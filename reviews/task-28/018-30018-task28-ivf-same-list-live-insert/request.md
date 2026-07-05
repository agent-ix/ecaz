# Review Request: Task 28 IVF Same-List Live Insert

Scope: Phase 5 live-insert coverage checkpoint. Sequential inserts into one
IVF list now have PG coverage for tail-page reuse, counter updates, and scan
reachability.

Task: `plan/tasks/28-ivf-access-method.md` Phase 5

Branch: `task28-ivf`

Head SHA: `610cdf4288628d187dedad89bc35d946696c0c7f`

Owner: coder2

Files:

- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_insert_reuses_same_list_tail_page`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy and the explicit user
  direction to test with PG18.
- The new PG test was run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice adds Phase 5 coverage rather than new production logic:

- Builds a single-list IVF index, inserts two additional rows into that same
  list, and verifies the index block count stays stable for small postings.
- Verifies per-list live count, metadata live count, and inserted-since-build
  drift counters advance after repeated same-list inserts.
- Confirms both live-inserted rows are reachable through the IVF scan debug
  path without duplicate heap IDs.
- Updates the task plan to separate same-list tail append coverage from the
  remaining true concurrency and duplicate heap-TID work.

## Review Focus

Please review for:

- Whether asserting stable block count is the right way to cover tail-page
  reuse for a small single-list fixture.
- Whether this coverage should also inspect directory tail block refs directly
  before the concurrency slice.
- Whether the remaining Phase 5 checklist is scoped correctly around
  concurrent inserts and duplicate heap-TID rejection.

## Non-Goals

This packet does not implement new append logic, duplicate heap-TID
coalescing/rejection, concurrent insert stress coverage, vacuum cleanup,
planner costing, heap/source rerank, or measurement gates.
