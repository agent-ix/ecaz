# Feedback: 647 Parallel Concurrent DSM Graph Workers

## Verdict: Accept

First end-to-end parallel graph assembly path. The two-phase design (existing
shm_mq heap ingest, then dedicated graph-assembly `ParallelContext`) is the
correct transitional shape. The wiring is consistent with the scaffolding
established in packets 633-646.

## Design Correctness

**Participant counting**: `graph_participant_count = requested_workers + 1`
(leader participates). Workers receive `worker_number` as their participant
index (0-based). Leader covers `workers_launched..participant_count`. Correct:
if 4 workers launch, they handle partitions 0-3, leader handles partition 4.
If fewer workers launch, leader covers the gap sequentially — functional fallback.

**DSM graph context**: Uses `InvalidOid` for heap/index relids and
`is_concurrent = false`. Correct — graph workers do not open heap/index
relations; they operate on the DSM image only.

**LWLock tranche**: `LWLockNewTrancheId` + `LWLockRegisterTranche` per build is
the right approach. Each graph build gets a fresh tranche, avoiding name
collisions across concurrent builds.

**Entry node bootstrap**: Entry node is pre-READY at DSM initialization
(packet 639). Workers begin searching from it immediately. Correct.

**`WaitForParallelWorkersToAttach`** before `insert_leader_partitions`: correct
barrier. The leader must not start inserting before workers attach, or the
leader's writes could race with workers deriving their partitions.

## One Observation

`parallel_graph_build_worker_main` reuses `PARALLEL_KEY_EC_HNSW_BUFFER_USAGE`
and `PARALLEL_KEY_EC_HNSW_WAL_USAGE` keys in the graph context TOC. These keys
are the same constants as in the heap ingest context. This is safe because the
graph context is a separate `ParallelContext` with its own TOC. No key collision.
Worth an explanatory comment if this becomes confusing to future readers.

## GUC Gate

`ec_hnsw.enable_parallel_build_concurrent_dsm` is the correct opt-in surface
before recall and build-time validation. Keeping it off by default is right.

## No Blocking Issues
