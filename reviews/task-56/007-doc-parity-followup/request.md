# Task 56.1 — SPIRE `/// # Safety` doc parity follow-up

**Branch:** `task-59-parallel-stream-burndown` (interleaved between
Task 59 slice 002 and slice 003 per reviewer seq 02 + seq 04
disposition).

**Scope-lock:** safety-doc additions only. No code logic changes; no
unsafe-block count changes; no signature changes.

## Why

Task 56 closeout reviewer seq 03 (`reviews/task-56/006-closeout/feedback/2026-05-24-03-reviewer.md`)
identified 12 missing `/// # Safety` docs across 8 SPIRE files that
were not addressed before the Task 56 branch merged into main. Task 59
slice 002 reviewer seq 02 and seq 04 confirmed those 12 docs remain
pending and HARD BLOCKED Task 59 close on them.

Per `feedback_dont_defer_safety_fixes` (HARD RULE, 2026-05-24):
> If a review finding is "missing `/// # Safety` docs" — BLOCK.

## Per-file count

| File | Missing pre-fix | Missing post-fix |
|---|---:|---:|
| `src/am/ec_spire/storage/relation_store.rs` | 4 | **0** |
| `src/am/ec_spire/update/publish/relation.rs` | 2 | **0** |
| `src/am/ec_spire/coordinator/debug.rs` | 1 | **0** |
| `src/am/ec_spire/update/materialization.rs` | 1 | **0** |
| `src/am/ec_spire/update/routing.rs` | 1 | **0** |
| `src/am/ec_spire/scan/types.rs` | 1 | **0** |
| `src/am/ec_spire/build/recursive.rs` | 1 | **0** |
| `src/am/ec_spire/build/tuples.rs` | 1 | **0** |
| **Total** | **12** | **0** |

