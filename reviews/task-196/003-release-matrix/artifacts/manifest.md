# Artifact manifest — Task 196 packet 003

- Candidate head / runner / attribution extension SHA:
  `a5e567c45a5c96f67a842163e2293843d0a3774a`
- Production fix commit: `77adfb6b40430e4fb4684bb9403717e1e1a42f68`
- Pre-fix baseline packet:
  `reviews/task-195/002-release-matrix/artifacts/candidate/`
- Pre-fix baseline runner / extension SHA:
  `51e5d614501742cb9d5db4b6b7d39ebcfba5d7c0`
- Task 196 base: `adcd95623aae91d960ff2f884ff64a95b0f6406e`
- Task bucket / packet: `reviews/task-196/003-release-matrix/`
- Lane / host: local Intel, three independent PG18 owner instances
- Fixture: physical hash-sharded `ec_distann`, one index per table
- Storage / rerank / search: production physical storage, exact training-
  landmark head, RaBitQ neighbors, lazy-10 payload materialization, BW4/H100
- Protocol: 200 recall queries, 2,000 trials, 10 warmups, 50 measured
  iterations; 10k/50k/100k staged real corpus
- Timestamp: 2026-07-22 America/Los_Angeles
- Suite config SHA-256:
  `7b5ae0f28ca1979f5ad49fdd957fc70ed0cfdcd745a40086588ce774a81250bd`

## Baseline reuse and commands

The pre-fix side is the same-day Task 195 production candidate matrix. Task
196 branched from its review head, and this command exited zero:

```text
git diff --quiet 51e5d614501742cb9d5db4b6b7d39ebcfba5d7c0..adcd95623aae91d960ff2f884ff64a95b0f6406e -- src/am/ec_distann/custom_scan.rs
```

The owning packet's `task196-release-suite.json` is byte-identical to the Task
195 config, including the historical suite/step labels, so the two sides have
the same config SHA and production search protocol. Candidate release install:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark
```

Candidate matrix:

```text
target/release/ecaz bench suite run --config reviews/task-196/003-release-matrix/artifacts/task196-release-suite.json --artifact-dir reviews/task-196/003-release-matrix/artifacts/candidate --log-file reviews/task-196/003-release-matrix/artifacts/suite-run-candidate.log
```

The final normal production install used the same install command with
`--features pg18`. Status reports three succeeded and zero failed, missing, or
stale steps; final suite audit passes all three inputs.

Focused identity/rank-shift coverage used:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --no-default-features --features pg18 materialized_payload_survives_rank_shift_without_refetch
```

It passed 1 test with 0 failures.

## Key results

| Scale | Recall A/B | Warm mean A/B | p95 A/B | Storage A/B bytes | Duplicate requests after |
|---|---:|---:|---:|---:|---:|
| 10k | 0.9990 / 0.9990 | 20.90 / 19.10 | 25.50 / 22.40 | 242745344 / 242761728 | 0 |
| 50k | 0.9685 / 0.9685 | 20.90 / 19.90 | 24.30 / 23.10 | 1242750976 / 1242750976 | 0 |
| 100k | 0.9625 / 0.9625 | 19.90 / 19.80 | 23.30 / 23.20 | 2496626688 / 2496626688 | 0 |

All 78 materialization work rows compare exactly, including zero duplicate
requests on both ordinary A/B sides. Query, training-slice, head-sample, and
seed-ID digests match. All topology, engagement, and traversal-reconciliation
gates pass. The latency result is treated as neutral-to-favorable rather than a
causal win; identity-keyed reuse only changes a deepened rejected-prefix case.
The 10k two-page storage variance is independent-build page layout, and the
patch changes no stored format.

