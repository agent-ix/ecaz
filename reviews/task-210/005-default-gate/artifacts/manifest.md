# Task 210 — default-config gate manifest

## Provenance

- Task bucket/packet: `reviews/task-210/005-default-gate/`
- Suite config: `artifacts/task210-default-gate.json`
  SHA-256 `c6622460de739bb0c7b43eef7e8ac1226f1e9ddd70c1f38d44050d4650374a31`
- Two runs, both against PG18 release builds with
  `distann-head-attribution-benchmark`:
  - **`run/`** (extension `81e816f32`'s predecessor at gate time; CLI with the
    populate hook): all 6 steps, `results.jsonl` 586 rows, SHA-256
    `ee76ff39772e9346e2f15793a2cc455ca23fbfcd95c737133119eca2ff7d2ee7`.
    Authoritative for the **default-gate arms**. Its replica arms ran inert —
    see "Replica-arm defect ledger".
  - **`run-replica/`** (extension `81e816f32`): the three replica arms rerun,
    `results.jsonl` 293 rows, SHA-256
    `55a55b0952e8f3fcf825aa2d5166c9ea5d68786fce46c0d05e81a639e4090ac0`.
    Authoritative for the **replica-serving arms**.
- Fixture: 3 owner nodes per step, one isolated cluster per step, BW=4,
  H=100, L=32, degree 32, head cap 4096, k=10, 200 queries, 50 iterations,
  10 warmups. Run dirs under `~/.ecaz/clusters/task210-gate-*`, removed
  after capture.
- Corpora: `data/staged-current`, prefixes `ec_real_{10k,50k,100k}`.

## The gate — NFR-021 clause 5 in the shipped default

`default-gate-*` arms run the shipped configuration: **no arm flags, no
session GUCs**. Since `fe5822f46` the sharded head is the default
(`shard_head_storage` + `sharded_head_search` on; single-owner rosters keep
full vectors).

| scale | coordinator resident bytes | recall@10 | warm mean (p50) |
|---|---:|---:|---:|
| 10k | 53,440 | 0.9990 | 31.80 (31.30) ms |
| 50k | 53,440 | 0.9545 | 40.50 (38.90) ms |
| 100k | 53,440 | 0.9290 | 38.60 (37.10) ms |

Constant 53,440 bytes at every scale (vs 25,894,607 for the pre-P2 central
head — the 003a A/B), recall equal to the 003a sharded arms, and the
conformance rows for `task210-default-gate` evaluate
`conforming, preregistration_matches=true`. The residual 53,440 bytes are
the empty-neighbour head-graph rows, still carried on the
`NFR_021_KNOWN_DISTRIBUTION_GAPS` allowlist and reported on every row
(003a reviewer question 1 remains open).

## Replica serving — §4.1 active in the measured window

`replica-serving-*` arms: shipped default + `head_replica_count=2`, with the
fixture distributing and attesting shard copies before benchmarking
(`physical_head_replicas populated replica_count=2 placed=6` on every arm —
3 shards × 2 replicas, coordinator-owned shards included).

From `run-replica/` (latency command, 50 scans):

| scale | head_replica_shards_served | head_replica_fallbacks | recall@10 | warm mean (p50) |
|---|---:|---:|---:|---:|
| 10k | **29** | 0 | 0.9990 | 30.20 (30.10) ms |
| 50k | **33** | 0 | 0.9540 | 41.30 (40.20) ms |
| 100k | **32** | 0 | 0.9265 | 48.30 (48.20) ms |

Serving is provably active (non-zero at every scale) and routing never
clamps (fallbacks 0: population is fully attested, so every server the
routing hash picks holds its copy). Recall stays within the default arms'
CI. Latency is neutral at 10k/50k and **+25% at 100k**: in a
single-stream benchmark, spreading head CPU across the roster buys nothing
and adds shard-build/routing overhead on first touch per backend. §4.1's
payoff is contended-load head-CPU spreading; this arm stays `context`, and
promoting replica routing under load needs its own measured case. Recorded,
not traded away.

## Replica-arm defect ledger (why three attempts)

Every layer below produced a **green suite run with an inert mechanism**,
caught only by the activation counters:

1. The fixture never called `ec_distann_populate_head_replicas`; attested
   routing correctly refused to route (first P2 run:
   `head_replica_fallbacks=96`). Fixed in `a3dccfd16`.
2. The `head_replica_count` session GUC was appended only under the legacy
   `--sharded-head` flag, which the default-config gate rightly no longer
   passes; the arms ran with `replica_count=0`
   (`served=0 AND fallbacks=0`). Fixed in `0b3e688e6`; bench child argv is
   now logged (`physical_bench_child` lines) so an arm's session GUCs are
   packet-provable.
3. Warmup queries served from the replica copy and populated the per-backend
   owner-shard cache (`953b502a8`); counters reset after warmup; measured
   queries hit the cache, whose hit path did not count. A no-warmup SQL probe
   on the same cluster showed `served=2`, isolating the counter. Fixed in
   `81e816f32`: cache entries carry `from_replica` provenance and hits count.

## Re-run

    ecaz bench suite run \
      --config reviews/task-210/005-default-gate/artifacts/task210-default-gate.json \
      --artifact-dir reviews/task-210/005-default-gate/artifacts/run
