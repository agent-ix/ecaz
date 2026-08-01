# Task 210 P2a/P2b — sharded head manifest

## Provenance

- Task bucket/packet: `reviews/task-210/003a-head-sharding/`
- Extension + CLI head: `161680298`, PG18 release build with
  `distann-head-attribution-benchmark`
- Suite config: `artifacts/task210-p2-head-sharding.json`
  SHA-256 `52c3ab5b592b889756df9bcf9724017d75b29175c7d915d2e3c61709f2feb31a`
- Structured results: `artifacts/run/results.jsonl` (870 rows)
  SHA-256 `73a0fdd1b4e2bb0eae44fb982a38d6d1f4f44f39e13aa22eded56134e6649d78`
- 9 steps, all succeeded. Fixture: 3 owner nodes, BW=4, H=100, L=32, degree 32,
  head cap 4096, k=10, 200 queries, 50 iterations, 10 warmups, no traversal
  replica. Run dirs under `~/.ecaz/clusters/`, removed after capture.
- Corpora: `data/staged-current`, prefixes `ec_real_{10k,50k,100k}`; SHAs on
  every row in `results.jsonl`.

## Arms

| id | role | what it is |
|---|---|---|
| `task210-p2-control` | control | coordinator-local head (today's shape) |
| `task210-p2-candidate` | candidate | head sharded across the roster |
| `task210-p2b-replica2` | context | sharded head + `head_replica_count=2` |

## P2a result — the property

Coordinator resident index bytes (`physical_benchmark_storage_node`,
`node_role=coordinator`), 100k:

| arm | head_sample_bytes | head_graph_bytes | coordinator resident |
|---|---:|---:|---:|
| control | 25,280,512 | 614,095 | **25,894,607** |
| candidate | 0 | 53,440 | **53,440** |

**485× reduction.** The 4,096 full-precision f32 landmarks no longer exist on
the coordinator: each landmark's vector lives on the owner its FR-078 placement
hash selects, and each owner materialises its own shard from local reads
(ADR-085 D11). What remains is bounded by capacity `C`, not by `N`.

Recall (`recall@10`, top_k=32 sweep point):

| scale | control | sharded | Δ |
|---|---:|---:|---:|
| 10k | 0.9990 | 0.9990 | 0 |
| 50k | 0.9545 | 0.9545 | 0 |
| 100k | 0.9275 | 0.9300 | +0.0025 |

Identical at 10k/50k and marginally better at 100k, consistent with the
unit-level proof that sharded exact head search is result-identical to the
unsharded head (`sharded_exact_head_search_is_identical_to_the_unsharded_head`).

Latency (`custom_scan_total` warm mean):

| scale | control | sharded | Δ |
|---|---:|---:|---:|
| 10k | 27.30 ms | 28.28 ms | +3.6% |
| 50k | 37.73 ms | 38.31 ms | +1.5% |
| 100k | 36.10 ms | 35.68 ms | **−1.2%** |

A small cost at 10k/50k where the head fan-out's RPC dominates a cheap local
scan, and a small win at 100k where spreading head work across three nodes beats
scanning 4,096 landmarks on one. Reported as measured; per NFR-021's
Verification clause a cost does not withhold the property.

## P2b result — the clamp, not replication

`head_replica_fallbacks=96` on the `head_replica_count=2` arm, `0` on every
other arm. Replica routing was requested and **every request clamped back to
the shard's owner**, because no publish-time step populates a replica with a
bounded shard copy. `ec_distann_head_shard_export` is the transport for that
population and is not yet called.

So this arm demonstrates that mis-routing is prevented — a node is never asked
for ids it does not own — and demonstrates **nothing about §4.1 replication**.
It is registered `context` with that stated in its rationale. Recall 0.9295 and
35.89 ms at 100k are, as expected, the sharded-head numbers with an extra
routing decision, not a replication result.

## Outstanding

`outstanding_distribution_gap` on the candidate is
`ec_distann_generation_head_graph:53440:task-210-P2`, not `none`. The remaining
53,440 bytes are the empty-neighbour row structure of the head graph: bounded
and constant in `N`, but non-zero. Either those rows stop being persisted
entirely, or the gap entry is retired with justification — it should not be
accepted silently because the number became small.

## Validation

- extension unit tests: `cargo test --no-default-features --features pg18 --lib
  am::ec_distann` — 185 pass, 1 pre-existing unrelated failure
  (`quantizer::simd_diff_...`, reproduces with all Task 210 changes stashed).
- suite tests: `cargo test -p ecaz-cli commands::bench::suite::tests::distann_`
  — 29 pass.

## Defects this run found (all fixed in the cited head)

Eleven attempts were needed; the causes are recorded because several are the
program's recurring failure modes:

1. head fan-out dialled the coordinator's own shard remotely (`65bfcd78b`)
2. **storage GUC set on `CREATE INDEX` rather than the build session — a silent
   no-op that produced byte-identical arms and would have passed review as a
   clean result. Caught only by the P0 coordinator-storage emitter** (`65c15a800`)
3. work-counter array size behind an unchecked feature (`ecea6c1da`)
4. fixture's exact attribution-row assertion (`b2acb4c1b`)
5. `head_sample.vector` was `NOT NULL` (`700b60532`)
6. state-row digest attested vectors the sharded head no longer stores (`4d9619709`)
7. epoch-manifest digest, same (`ce293ffef`)
8. dropping the coordinator head graph made any non-sharded read return one
   seed; the read path now derives sharding from the persisted shape rather than
   a session GUC, and a membership-only head on a single-owner roster errors
   instead of degrading (`161680298`)
9. orphaned postmasters from earlier attempts held fixture ports (teardown fixed
   in the runner script)

## Re-run

```text
ecaz bench suite run \
  --config reviews/task-210/003a-head-sharding/artifacts/task210-p2-head-sharding.json \
  --artifact-dir reviews/task-210/003a-head-sharding/artifacts/run \
  --results-output reviews/task-210/003a-head-sharding/artifacts/run/results.jsonl \
  --manifest-output reviews/task-210/003a-head-sharding/artifacts/run/suite-manifest.json \
  --log-file reviews/task-210/003a-head-sharding/artifacts/suite-run.log
```
