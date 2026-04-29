# Task 28 IVF A3 Page Ownership Diagnostic

## Scope

This packet records the first A3 diagnostic requested in the round-2 feedback:
dump per-block IVF posting ownership after the same-slice churn fixture so the
remaining nlists=64 growth can be attributed instead of guessed.

Commit `06701988` adds `ec_ivf_index_page_ownership(index_oid)` as an admin
diagnostic. For each index block it reports line-pointer count, unused line
pointers, non-posting tuples, posting tuple counts, deleted posting tuples,
heap-TID refs, and the list IDs represented on that block.

## Fixture

The existing packet-30124 same-slice churn SQL was rerun on local PG18:

- 50k rows
- 4 dimensions
- `quantizer='turboquant'`
- `nlists in {32,64}`
- three delete/vacuum/refill cycles of 25k rows

## Result

Current-head churn size:

| phase | n32 index bytes | n64 index bytes |
|---|---:|---:|
| cycle0 build | 4,464,640 | 4,472,832 |
| cycle1 refill | 4,464,640 | 4,472,832 |
| cycle2 refill | 4,464,640 | 4,489,216 |
| cycle3 refill | 4,464,640 | 4,538,368 |

Page-ownership summary after cycle3:

| nlists | posting blocks | cross-list blocks | mixed metadata/posting blocks | unused line pointers | posting tuples | deleted posting tuples |
|---|---:|---:|---:|---:|---:|---:|
| 32 | 530 | 0 | 1 | 0 | 50,000 | 0 |
| 64 | 549 | 21 | 2 | 0 | 50,000 | 0 |

The detailed diagnostic rows show the n64 cross-list pages are concentrated in
tail blocks and include pages such as:

- block 517: lists `31,43,61`
- block 526: lists `31,43,62`
- block 539: lists `31,43,63`
- block 545: lists `31,43,63` plus non-posting tuples
- block 550: lists `16,62,63`

## Interpretation

The diagnostic points at cross-list page sharing as the remaining A3 pressure,
not unreclaimed tombstones:

- `deleted_posting_tuples=0` and `unused_line_pointers=0` after vacuum/refill.
- n32 has no cross-list posting blocks and remains size-stable across cycles.
- n64 has 21 cross-list posting blocks and two mixed metadata/posting blocks,
  and is the only shape that grows across cycles.

This makes the next A3 implementation decision narrower: either segregate list
posting pages at build time, or add reusable free-space metadata that can cross
list boundaries. The current list-local reuse path is functioning, but it cannot
fully reason about shared pages.

## Validation

- `cargo test -p ecaz --lib am::ec_ivf`
- `cargo pgrx test pg18 test_ec_ivf_page_ownership_snapshot_reports_posting_blocks`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.sql --raw --log-output review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/ivf_same_slice_churn_current.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/page_ownership_diagnostic.sql --raw --log-output review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/page_ownership_diagnostic.log`
- `git diff --check`

## Artifacts

- `artifacts/ivf_same_slice_churn_current.log`
- `artifacts/page_ownership_diagnostic.sql`
- `artifacts/page_ownership_diagnostic.log`
- `artifacts/manifest.md`
