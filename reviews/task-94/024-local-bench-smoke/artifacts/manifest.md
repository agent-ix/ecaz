# Task 94 Packet 024 Artifact Manifest

- head SHA: `187be1af1dea26f1576245bd2f2f7f4d0c247b9f`
- task bucket: `reviews/task-94/024-local-bench-smoke/`
- lane / fixture / storage: LUT lane / local PG18 / `task94_local_pqfs10k_roff` / IVF `storage_format=pq_fastscan`, `pq_group_size=8`, `rerank=off`
- timestamp: `2026-06-09T14:14:16-07:00`
- host class: local x86_64 / AVX2; no AWS and no GitHub CI were run for this packet
- local PG18: `/home/peter/.pgrx`, port `28818`, database `postgres`
- installed backend SHA-256 after code fix: `c5561017c3b7348416f7734811cf526098ac4549a89cf3f959910b0f57324793`

## Code Checkpoint

`187be1af1` (`Batch IVF PqFastScan scratch scoring`)

- Adds `IvfQuantizer::score_grouped_pq_batch_from_payloads(...)`.
- Enables `StorageFormat::PqFastScan` for the existing IVF scratch SoA batch decode path.
- Drains scratch postings through `score_grouped_pq_batch_for(surface=ivf, quant=grouped_pq)` and records scalar tails separately.

## Focused Local Tests

### `cargo-test-ivf-pqfastscan-payload-batch.log`

- command: `script -q -c "cargo test -p ecaz --lib pq_fastscan_payload_batch_scores_match_scalar_and_records_counters --no-default-features --features pg18" reviews/task-94/024-local-bench-smoke/artifacts/cargo-test-ivf-pqfastscan-payload-batch.log`
- result: passed, `1 passed; 0 failed`
- key coverage: 39 PqFastScan payloads score bit-exact with scalar and record `(surface=ivf, quant=grouped_pq)` kernel and scalar-tail counters.

### `cargo-test-ivf-scratch-pqfastscan-gate.log`

- command: `script -q -c "cargo test -p ecaz --lib scratch_soa_batch_decode_gate_admits --no-default-features --features pg18" reviews/task-94/024-local-bench-smoke/artifacts/cargo-test-ivf-scratch-pqfastscan-gate.log`
- result: passed, `1 passed; 0 failed`
- key coverage: `use_scratch_soa_batch_decode_for_format(true, PqFastScan, 4)` is admitted.

### `cargo-fmt-check.log`

- command: `script -q -c "cargo fmt --check" reviews/task-94/024-local-bench-smoke/artifacts/cargo-fmt-check.log`
- result: passed
- note: stable-rust warnings for unstable rustfmt config keys are pre-existing repo behavior.

## Local Fixture Setup

### `install-ecaz-pg18-after-ivf-pqfastscan-batch.log`

- command: `target/debug/ecaz --log-file reviews/task-94/024-local-bench-smoke/artifacts/install-ecaz-pg18-after-ivf-pqfastscan-batch.log dev install ecaz-pg-test --pg 18 --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- result: installed updated `ecaz.so`
- installed backend SHA-256: `c5561017c3b7348416f7734811cf526098ac4549a89cf3f959910b0f57324793`

### `create-local-ivf-pqfastscan-rerank-off.sql`

- command: `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file reviews/task-94/024-local-bench-smoke/artifacts/create-local-ivf-pqfastscan-rerank-off.sql --log-output reviews/task-94/024-local-bench-smoke/artifacts/create-local-ivf-pqfastscan-rerank-off.log`
- result: created local 10k copied fixture from `task28_ivf_pqg10k_g8` with `rerank=off`
- fixture reloptions: `nlists=64`, `nprobe=64`, `training_sample_rows=2000`, `storage_format=pq_fastscan`, `pq_group_size=8`, `rerank=off`

## Suite Evidence

### `task94-local-ivf-pqfastscan-suite.json`

- command config for the local suite; four steps: recall batch-off, recall batch-on, latency batch-off, latency batch-on.
- batch-on steps use `ivf_scratch_soa_batch_decode=true`.

### `suite-audit-cli.log`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/024-local-bench-smoke/artifacts/suite-audit-cli.log bench suite audit --config reviews/task-94/024-local-bench-smoke/artifacts/task94-local-ivf-pqfastscan-suite.json`
- result: `audit passed: 4 steps`

### `suite-run-cli.log`, `suite-manifest.json`, `results.jsonl`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/024-local-bench-smoke/artifacts/suite-run-cli.log bench suite run --config reviews/task-94/024-local-bench-smoke/artifacts/task94-local-ivf-pqfastscan-suite.json --artifact-dir reviews/task-94/024-local-bench-smoke/artifacts --manifest-output reviews/task-94/024-local-bench-smoke/artifacts/suite-manifest.json --results-output reviews/task-94/024-local-bench-smoke/artifacts/results.jsonl`
- result: completed 4, failed 0
- recall equality:
  - batch off nprobe 32: `recall@k=0.4275`, `ndcg@k=0.9022`
  - batch on nprobe 32: `recall@k=0.4275`, `ndcg@k=0.9022`
  - batch off nprobe 64: `recall@k=0.4325`, `ndcg@k=0.9038`
  - batch on nprobe 64: `recall@k=0.4325`, `ndcg@k=0.9038`
- latency:
  - batch off nprobe 32: `p50=2.88 ms`, `p95=3.17 ms`, `p99=3.41 ms`
  - batch on nprobe 32: `p50=2.83 ms`, `p95=3.16 ms`, `p99=3.50 ms`
  - batch off nprobe 64: `p50=4.56 ms`, `p95=5.04 ms`, `p99=5.62 ms`
  - batch on nprobe 64: `p50=4.55 ms`, `p95=5.02 ms`, `p99=5.81 ms`
- direct block kernel rows from `results.jsonl`:
  - nprobe 32 AVX2: `surface=ivf`, `quant=grouped_pq`, `isa=avx2`, `kernel_candidates=579392`, `scalar_candidates=0`, `kernel_elapsed_ms=105.397059`
  - nprobe 32 scalar tail: `surface=ivf`, `quant=grouped_pq`, `isa=scalar`, `kernel_candidates=0`, `scalar_candidates=1775`, `scalar_elapsed_ms=0.379304`
  - nprobe 64 AVX2: `surface=ivf`, `quant=grouped_pq`, `isa=avx2`, `kernel_candidates=1198080`, `scalar_candidates=0`, `kernel_elapsed_ms=218.736765`
  - nprobe 64 scalar tail: `surface=ivf`, `quant=grouped_pq`, `isa=scalar`, `kernel_candidates=0`, `scalar_candidates=1920`, `scalar_elapsed_ms=0.403762`

### `suite-report-cli.log`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/024-local-bench-smoke/artifacts/suite-report-cli.log bench suite report --manifest reviews/task-94/024-local-bench-smoke/artifacts/suite-manifest.json`
- result: report generated from the suite manifest; parsed result rows match `results.jsonl`.

## Notes

- Earlier direct local latency runs in this packet (`latency-ivf-pqfastscan-10k*.log`) intentionally remain as negative evidence: existing rerank-on and default scratch-off paths emitted zero block-kernel counters. That exposed the production IVF call-site gap fixed by `187be1af1`.
- This packet does not claim Graviton 4, NEON, or SVE2 evidence. Those remain AWS-final evidence after user approval.
