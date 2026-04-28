# Task 28 IVF Adjacent Posting Page Reuse

## Scope

This packet records the A3/F2 follow-up in commit `4ed20913`.

The change keeps normal list-local reuse first, then tries only the immediate left and right neighbor blocks before allocating a new posting page. If a neighbor accepts the posting, the list directory can extend its head or tail to cover that one block.

The reuse is deliberately bounded to one neighboring block per insert attempt so a list cannot expand into a wide scan range just to chase arbitrary old free pages.

## Result

### Same-Slice Churn

At head `4ed20913`, using the same-slice diagnostic from packet 30124:

- nlists=32 converged through cycle3 at `4,464,640` bytes.
- nlists=64 cycle3 was `4,538,368` bytes and `43,756.698 ms`.

Comparison to packet 30124:

- nlists=64 cycle3 size improved from `4,825,088` bytes to `4,538,368` bytes.
- nlists=64 cycle3 refill improved from `124,588.292 ms` to `43,756.698 ms`.

### Original Drifting Churn

At head `4ed20913`, using the original churn script from packet 30120:

- nlists=32 converged through cycle3 at `4,464,640` bytes.
- nlists=64 cycle3 was `4,497,408` bytes and `34,070.797 ms`.

Comparison to packet 30123:

- nlists=64 cycle3 size improved from `4,964,352` bytes to `4,497,408` bytes.
- nlists=64 cycle3 refill improved from `134,251.989 ms` to `34,070.797 ms`.

## Interpretation

This materially improves A3's sustained churn story. The nlists=64 index still does not return exactly to its build size, so this is not a claim that vacuum fully shrinks the index. It does show that tuple compaction plus bounded adjacent-page reuse keeps size close to live data and removes the prior multi-minute refill cliff on the local churn fixture.

## Validation

- `cargo test -p ecaz --lib directory_insert_stats_extends_head_backward`
- `cargo test -p ecaz --lib am::ec_ivf::insert::tests`
- `cargo test -p ecaz --lib am::ec_ivf::page::tests`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.sql --raw --log-output review/30125-task28-ivf-adjacent-page-reuse/artifacts/ivf_same_slice_churn_adjacent_smoke.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30125-task28-ivf-adjacent-page-reuse/artifacts/ivf_sustained_churn_adjacent_smoke.log`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- `cargo pgrx test pg18 test_ec_ivf_insert_vacuum_scan_safety`
- `git diff --check -- src/am/ec_ivf/page.rs src/am/ec_ivf/insert.rs`

## Artifacts

- `artifacts/ivf_same_slice_churn_adjacent_smoke.log`
- `artifacts/ivf_sustained_churn_adjacent_smoke.log`
- `artifacts/manifest.md`
