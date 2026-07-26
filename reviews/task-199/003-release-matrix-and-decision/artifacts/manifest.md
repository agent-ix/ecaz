# Task 199 packet manifest

- Head SHA: `2a4a70b23161f556c44d6d1d2c960541fbcb1bdb`
- Packet: `reviews/task-199/003-release-matrix-and-decision/`
- Runner: `ecaz bench suite`; config: `task199-normal-release-10k-50k-100k.json`
- Exact release matrix suite manifest: `artifacts/run/suite-manifest.json`
- Result rows: `artifacts/run-10k/results.jsonl` (10k) and `artifacts/run/results.jsonl` (100k). The 50k dedicated rerun repeatedly stalls in the harness before summary emission; its completed final-SHA summary remains under `artifacts/run/normal-release-ab-50k/`, while the machine-readable 50k ancestor remains an open provenance gap pending a host where the suite cleanup completes.
- Command: `ecaz bench suite run --config .../task199-normal-release-10k-50k-100k.json --artifact-dir .../artifacts/run`; 50k and 100k resumed with `--resume-from` and `--only`.
- Timestamp: 2026-07-25; isolated three-node local-multinode PG18, shared-table physical surface, release profile.

## Results

| scale | owner recall | replica recall | owner latency ms | replica latency ms | no-replica rows/s | physical generation bytes | replica relation / WAL / build |
|---|---:|---:|---:|---:|---:|---:|---:|
| 10k | .9990 | .9990 | 18.30 | 16.00 | 2292.764 | 311,910,400 | 158,326,784 / 137,540,056 / 4,980 ms |
| 50k | .9685 | .9685 | 19.40 | 16.40 | 2564.925 | 1,588,461,568 | 823,705,600 / 714,877,096 / 23,580 ms |
| 100k | .9625 | .9625 | 19.90 | 16.20 | 2524.220 | 3,188,056,064 | 1,659,518,976 / 1,937,700,656 / 46,854–54,302 ms |

Storage lines report identical owner and coordinator-replica generation,
coordinator-source, and single-index bytes because that step measures shared
generation storage, not the replica image. Replica relation/WAL/build costs
are listed above; at 100k the relation is 52.0% of the 3,188,056,064-byte
physical generation. The 2,000-trial recall mean includes setup/cold effects;
the 50-sample latency step is warm-cache steady state. At 100k those answer
different questions (21.43→21.50 ms bulk mean versus 19.90→16.20 ms warm
repeat), so promotion claims the warm steady-state benefit explicitly.

The no-replica read-latency before arm was not measured at release sample
size; the pre-boundary run had only 10 queries/2 iterations and is cited only
for its insert throughput. The decision therefore makes no no-replica read
latency claim. The before insert comparison is 2315.234 rows/s at 10k versus
2292.764 after (−0.97%); 50k/100k before arms were not collected.

The 10k profile intentionally trains landmarks from the shared 100k training
file, matching the reference configuration. This bespoke three-scale config
is used instead of the canonical four-profile sweep because AC5 requires one
identical A/B matrix across the three staged scales.

The packet's Graviton run proves ordered identity and lifecycle/fault-drill
agreement within aarch64 (Graviton4/Neoverse V2, SVE2-128); it does not compare
one shared generation across x86 and ARM. Cross-ISA final-order equivalence is
waived for this task's promotion, with the limitation recorded explicitly.

The packet-local `distann-multinode-summary.log` files are the cited raw
result artifacts. SHA-256 hashes of all committed artifacts are recorded in
the review request's follow-up hash table.

## Additional evidence

- Lifecycle and ENOSPC regression closure: `../002-operations-lifecycle-and-isolation/`.
- Historical no-replica baseline: `artifacts/no-replica-before/pre-task199-no-replica-10k/distann-multinode-summary.log` (2315.234 rows/s, pre-extension SHA `ebf9950c1e8a3a6cbbf66a19e8117f9c64b17436`).
- That baseline uses `--queries 10 --benchmark-iterations 2 --benchmark-warmup-iterations 1`; only its insert-throughput line is cited, not read latency.
- Graviton ordered-identity and teardown evidence: `artifacts/graviton-run/` and `artifacts/cloud-teardown-verification-r25.log`; the cloud runner predates the guard-only follow-up, while read-path code is unchanged.
- All cited suite summaries carry extension SHA `2a4a70b23` and `release` profile.
