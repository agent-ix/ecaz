# Review Request: A6 Vacuum Finalize And Duplicate Guard

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `spec/functional/FR-010-hnsw-vacuum.md`
- `spec/functional/FR-022-vacuum-implementation.md`

This is the next narrow A6 checkpoint after pass-1 dead-heap-TID stripping. It
does not attempt graph repair yet, but it closes a concrete correctness gap in
the pass-1-only state.

Checkpoint scope:

1. finalize fully-dead element tuples to `deleted = true`
2. keep the implementation one page at a time with no new cross-page lock protocol
3. teach duplicate discovery to skip deleted / empty-heaptid elements
4. add regression coverage for reinserting the same encoded vector after vacuum
5. update task/status/review docs to reflect “mark + finalize landed; repair pending”

## Scope

- `src/am/vacuum.rs`
- `src/am/insert.rs`
- `src/lib.rs`
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `review/README.md`

## What Landed

### 1. Fully-dead vacuum candidates now finalize to `deleted = true`

The previous checkpoint stopped at the intermediate state:

- `heaptids = []`
- `deleted = false`

That was enough for scan invisibility, but it left an ambiguous on-disk tomb
state behind.

The vacuum path now collects fully-dead element TIDs during pass 1 and runs a
separate finalize phase that:

- sorts/deduplicates those element TIDs
- groups them by block
- rewrites one page at a time under `EXCLUSIVE`
- sets `deleted = true` only when the live tuple is still empty-heaptid and not
  already deleted

This keeps the implementation structurally close to the eventual three-pass A6
shape while still avoiding pass-2 graph repair in the same checkpoint.

### 2. This closes a real duplicate-insert reanimation bug

While working the pass-1 slice, I found that `find_duplicate_element_tid(...)`
still considered any matching element tuple by `(code, gamma)`, even if that
element had:

- `deleted = true`, or
- `heaptids.is_empty()`

That meant a live insert of the same encoded vector after vacuum could
incorrectly select a dead element tuple for duplicate coalescing.

In the worst case, the insert would append the new heap TID to a finalized dead
node and keep `deleted = true`, making the new row invisible.

This checkpoint fixes that by teaching duplicate discovery to skip dead and
empty-heaptid element tuples outright.

### 3. Lock scope remains narrow

There is still no new multi-page write protocol here.

Both the mark and finalize phases operate:

- one page at a time
- with read-side scanning first
- and page-local `EXCLUSIVE` rewrites only when a page actually changes

So this slice does not introduce the kind of lock-ordering ADR work that A5
needed.

### 4. Coverage now locks in the reinsert behavior

The new regression surface proves that:

- fully-dead elements become `deleted = true`
- repeated vacuum passes remain stable
- reinserting the same encoded vector after vacuum creates or selects a live
  element instead of reattaching to the dead tombstone
- graph/runtime scan can still reach the replacement row

This is the main correctness reason for taking the finalize slice before graph
repair.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `tests::pg_test_tqhnsw_vacuum_pass1_makes_deleted_row_unreachable`
- `tests::pg_test_tqhnsw_vacuum_pass1_is_stable_across_repeated_replays`
- `tests::pg_test_tqhnsw_vacuum_finalized_nodes_skip_duplicate_coalesce`

## Review Focus

- Is landing pass-3 finalize ahead of pass-2 graph repair a defensible narrow
  checkpoint, given that the runtime already skipped empty/deleted elements and
  the duplicate-reanimation bug is now closed?
- Does the finalize rewrite path have any remaining race or stale-read edge case
  before graph repair starts mutating neighboring nodes?
- Are the duplicate-discovery guards now sufficient, or is there any other live
  insert path that could still accidentally target a dead vacuum tombstone?
