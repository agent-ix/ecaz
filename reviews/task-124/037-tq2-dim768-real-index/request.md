# Review Request: Task 124 Packet 037 - TQ2 768D Real-Index Validation

## Summary

This packet closes the remaining reviewer gap from packet 035: reduced-dimension TQ is no longer micro-benchmark-only.

Code commit `0b3fd57f713297b9c07ceadc364ad6a698021a75` adds a real IVF rerank format:

- `rerank_format=turboquant2_768`
- aliases: `turboquant_2_768`, `tq2_768`
- persisted index-side TQ2 payload over the first 768 dimensions
- full source/query dimension validation remains 1536D for the index
- reduced-dim path is direct prefix-subspace scoring, not centroid-relative residual scoring, so it does not apply full-centroid IP correction to a partial payload

The packet then builds fresh real IVF indexes at 10k / 50k / 100k and runs recall, latency with TQ scorer attribution, and storage through `ecaz bench suite`.

## Result

The reduced-dim scorer speed win is real in the workload, but it is not usable: recall remains broken at 50k/100k.

Scorer-level delta versus packet 036 full-dim TQ2:

| Scale | nprobe | full TQ2 scorer | TQ2 768D scorer | Delta |
| --- | ---: | ---: | ---: | ---: |
| 10k | 32 | 4.027461 ms | 2.036876 ms | -49.4% |
| 10k | 64 | 4.183670 ms | 2.062286 ms | -50.7% |
| 50k | 32 | 4.297787 ms | 2.138413 ms | -50.2% |
| 50k | 64 | 4.354213 ms | 2.140495 ms | -50.8% |
| 100k | 32 | 4.444708 ms | 2.220708 ms | -50.0% |
| 100k | 64 | 4.602538 ms | 2.160324 ms | -53.1% |

Recall and latency for the new real format:

| Scale | nprobe | recall@10 | NDCG@10 | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.9530 | 0.9989 | 0.70 ms | 0.81 ms | 0.99 ms |
| 10k | 64 | 0.9530 | 0.9989 | 1.17 ms | 1.25 ms | 1.32 ms |
| 50k | 32 | 0.7700 | 0.9914 | 2.36 ms | 2.58 ms | 2.67 ms |
| 50k | 64 | 0.7710 | 0.9915 | 4.73 ms | 4.91 ms | 5.07 ms |
| 100k | 32 | 0.7620 | 0.9898 | 4.95 ms | 5.45 ms | 5.81 ms |
| 100k | 64 | 0.7710 | 0.9928 | 9.21 ms | 9.40 ms | 9.70 ms |

The latency logs show the real TQ scorer path:

- SIMD row: `quant=turboquant_qjl isa=neon candidates=9600`
- scalar tail row: `quant=turboquant_qjl isa=scalar candidates=400`

At 100k/nprobe64 the rows are:

- NEON: `elapsed_ms=1.376537`
- scalar tail: `elapsed_ms=0.783787`

## Validation

Static/focused checks run before the packet:

- `cargo fmt`
- `cargo test -p ecaz --lib --no-default-features --features pg18 turboquant2_768 -- --nocapture`
- `cargo check -p ecaz --lib --no-default-features --features pg18`
- `git diff --check`

Build/install and suite:

- `cargo build --release -p ecaz`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `./target/release/ecaz bench suite audit --config reviews/task-124/037-tq2-dim768-real-index/artifacts/task124-tq2-dim768-final15-suite.json --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-audit.log`
- `./target/release/ecaz bench suite run --config reviews/task-124/037-tq2-dim768-real-index/artifacts/task124-tq2-dim768-final15-suite.json --artifact-dir reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-run.log`
- `./target/release/ecaz bench suite status --manifest reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite/suite-manifest.json --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-status.log`
- `./target/release/ecaz bench suite report --manifest reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite/suite-manifest.json --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-report.log`

Status result: `completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/task124-tq2-dim768-final15-suite.json`
- `artifacts/suite-audit.log`
- `artifacts/suite-run.log`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/tq2-dim768-final15-suite/suite-manifest.json`
- `artifacts/tq2-dim768-final15-suite/results.jsonl`
- `artifacts/tq2-dim768-final15-suite/load-*.log`
- `artifacts/tq2-dim768-final15-suite/recall-*.log`
- `artifacts/tq2-dim768-final15-suite/latency-*.log`
- `artifacts/tq2-dim768-final15-suite/storage-*.log`

Regenerable `truth-*-k10.json` files were emitted by the suite and intentionally left uncommitted.

## Review Focus

- Confirm the new `turboquant2_768` format is narrow enough for this task and does not overclaim a general dimension reloption.
- Confirm the reduced-dim direct-prefix scoring contract is acceptable for workload validation, and that it avoids invalid centroid-relative correction.
- Confirm packet 037 satisfies the reviewer requirement: real format, real index, 10k/50k/100k recall + latency + TQ scorer elapsed.
