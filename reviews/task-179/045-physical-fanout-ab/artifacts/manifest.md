# Artifact manifest

- Head SHA when the packet was produced: `3c883fb32de031eb030753dccc0ffec8cd5e4ee7`
- Baseline extension checkout: `c213af204d9979ea64bbb593dc09c0ee3876ff94`
- Candidate fanout implementation: `5a48c7ee93fcc2f1f201c7d2231f13cae467073e`
- Candidate extension checkout installed: `3c883fb32de031eb030753dccc0ffec8cd5e4ee7`
  (no extension-library changes after `5a48c7ee9`; later changes are the CLI
  warmup runner and review packets)
- Release suite runner: `f11ffcafcdf1e80464540bad05d8364559f2598b`
- Task bucket / packet: `reviews/task-179/045-physical-fanout-ab`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local Intel PG18 physical DistANN A/B, x86_64 Intel Core i9-10900K
- Baseline run: `2026-07-12T19:42:28-07:00` through
  `2026-07-12T20:33:24-07:00`
- Candidate run: `2026-07-12T20:39:12-07:00` through
  `2026-07-12T21:29:57-07:00`
- Fixture: three physical PostgreSQL owner instances plus a same-data
  single-index control in every scale
- Storage format: WAL-logged distributed-control physical graph, row, and
  directory relations; graph degree 32; head index cap 4096
- Rerank mode: exact frozen-row materialization from the physical owner
- Isolation surface: isolated source/control tables and one generation per
  physical owner; no shared-table benchmark surface

## Corpus provenance

Corpus and query TSV files are deliberately not committed. Both arms read the
same staged files under `/home/peter/dev/ecaz/data/staged-current`.

| Prefix | Rows | Dimension | Corpus SHA-256 | Query SHA-256 |
| --- | ---: | ---: | --- | --- |
| `ec_real_10k` | 10,000 | 1536 | `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` |
| `ec_real_50k` | 50,000 | 1536 | `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| `ec_real_100k` | 100,000 | 1536 | `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

## Commands

Baseline extension installation, from the detached baseline worktree:

```text
cargo pgrx install --release \
  -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features -F pg18
```

Baseline suite:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/045-physical-fanout-ab/artifacts/baseline-suite.json \
  --continue-on-error \
  --log-file reviews/task-179/045-physical-fanout-ab/artifacts/baseline-suite-run.log
```

Candidate extension installation used the same `cargo pgrx install` command
from the candidate worktree. Candidate suite:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/045-physical-fanout-ab/artifacts/candidate-suite.json \
  --continue-on-error \
  --log-file reviews/task-179/045-physical-fanout-ab/artifacts/candidate-suite-run.log
```

Each final manifest was then checked with `ecaz bench suite status`, each
config with `ecaz bench suite audit`, and each final manifest with
`ecaz bench suite report` to independently parse its normalized results.

## Artifact index

### Arm-level provenance

- `baseline-suite.json`, `candidate-suite.json`: checked-in canonical matrix
  configs. They differ only in arm names, packet output paths, run directory,
  and base ports 40420/40430.
- `baseline/suite-manifest.json`, `candidate/suite-manifest.json`: exact
  expanded commands, release runner SHA, duration, exit status, expected
  artifacts, and threshold results for all six steps.
- `baseline/results.jsonl`, `candidate/results.jsonl`: normalized topology,
  recall, latency, storage, engagement, and threshold source rows cited by the
  request and `comparison.md`.
- `baseline-suite-run.log`, `candidate-suite-run.log`: suite driver output and
  key result lines for all three scales in each arm.
- `baseline-status.log`, `candidate-status.log`: post-run completion checks.
- `baseline-audit-final.log`, `candidate-audit-final.log`: post-run config
  audits.
- `baseline-install.log`, `candidate-install.log`: exact PG18 release extension
  build/install output for each arm.
- `comparison.md`: derived A/B table; every number traces to the two committed
  `results.jsonl` files.

### Per-scale evidence

For each arm and each of `10k`, `50k`, and `100k`:

- `distann-local-multinode.log` records Ready/Published owner topology,
  physical serving, remote-owner verification, build, recall, fully warmed
  latency, storage, engagement, and topology-gate lines.
- `distann-multinode-summary.log` is the decision-grade physical summary.
- `physical-recall.log` and `physical-latency.log` contain the physical arm's
  detailed recall and latency runs.
- `single-recall.log` and `single-latency.log` contain the same-data control.

PostgreSQL server logs, run directories, truth caches, corpus/query TSVs, and
dry-run snapshots are not part of the committed evidence set.

## Key cited results

```text
baseline status:  completed=3 failed=0 missing_artifacts=0 stale=0
candidate status: completed=3 failed=0 missing_artifacts=0 stale=0

10k:  recall 1.0000 -> 1.0000; physical mean/p95 72.40/91.20 -> 42.40/54.90 ms
50k:  recall 0.9800 -> 0.9800; physical mean/p95 94.60/122.10 -> 59.00/75.10 ms
100k: recall 0.9500 -> 0.9500; physical mean/p95 83.50/119.90 -> 50.30/68.80 ms

all six topology gates: pass=true, owners=3, remote_verified=2
all six latency rows: cache=warm, warmup_iterations=10, count=50, concurrency=1
all eighteen configured thresholds: pass
```
