# Artifact Manifest

- head SHA: `ee876d09089dbc67a2faa824f7545e92227c3a8d`
- task bucket: `reviews/task-51`
- packet path: `reviews/task-51/008-ivf-rabitq-sidecar-rerank-bench`
- slice: suite-driven IVF/RaBitQ sidecar upper-bound harness and local 50k preflight
- isolated one-index-per-table surface: yes, benchmark prefix `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- lane / fixture / storage / rerank:
  - local PG18
  - `ec_real_50k`
  - `ec_ivf` with `storage_format=rabitq`, `quant_bits=1`, `nlists=128`, `rerank=off`
  - sidecar variants `f32`, `f16`, `rabitq8`
- AWS: not used
- vchord / pgvectorscale: not used

## Scope Clarification From Reviewer Feedback

This packet measures a free-I/O upper bound. `ecaz bench sidecar-rerank` loads
the sidecar representation into the CLI process before timed reranking, so the
reported `sidecar_p50` is resident scoring time, not product sidecar storage
latency. The benchmark does not model random sidecar table lookup,
TID-sorted sidecar fetch, prefetch behavior, or an in-index read path. A
real-I/O sidecar microbenchmark remains open before Exp 7 can support a product
decision.

The `candidate_sql_p50` values are IVF `rerank=off` top-50 candidate-id
queries returned to the client, not full posting-frontier materialization.

## Validation Artifacts

- `cargo-check-ecaz-cli.log`
  - command: `cargo check -p ecaz-cli`
  - result: passed
  - key line: `Finished dev profile [unoptimized + debuginfo]`
- `cargo-test-sidecar-suite.log`
  - command: `cargo test -p ecaz-cli expands_sidecar_rerank_with_variants`
  - result: passed
  - key line: `test result: ok. 1 passed; 0 failed`
- `diff-check.log`
  - command: `git diff --check`
  - result: passed, no output

## Benchmark Artifacts

Benchmark packet: `benchmarks/task51-local-ivf-rabitq-sidecar`

- `benchmarks/task51-local-ivf-rabitq-sidecar/suite.json`
- `benchmarks/task51-local-ivf-rabitq-sidecar/manifest.md`
- `benchmarks/task51-local-ivf-rabitq-sidecar/request.md`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-manifest.json`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/results.jsonl`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-run.log`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-status.log`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-report.log`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-audit.log`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/load-50k-rabitq1-n128-rerank-off.log`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/sidecar-50k-rabitq1-n128-k50.log`
- `benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/storage-50k-rabitq1-n128-rerank-off.log`

Key benchmark lines cited by `request.md`:

```text
completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
nprobe=64 f32 recall@10=0.9940 candidate_sql_p50=107.509 ms sidecar_p50=1.552 ms total_bound_p50=109.450 ms sidecar_size=292.97 MiB
nprobe=64 f16 recall@10=0.9940 candidate_sql_p50=107.509 ms sidecar_p50=2.561 ms total_bound_p50=110.184 ms sidecar_size=146.48 MiB
nprobe=64 rabitq8 recall@10=0.9505 candidate_sql_p50=107.509 ms sidecar_p50=1.120 ms total_bound_p50=108.628 ms sidecar_size=73.81 MiB
```
