# Manifest: Task 97 Local Closeout Bench Matrix

- Head SHA under test: `4804791fda0f6d62c1c520e349bd71798b436247`
- Task bucket: `reviews/task-97`
- Packet path: `reviews/task-97/026-local-closeout-bench-matrix`
- Timestamp: `2026-06-10T15:39:05Z`
- Lane: coder-1 / QJL32 TurboQuant
- Fixture: generated local deterministic corpora, `n=512` and `n=4096`, `dim=1024`, `queries=64`, `bits=4`
- Storage format: `turboquant`
- Rerank mode: forced index path, `k=10`
- Surface isolation: one prefix per AM/fixture (`task97_qjl32_{512,4096}_{ivf,spire,hnsw}`)
- Host class: local Intel AVX2
- PostgreSQL: PG18, host `/home/peter/.pgrx`, port `28818`
- CI/AWS: not run

## Commands

Install under test:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/026-local-closeout-bench-matrix/artifacts/local-ecaz-pg18-install.log dev install ecaz-pg-test --pg 18
```

Suite audit:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/026-local-closeout-bench-matrix/artifacts/suite-audit-after-path-fix-cli.log bench suite audit --config reviews/task-97/026-local-closeout-bench-matrix/artifacts/task97-local-closeout-qjl32-suite.json
```

Suite run:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/026-local-closeout-bench-matrix/artifacts/suite-run-cli.log bench suite run --config reviews/task-97/026-local-closeout-bench-matrix/artifacts/task97-local-closeout-qjl32-suite.json --artifact-dir reviews/task-97/026-local-closeout-bench-matrix/artifacts --manifest-output reviews/task-97/026-local-closeout-bench-matrix/artifacts/suite-manifest.json --results-output reviews/task-97/026-local-closeout-bench-matrix/artifacts/results.jsonl
```

Suite status:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench suite status --manifest reviews/task-97/026-local-closeout-bench-matrix/artifacts/suite-manifest.json
```

Suite report:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/026-local-closeout-bench-matrix/artifacts/suite-report-cli.log bench suite report --manifest reviews/task-97/026-local-closeout-bench-matrix/artifacts/suite-manifest.json
```

## Artifacts

- `task97-local-closeout-qjl32-suite.json`: checked-in suite config.
- `suite-manifest.json`: suite run manifest.
- `results.jsonl`: structured suite results.
- `local-closeout-summary.md`: curated summary of recall, latency, and counter rows.
- `local-ecaz-pg18-install.log`: local PG18 install log. Installed `ecaz.so` sha256 was `350cf9c3d2bdef5e0604fa7504dc04c01f55a8da24480b1373028f104c108e74`.
- `suite-audit-after-path-fix-cli.log`: final audit log.
- `suite-run-cli.log`: suite run log.
- `suite-status-cli.log`: status log.
- `suite-report-cli.log`: report log.
- `recall-*.log`: per-surface recall logs.
- `latency-*.log`: per-surface latency logs with `[block-kernel-counters]` rows where emitted.
- `load-*.log`: per-surface corpus/index load logs.
- `generate-*.log` and `task97_qjl32_*_{corpus,queries}.tsv`: deterministic local fixture generation outputs.

## Key Result Lines

Suite status:

```text
[suite:task97-local-closeout-qjl32-suite] completed=34 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall parity:

```text
All 14 batch-on recall cells exactly matched their batch-off recall cells.
```

Representative local AVX2 end-to-end p50 speedups:

```text
IVF 4096 nprobe=8/16/32: 1.22x, 1.44x, 1.71x
SPIRE 512 nprobe=8/16: 1.03x, 1.03x
SPIRE 4096 nprobe=8/16/32: 1.04x, 1.02x, 1.00x
HNSW 4096 ef_search=32/64: 1.10x, 1.04x
```

SPIRE direct scoring counter speedups:

```text
512 nprobe=8: 2.51x
512 nprobe=16: 2.48x
4096 nprobe=8: 2.97x
4096 nprobe=16: 2.83x
4096 nprobe=32: 2.81x
```

Known local caveats:

```text
HNSW 512 ef_search=64 regressed end-to-end p50 (0.96x).
SPIRE 4096 nprobe=32 is flat/slightly negative end-to-end despite direct scoring counters being faster.
ARM/G4 execution remains approval-gated and was not run in this packet.
```