Candidate target and installed attribution binaries were 24,269,976 bytes and
byte-identical at SHA-256
`5b852a06ba4578c109a7d03a271a75ace1abe1d0c25ffafe08de7bb252be105b`.
The release runner was 23,198,448 bytes at SHA-256
`8414483d535fd1d6c13f4d3729b50251d414fafb89a50f27a73c93a1cf660909`.
The final normal target/installed binaries were 23,866,928 bytes and
byte-identical at SHA-256
`d5a4e92a9d13310a045f26753a41b6fea00b61661a5bfc384a9804e66d00a1ad`.
Normal installed SQL has none of the attribution profile, benchmark counters,
or rank-shift diagnostics.

## Files

| Artifact | SHA-256 | Purpose / cited result |
|---|---|---|
| `task196-release-suite.json` | `7b5ae0f28ca1979f5ad49fdd957fc70ed0cfdcd745a40086588ce774a81250bd` | Checked-in byte-identical 10k/50k/100k production config |
| `ab-comparison.log` | `eb3a039fbca889be5467da2c7eb9e6ae94240ed24dd5b6011d5240f98f34aeec` | Exact recall/work/digest/gate comparison and decision |
| `candidate-binary-identity.log` | `9f56f4b90ae6dd2cc5744d88bec4c9455feae5f1297f195e0128793162789484` | Candidate release target/installed/runner identity |
| `production-binary-identity.log` | `afa27b23f1c13b8929e525a4184d31b4b47ef09e9c33ca3e875fe26f7871505d` | Final normal release target/installed identity |
| `feature-isolation-audit.log` | `8e919e2b62eadb3e3bae421419b27551c8a736dcb018d4c5499b4bc2ec899b44` | No attribution/profile/diagnostic surface in normal SQL |
| `focused-unit-test.log` | `7fefebf8f0cf68b140cca5356514fe480deb670f021f333ce186f39da9732cd6` | Identity-keyed reuse survives an equal-distance rank shift |
| `candidate-release-install.log` | `0f072ebe7ae64818fa25301f60ee9fbae90a35bd82a0d8a7c100fcc4f3eb38d5` | Candidate release build/install transcript |
| `production-release-install.log` | `825907743ec862be7256fad8196c1ddabb5c7beafa9c22f1433b50484983c08f` | Normal release build/install transcript |
| `suite-audit-final.log` | `4429b5b737f3734170be3455b99d8193ce713066fd127c400b3419369d194582` | Final input audit passed all three steps |
| `suite-run-candidate.log` | `fa09b8c0f7de5e618febc91a79fbb6b4ae1cd56dabc8caf5290de8daef3020a8` | Compact runner transcript |
| `candidate-status.log` | `1e9c62619c4ab520a0a70f7092799c7997d93519b512967553e8793466c367e1` | Three succeeded, zero failed/missing/stale |
| `candidate/suite-manifest.json` | `ce69679ffa921742ec4d802978a02e7906018cda99da70a055e38edd07a3611f` | Clean runner/config/step provenance and durations |
| `candidate/results.jsonl` | `b9e0801e30f43ae3ef8005f4703fffb23abf4bde656589b0a4a7608f41ce1846` | Structured 10k/50k/100k result rows |
| `candidate/owner-schema-production-10k/distann-multinode-summary.log` | `a117ce190a1f72d1f3391d8d841d4f88d5ce809ba78866cd675541776fb79bec` | 10k summary, gates, work, recall, latency, storage |
| `candidate/owner-schema-production-50k/distann-multinode-summary.log` | `eaa9153b9560f071aba1f8b595111fedb99be9daf9f7eba82e0f6f96e3363974` | 50k summary, gates, work, recall, latency, storage |
| `candidate/owner-schema-production-100k/distann-multinode-summary.log` | `c0a787597b85acc993b9c1284dee77f1bc94a258fa881757df986b9afe1d8170` | 100k summary, gates, work, recall, latency, storage |

Each scale additionally retains the compact production recall and latency logs
cited by the structured results. Node PostgreSQL logs, full fixture transcripts,
single-control logs, and other operational exhaust are deliberately not
committed.
