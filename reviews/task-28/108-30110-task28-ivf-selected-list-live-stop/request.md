# Task 28 IVF Selected-List Live Stop

## Scope

This packet covers the A7 code slice in `69ec3df1`: IVF scan materialization now carries each selected list's directory `live_count` into the posting scan and stops scoring candidates for that list once the live heap-TID budget is consumed.

This is a conservative posting-scan early-stop lever for the current page layout. It does not change the probed block sequence or rely on a new score-bound model.

## Behavior

Before this slice, scan materialization decoded and scored every non-deleted posting tuple in the selected list ranges, even after VACUUM had preserved a broad range with stale/dead tail space.

After this slice:

- `build_selected_probe_plan` records `remaining_live_tids_by_list` from directory entries.
- `materialize_probe_candidates` decrements that budget by each live posting's heap-TID count.
- Once a selected list reaches zero remaining live TIDs, later live-looking tuples in that list's preserved range are ignored instead of scored or inserted into candidate state.

The block read pattern is intentionally unchanged; this slice reduces candidate scoring/work after churn, not page IO.

## Validation

- `cargo test -p ecaz --lib consume_live_tid_budget`
- `cargo test -p ecaz --lib am::ec_ivf::scan::tests`
- `cargo pgrx test pg18 test_ec_ivf_insert_vacuum_scan_safety`
- `git diff --check`

## Next

Rerun the 100k+ recall/latency frontier after this change to see whether the reduced post-vacuum scoring path materially affects the local tuning frontier.
