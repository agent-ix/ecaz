# Review Request: Task 28 IVF Vacuum Safety Coverage

Scope: Phase 6 vacuum checkpoint. IVF now has PG coverage for repeated vacuum
cleanup and live-insert/delete/vacuum scan safety.

Task: `plan/tasks/28-ivf-access-method.md` Phase 6

Branch: `task28-ivf`

Head SHA: `0fc431fa31cbc9a8809c382ca64235a3fdbdc3e8`

Owner: coder2

Files:

- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repeated_bulkdelete_is_idempotent`
- `cargo pgrx test pg18 test_ec_ivf_insert_vacuum_scan_safety`
- `git diff --cached --check`

Validation notes:

- Validation was PG18-only per the current user direction to focus on PG18.
- The PG tests were run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice closes the first-baseline Phase 6 vacuum safety item:

- Adds repeated bulkdelete coverage proving a second cleanup pass does not
  remove the same dead heap TID twice.
- Adds live-insert/delete/vacuum coverage proving a live-inserted row is
  reachable before vacuum and excluded from scan output after vacuum.
- Checks live/dead counters, metadata live totals, inserted-since-build drift,
  and finite post-vacuum scan scores.
- Updates the task plan to mark Phase 6 complete for the first IVF baseline.

## Review Focus

Please review for:

- Whether the repeated-vacuum test is sufficient for idempotence at this stage.
- Whether the live-insert/delete/vacuum test should also assert exact result
  ordering, or whether absence of the deleted heap TID is the right safety gate.
- Whether Phase 6 should include any additional non-concurrent scan/vacuum
  behavior before moving back to Phase 5 shape/concurrency gaps.

## Non-Goals

This packet does not run SQL `VACUUM`, add concurrent vacuum/scan coverage,
compact or reclaim pages, implement planner costing, or make measurement claims.
