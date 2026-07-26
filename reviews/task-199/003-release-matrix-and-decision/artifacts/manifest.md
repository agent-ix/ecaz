# Task 199 packet manifest

- Head SHA: `2a4a70b23161f556c44d6d1d2c960541fbcb1bdb`
- Packet: `reviews/task-199/003-release-matrix-and-decision/`
- Runner: `ecaz bench suite`; config: `task199-normal-release-10k-50k-100k.json`
- Exact release matrix suite manifest: `artifacts/run/suite-manifest.json`
- Result rows: `artifacts/run-10k/results.jsonl` (10k), `artifacts/run-50k-final/results.jsonl` (50k), and `artifacts/run/results.jsonl` (100k). Each scale has its own suite manifest so resume operations cannot clobber another scale.
- Command: `ecaz bench suite run --config .../task199-normal-release-10k-50k-100k.json --artifact-dir .../artifacts/run`; 50k and 100k resumed with `--resume-from` and `--only`.
- Timestamp: 2026-07-25; isolated three-node local-multinode PG18, shared-table physical surface, release profile.

## Results

| scale | owner recall | replica recall | owner latency ms | replica latency ms | no-replica rows/s | physical generation bytes | replica relation / WAL / build |
|---|---:|---:|---:|---:|---:|---:|---:|
| 10k | .9990 | .9990 | 18.30 | 15.30 | 2292.764 | 311,910,400 | 158,326,784 / 137,540,056 / 4,980 ms |
| 50k | .9685 | .9685 | 20.40 | 16.40 | 2564.925 | 1,588,461,568 | 823,705,600 / 716,628,840 / 24,335 ms |
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

## Run dispersion and configuration identity

The selected release rows are the dedicated per-scale runs listed above. The
previous release run provides a dispersion check: 10k owner/replica latency
was 18.60/15.70 ms versus selected 18.30/15.30 (−0.30/−0.40 ms), 50k was
19.30/16.60 versus selected 20.40/16.40 (+1.10/−0.20 ms), and 100k was
20.50/16.80 versus selected 19.90/16.20 (−0.60/−0.60 ms). These deltas are
reported so the 100k warm-repeat benefit is not presented as more precise than
the observed run-to-run spread.

The per-scale suite manifests preserve actual configuration identity:
10k/100k use config SHA `515bf5b80c06356ce29a76637525e364ef16c3bc3fe16689032be6bb7fd0c730`;
the corrected 50k run uses `db9985d3cc97f9ff82cfbac53d29db535ae6c5dbd48d8402e32d1fe95c3e196c`.
The differences are limited to run directory/base port; benchmark arguments,
corpus, seed variants, and result values are unchanged.

The packet-local `distann-multinode-summary.log` files are the cited raw
result artifacts. SHA-256 hashes of all committed artifacts are recorded in
the review request's follow-up hash table.

## Additional evidence

- Lifecycle and ENOSPC regression closure: `../002-operations-lifecycle-and-isolation/`.
- Historical no-replica baseline: `artifacts/no-replica-before/pre-task199-no-replica-10k/distann-multinode-summary.log` (2315.234 rows/s, pre-extension SHA `ebf9950c1e8a3a6cbbf66a19e8117f9c64b17436`).
- That baseline uses `--queries 10 --benchmark-iterations 2 --benchmark-warmup-iterations 1`; only its insert-throughput line is cited, not read latency.
- Graviton ordered-identity and teardown evidence: `artifacts/graviton-run/` and `artifacts/cloud-teardown-verification-r25.log`; the cloud runner predates the guard-only follow-up, while read-path code is unchanged.
- All cited suite summaries carry extension SHA `2a4a70b23` and `release` profile.

## Committed artifact SHA-256

```text
db9985d3cc97f9ff82cfbac53d29db535ae6c5dbd48d8402e32d1fe95c3e196c  task199-normal-release-10k-50k-100k.json
bd91ebcfda37b87ce57bff5a334e226eeb758c15cf4ebdadab1ef55affb17a89  run-10k/results.jsonl
33ba8637e81fecdd5243ee11c486d51fd745da96f9c684ac4194f45489e47177  run-10k/suite-manifest.json
abdfbe5673fb207b87275a22da2ffe719bc685bc0135d991372ed39d08548983  run-50k-final/results.jsonl
0e55b11aace5c4d5609bd97abf07fbc4503b5c61b5946a764dded687456ca237  run-50k-final/suite-manifest.json
18a2ad2330c25ae429a5ae4c465b1d56f7c23f1801a9331cd761ff61d948bbc9  run/results.jsonl
2fd1a047c8ae2b473822b5d2e42fe13944858ad71a0fd43cabc18f9303e2ed98  run/suite-manifest.json
0482fee8e4da07234d1387d5bc9efd705ed1fd60819008e1e79e02801925fbe5  run-10k/normal-release-ab-10k/distann-multinode-summary.log
310816451460015bf15e78abd9094814a38a271cf236c843fcf5c6bb573d0a4a  run-50k-final/normal-release-ab-50k/distann-multinode-summary.log
d648981bd0565387d75b1356c54dc8178aea81a84a012df6a07f061a728d21dc  run/normal-release-ab-100k/distann-multinode-summary.log
```
