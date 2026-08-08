# Task 172 packet manifest

## Provenance

- Task bucket: `reviews/task-172/011-final-gate/`
- Head SHA: `c14c87aab351112963f8257bcd8b416943584a3c`
- Suite runner commit: `c14c87aab351112963f8257bcd8b416943584a3c`
- Extension git SHA: `22ed70bb9d5a39685f0c06db40a4491489516da6`
- Build profile: `release`
- Suite config SHA-256: `8da5b360c5df5fbbadf8aee2ec316444aa03753190316d66eca0f8dd4e3437b3`
- Run date: 2026-08-08 (America/Los_Angeles)
- Host surface: local PostgreSQL 18 socket, three local PostgreSQL instances
- Fixture: `distann-multicluster local-multinode-pg18`, three nodes
- Corpus prefixes: `ec_real_10k`, `ec_real_50k`, `ec_real_100k`
- Query counts: 20 recall queries, 20 latency iterations, 10 warmups
- Concurrency sweep: 1, 2, 4, 8, 16
- Decision arms use `metrics_mode=benchmark`; the 10k diagnostic uses
  `metrics_mode=full_metrics` with stage counters and backend-memory sampling.
- Storage is the summed physical owner surface; no coordinator copy is
  included. The control is one isolated single-index table.
- Corpus TSVs, truth caches, and PostgreSQL cluster directories are not packet
  artifacts and are not committed.

## Commands

The packet-local configuration is `task172-final-suite.json`.

```text
/home/peter/.cargo-target/release/ecaz bench suite audit \
  --config reviews/task-172/011-final-gate/artifacts/task172-final-suite.json

/home/peter/.cargo-target/release/ecaz bench suite run \
  --config reviews/task-172/011-final-gate/artifacts/task172-final-suite.json \
  --resume-from reviews/task-172/011-final-gate/artifacts/suite-manifest.json \
  --manifest-output reviews/task-172/011-final-gate/artifacts/suite-manifest.json \
  --log-file reviews/task-172/011-final-gate/artifacts/suite-run.log
```

The final manifest reports four succeeded steps:

| Step | Scale | Mode | Duration |
| --- | --- | --- | ---: |
| `physical-benchmark-10k` | 10k | benchmark | 480,948 ms |
| `physical-benchmark-50k` | 50k | benchmark | 1,190,794 ms |
| `physical-benchmark-100k` | 100k | benchmark | 2,314,225 ms |
| `physical-overhead-full-metrics-10k` | 10k | full_metrics diagnostic | 193,589 ms |

## Artifact inventory

- `task172-final-suite.json`: checked-in `SuiteConfig` used by the run.
- `dry-run-manifest.json`: preflight expansion of the four selected steps.
- `suite-manifest.json`: final runner manifest; all steps succeeded and the
  cross-scale NFR-021 result is conforming.
- `results.jsonl`: normalized, decision-identity-deduplicated suite results.
  The result rows cite the packet-local per-step logs.
- `suite-run.log`: runner-level output.
- `physical-benchmark-10k/`, `physical-benchmark-50k/`,
  `physical-benchmark-100k/`: benchmark-mode logs, summaries, topology,
  storage, recall, latency, and preflight evidence.
- `physical-overhead-full-metrics-10k/`: full-metrics stage/materialization
  counters and backend-memory diagnostic output.

## Cited result lines

From `results.jsonl`:

- `physical_benchmark_nfr_021_conformance`: `scales=100k,10k,50k`,
  `actual_admissibility=conforming`, `evidence_complete=true`,
  `max_non_owned_records=0`, `max_orphan_vectors=0`,
  `coordinator_resident_unsharded_bytes=0`.
- `physical_benchmark_engagement`: `remote_owners=2`,
  `materialize_probes=2`, `pass=true` at 10k, 50k, and 100k.
- `physical_benchmark_recall`: physical/control pairs are
  `1.0000/1.0000`, `0.9750/0.9800`, and `0.9550/0.9500` at 10k, 50k, and
  100k.
- `physical_benchmark_storage_ratio`: physical amplification is
  `1.235600`, `1.332693`, and `1.351147` at 10k, 50k, and 100k.
- `physical_benchmark_build`: physical/publish/single milliseconds are
  `59736/74508/16570`, `395874/460809/162864`, and
  `873252/1001537/408204` at 10k, 50k, and 100k.
- `physical_benchmark_latency`: the complete benchmark-mode concurrency
  sweep is present for both physical and single arms at all three scales.

The 100k cluster graph-side bytes are 830,144,512. A simple linear capacity
extrapolation from this measured point is approximately 8.3 GB at 1m and
83.0 GB at 10m, before head, metadata, and operational overhead. These are
planning estimates only, not measured promotion numbers.
