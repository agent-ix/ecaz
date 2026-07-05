## Addendum: Post-420 Merge-Readiness State

Updates the `reviewer-1.md` assessment after packet `420` landed.
Supersedes the blocker list in the original where it has moved.

### Blocker status after `420`

| # | Blocker (from original) | Status after `420` |
|---|-------------------------|--------------------|
| 1 | Capture `cargo test` output naming the previously-blocked tests green | **Partially closed.** `420` names `test_tqhnsw_storage_format_switch_reindex_restores_runtime` and `grouped_binary_traversal_score_gate_requires_pq_fastscan_storage` as locally-green by test name. Still open: a single captured artifact naming the full set (`393`, `403`/`409`/`420`, `404`, `411`, `408`/`410`) green at one SHA. |
| 2 | No GitHub CI | **Unchanged.** Post-merge follow-up. Should have a named owner before merge, not block it. |
| 3 | `409` guardrail coverage breadth (reverse-direction, happy-path, REINDEX-clears) | **Closed.** `420` added reverse-mismatch on all three AM paths + the full REINDEX-restores-runtime happy-path (metadata flip + scan + insert + vacuum). |
| 4 | Local uncommitted `416`/`417`/`418` batch needs landing | **Unchanged.** Still uncommitted. `417`'s AM-logic change should still be split out — but `420` now provides direct unit coverage for that change (`grouped_binary_traversal_score_gate_requires_pq_fastscan_storage`), so the split is hygiene, not safety. |
| 5 | `418`'s `BuildCodeDistance::new` offset change unmeasured on hot build path | **Unchanged.** One before/after build-time row at `50k`. |
| 6 | Shared-table planner cross-choosing | **Unchanged.** Post-merge follow-up with named task. |

### Remaining work before merge

Three items, in order:

1. **Land `416` → `418` on branch.** Re-split `417`'s
   `grouped_binary_traversal_score_enabled` tightening into its
   own packet first; `420` now provides the unit-test that
   change needed. Capture `418`'s build-time delta at `50k`.

2. **Capture one landing-proof artifact.** Run at the final SHA
   after `416`/`417`/`418` land, capture `cargo test 2>&1 |
   tail -50` showing the specific previously-blocked tests
   green. Minimum test names to include:
   - `test_tqhnsw_turboquant_reloption_round_trip`
   - `test_tqhnsw_pq_fastscan_reloption_round_trip`
   - `test_tqhnsw_storage_format_switch_rejects_insert_until_reindex`
   - `test_tqhnsw_storage_format_switch_rejects_vacuum_until_reindex`
   - `test_tqhnsw_storage_format_switch_reverse_requires_reindex`
   - `test_tqhnsw_storage_format_switch_reverse_rejects_insert`
   - `test_tqhnsw_storage_format_switch_reverse_rejects_vacuum`
   - `test_tqhnsw_storage_format_switch_reindex_restores_runtime`
   - `test_pq_fastscan_default_source_rerank_emits_heap_scores`
   - `test_pq_fastscan_default_rerank_matches_explicit_heap`
   - `grouped_binary_traversal_score_gate_requires_pq_fastscan_storage`
   - any `tqhnsw_debug_pq_fastscan_runtime_settings_for_index`
     pg tests added in `408` / `410`

   Bonus if a `cargo pgrx test pg17` run (outside the `~/.pgrx`
   read-only sandbox) can be captured alongside. Ideal shape:
   one small packet containing the captured output and the head
   SHA.

3. **Name owners + targets for post-merge items.** Two entries:
   - CI standup: `cargo test` on Linux/x86_64 on every PR
   - Shared-table planner investigation: why the planner
     cross-chooses between sibling `m=8` / `m=16` indexes

   These should live in whatever tracker the project uses — not
   block this merge, but not be left as "we'll see."

### Readiness

**~92% ready.** The specific merge question — "does `pq_fastscan`
as a first-class format preserve scan/insert/vacuum correctness
under REINDEX and storage-format switches" — is now answered in
code *and* in tests that run locally. What remains is evidence
packaging (#2) and pre-merge cleanup (#1). Neither requires new
design work.

When #1 and #2 are done, this becomes **"ready"** without
asterisks. #3 is a hygiene ask, not a gate.

### For an agent picking this up

If driving toward merge, the work list is:
1. rebase/split `417`, commit `416`+`417`+`418`, measure `418`
2. run full `cargo test` at final SHA, capture output, file as
   a landing-proof evidence packet
3. open two follow-up tasks (CI, planner) with owners
4. propose merge

Nothing else on the 378–420 arc needs code changes before
merge.
