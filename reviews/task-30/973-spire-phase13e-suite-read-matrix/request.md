# Review Request: SPIRE Phase 13e Suite Read Matrix

## Summary

This slice closes the remaining Phase 13e suite-runner gap for read matrices. AWS correctness, representative, and stress read tiers now run through `ecaz bench suite`, with production-read-profile `spire-pipeline` rows included directly in the tier configs.

Changes:

- `ecaz bench suite` now uses `artifact_dir` to provide default packet-local `--log-output` paths for recall, latency, spire-pipeline, and sidecar-rerank steps when a step omits an explicit log path.
- `scripts/spire-aws/bench.sh` injects the operator-supplied artifact directory into the selected tier suite before invoking `ecaz bench suite run`.
- `scripts/spire-aws/suite-{correctness,representative,stress}.json` include `spire-pipeline --include-production-read-profile` rows.
- Representative suite covers the remote tuple transport sweep values `auto`, `json_tuple_payload_v1`, and `pg_binary_attr_v1` as suite steps.

## Key Evidence

- `artifacts/suite-dry-run-correctness.log`: expands 3 suite steps with packet-local logs and production read profile.
- `artifacts/suite-dry-run-representative.log`: expands 11 suite steps, including k=10/k=100 production read profiles and all three transport sweep rows.
- `artifacts/suite-dry-run-stress.log`: expands stress recall, latency, and production read profile.
- `artifacts/suite-audit-correctness.log`, `artifacts/suite-audit-representative.log`, `artifacts/suite-audit-stress.log`: all pass.

## Validation

- `cargo test -p ecaz-cli suite`
- `cargo check -p ecaz-cli`
- `cargo build -p ecaz-cli`
- `cargo fmt --all -- --check`
- `bash -n scripts/spire-aws/bench.sh`
- dry-run and audit of generated correctness, representative, and stress suite configs

## Remaining Phase 13e Work

- Run the AWS correctness tier and capture production profile fields from real remotes.
- Run representative AWS latency/recall and record p50/p95/p99.
- Evaluate connection pooling only after AWS profile evidence says setup cost meets the gate.
