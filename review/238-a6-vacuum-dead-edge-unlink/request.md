# Review Request: A6 Vacuum Dead-Edge Unlink

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `spec/functional/FR-022-vacuum-implementation.md`

This is the next narrow A6 checkpoint after mark + finalize. It still does not
attempt replacement search, but it closes the first real pass-2 graph-repair
gap by removing stale dead-node references from persisted neighbor tuples.

Checkpoint scope:

1. implement pass-2 dead-edge unlink by scanning neighbor tuples and clearing
   any slot that points at a fully-dead element TID
2. keep the write protocol one page at a time in ascending block order, with a
   dedicated ADR for the new multi-page repair surface
3. run pass 2 before finalize so the three-pass shape now reads mark → unlink →
   finalize
4. add regression coverage for persisted dead-edge removal and repeated repair
   stability
5. update task/status/spec docs to reflect that replacement search is now the
   remaining A6 graph-repair step

## Scope

- `src/am/vacuum.rs`
- `src/lib.rs`
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/adr/ADR-027-vacuum-graph-repair-lock-ordering.md`
- `review/README.md`

## What Landed

### 1. Vacuum now performs a real pass-2 unlink repair

After pass 1 identifies fully-dead element TIDs, vacuum now runs a second scan
over persisted neighbor tuples and clears any slot that still points at one of
those dead element TIDs.

This is intentionally broader than “walk the deleted node's outgoing adjacency.”
The current slice scans all neighbor tuples so it catches asymmetric stale edges
too, not just reciprocal ones.

### 2. The write protocol stays narrow and explicit

The pass-2 rewrite shape matches the existing narrow vacuum rewrite model:

1. scan a data page under `SHARE`
2. if that page needs repair, release it
3. reopen that same page alone under `EXCLUSIVE`
4. replan against the current page image
5. rewrite through GenericXLog

There is never more than one data-page `EXCLUSIVE` lock held at a time. The new
ADR-027 records that lock ordering for this pass-2 surface and reserves the same
rule for future replacement search.

### 3. The checkpoint now matches the intended three-pass order structurally

`run_pass1_vacuum(...)` now executes:

1. pass 1 mark / heap-TID stripping
2. pass 2 dead-edge unlink
3. pass 3 finalize

That still leaves replacement search open, but the overall A6 shape is now much
closer to the final intended algorithm than the earlier mark+finalize-only
checkpoint.

### 4. Coverage proves persisted stale dead edges are removed

The new regression surface proves that:

- a real deleted element starts with at least one persisted inbound neighbor ref
- after vacuum, no persisted neighbor tuple still contains that deleted element
  TID
- repeated vacuum replays keep the dead element finalized and fully unlinked

The earlier pass-1 / finalize / duplicate-guard regressions remain in place.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `tests::pg_test_tqhnsw_vacuum_pass2_unlinks_deleted_neighbor_refs`
- `tests::pg_test_tqhnsw_vacuum_pass1_is_stable_across_repeated_replays`

## Review Focus

- Is the “scan every neighbor tuple for dead refs” choice the right first pass-2
  checkpoint, given the need to catch asymmetric stale edges before replacement
  search exists?
- Does the share-then-exclusive page-local rewrite shape in `src/am/vacuum.rs`
  look like the right concurrency boundary for future replacement-search work?
- Is ADR-027 scoped correctly, or should vacuum graph repair share a different
  durable lock-ordering document with A5 insert backlink mutation?
