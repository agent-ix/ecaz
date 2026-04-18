# Review Request: C1 ADR-030 V2 Final Local Landing Proof Artifact

Code checkpoint under test: `ea9ec05`

This packet adds no code. It captures the final local execution evidence the
latest `419` / `420` feedback asked for.

## Context

The remaining merge-readiness gap after `420` was evidence packaging:

1. one captured local artifact naming the previously-blocked tests green
2. one explicit statement separating sandbox filesystem restrictions from code
   health

This packet closes that gap.

## Problem

The branch already had:

1. repaired local `cargo test` coverage via `415`
2. the source-backed pq-fastscan/runtime/test fixes from `404`, `411`, `417`,
   and `420`
3. full storage-format guardrail coverage from `403`, `409`, and `420`

But the review flow still lacked one small packet with the exact green output.

## Captured Artifacts

Fresh local logs captured against code checkpoint `ea9ec05`:

1. outside-sandbox `cargo test`
   - local log: `tmp/landing-proof-ea9ec05-cargo-test-escalated.log`
2. outside-sandbox `bash scripts/run_pgrx_pg17_test.sh`
   - local log: `tmp/landing-proof-ea9ec05-pgrx-wrapper.log`
3. inside-sandbox `cargo test`
   - local log: `tmp/landing-proof-ea9ec05-cargo-test.log`
   - used only to show the read-only `.pgrx` failure mode

## Evidence

### 1. Outside-sandbox `cargo test` names the required tests green

Excerpt from `tmp/landing-proof-ea9ec05-cargo-test-escalated.log`:

```text
test am::scan::tests::grouped_binary_traversal_score_gate_requires_pq_fastscan_storage ... ok
test tests::pg_test_pq_fastscan_default_rerank_matches_explicit_heap ... ok
test tests::pg_test_pq_fastscan_default_source_rerank_emits_heap_scores ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_binary_default ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_binary_fallback ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_heap_override ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_env_override ... ok
test tests::pg_test_tqhnsw_pq_fastscan_reloption_round_trip ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reindex_restores_runtime ... ok
test tests::pg_test_tqhnsw_turboquant_reloption_round_trip ... ok
test tests::pg_test_tqhnsw_storage_format_switch_rejects_insert_until_reindex - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_rejects_vacuum_until_reindex - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reverse_rejects_insert - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reverse_rejects_vacuum - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reverse_requires_reindex - should panic ... ok
test result: ok. 461 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 48.41s
```

That closes the original `419` blocker asking for a concrete local run naming
the formerly-blocked tests green.

### 2. Outside-sandbox wrapper pass confirms the pg17 lane too

Excerpt from `tmp/landing-proof-ea9ec05-pgrx-wrapper.log`:

```text
test tests::pg_test_pq_fastscan_default_rerank_matches_explicit_heap ... ok
test tests::pg_test_pq_fastscan_default_source_rerank_emits_heap_scores ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_binary_default ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_binary_fallback ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_heap_override ... ok
test tests::pg_test_pq_fastscan_index_runtime_settings_report_env_override ... ok
test tests::pg_test_tqhnsw_pq_fastscan_reloption_round_trip ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reindex_restores_runtime ... ok
test tests::pg_test_tqhnsw_turboquant_reloption_round_trip ... ok
test tests::pg_test_tqhnsw_storage_format_switch_rejects_insert_until_reindex - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reverse_rejects_insert - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_rejects_vacuum_until_reindex - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reverse_rejects_vacuum - should panic ... ok
test tests::pg_test_tqhnsw_storage_format_switch_reverse_requires_reindex - should panic ... ok
test result: ok. 461 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 47.69s
```

So the local wrapper is green when it can use the normal writable `.pgrx`
install destination.

### 3. The sandbox failure is environmental, not a code regression

Excerpt from `tmp/landing-proof-ea9ec05-cargo-test.log`:

```text
failed writing `/home/peter/dev/tqvector/tqvector.control` to `/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector.control`
Read-only file system (os error 30)
```

The following pg tests then cascade on the poisoned mutex. That reproduces the
same environment restriction already seen on the wrapper lane, and explains why
inside-sandbox failures are not evidence of a code defect on this checkpoint.

## Outcome

This packet closes the remaining local-proof gap:

1. the repaired `cargo test` lane is green on the final code checkpoint when
   run outside the sandbox
2. the pg17 wrapper lane is green on the same code checkpoint when run outside
   the sandbox
3. the previously-requested reloption, REINDEX, rerank, runtime-settings, and
   binary-gate tests are now named explicitly in captured output
4. the only remaining failure mode in this environment is the sandbox's
   read-only `.pgrx` install destination

## Remaining Work

No new runtime code change is implied by this packet.

The only remaining tasks are post-merge follow-ups:

1. stand up the Linux/x86_64 `cargo test` PR lane
2. investigate shared-table planner cross-choosing between sibling indexes
