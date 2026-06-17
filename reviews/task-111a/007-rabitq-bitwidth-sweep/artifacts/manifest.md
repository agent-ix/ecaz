# Artifact Manifest

- Task: `111a`
- Packet: `reviews/task-111a/007-rabitq-bitwidth-sweep`
- Head SHA: `5cdddc04f`
- Code commits under measurement:
  - `69cf0030d` - packed dense EXPLAIN counters and single-segment borrow fast path
  - `6594ccc8e` - packed dense vacuum/rewrite coverage and PG18 span fixture
- Timestamp: `2026-06-17T09:25:08-07:00`
- Database: `task111a_dense_bench`
- PG socket/port: `/home/peter/.pgrx`, `28818`
- Surface isolation: isolated one-index-per-table prefixes, `task111a_007_*`
- Fixture scales:
  - 50k: `data/task111a_real50k/ec_real_50k_corpus.tsv`,
    `data/task111a_real50k/ec_real_50k_queries.tsv`,
    `data/task111a_real50k/ec_real_50k_manifest.json`
  - 100k: `data/task106_full_sweep_100k/ec_real_100k_corpus.tsv`,
    `data/task106_full_sweep_100k/ec_real_100k_queries.tsv`,
    `data/task106_full_sweep_100k/ec_real_100k_manifest.json`
- Runner: `ecaz bench suite`
- Suite config: `artifacts/task111a-rabitq-bitwidth-suite.json`
- Suite result stream: `artifacts/suite/results.jsonl`
- Suite report result stream: `artifacts/suite/results-report.jsonl`
- Suite manifest: `artifacts/suite/suite-manifest.json`
- Summary: `artifacts/summary.md`

## Scope

This packet covers RaBitQ `quant_bits` values 2, 4, and 8. Packet
`reviews/task-111a/004-all-dense-options-benchmark` covers TurboQuant and rb1.
Together the benchmark evidence covers TQ plus RaBitQ `{1,2,4,8}` across:

- row postings
- original dense postings
- original dense with coalescing
- original dense with typed views
- page-spanning packed dense
- page-spanning packed dense with typed views

## Commands

Build and install:

```text
cargo build --release -p ecaz-cli --bin ecaz
target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/install-ecaz-pg18-release.log
```

Audit and dry-run:

```text
target/release/ecaz --log-file reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite-audit.log bench suite audit --config reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/task111a-rabitq-bitwidth-suite.json --database task111a_dense_bench --host /home/peter/.pgrx --port 28818
target/release/ecaz --log-file reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite-dry-run.log bench suite run --config reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/task111a-rabitq-bitwidth-suite.json --dry-run --database task111a_dense_bench --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite-dry-run-manifest.json
```

Suite run:

```text
target/release/ecaz --log-file reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite-run.log bench suite run --config reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/task111a-rabitq-bitwidth-suite.json --database task111a_dense_bench --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite/suite-manifest.json --results-output reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite/results.jsonl
```

Post-run checks:

```text
target/release/ecaz --log-file reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite-status.log bench suite status --manifest reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite/suite-manifest.json
target/release/ecaz --log-file reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite-report.log bench suite report --manifest reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite/suite-manifest.json --results-output reviews/task-111a/007-rabitq-bitwidth-sweep/artifacts/suite/results-report.jsonl
```

## Artifact Notes

- `artifacts/install-ecaz-pg18-release.log` records the PG18 install of the
  release backend.
- `artifacts/suite-audit.log` records `audit passed: 180 steps`.
- `artifacts/suite-status.log` records `completed=180 failed=0 skipped=0
  dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-run.log` is the compact suite-level run log.
- `artifacts/suite/results.jsonl` and `artifacts/suite/results-report.jsonl`
  contain extracted recall, latency, storage, planner, build, and block-kernel
  counter rows.
- `artifacts/suite/suite-manifest.json` records every step command and
  generated artifact path.
- The truth-cache files under `artifacts/suite/truth-*.json` are regenerable and
  intentionally not committed.
- Most per-step logs are intentionally not committed to avoid packet log
  exhaust. The structured result stream is the durable source for the recall,
  latency, storage, and batch-counter values cited in `request.md` and
  `summary.md`.
- Selected EXPLAIN logs and SQL files are committed because `summary.md` cites
  the page-spanning packed counters:
  - `artifacts/suite/explain-100k-rb2-dense-b.log`
  - `artifacts/suite/explain-100k-rb2-dense-b-typed.log`
  - `artifacts/suite/explain-100k-rb4-dense-b.log`
  - `artifacts/suite/explain-100k-rb4-dense-b-typed.log`
  - `artifacts/suite/explain-100k-rb8-dense-b.log`
  - `artifacts/suite/explain-100k-rb8-dense-b-typed.log`

## Key Result Lines

- Suite health: 180 completed, 0 failed, 0 skipped, 0 stale, 0 missing artifacts.
- nprobe=32 recall is identical across surfaces at each bit-width/scale:
  - rb2: 50k 0.8840, 100k 0.8670
  - rb4: 50k 0.9410, 100k 0.9290
  - rb8: 50k 0.9460, 100k 0.9390
- nprobe=32 latency winners:
  - rb2 50k: dense-typed 58.0 ms vs row 77.1 ms
  - rb2 100k: dense-old 124.5 ms vs row 140.7 ms
  - rb4 50k: dense-old 15.3 ms vs row 18.0 ms
  - rb4 100k: dense-typed 32.4 ms vs row 38.6 ms
  - rb8 50k: dense-old 13.4 ms vs row 21.5 ms
  - rb8 100k: dense-old 32.3 ms vs row 43.7 ms
- EC IVF index size:
  - rb2 100k: row 49.6 MiB, dense-old 41.8 MiB, dense-b 49.4 MiB
  - rb4 100k: row 87.6 MiB, dense-old 78.9 MiB, dense-b 98.1 MiB
  - rb8 100k: row 196.0 MiB, dense-old 157.0 MiB, dense-b 171.4 MiB
- rb2 nprobe=32 batch counters:
  - 100k row: 20,380 flushes, 20,363 width >=32
  - 100k dense-old: 275,136 flushes, 272,838 width 16-31, 0 width >=32
  - 100k dense-a: 21,778 flushes, 21,443 width >=32
  - 100k dense-b: 164,473 flushes, 161,307 width >=32
- 100k page-spanning packed EXPLAIN counters:
  - rb2 dense-b-typed: 1,314 groups assembled, 2,628 segments assembled,
    16,622,496 copied bytes, 22 borrowed groups
  - rb4 dense-b-typed: 1,323 groups assembled, 5,264 segments assembled,
    32,848,140 copied bytes, 13 borrowed groups
  - rb8 dense-b-typed: 1,327 groups assembled, 9,222 segments assembled,
    65,243,556 copied bytes, 9 borrowed groups
