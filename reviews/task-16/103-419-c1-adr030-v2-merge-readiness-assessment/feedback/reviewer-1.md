## Cross-Arc Merge-Readiness Assessment: adr030-v2-heap-source-experiment → main

Synthesis across packets `378`–`418`. Not a per-packet review —
a consolidated view of where the branch stands against the
task-15 merge bar. Grounded in the per-packet feedback already
filed under `review/{401..418}/feedback/reviewer-1.md`.

### Where the branch stands

**Landing case is strong on measurement terms.** Packets `413`
and `414` together give the first honest `corpus × format × m ×
ef` matrix on a single clean runtime lane, and the `50k, m=16,
ef>=128` cell has `pq_fastscan` ahead on both recall (`0.9635`
vs `0.9342` at `ef=128`) and SQL latency (`4.263ms` vs `4.437ms`
at `ef=128`). That is the durable landing quote — bilateral-win,
plan-verified, scratch-cluster only, no `~/.pgrx` dependency.

**Code-side landing is mechanically right.** The runtime-decision
refactor (`401` / `404` / `408` / `410`) consolidated
`PqFastScan` traversal + rerank behavior behind
`PqFastScanTraversalScoreModeResolution` and
`PqFastScanRerankModeResolution` enum-decisions with a shared
index-aware debug helper. The REINDEX guardrail (`403` / `409`)
has turboquant↔pq_fastscan mismatch coverage on all three AM
entry paths (scan, insert, vacuum) via real AM callbacks, not
bypasses.

### Merge blockers (in priority order)

1. **Test execution is no longer a hypothetical — capture it.**
   Packet `415` shipped a 240-line C stub that makes `cargo test`
   a real checkpoint on Linux/x86_64. But `415` does not name
   *which* of the 378–418 `#[pg_test]`s are now actually green.
   Before merge there needs to be a captured run of `cargo test
   2>&1 | tail -50` at the final SHA showing the previously-
   blocked tests green:
   - `test_tqhnsw_turboquant_reloption_round_trip` (`393`)
   - `test_tqhnsw_pq_fastscan_reloption_round_trip` (`393`)
   - `test_tqhnsw_storage_format_switch_rejects_insert_until_reindex` (`409`)
   - `test_tqhnsw_storage_format_switch_rejects_vacuum_until_reindex` (`409`)
   - `test_pq_fastscan_default_source_rerank_emits_heap_scores` (`404`)
   - `test_pq_fastscan_default_rerank_matches_explicit_heap` (`411`)
   - `tqhnsw_debug_pq_fastscan_runtime_settings_for_index` coverage (`408` / `410`)

   Without that artifact, the landing proof still rests on
   `cargo check`.

2. **No GitHub CI exists.** `415` unblocks the technical
   prerequisite, but no CI lane is actually running these tests
   on every PR. That is a permanent merge risk, not a one-time
   gate. Either stand up a minimal GitHub Actions workflow
   running `cargo test` on Linux/x86_64, or explicitly name this
   as accepted technical debt at merge with a named owner and a
   target date.

3. **Test coverage breadth is still narrow.** `409`'s guardrail
   tests only cover:
   - one mismatch direction (`turboquant`→`pq_fastscan`); the
     reverse is untested
   - the mismatch case only — no happy-path-after-matching-ALTER
     assertion
   - no REINDEX-clears-guardrail assertion

   The tests that exist are correctly shaped and exercise real
   AM routing, but the matrix is incomplete. A follow-up packet
   adding ~4–6 cases would turn the guardrail from "protects
   against the one case we tested" to "protects against the
   class."

4. **Local uncommitted batch (`416` / `417` / `418`) needs
   landing.** These are pre-merge cleanup that the `413` / `414`
   measurements implicitly depend on:
   - `417`'s fixture alignment (binary-capable fixture, exact-
     score derivation from source, row-ids by observed
     self-rank)
   - `418`'s `BuildCodeDistance::new` offset fix on the hot
     build path
   - `416`'s debug-helper heap-backed repair

   Specifically `417` bundles 7 test changes + 1 runtime change
   (`grouped_binary_traversal_score_enabled` tightening in
   `src/am/scan.rs`). For clean merge story, the AM-logic change
   should be split into its own packet so the test-alignment
   diff stays mechanical.

5. **`418`'s offset change is unmeasured on the hot build path.**
   `BuildCodeDistance::new(...)` now takes the full `BuildTuple`
   slice and runs `score_code_inner_product` over every tuple
   before HNSW begins. Negligible on small corpora; at `50k`+
   this should have a one-row before/after build-time
   measurement. Easy to capture, and without it the correctness
   improvement is locked in unmeasured.

6. **Shared-table planner cross-choosing is an open product
   question.** `414` honestly scoped itself to isolated
   one-index-per-table surfaces. The real production shape —
   many indexes on one table — still shows the planner
   cross-choosing between sibling `m=8` / `m=16` indexes. Not a
   task-15 merge blocker, but it is the next thing an operator
   will hit and the branch has no answer for it yet. Should be a
   named follow-up task, not a "we'll see."

### My read

**Roughly 85% ready.** Code and measurements are there. What is
missing is evidence-of-execution, not evidence-of-correctness.

Three concrete steps before merge:

1. Land `416` → `418` on branch (re-split `417`'s AM change
   first).
2. Capture `cargo test` output at the final SHA showing the
   specific previously-blocked tests green, and attach it to a
   landing-proof packet.
3. Add the 4–6 missing coverage cases for `409` (reverse-
   direction mismatch on all three paths, happy-path-after-
   matching-ALTER, REINDEX-clears-guardrail).

A merge today based on current artifacts works *if* the reviewer
accepts that every `#[pg_test]` on the branch is source-
inspected-only. A merge with step 2 done works without that
asterisk. Step 3 is the difference between "merged and safe" and
"merged and safe even against future mistakes in this area."

The CI question (blocker 2) is orthogonal to this merge but will
be the gating question for the *next* one. It should not hold
task-15 specifically — but it should be a named follow-up with
an owner.

### What would move this to "ready"

- a captured `cargo test` landing artifact (closes blocker 1)
- `416` / `418` landed, `417` split and landed (closes blocker 4)
- a follow-up packet adding `409`'s missing coverage cases
  (closes blocker 3)
- a named CI-standup task with owner and target (addresses
  blocker 2 without gating this merge)
- a named planner-investigation task (addresses blocker 6
  without gating this merge)

At that point there is no honest reason not to merge. Until
then, "merge readable" is the honest description, not "merge
ready."
