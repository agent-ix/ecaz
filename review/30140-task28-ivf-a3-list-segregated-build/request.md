# Task 28 IVF A3 List-Segregated Build

## Scope

This packet records commit `4e568d22`, which addresses the A3 diagnostic from
packet 30139 by segregating build-time posting pages by IVF list.

The change adds a `DataPageChain` separator and uses it in IVF build staging:

- centroid/codebook tuples are separated from posting tuples;
- each non-empty posting list starts on a fresh page;
- directory tuples are separated from posting pages.

This preserves list-local range reuse after vacuum and prevents build-time
cross-list posting-page sharing.

## Result

The packet-30124 same-slice churn fixture was rerun after the change.

| phase | n32 index bytes | n64 index bytes |
|---|---:|---:|
| cycle0 build | 4,603,904 | 4,734,976 |
| cycle1 refill | 4,603,904 | 4,734,976 |
| cycle2 refill | 4,603,904 | 4,734,976 |
| cycle3 refill | 4,603,904 | 4,734,976 |

Page ownership after cycle3:

| nlists | posting blocks | cross-list blocks | mixed metadata/posting blocks | unused line pointers | posting tuples | deleted posting tuples |
|---|---:|---:|---:|---:|---:|---:|
| 32 | 530 | 0 | 0 | 0 | 50,000 | 0 |
| 64 | 550 | 0 | 0 | 0 | 50,000 | 0 |

## Interpretation

This closes the specific A3 issue diagnosed in packet 30139:

- n64 cross-list posting blocks dropped from `21` to `0`;
- n64 mixed metadata/posting blocks dropped from `2` to `0`;
- n64 index size stayed flat through all three churn cycles.

The tradeoff is expected build-time space rounding: the n64 initial index grew
from `4,472,832` bytes in packet 30139 to `4,734,976` bytes here, about 5.9%.
That is the cost of page-level list isolation in this 50k/4D fixture. The next
A3 closure step should run the agreed longer 100k sustained-churn pass before
calling the convergence gate fully closed.

## Validation

- `cargo test -p ecaz --lib build_state_segregates_posting_blocks_by_list`
- `cargo test -p ecaz --lib data_page_chain_can_start_next_tuple_on_fresh_page`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.sql --raw --log-output review/30140-task28-ivf-a3-list-segregated-build/artifacts/ivf_same_slice_churn_list_segregated.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/page_ownership_diagnostic.sql --raw --log-output review/30140-task28-ivf-a3-list-segregated-build/artifacts/page_ownership_list_segregated.log`
- `git diff --check`

## Artifacts

- `artifacts/ivf_same_slice_churn_list_segregated.log`
- `artifacts/page_ownership_list_segregated.log`
- `artifacts/manifest.md`
