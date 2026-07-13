# Artifact manifest

- Baseline source head: `8d0c7d6bb463513ba4f7c65316c723ca0f551e1c`
- Candidate source head: `9387f72b3209c751ba561f5f976f57954bd30b90`
- Prompt-cancellation implementation: `a94e5e9be83b523a907ca3590dc62cafeca3cb3a`
- Release runner git commit: `24ec63788cc5c8ea361eb8c0ceff6c5a966e5323`
- Release runner SHA-256: `ad52902025faeed5c79629dabc23b8dd3e5a48d94d06e92f44e8af3259959320`
- Feature set for both arms: `pg18`
- Task bucket / packet: `reviews/task-179/052-prompt-cancellation-ab`
- Immutable baseline packet: `reviews/task-179/050-direct-graph-reader-ab/artifacts/candidate`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local Intel PG18 physical DistANN A/B
- Host: x86_64 Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM,
  Linux 6.18.33.2 WSL2, 1 TiB ext4 virtual disk
- PostgreSQL: 18.3, release extension builds, three loopback instances per
  scale with `shared_preload_libraries=ecaz`
- Baseline run: `2026-07-13T01:33:59-07:00` through
  `2026-07-13T02:24:52-07:00`
- Candidate run: `2026-07-13T03:01:51-07:00` through
  `2026-07-13T03:52:03-07:00`
- Fixture: three physical PostgreSQL owner instances plus a same-data
  single-index control in every scale
- Storage format: WAL-logged distributed-control physical graph, row, and
  directory relations; graph degree 32; head index cap 4096
- Rerank mode: exact frozen-row materialization from the physical owner
- Isolation surface: isolated source/control tables and one generation per
  physical owner; no shared-table benchmark surface

The baseline is packet 050's immutable direct-reader candidate. Relative to
that source, the only production-code difference in the installed candidate
is implementation commit `a94e5e9be`; intervening commits add review packets.
The unchanged runner records its own earlier build commit separately from the
candidate extension source head.

## Suite configuration

| Arm | Config path / SHA-256 | Ports |
| --- | --- | --- |
| Baseline | `reviews/task-179/050-direct-graph-reader-ab/artifacts/candidate-suite.json` / `55535fe4fe82996af58eb4fd0fa44249d203481e525946f35feed13dd2dd3b9c` | 40460-40462 |
| Prompt poll | `artifacts/candidate-suite.json` / `77f7263969f4fe05eb85e2cdfba9d21d647000e31f903d78c0d0320950711297` | 40470-40472 |

Both configs cover 10k/50k/100k with 20 recall queries, top-k 10, 10 untimed
same-connection warmups, 50 latency iterations, concurrency 1, three owners,
degree 32, head cap 4096, and a same-data single-index control. Both require
`seed_strategy=persisted_head`.

## Corpus provenance

Corpus and query TSV files are deliberately not committed. Both arms read the
same staged files under `/home/peter/dev/ecaz/data/staged-current`.

| Prefix | Rows | Dimension | Corpus SHA-256 | Query SHA-256 |
| --- | ---: | ---: | --- | --- |
| `ec_real_10k` | 10,000 | 1536 | `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` |
| `ec_real_50k` | 50,000 | 1536 | `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| `ec_real_100k` | 100,000 | 1536 | `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

The hashes were recomputed after the candidate suite and match packet 050's
recorded baseline hashes.

## Commands

Candidate extension installation:

```text
cargo pgrx install --release \
  -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features -F pg18
```

Candidate suite:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/052-prompt-cancellation-ab/artifacts/candidate-suite.json \
  --continue-on-error \
  --log-file reviews/task-179/052-prompt-cancellation-ab/artifacts/candidate-suite-run.log
```

Post-run validation:

```text
target/release/ecaz bench suite status \
  --manifest reviews/task-179/052-prompt-cancellation-ab/artifacts/candidate/suite-manifest.json \
  --log-file reviews/task-179/052-prompt-cancellation-ab/artifacts/candidate-status.log

target/release/ecaz bench suite audit \
  --config reviews/task-179/052-prompt-cancellation-ab/artifacts/candidate-suite.json \
  --log-file reviews/task-179/052-prompt-cancellation-ab/artifacts/candidate-audit-final.log
```

## Artifact index

### Baseline provenance

- `reviews/task-179/050-direct-graph-reader-ab/artifacts/candidate/suite-manifest.json`
  records exact expanded commands, runner SHA, duration, exit status, expected
  artifacts, and all 12 passing baseline thresholds.
- `reviews/task-179/050-direct-graph-reader-ab/artifacts/candidate/results.jsonl`
  is the immutable baseline source for every comparison value.

### Candidate arm

- `candidate-suite.json`: canonical matrix config.
- `candidate/suite-manifest.json`: exact expanded commands, runner SHA,
  durations, exit status, expected artifacts, and 12/12 passing thresholds.
- `candidate/results.jsonl`: normalized topology, recall, latency, storage,
  engagement, and threshold source rows.
- `candidate-install.log`: exact release extension build and installation.
- `candidate-suite-run.log`: suite driver output.
- `candidate-status.log`: post-run 3/3 completion and missing/stale check.
- `candidate-audit-final.log`: post-run config audit.
- `comparison.md`: derived A/B tables; every number traces to the two cited
  JSONL files.

### Per-scale candidate evidence

For each of `10k`, `50k`, and `100k`:

- `distann-local-multinode.log` records Ready/Published topology, physical
  serving, remote-owner verification, build, recall, fully warmed latency,
  storage, engagement, and topology-gate lines.
- `distann-multinode-summary.log` is the decision-grade physical summary.
- `physical-recall.log` and `physical-latency.log` contain the physical arm's
  detailed recall and latency runs.
- `single-recall.log` and `single-latency.log` contain the same-data control.

PostgreSQL server logs, run directories, truth caches, corpus/query TSVs,
initial pre-run audit logs, and polling output are not committed.

## Key cited results

```text
baseline status:  completed=3 failed=0 missing_artifacts=0 stale=0
candidate status: completed=3 failed=0 missing_artifacts=0 stale=0

10k:  recall 1.0000 -> 1.0000; mean/p95 43.20/55.50 -> 43.50/55.70 ms
50k:  recall 0.9800 -> 0.9800; mean/p95 54.10/68.30 -> 54.50/67.90 ms
100k: recall 0.9500 -> 0.9500; mean/p95 51.90/70.00 -> 49.50/67.40 ms

all six topology gates: pass=true, owners=3, remote_verified=2
all six physical latency rows: cache=warm, warmup_iterations=10, count=50, concurrency=1
all twenty-four configured thresholds across both arms: pass
```
