# Review Request: C1 ADR-030 V2 Merge Readiness Assessment

Current head: `4fec776`

This packet covers local uncommitted work on top of that head.

## Context

Reviewer feedback on the merge-readiness assessment asked for one thing above
all else: stop inferring that the branch is green and capture the local
execution proof directly.

For the current task, the scope is local build/test readiness only:

1. ignore GitHub CI for now
2. prove the branch is locally executable
3. show the specific formerly-blocked tests green on the final tree

This packet is the evidence slice for that request.

## Problem

Before this slice, the branch had strong code and measurement evidence, but the
merge story still had one avoidable hole:

1. `cargo test` had been repaired in packet `415`
2. the runtime/test-alignment work from packets `416` to `418` was present
3. but there was no final captured statement saying which load-bearing tests are
   now actually green on the repaired tree

That left the landing proof weaker than it needed to be.

## Evidence

Final local validation on the current tree:

1. `cargo test`
   - passed
   - `461 passed; 0 failed; 7 ignored`
2. `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
   - passed
3. `bash scripts/run_pgrx_pg17_test.sh`
   - fails in the sandbox because `cargo pgrx install --test` cannot write
     `/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector.control`
     on a read-only filesystem
   - rerun outside the sandbox passes on the same tree

The specific tests the reviewer called out are now green under the final local
tree:

1. `test_tqhnsw_turboquant_reloption_round_trip`
2. `test_tqhnsw_pq_fastscan_reloption_round_trip`
3. `test_tqhnsw_storage_format_switch_rejects_insert_until_reindex`
4. `test_tqhnsw_storage_format_switch_rejects_vacuum_until_reindex`
5. `test_tqhnsw_storage_format_switch_reverse_requires_reindex`
6. `test_tqhnsw_storage_format_switch_reverse_rejects_insert`
7. `test_tqhnsw_storage_format_switch_reverse_rejects_vacuum`
8. `test_tqhnsw_storage_format_switch_reindex_restores_runtime`
9. `test_pq_fastscan_default_source_rerank_emits_heap_scores`
10. `test_pq_fastscan_default_rerank_matches_explicit_heap`
11. `test_pq_fastscan_index_runtime_settings_report_binary_default`
12. `test_pq_fastscan_index_runtime_settings_report_binary_fallback`
13. `test_pq_fastscan_index_runtime_settings_report_heap_override`
14. `test_pq_fastscan_index_runtime_settings_report_env_override`
15. `am::scan::tests::grouped_binary_traversal_score_gate_requires_pq_fastscan_storage`

The wrapper proof is also now explicit:

1. inside the sandbox, the wrapper still trips on the `.pgrx` install target
2. outside the sandbox, the same wrapper completes successfully on the current
   tree
3. the earlier wrapper failure is therefore an environment restriction, not a
   code or regression signal

## Outcome

Within the current scope of local readiness, the branch is now at merge-proof
quality:

1. the repaired `cargo test` lane is green
2. the full local wrapper is green when allowed to use the normal `.pgrx`
   install destination
3. the formerly-requested storage-format and pq_fastscan runtime tests are now
   explicitly captured as green
4. the remaining caveat is environmental sandbox policy, not local code health

## Remaining Non-Goals

These are intentionally not treated as blockers in this packet:

1. GitHub CI bring-up
2. planner behavior across multiple sibling indexes on one table
3. further measurement work unrelated to local build/test proof

## Next Slice

The branch can move back to the normal review loop:

1. commit the code checkpoint
2. commit these packet updates separately
3. push both so outside reviewers are looking at the same green local state
