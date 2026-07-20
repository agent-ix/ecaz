# Task 182 production A/B manifest

- Status: complete; nine of nine cells succeeded
- Configuration checkpoint: `8769d57834170dcdc586fa8ac85b99e50b656bd8`
- Task bucket / packet: `reviews/task-182/006-production-ab/`
- Suite config: `production-ab-suite.json`
- Matrix: current production vs trained production vs owner-oracle diagnostic
  at 10k / 50k / 100k
- Fixture: three physical owners, fresh generation per step
- Query shape: 200 held-out queries, top-10, BW4/H100
- Latency: 50 warm iterations after 10 warmups, concurrency 1
- Storage: physical generation/control/source/single plus persisted and cached
  head estimates
- Neighbor/rerank: RaBitQ neighbor scoring; existing exact final rerank
- Training: rows 201–400 from each declared staged query file (10k uses the
  100k query file, matching the reviewed Task 181 disjoint slice)
- Corpus/query TSVs and truth caches are not committed

## Dry-run validation

- Command: `cargo run -p ecaz-cli -- bench suite run --config reviews/task-182/006-production-ab/artifacts/production-ab-suite.json --dry-run`
- Timestamp: 2026-07-16 (America/Los_Angeles)
- Result: success; nine selected steps, all with status `dry-run`
- Generated manifest: `run/suite-manifest.json`
- Expanded steps: `current`, `trained`, and `oracle` at 10k, 50k, and 100k

## Measurement build

- Build / install SHA: `f02cf58a0` (full SHA will be attested by every suite
  node and recorded with the results)
- Extension command: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
- Extension result: success; release library installed and SQL entities
  generated for PG18
- Extension log: `implementation-install.log`
- Runner command: `cargo build --release -p ecaz-cli`
- Runner result: success; one pre-existing unused-field warning
- Runner log: `cli-release-build.log`

## Execution provenance

- Measurement implementation / installed extension / runner SHA:
  `f02cf58a0224dc8a420dbb4964425fe31338e1e2`
- Installed profile/features: release, PG18,
  `distann-head-attribution-benchmark`; the benchmark feature is used only by
  the diagnostic oracle arm
- Command: `target/release/ecaz bench suite run --config reviews/task-182/006-production-ab/artifacts/production-ab-suite.json`
- Execution date: 2026-07-16 (America/Los_Angeles)
- Isolation: one fresh three-owner physical generation and one single-index
  reference per step; the live run directory was replaced between steps
- Suite status: completed 9, failed 0, skipped 0, missing artifacts 0, stale 0
- Suite manifest: `run/suite-manifest.json`
- Normalized results: `run/results.jsonl` (171 rows)
- Runner report/status: `run/report.md`, `run/status.log`
- Config SHA-256: `1a398196c40f70aef9b788fa52afd1eada307f8aec7ec87bb7a921936277f74d`
- Suite-manifest SHA-256: `7c86229b7eb461eeebc159c19f9890a82d7f675a0fc49ce7f2ac98d526aa6c1e`
- Results SHA-256: `e5b2ecb9f95cc60ac298b664185f480c531d0cfcec8ba0dd274c54c74a9625d1`
- Report SHA-256: `18c7396131f169a06f7d44f58b0582684efe304c5abf9da750ddbae81fc3382b`

Every cell attested the same release SHA unanimously. Every Ready/Published
row had `non_owned=0` and `orphans=0`; every serving, topology, and remote
engagement gate passed with three owners and two remote probes.

## Recall and latency

| Scale | Arm | Distinct recall@10 (95% CI) | Warm p50 / p95 / p99 / max |
| --- | --- | ---: | ---: |
| 10k | current production | 0.9990 (0.9964-0.9997) | 34.2 / 39.2 / 45.8 / 45.8 ms |
| 10k | trained production | 0.9990 (0.9964-0.9997) | 38.5 / 43.3 / 48.3 / 51.1 ms |
| 10k | owner oracle | 0.9995 (0.9972-0.9999) | 264.7 / 279.4 / 282.4 / 283.9 ms |
| 50k | current production | 0.9545 (0.9445-0.9628) | 44.1 / 54.5 / 57.5 / 58.1 ms |
| 50k | trained production | 0.9685 (0.9599-0.9753) | 39.3 / 48.3 / 55.8 / 59.7 ms |
| 50k | owner oracle | 0.9970 (0.9935-0.9986) | 1229.4 / 1265.1 / 1279.6 / 1286.1 ms |
| 100k | current production | 0.9275 (0.9153-0.9381) | 40.7 / 56.2 / 58.7 / 60.1 ms |
| 100k | trained production | 0.9625 (0.9532-0.9700) | 41.4 / 53.3 / 60.2 / 62.2 ms |
| 100k | owner oracle | 0.9970 (0.9935-0.9986) | 2554.1 / 2577.5 / 2647.6 / 2696.2 ms |

The trained production arm is recall-neutral at 10k and improves current by
0.0140 at 50k and 0.0350 at 100k. It costs 4.3 ms p50 at 10k, improves p50 by
4.8 ms at 50k, and costs 0.7 ms p50 at 100k while improving 100k p95 by 2.9
ms. These are relative A/B findings; no unapproved hard recall or latency gate
is applied.

## Build, storage, and head accounting

| Scale | Arm | Physical / publish ms | Physical generation bytes | Head cache estimate |
| --- | --- | ---: | ---: | ---: |
| 10k | current | 76,237 / 89,322 | 242,745,344 | 25,794,612 |
| 10k | trained | 78,176 / 91,448 | 242,761,728 | 25,826,119 |
| 50k | current | 418,683 / 484,946 | 1,242,734,592 | 25,814,233 |
| 50k | trained | 426,094 / 492,213 | 1,242,742,784 | 25,900,434 |
| 100k | current | 909,343 / 1,041,453 | 2,496,626,688 | 25,894,607 |
| 100k | trained | 912,404 / 1,043,550 | 2,496,626,688 | 25,892,203 |

Control-index bytes were 24,576 at every scale. Coordinator-source bytes were
166,699,008 / 833,208,320 / 1,666,326,528 and single-index bytes were
115,687,424 / 444,186,624 / 854,810,624 at 10k/50k/100k. All production heads
stored 4,096 samples and returned at most 32 seeds.

## Policy attestation and decision

Current production attested `current_sample_graph` /
`persisted_head_graph`, zero training rows, cap 4,096, and 32 returned seeds.
Trained production attested `training_landmarks_exact` /
`exact_landmark_scan`, exactly 200 training rows, a nonzero persisted training
digest, cap 4,096, 4,096 stored samples, and 32 returned seeds at every scale.
The two production arms set no benchmark seed GUC.

Decision: **PROMOTE** the trained exact-landmark policy as a supported explicit
production build policy. Keep current-sample legacy/default builds unchanged
and byte-compatible; trained construction requires its explicit disjoint
training relation. Do not promote owner scan. Task 183 owns bounded work to
recover the remaining 50k/100k recall headroom and optimize latency.

Only decision-grade compact artifacts are committed: suite config/manifest,
results/report/status, per-step summaries, and cited physical recall/latency
logs. Corpus/query TSVs, truth caches, PostgreSQL node logs, reusable live run
directories, and redundant single-index/full driver logs are not committed.
