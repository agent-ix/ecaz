# Artifact manifest — Task 192 packet 007

- Task bucket / packet: `reviews/task-192/007-full-scale-decision/`
- Runner SHA: `8ae76282689daf7087524718229a870e9d82d0d7`
- Extension SHA: `0ef3a3b9e96e9357953d983ad4aa6338672745e0`
- Host/lane: local Intel, three isolated PG18 owner instances
- Fixture: `ecaz bench suite`; one physical generation shared by both A/B arms
  at each scale plus a separate same-data single-index control
- Storage / rerank / search: trained exact landmark head, RaBitQ stored neighbor
  codes, exact co-located row-tier rerank, lazy10, BW=4/H=100
- Protocol: 200 recall queries / 2,000 trials and 10 warmups + 50 measured
  latency iterations per arm at 10k, 50k, and 100k
- Suite duration: 214,861 ms (10k), 1,009,005 ms (50k), and 2,087,662 ms
  (100k); 3 succeeded, 0 failed, 0 missing, 0 stale
- Installed extension: release profile; installed and target files both
  24,212,096 bytes with SHA-256
  `1776ce3af283303a1f15706924418a2158dd1d1c0255d30d350c30f33e0abf1d`
- Command: `target/debug/ecaz bench suite run --config reviews/task-192/007-full-scale-decision/artifacts/full-scale-suite.json --database tqvector_bench --log-file reviews/task-192/007-full-scale-decision/artifacts/suite-run.log`
- Corpus/query: `ec_real_10k`, `ec_real_50k`, and `ec_real_100k`; corpus TSVs
  are intentionally not committed. The 10k training slice uses rows 201–400
  of the staged 100k query file because the 10k file contains only the 200-row
  evaluation slice.

## Files

| Artifact | SHA-256 | Purpose |
|---|---|---|
| `full-scale-suite.json` | `619ca33c73b132e836b73df21e65cf8bdf500acb94ecad1845791ef11bc01897` | Checked-in 10k/50k/100k A/B config |
| `full-run/suite-manifest.json` | `a640222847affb291b3639eee796843d975f7e530c0fe872004f41b61a89c60d` | Commands, runner SHA, timings, and success state |
| `full-run/results.jsonl` | `42dab880251b81225469be7dc3a9ab5e564ed55aedab81a84c52750eec0ef34e` | Structured recall, latency, storage, topology, stage, and work rows |
| `full-run/owner-validation-cache-ab-10k/distann-multinode-summary.log` | `3a7cd6ec3c8efe75c30416d4230e584ae5bb53ab4fcb0fd4d1698ac08aee9761` | Complete parsed 10k summary |
| `full-run/owner-validation-cache-ab-50k/distann-multinode-summary.log` | `f0d5c18bbc813e2451b35641e0b2ff9286bc088489a7fcf38c31d6cb2449139f` | Complete parsed 50k summary |
| `full-run/owner-validation-cache-ab-100k/distann-multinode-summary.log` | `1640a37d8d459c449b3b93502a0ce854a03b35ac0c4b28085e8e64b562f9a853` | Complete parsed 100k summary |
| `release-install.log` | `3e4a4fbee75475db6918bf0a66c440990a92009f4c205a757e58141a37c4d74e` | Release install and binary identity preflight |
| `suite-audit.log` | `d3c22f5d53191be96031f80576cf04b24b1dd0a57a956bc4773fda619d46080b` | Suite preflight audit |
| `full-run/owner-validation-cache-ab-10k/physical-owner-validation-uncached-recall.log` | `778b2a95f5ab2a1e47f092f8b54c398d22b2e6ba77ac319e11e63c24bc8a7c2d` | 10k baseline recall |
| `full-run/owner-validation-cache-ab-10k/physical-owner-validation-uncached-latency.log` | `c752be57b7d96d878e84ddeab345ebd68883d6380cdf39386876bd12e3e261e1` | 10k baseline latency/counters |
| `full-run/owner-validation-cache-ab-10k/physical-owner-validation-cached-recall.log` | `d9089b8c6b40257b70917f241c00728e8d2ba6d50c192664a84efa1aab0ddca0` | 10k candidate recall |
| `full-run/owner-validation-cache-ab-10k/physical-owner-validation-cached-latency.log` | `a72282d114d8462373aeafcd814878802317416f30c39b11a44925e6ee31c515` | 10k candidate latency/counters |
| `full-run/owner-validation-cache-ab-50k/physical-owner-validation-uncached-recall.log` | `d07f554391713d01f4c2651f7a3d4042293137960b660b0e50ec8ab91d356032` | 50k baseline recall |
| `full-run/owner-validation-cache-ab-50k/physical-owner-validation-uncached-latency.log` | `55badc664b5223a33dd9dce8fc3d591716efb9444076236a6e28417c350c8703` | 50k baseline latency/counters |
| `full-run/owner-validation-cache-ab-50k/physical-owner-validation-cached-recall.log` | `5c9aaa707cebe5afd0794a8226b5c63b202393eb115c7db58a6e6ab8ae1cb52b` | 50k candidate recall |
| `full-run/owner-validation-cache-ab-50k/physical-owner-validation-cached-latency.log` | `fb1d502bb2fb560e32db5f297454eb0b18c111601a1d2b2d3fb33894fe578d40` | 50k candidate latency/counters |
| `full-run/owner-validation-cache-ab-100k/physical-owner-validation-uncached-recall.log` | `02077a81166ed12ce6a6529562d727db3ec6a0883f10f1e97bd90627f3d747a0` | 100k baseline recall |
| `full-run/owner-validation-cache-ab-100k/physical-owner-validation-uncached-latency.log` | `384ba1bb9a725fee378f2818bdb381ce601c8cea676c674ab00c6c1afcb458cb` | 100k baseline latency/counters |
| `full-run/owner-validation-cache-ab-100k/physical-owner-validation-cached-recall.log` | `9677b2c8fafcd9604035caa8b0e0f6548878e016d6b40e527dd8d3064acecfb6` | 100k candidate recall |
| `full-run/owner-validation-cache-ab-100k/physical-owner-validation-cached-latency.log` | `1a3b13008786197aaba8602c876932a00e2c40232fead75f9ed63918c7ff38f5` | 100k candidate latency/counters |

Operational node logs, the fixture transcript, polling output, and
single-control raw logs are intentionally not part of this packet.

## Key result rows

- Recall uncached/cached: 10k `0.9990/0.9990`; 50k `0.9685/0.9685`; 100k
  `0.9625/0.9625`.
- Warm mean uncached/cached: 10k `24.70/19.30 ms`; 50k `23.50/19.80 ms`;
  100k `23.70/19.70 ms`.
- Warm p95 uncached/cached: 10k `28.60/22.50 ms`; 50k `26.70/22.80 ms`;
  100k `26.60/22.80 ms`.
- Owner open/validate uncached/cached: 10k `7.818124/0.026003 ms`; 50k
  `6.708479/0.023115 ms`; 100k `6.889008/0.023703 ms`.
- Physical-generation bytes are identical by arm at every scale:
  `242,745,344`, `1,242,734,592`, and `2,496,659,456`.