Verification (reviewer's seq-01 awk pattern, with the existing
cfg-attribute-skipping refinement so attributes between doc and
unsafe fn don't trigger false positives):

```
$ for f in src/am/ec_spire/{storage/relation_store,update/publish/relation,coordinator/debug,update/materialization,update/routing,scan/types,build/recursive,build/tuples}.rs; do
    echo "=== $f ==="
    awk 'BEGIN{d=0;s=0}
      /^[ \t]*\/\/\/ # Safety/{s=1;d=1;next}
      /^[ \t]*\/\/\//{d=1;next}
      /^[ \t]*#\[/{next}
      /unsafe fn/{
        if(!d) print "NO_DOC: L"NR
        else if(!s) print "DOC_NO_HEADING: L"NR
        d=0;s=0;next
      }
      /^[ \t]*$/{next}
      {d=0;s=0}' "$f"
  done
=== ... === (all 8 files clean)
```

## Per-function dispositions

### `storage/relation_store.rs` (4 fns)

1. **`SpireRelationObjectStore::for_index_relation`** —
   - Names: index_relation null-or-open-with-lock; remains open for
     store lifetime; returns Err on null/invalid OID.
2. **`SpireRelationObjectStore::for_store_relation_id`** —
   - Names: store_relation null-or-open; remains open for store
     lifetime (caller's relation guard owns the close); store_relid
     must match relation OID; local_store_id must match local-store
     config — passing inconsistent identifiers misroutes writes.
3. **`SpireRelationObjectStoreSet::for_index_relation_and_config`** —
   - Names: index_relation null-or-already-open at lockmode and
     remains open for set lifetime; per-store relations are opened
     internally by `OpenedRelationsGuard` and closed on drop, so
     callers must not concurrently close them.
4. **`SpireRelationObjectStoreSet::for_index_relation_and_placements`** —
   - Same contract as `_and_config`; placement directory is a borrowed
     snapshot copied into the set.

### `update/publish/relation.rs` (2 fns)

5. **`publish_relation_scheduled_replacement_epoch`** —
   - Names: index_relation must be the live SPIRE index relation
     opened at RowExclusiveLock; remains open for the call;
     object_store backed by the same relation; previous_epoch_manifest
     must reflect the manifest live before this publish.
6. **`publish_relation_selected_scheduled_replacement_epoch`** —
   - Same lock/lifetime contract; selected.lock_plan.pid_plan must
     encode the PID schedule selected for the previous-epoch manifest.

### `coordinator/debug.rs` (1 fn)

7. **`debug_spire_manifest_bundle`** —
   - Names: test-only helper (cfg-gated to test / pg_test); requires
     live SPIRE index relation and matching root_control_state read
     from the same relation.

### `update/materialization.rs` (1 fn)

8. **`fetch_split_replacement_source_vectors`** —
   - Names: heap_relation, snapshot, slot all come from caller's
     materialization scope; relation opened with appropriate lock,
     snapshot is the active scan snapshot, slot is a reusable slot
     whose lifetime covers the entire call; indexed_attribute must
     match the attribute number that produced the heap TIDs.

### `update/routing.rs` (1 fn)

9. **`build_relation_selected_scheduled_split_replacement_execution_input_from_heap_sources`** —
   - Same heap-relation/snapshot/slot contract; indexed_attribute
     must match the attribute that produced heap TIDs in
     selected.lock_plan; object_store backed by the same SPIRE index
     relation that produced snapshot / selected.

### `scan/types.rs` (1 fn)

10. **`root_control_for_rescan`** —
    - Names: index_relation is the live SPIRE index relation that the
      AM scan rescan callback is operating on; PostgreSQL guarantees
      open + lock for the duration of the rescan call; only reads
      the root-control page; AM scan state machine ensures no
      concurrent publish to that page during rescan.

### `build/recursive.rs` (1 fn)

11. **`publish_relation_recursive_routing_epoch_draft`** —
    - Names: index_relation is the SPIRE index relation opened by
      ambuild / publish phase at AccessExclusiveLock; remains open
      for the call; draft.placement_directory must have been
      validated by recursive routing against the same relation;
      next_local_vec_seq + local_store_config must reflect the state
      captured at the start of the recursive build.

### `build/tuples.rs` (1 fn)

12. **`detoasted_varlena_bytes`** —
    - Names: datum must be a valid varlena Datum sourced from a live
      PostgreSQL tuple slot / HeapTuple — pointed-at region must be
      a well-formed PG varlena (packed / compressed / fully-detoasted)
      and live for the duration of the call; detoast wrapper copies
      bytes before returning so the returned Vec outlives the input
      Datum safely.

## Style alignment

Each new doc:

- Leads with a one-line summary of what the function does (the
  business-level "what").
- Has an explicit `# Safety` heading on a separate doc line so
  clippy `missing_safety_doc` accepts them.
- Names: pointer-validity, lifetime, and any concurrency / lock
  invariants the caller must honor; failure mode if the contract is
  violated where the failure mode is not "UB" (e.g., "misroutes
  writes", "publishes inconsistent manifest").
- Matches the style of the Task 59 slice 002 fix-up
  (`reviews/task-59/002-parallel-typed-views/feedback/2026-05-24-03-coder.md`
  §"HARD BLOCK 2") and the Task 57 doc-fix precedent at commit
  `fdee08894`.

## Validation

- `cargo check --no-default-features --features pg18 --lib` — green
  (one pre-existing unused-import warning in `src/am/ec_spire/update.rs`,
  unrelated, unchanged).
- Reviewer's awk audit (with cfg-attribute-skipping refinement) on
  the 8 files: zero missing.
- No `unsafe { ... }` block count changes (this packet only adds
  documentation comments above existing `unsafe fn` declarations).
- No code logic / signature changes; binary-compatible with all
  consumers.

## Disposition

**Task 56.1 ready for reviewer signoff.** All 12 missing SPIRE
`/// # Safety` docs landed with substantive contracts. The
Task-59-close-blocker from reviewer seq 02 (HARD BLOCK 3) is now
clear; pending reviewer ack, Task 59 slice 003 can open.

## Cross-references

- Originating reviewer ask: `reviews/task-56/006-closeout/feedback/2026-05-24-03-reviewer.md`.
- Task 59 hold-open: `reviews/task-59/002-parallel-typed-views/feedback/2026-05-24-02-reviewer.md` §"HARD BLOCK 3" and `2026-05-24-04-reviewer.md` §"Required ordered next steps" step 2.
- Doc-fix style precedent (Task 57): commit `fdee08894`.
- Task 56 partial-fix precedent (Task 56 main, 20/32 fixed under
  pressure): commit `af69128e8`.
- Anti-pattern + view-op memory rules honored throughout — no code
  changes, just docs.

## Artifacts

- `artifacts/spire_safety_doc_audit_post.txt` — awk audit output
  showing zero missing across the 8 files at HEAD post-fix.
- `artifacts/manifest.md` — packet-local source of truth.
