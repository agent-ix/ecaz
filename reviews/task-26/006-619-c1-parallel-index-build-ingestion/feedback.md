# Feedback: 619 Parallel Index Build Ingestion

## Verdict: Accept

First executable parallel build path is correctly implemented and scoped.

## Findings

**DSM/MQ topology**: One `shm_mq` per worker with the leader as receiver is
the correct starting shape. Per-worker queues avoid contention on a single
shared queue and give the leader independent drain ordering. The 1 MiB queue
size is adequate for initial work.

**Leader as dedicated drainer**: `leader_participates = false` is correct here.
The leader is blocking on `shm_mq_receive` for all workers simultaneously;
adding a heap scan on the leader while draining would require non-blocking
queue reads from a second thread or a shared sorter — neither of which exists
yet. The request correctly describes this as the limiting shape of the slice.

**Binary message format**: The encode/decode pair is verbose but
consistent. Cursor-based reads with bounds checks (`read_bytes` panics on
truncation) are correct. Round-trip test covers source_vector presence and
absence, which are the two distinct code paths.

**`pg_usleep(1000)`** in drain loop: Pragmatic. The `SHM_MQ_WOULD_BLOCK` path
should spin or yield; 1 ms is a reasonable first-pass yield interval for a
build (not a hot scan path). A `WaitLatch`-based approach would be cleaner
long-term but is not required now.

**`reserved0: [u8; 3]`**: Correct padding to align `workersdonecv` after the
`is_concurrent: bool` field. The `workersdonecv` and `mutex` fields are
initialized via `ConditionVariableInit` / `SpinLockInit` in `begin` rather
than through the Rust constructor — that is the correct pattern for PG shared
memory objects.

**`build_source_column` fallback**: Explicitly falling back to serial for
source-column builds is correct. Source column reads go through a different
index tuple path and workers would need separate source-column relation opens.
Deferring this is the right call.

**Snapshot handling**: `is_concurrent` path uses `RegisterSnapshot` /
`UnregisterSnapshot` on the transaction snapshot. The non-concurrent path uses
`SnapshotAnyData` directly. Both match PostgreSQL index build convention.

**`WaitForParallelWorkersToAttach`**: Called after setting up queue handles.
This is the correct barrier before the leader starts draining — workers must
attach their queue sender end before `shm_mq_receive` makes sense.

**`nworkers_launched == 0` early return**: Correct defensive path. If PG
declines to launch workers (e.g., max_parallel_maintenance_workers is 0), the
leader falls back to serial via `None`.

**Test**: `test_pg18_parallel_index_build_uses_workers` asserts at least one
worker launched and all heap TIDs are present in the index with a valid entry
point. This is the right contract to prove at this stage.

## One Observation

`encode_build_tuple_message` errors with `pgrx::error!` if
`tuple.heap_tids.len() != 1`. Workers always emit one TID per tuple (one row
per callback invocation), so this cannot fire in normal use. The check is a
correct defensive assertion, not a recoverable error — the current form is fine.

## No Blocking Issues
