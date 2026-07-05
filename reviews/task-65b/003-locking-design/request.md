# Task 65b Review Request: Locking And Coherency Design

## Scope

This packet covers Task 65b Slice C: the locking/coherency design for parallel
Vamana graph construction. There is no code change in this packet.

The design is grounded in:

- Slice A measurement floor: real10k SQL index build `6.72s`, real100k SQL
  index build `243.29s`, real10k in-degree p95/p99/max `52/79/2881`.
- Slice B serial cache: `BuilderNeighborCache` now owns build-loop adjacency
  reads and writes without recall or degree-shape movement on real10k.
- Existing local precedents: `ec_ivf` and `ec_hnsw` use PostgreSQL
  `ParallelContext` for build workers; HNSW also has a DSM/LWLock graph
  experiment. `ec_diskann` is still `amcanbuildparallel = false`.

## Decision

Use **deterministic epoch/batch proposal with ordered commit** for the first
parallel graph-construction implementation.

- Worker mechanism: **rayon stepping stone** for the Vamana graph core.
- Worker count source: PostgreSQL `IndexInfo::ii_ParallelWorkers`, not a
  DiskANN-specific worker-count reloption. This preserves the peer AM control
  surface while the graph-core implementation remains a stepping stone.
- Determinism model: fixed seed, fixed pivot permutation, fixed epoch ranges,
  worker-independent proposal ordering, and leader/reducer commit in pivot
  order.
- Write strategy for Slices D/E: workers do not mutate the shared graph
  directly. They compute proposed out-neighbors from an immutable epoch
  snapshot; the leader applies out-edge replacement, backlinks, and reprunes
  through `BuilderNeighborCache` in deterministic order.
- Locking strategy for a future live shared cache or PostgreSQL DSM version:
  **sharded `RwLock`/LWLock stripes**, acquired in ascending stripe order for
  multi-node operations. The initial ordered-commit path deliberately avoids
  concurrent adjacency mutation, so these locks are not required in Slice D.

This is closest to the task's determinism option 3 for the first executable
slice, with a deliberate path toward option 2 once correctness and measurement
show where the ordered reducer bottlenecks.

ADR-075 records the coordinator decision and the Gate #6 disposition: the rayon
path is a graph-core stepping stone only, while PostgreSQL remains the
worker-count authority. Full `pg_stat_progress_create_index`, WAL/buffer worker
attribution, and `pg_stat_activity` worker visibility are not claimed by this
stepping stone; they become mandatory if the production path migrates to
`ParallelContext`.

## Why Not Per-Node Locks First

Slice A's degree distribution makes per-node locking a poor first default:

- final out-degree is bounded by `R=32`
- real10k in-degree p95/p99/max is `52/79/2881`
- backlinks and reprunes concentrate on a small set of hub nodes

A per-node `Mutex<Vec<u32>>` or `RwLock<Vec<u32>>` would be simplest, but it
puts the hottest write contention exactly on the medoid/high-degree hubs. That
does not match the observed hub shape and risks spending the first
multi-worker slice debugging lock convoy behavior instead of graph correctness.

The ordered-commit path keeps writes single-owner and deterministic while still
allowing the expensive proposal work, especially greedy search and prune
distance evaluation, to fan out. If later measurements show the reducer is the
dominant bottleneck, the next step is sharded live commit rather than per-node
locks.

## Reducer Floor

Slice A real10k recorded `142105` backlink additions and `61593` reprunes. The
current serial build-probe attributed `backlink_ms=12362` across pass 0/1 in
packet 002. That is the conservative upper bound for a naive ordered reducer
that simply preserves today's backlink/reprune work on one thread.

The lower bound is much smaller because proposal fanout removes greedy search,
candidate-pool construction, and robust-prune proposal work from the reducer.
The reducer still owns:

- pivot out-edge replacement, bounded by `10000 * R32 = 320000` u32 writes
- backlink insertion checks for `142105` additions
- reprune replacement for `61593` hot-target overflows

