# Review Request: SPIRE k=100 Local Payload-Cap Gate

## Summary

This packet fixes a local product-readiness blocker exposed by the AWS representative run: production remote reads at `top_k=100` could fail before heap merge because the default `ec_spire.max_remote_payload_rows_per_batch` was 64.

Code commit: `553cd24ec3523216d68dc0a9311d4cd5fbf99d38`

Changes:

- Raise the default remote payload batch row cap from 64 to 128, so the required Phase 13 representative `k=100` lane works without per-session tuning.
- Update the GUC help text to explain the Phase 13 `k=100` default.
- Add a focused assertion that the default cap admits a `k=100` remote payload batch.
- Parameterize `scripts/run_spire_phase13e_static_remote_placement_pg18.sh` with `--fixture-rows`, `--bench-top-k`, `--bench-queries-limit`, and `--bench-sweep`, so the AWS failure shape is repeatable locally.

## Evidence

Before fix, the enlarged local PG18 static remote-placement gate reproduced the AWS failure locally:

- fixture rows: `480`
- bench top-k: `100`
- failure: `ERROR: ec_spire production executor cannot merge remote heap candidates while node_id 2 is in state CandidateReceiveFailed with status remote_candidate_receive_failed`
- artifact: `artifacts/before-fix/bench-suite/suite-run.log`

After fix, the same local PG18 gate passed:

- smoke profile: `ready|3|3|3|3|6`
- suite: `bench_suite_summary=passed|...`
- production profile: `status ready`, `result_source remote_heap_candidates`, `dispatch_sum 3`, `candidate_query_sum 3`, `heap_query_sum 3`, `returned_sum 100`
- recall@k: `1.0000`
- final gate: `SPIRE Phase 13e static remote placement PG18 fixture passed`
- artifacts: `artifacts/after-fix/phase13e-static-remote-placement.log`, `artifacts/after-fix/bench-suite/spire-pipeline.log`, `artifacts/after-fix/bench-suite/results.jsonl`

AWS safety check:

- `artifacts/aws-running-check.log` returned no pending/running/stopping instances in `us-west-2`.

## Review Focus

Please review whether 128 is the right production default for `ec_spire.max_remote_payload_rows_per_batch` given the Phase 13 `k=100` requirement, and whether the parameterized local gate is sufficient to prevent another AWS retry before local completion.
