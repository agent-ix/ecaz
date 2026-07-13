# Artifact manifest

- Head SHA for both arms: `24ec63788cc5c8ea361eb8c0ceff6c5a966e5323`
- Benchmark-control implementation: `2bf203e4c7ed8091932bd2c01b134591d21bca73`
- Release runner git commit: `24ec63788cc5c8ea361eb8c0ceff6c5a966e5323`
- Release runner SHA-256: `ad52902025faeed5c79629dabc23b8dd3e5a48d94d06e92f44e8af3259959320`
- Baseline feature set: `pg18 distann-legacy-seed-benchmark`
- Candidate feature set: `pg18`
- Task bucket / packet: `reviews/task-179/048-persisted-head-ab`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local Intel PG18 physical DistANN A/B
- Host: x86_64 Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM,
  Linux 6.18.33.2 WSL2, 1 TiB ext4 virtual disk
- PostgreSQL: 18.3, release extension builds, three loopback instances per
  scale with `shared_preload_libraries=ecaz`
- Baseline run: `2026-07-12T22:58:34-07:00` through
  `2026-07-12T23:52:06-07:00`
- Candidate run: `2026-07-12T23:58:30-07:00` through
  `2026-07-13T00:49:02-07:00`
- Fixture: three physical PostgreSQL owner instances plus a same-data
  single-index control in every scale
- Storage format: WAL-logged distributed-control physical graph, row, and
  directory relations; graph degree 32; head index cap 4096
- Rerank mode: exact frozen-row materialization from the physical owner
- Isolation surface: isolated source/control tables and one generation per
  physical owner; no shared-table benchmark surface

The only A/B code-path difference is scan-time seed acquisition. Both arms
were built from the same head and build the same physical generation plus
persisted head sample. The baseline restores a full graph scan/score on each
owner using the current pooled, bounded, concurrent transport; the candidate
uses normal persisted-head search.

## Suite configuration

| Arm | Config SHA-256 | Strategy required by thresholds | Ports |
| --- | --- | --- | --- |
| Baseline | `c2ac2d1e2cbf525775e1931d4587a5afdb59c6bc5d091f15ddcc5760267dc9f0` | `owner_scan` | 40440-40442 |
| Candidate | `a83751662e07ada66dd4f04ef9fe47813c7f7d049fefa30738421e02f3932889` | `persisted_head` | 40450-40452 |

Both configs cover 10k/50k/100k with 20 recall queries, top-k 10, 10 untimed
same-connection warmups, 50 latency iterations, concurrency 1, and a same-data
single-index control.

## Corpus provenance

Corpus and query TSV files are deliberately not committed. Both arms read the
same staged files under `/home/peter/dev/ecaz/data/staged-current`; hashes were
recomputed after both suites completed.

| Prefix | Rows | Dimension | Corpus SHA-256 | Query SHA-256 |
| --- | ---: | ---: | --- | --- |
| `ec_real_10k` | 10,000 | 1536 | `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` |
| `ec_real_50k` | 50,000 | 1536 | `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| `ec_real_100k` | 100,000 | 1536 | `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

## Commands

Release runner build:

```text
cargo build --release -p ecaz-cli
```

Baseline extension installation:

```text
cargo pgrx install --release \
  -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features -F "pg18 distann-legacy-seed-benchmark"
```

Baseline suite:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/048-persisted-head-ab/artifacts/baseline-suite.json \
  --continue-on-error \
  --log-file reviews/task-179/048-persisted-head-ab/artifacts/baseline-suite-run.log
```

Candidate installation used the same `cargo pgrx install` command with
`-F pg18`. Candidate suite:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/048-persisted-head-ab/artifacts/candidate-suite.json \
  --continue-on-error \
  --log-file reviews/task-179/048-persisted-head-ab/artifacts/candidate-suite-run.log
```

Each final manifest was checked with `ecaz bench suite status`, and each config
was checked again with `ecaz bench suite audit` after its run.

## Artifact index

### Arm-level provenance

- `baseline-suite.json`, `candidate-suite.json`: canonical matrix configs.
- `baseline/suite-manifest.json`, `candidate/suite-manifest.json`: exact
  expanded commands, runner SHA, duration, exit status, expected artifacts,
  and 12/12 passing threshold results per arm.
- `baseline/results.jsonl`, `candidate/results.jsonl`: normalized topology,
  recall, latency, storage, engagement, and threshold source rows.
- `baseline-suite-run.log`, `candidate-suite-run.log`: suite driver output.
- `baseline-status.log`, `candidate-status.log`: post-run 3/3 completion and
  missing/stale checks.
- `baseline-audit-final.log`, `candidate-audit-final.log`: post-run config
  audits.
- `baseline-install.log`, `candidate-install.log`: exact release extension
  builds and installed feature sets.
- `release-cli-build.log`: exact-head release runner build.
- `comparison.md`: derived A/B tables; every number traces to the committed
  JSONL files.

### Per-scale evidence

For each arm and each of `10k`, `50k`, and `100k`:

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

10k:  recall 1.0000 -> 1.0000; mean/p95 283.20/296.20 -> 42.40/55.00 ms
50k:  recall 1.0000 -> 0.9800; mean/p95 1266.60/1301.40 -> 57.10/74.40 ms
100k: recall 0.9950 -> 0.9500; mean/p95 2613.40/2663.00 -> 50.90/69.10 ms

all six topology gates: pass=true, owners=3, remote_verified=2
all six latency rows: cache=warm, warmup_iterations=10, count=50, concurrency=1
all twenty-four configured thresholds: pass
```