Slice E must log reducer wall time directly. If ordered commit costs more than
`1500ms` on real10k, the Task 65b `<= 3s` gate is likely unreachable with this
design and Slice F should open the sharded live-commit path instead of tuning
only batch size.

## Epoch Semantics

Each pass over the pivot permutation is split into fixed epochs:

1. The leader exposes an immutable graph snapshot at epoch start.
2. Workers compute proposals for a fixed pivot range. Partitioning must be based
   on pivot ordinal, not dynamic work-stealing result order.
3. Proposals are returned as `(pass, epoch, pivot_ordinal, pivot_id, edges,
   counters)`.
4. The leader sorts by `(pass, epoch, pivot_ordinal)` and commits:
   - replace pivot out-edges
   - apply backlinks in pivot order
   - reprune target nodes with stable candidate ordering
5. The next epoch observes the committed cache state.

For `parallel_workers = 1` and `parallel_batch_size = 1`, this must match the
serial cache-backed path exactly. Larger batch sizes intentionally permit
bounded stale reads inside an epoch; recall and speed decide the production
default in Slice F.

Slice E starts with `parallel_batch_size = 1` for byte-equivalence. Any larger
batch size must report `stale_read_fraction` and fails review if real10k
Recall@10 moves more than 0.5 percentage points below the packet 001/002
baseline.

## Stale-Read Accounting

The implementation should log enough to explain recall changes:

- `parallel_workers`
- `parallel_batch_size`
- `parallel_epochs`
- per-worker proposal time
- leader commit/reducer time
- greedy-search distance calls
- robust-prune distance calls
- backlink additions
- reprunes
- same-epoch candidate reads
- total candidate/neighbor reads
- `stale_read_fraction = same_epoch_candidate_reads / total_candidate_reads`

For the ordered-snapshot path, "stale" means a worker proposal read a graph view
that omitted an earlier pivot from the same epoch that serial insertion would
already have committed.

## Concurrency Test Surface

Slice E should use a hand-rolled deterministic interleaving model for the
ordered reducer rather than loom over rayon internals. The model surface is the
epoch state machine:

- an immutable snapshot is created once per epoch
- workers can only produce proposal records
- the reducer is the only writer to `BuilderNeighborCache`
- reducer commits are sorted by `(pass, epoch, pivot_ordinal)`
- epoch numbers are monotonic and no proposal from epoch N can observe commits
  from epoch N

The unit/model tests should run without PostgreSQL and assert single-writer
cache ownership, snapshot immutability, and fixed output for repeated schedules.

## Reloptions And Slices

Proposed reloptions for implementation slices:

- `parallel_build_batch_size`: epoch size; default TBD by Slice F.
- `parallel_build_flush_rate`: reserved for the paged-cache/DSM version; log the
  configured value even if the in-memory ordered-commit path does not flush.

Slice D should wire the worker scaffolding behind reloptions with
PostgreSQL `ii_ParallelWorkers = 1` and `parallel_build_batch_size = 1`, then
prove byte-equivalent output and no meaningful performance movement.

Slice E should enable `parallel_build_workers = 4`, keep epoch ranges
worker-independent, and enforce recall plus deterministic repeat runs for fixed
seed, worker count, and batch size.

Slice F should sweep batch size and worker count with `ecaz bench suite` on
real10k and real100k, then either keep ordered commit or open the sharded-lock
live-commit follow-up if reducer time blocks the task's `<= 3s` real10k goal.

## Review Focus

- Whether rayon is acceptable as the first graph-core worker mechanism before
  PostgreSQL `ParallelContext` integration.
- Whether deterministic epoch proposal plus ordered commit is the right
  default before introducing live concurrent adjacency writes.
- Whether sharded locks/LWLocks are the right future live-cache strategy if the
  ordered reducer becomes the bottleneck.
- Whether the proposed stale-read and reducer counters are sufficient for Slice
  E/F review packets.
