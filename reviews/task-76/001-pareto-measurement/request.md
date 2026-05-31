# Review Request: Task 76 Intel-Local SPIRE Pareto Measurement

Task 76 benchmark evidence is ready for review.

## Scope

- Added the Task 76 suite config at `benchmarks/task76-intel-local-spire-pareto/suite.json`.
- Ran the suite locally on the Intel desktop against PG18.
- Captured all raw logs and parsed outputs under `benchmarks/task76-intel-local-spire-pareto/artifacts/`.
- Recorded the durable result in `benchmarks/task76-intel-local-spire-pareto/manifest.md`.

## Result

The suite completed 33/33 steps with no failures.

Recommended decision: do not change SPIRE defaults from this local Intel packet.

Key reason: 100k high-recall SPIRE remains far slower than IVF at comparable recall:

- SPIRE tg96/nprobe96: recall@10 0.9975, p50 146.693 ms, p95 175.128 ms.
- IVF nprobe96: recall@10 0.9980, p50 37.7 ms, p95 46.5 ms.

The 100k SPIRE candidate surface plateaus after nprobe64:

- nprobe64: recall@10 0.9825, leaf routes 3,556, candidates 2,784,952.
- nprobe96: recall@10 0.9975, leaf routes 3,556, candidates 2,784,952.
- nprobe128: recall@10 1.0000, leaf routes 3,556, candidates 2,784,952.

The local 1M TSV fixture was unavailable, so this packet explicitly does not promote a 1M-informed default.

## Evidence

- Manifest: `benchmarks/task76-intel-local-spire-pareto/manifest.md`
- Summary: `benchmarks/task76-intel-local-spire-pareto/artifacts/summary.md`
- Suite report: `benchmarks/task76-intel-local-spire-pareto/artifacts/suite-report.md`
- Normalized results: `benchmarks/task76-intel-local-spire-pareto/artifacts/normalized-results.jsonl`
- Raw results: `benchmarks/task76-intel-local-spire-pareto/artifacts/results.jsonl`
- Full run log: `benchmarks/task76-intel-local-spire-pareto/artifacts/suite-run.log`
- Clippy log: `reviews/task-76/001-pareto-measurement/artifacts/clippy-pg18.log`
- AWS status after local work: `reviews/task-76/001-pareto-measurement/artifacts/aws-status-1m-after-local-work.log`, `reviews/task-76/001-pareto-measurement/artifacts/aws-status-10k-medium-after-local-work.log`

## Validation

```text
target/debug/ecaz bench suite audit --config benchmarks/task76-intel-local-spire-pareto/suite.json --database task76_spire_pareto --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-audit.log
target/debug/ecaz bench suite run --dry-run --config benchmarks/task76-intel-local-spire-pareto/suite.json --database task76_spire_pareto --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task76-intel-local-spire-pareto/artifacts/suite-dry-run-manifest.json --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-dry-run.log
target/debug/ecaz bench suite run --config benchmarks/task76-intel-local-spire-pareto/suite.json --database task76_spire_pareto --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task76-intel-local-spire-pareto/artifacts/suite-manifest.json --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-run.log
target/debug/ecaz bench suite report --manifest benchmarks/task76-intel-local-spire-pareto/artifacts/suite-manifest.json --results-output benchmarks/task76-intel-local-spire-pareto/artifacts/normalized-results.jsonl --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-report.md
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
script -q -c "target/debug/ecaz cloud status --profile 1m" reviews/task-76/001-pareto-measurement/artifacts/aws-status-1m-after-local-work.log
script -q -c "target/debug/ecaz cloud status --profile 10k-medium" reviews/task-76/001-pareto-measurement/artifacts/aws-status-10k-medium-after-local-work.log
```
