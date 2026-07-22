# Artifact manifest — Task 195 packet 002

- Head / runner SHA: `51e5d614501742cb9d5db4b6b7d39ebcfba5d7c0`
- Production code SHA: `65e746fc1a85f8efa8032a81aa2cc95b55c23503`
- Baseline extension SHA: `cf862b9507f8ba211bb3e903df54c1f127714a51`
- Task bucket / packet: `reviews/task-195/002-release-matrix/`
- Lane / host: local Intel, three independent PG18 owner instances
- Fixture: physical hash-sharded `ec_distann`, one index per table
- Storage / rerank / search: production physical storage, exact training-
  landmark head, RaBitQ neighbors, lazy-10 payload materialization, BW4/H100
- Protocol: 200 recall queries, 2,000 latency trials, 10 warmups, 50 measured
  iterations; 10k/50k/100k staged real corpus
- Timestamp: 2026-07-22 America/Los_Angeles
- Suite config SHA-256:
  `7b5ae0f28ca1979f5ad49fdd957fc70ed0cfdcd745a40086588ce774a81250bd`

## Commands

Release installs used the corresponding checkout and:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark
```

The final normal production install used the same command with `--features
pg18`. The binary-identity artifacts record target/installed sizes and hashes.

Both A/B matrices used:

```text
/home/peter/dev/ecaz/target/release/ecaz bench suite --config reviews/task-195/002-release-matrix/artifacts/task195-release-suite.json --artifact-dir reviews/task-195/002-release-matrix/artifacts/{baseline|candidate}
```

Status and audit used the same config/artifact directory with `--status` and
`--audit`. The checked-in final dry run used `--dry-run` and
`artifacts/dry-run-final`.

## Key results

| Scale | Recall A/B | Warm mean A/B | p95 A/B | Open/validate A/B | Payload SQL A/B | Remote materialize A/B | Storage A/B bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| 10k | 0.9990 / 0.9990 | 22.80 / 20.90 | 26.00 / 25.50 | 7.030095 / 0.028399 | 8.748595 / 8.994472 | 10.473932 / 7.229717 | 242761728 / 242745344 |
| 50k | 0.9685 / 0.9685 | 24.10 / 20.90 | 27.50 / 24.30 | 6.791688 / 0.023934 | 9.018823 / 8.978053 | 10.699518 / 7.154837 | 1242742784 / 1242750976 |
| 100k | 0.9625 / 0.9625 | 24.30 / 19.90 | 27.20 / 23.30 | 7.122363 / 0.024107 | 8.909927 / 8.804038 | 10.721241 / 7.027564 | 2496651264 / 2496626688 |

`ab-comparison.log` also records 78/78 materialization metrics compared with
zero mismatches, equality of query/training/head/seed digests, and all
topology, engagement, and traversal-reconciliation gates passing.

## Files

| Artifact | SHA-256 | Purpose / cited result |
|---|---|---|
| `task195-release-suite.json` | `7b5ae0f28ca1979f5ad49fdd957fc70ed0cfdcd745a40086588ce774a81250bd` | Checked-in 10k/50k/100k suite configuration |
| `suite-audit.log` | `4429b5b737f3734170be3455b99d8193ce713066fd127c400b3419369d194582` | Final candidate audit passed all 3 steps |
| `suite-dry-run-final.log` | `2117200f781ff894c02da6b561b12b156978e305ccac89b14f0c3616b55e8ebc` | Final encoded production variant and per-step command audit |
| `ab-comparison.log` | `71b82035b61d6973e5045b8b16f2cf7ee7c823dcde2ee00a447b7c7803ab3905` | Exact A/B result, work equality, digests, and gates |
| `baseline-binary-identity.log` | `f254cb9f849dbcacc89f7ce59de0dedeba404eed061d6cd681fd97d8e0c67a3d` | Release attribution baseline target/installed identity |
| `candidate-binary-identity.log` | `f408d8bf08f3a44341e456168f203541b8c8b9f61e84f46f0bdf97c0738e7d94` | Release attribution candidate target/installed identity |
| `production-binary-identity.log` | `6a7e8b87e94408213d6ad2765a1c001d12a9f3921524ac89bd8beb551ecddd6d` | Normal release PG18 target/installed identity |
| `feature-isolation-audit.log` | `eea3d2553d29f9a9bb9c95fb5b39e2a176d6aacc6b3d0796a0ee8856fe21ef46` | No attribution/profile/selector surface in normal installed SQL |
| `baseline-release-install.log` | `e3f6dc36b1605733be545c516c4467c2f7f719e01b7974e0b38d7c15f58ea359` | Baseline release build/install transcript |
| `candidate-release-install.log` | `21d186996ba188bf31b4b08ad3209cda8c3dc165f24a4eb80ed3169c15d578bd` | Candidate release build/install transcript |
| `production-release-install.log` | `f8f1271306367c3ef5eeb361a2009f2705b1afaceaa2fe10ee396cf08c83bcb1` | Final normal release build/install transcript |
| `baseline/suite-manifest.json` | `f05c13c288c8036f7b708e6615477e3bce22cb74e299607f7339608da48d8fc6` | Baseline runner/config/step provenance |
| `baseline/results.jsonl` | `a06ad425d7f5822531779435054e6a9c12fb426cf7e11fb3aff1a576aa9f34e3` | Baseline structured results |
| `baseline-status.log` | `8557f59e26092c7cc723a3fb870daffda0cde6ffc5e96f1055d14830a739fe57` | 3 succeeded, 0 failed/missing/stale |
| `candidate/suite-manifest.json` | `d8e1603ecc2a510cc3778078aef7e16a18a73e045d1fcaf45bfb1c0b10073450` | Candidate runner/config/step provenance |
| `candidate/results.jsonl` | `df16a8e193d6cd2a9e56847b10ae32e89da27063f21e7c3dcb74792b32e58196` | Candidate structured results |
| `candidate-status.log` | `c6da312e6f77095fdb40ffa8866333f5fc57e0900551d8e28e0ec4701ac883b7` | 3 succeeded, 0 failed/missing/stale |

Each baseline/candidate scale directory additionally retains the compact
`distann-multinode-summary.log`, `physical-production-recall.log`, and
`physical-production-latency.log` cited by the structured results. Corpus TSVs,
node PostgreSQL logs, tunnel/session state, and full fixture transcripts are
not committed.
