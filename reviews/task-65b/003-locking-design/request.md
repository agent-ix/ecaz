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

## Reloptions And Slices

Proposed reloptions for implementation slices:

- `parallel_build_workers`: `0` means Task 65/Slice B serial fallback.
- `parallel_build_batch_size`: epoch size; default TBD by Slice F.
- `parallel_build_flush_rate`: reserved for the paged-cache/DSM version; log the
  configured value even if the in-memory ordered-commit path does not flush.

Slice D should wire the worker scaffolding behind reloptions with
`parallel_build_workers = 1` and `parallel_build_batch_size = 1`, then prove
byte-equivalent output and no meaningful performance movement.

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
