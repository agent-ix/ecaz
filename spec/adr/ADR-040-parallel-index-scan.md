---
id: ADR-040
title: "Parallel Index Scan via amcanparallel"
status: PROPOSED
impact: Affects FR-014, NFR-001, ADR-014, ADR-016, ADR-026, ADR-027
date: 2026-04-18
---
# ADR-040: Parallel Index Scan

## Context

Postgres supports parallel index scans via the access method's
`amcanparallel` flag and the parallel-scan callbacks
(`amestimateparallelscan`, `aminitparallelscan`,
`amparallelrescan`). When enabled, the planner can split an
index scan across multiple background workers that share a
common `ParallelIndexScanDesc` and return tuples to a leader for
merging.

tqvector today sets `amcanparallel = false`. Every scan is
single-worker. For a graph ANN index whose per-query latency is
dominated by graph traversal and candidate scoring, parallel
workers could substantially shorten wall-clock query time on
multi-core hardware — especially on hot workloads where
ef_search is large and a single worker can't saturate the SIMD
units in its CPU core.

No Postgres vector extension currently ships parallel index scan:

- **pgvector** — sets `amcanparallel = false`.
- **pgvectorscale** — sets `amcanparallel = false`.
- **VectorChord** — sets `amcanparallel = false`.
- **Lantern** — sets `amcanparallel = false`.

This is unclaimed territory. It is also a non-trivial engineering
lift because Postgres's parallel scan model assumes the index AM
can deterministically partition work across workers — easy for
btree ordered scans, subtle for graph ANN where the work is
inherently interdependent (each hop depends on previous hops'
scored candidates).

## Decision

tqvector will enable **`amcanparallel = true`** for `ec_hnsw` and
future `ec_diskann` / `tqspann` access methods. The parallel scan
model is:

### 2026-04-24 amendment: Gather Merge output compatibility

The PG18 implementation may expose a planner-visible
`Parallel Index Scan`, but it must not let multiple workers emit
tuples directly to `Gather Merge` until the worker streams are
rank-compatible with the SQL merge key.

The local PG18 diagnostic fixture showed why. The single-emitter
serial HNSW output can contain adjacent inversions when compared
against the exact SQL `ORDER BY` expression, for example:

```text
177(-1.646769881) before 379(-1.742236257)
472(-1.641386509) before 473(-1.769334435)
172(-1.507030725) before 93(-1.649150252)
```

Those rows are acceptable when one backend emits the serial AM
order, because `Gather Merge` only sees one non-empty stream. If
the same serial order is partitioned across multiple emitters,
`Gather Merge` compares each stream head by the exact sort key and
can legally pull a later exact-smaller row ahead of an earlier
serial row. Hash partitioning, worker-local direct emission, and
rank-modulo partitioning all have this failure mode unless a
coordinator withholds rows until the serial prefix is safe.

Therefore the production PG18 path starts with one elected tuple
emitter for serial-equivalent correctness. The remaining
multi-emitter design must provide one of these contracts:

- a coordinator-owned serial-rank sequencer whose exposed ordering
  is compatible with the planner pathkeys;
- per-worker streams that are actually sorted by the exact SQL
  `ORDER BY` key, accepting exact-key order rather than serial HNSW
  order;
- a planner/runtime contract that avoids `Gather Merge` for
  approximate serial-order output.

Until one of those contracts is implemented, multi-emitter output
remains diagnostic-only and is expected to fail strict
serial-equivalence checks.

### Shared coordinator, independent beams

The leader initializes a shared scan descriptor holding the query
vector, rotation, LUT, and a shared result heap. Each worker runs
an **independent graph traversal from a different entry point**,
maintaining its own visited set and candidate heap.

- **ef_search is split across workers.** With `n_workers = 4` and
  user-provided `ef_search = 100`, each worker runs with
  `ef_per_worker = 100 / 4 + overlap`, where overlap is a small
  fixed additive term (e.g., 10) to preserve recall at worker
  boundaries.
- **Entry points are chosen deterministically** from the index
  meta — multiple upper-layer entry nodes or a seeded distribution.
- Workers do **not** share their visited sets. This is a
  deliberate tradeoff: sharing the visited set avoids duplicate
  work but requires atomic operations on every visit check,
  which dominates latency on small workloads. Duplicate visits
  are bounded and acceptable.

### Shared result aggregation

Workers write their top-k candidates into a shared result heap
under a lightweight lock (spinlock or lock-free heap). The
leader reads the merged top-k after all workers complete.

### Rescan and partial parallelism

`amparallelrescan` resets shared state for nested-loop-like
parallel plans. Single-row query cases (one query per scan)
won't benefit — parallel workers have setup overhead that
dominates sub-millisecond queries. The planner's cost model
(ADR-011 replacement work) must account for this.

## Consequences

### Scan state must become worker-safe

Today's `TqScanOpaque` holds single-worker mutable state:
visited sets, grouped rerank slots, heap rerank state, tuple
slot caches. For parallel scan:

- **Per-worker state** stays in a worker-local `TqScanOpaque`.
- **Shared state** moves into a new `TqParallelScanOpaque` in
  dynamic shared memory (DSM), allocated during
  `aminitparallelscan`.
- Heap rerank tuple slots must be **per-worker** — `TupleTableSlot`
  is not cross-process-safe.
- The grouped-rerank snapshot can be shared (snapshots are
  cross-worker-safe in Postgres's snapshot management) but each
  worker holds its own `Relation` reference.

### Locking discipline revisited

ADR-026 (insert lock ordering) and ADR-027 (vacuum lock
ordering) assume single-reader semantics on graph pages.
Parallel scan adds concurrent readers across multiple backends.
The changes:

- **Page shared locks** work correctly across workers — they're
  reader locks, so multiple workers can hold them concurrently.
- **The upgrade-to-exclusive path** during insert is unchanged;
  inserts and scans coordinate through the buffer manager as
  before.
- **Visited-set coordination** is skipped by design (see above).

No new ADR on lock ordering is required, but ADR-026 and ADR-027
should be reviewed to confirm their reasoning holds under
multiple concurrent readers.

### ef_search budget and recall

Splitting ef_search across workers risks recall loss if the
split is naive. The `overlap` additive term (expected 5–15%
of per-worker budget) preserves recall at boundaries. A worked
example at `ef_search = 100, n_workers = 4`:

- Naive split: each worker gets 25, total work = 100, recall
  drops because no single worker sees enough of the graph to
  find far candidates.
- With overlap = 10: each worker gets 35, total work = 140,
  recall matches or exceeds single-worker ef=100 because
  diverse entry points expand graph coverage.

Net effect: ~40% extra total graph traversal work, completed in
~25% wall-clock time, for neutral-or-better recall. Users with
latency-sensitive workloads on multi-core hardware win
substantially; users on single-core tiers see no regression
(planner won't dispatch parallel workers when cost model
rejects).

### Planner and cost model

Parallel-scan path cost must be computed correctly. Currently
ADR-011 uses an `f64::MAX` cost gate as a hack. That work is
already slated for removal; parallel path costing is a natural
extension:

- **Startup cost:** worker launch overhead (~100–500 µs on
  Linux).
- **Run cost:** per-worker traversal divided by worker count.
- **Parallel_setup_cost and parallel_tuple_cost:** standard
  Postgres GUCs already handled by `add_partial_path`.

The parallel plan is rejected when ef_search is small or the
query is expected to return few rows — standard parallel
planner behavior, no special logic needed beyond cost estimates.

### Determinism

Parallel top-k can return different tie-breaking order across
runs because worker completion order is non-deterministic.
Acceptable for approximate NN search (recall target, not
deterministic ranking), but should be documented. Users who
need deterministic results can set
`max_parallel_workers_per_gather = 0` at session level.

### Insert and vacuum unchanged

Parallel scan affects only the read path. Insert and vacuum
remain single-worker operations. ADR-026 and ADR-027 continue
to govern concurrency between writers; parallel scan adds only
more readers.

## Scoping

### What ships in the first parallel-scan release

- `amcanparallel = true` for `ec_hnsw`.
- Shared scan descriptor with query, rotation, LUT, result heap.
- Serial-equivalent elected tuple emitter while multi-emitter
  output remains rank-incompatible with PG18 `Gather Merge`.
- Per-worker independent beams with overlap term once a
  rank-compatible output contract is available.
- Result heap merge via shared-memory spinlock once a
  rank-compatible output contract is available.
- Cost model entries for parallel path.

### What does not ship in the first release

- **Parallel insert.** Insert throughput is a separate NFR
  (task 13); parallel insert is substantially harder because
  writers must coordinate on graph mutation.
- **Parallel vacuum.** Bulk delete could parallelize but is
  not urgent.
- **Gang scheduling across queries.** Each scan stands alone.
- **Shared visited set.** Deliberately rejected; revisit if
  profiling shows duplicate work dominates.

## Alternatives considered

### Stay serial

Defensible if we believe parallel scan adds complexity out of
proportion to benefit. Against: no Postgres vector extension
offers this today; it's a real differentiator with measurable
multi-core wins.

### Share visited set across workers

Would eliminate duplicate work. Requires atomic visited-set
operations on every node check. Profiling in FAISS-parallel
implementations suggests the atomic cost dominates for typical
graph sizes. Rejected in favor of independent beams with
overlap.

### Parallel only for DiskANN / SPANN, not HNSW

Considered because DiskANN's structure (single layer, predictable
I/O) is a cleaner fit. Rejected because HNSW is where the current
users are, and HNSW also benefits — the shared-query, independent-
beam model works identically across all three graph structures.

### Implement fully async scan via effective_io_concurrency

Postgres's async I/O path is different from parallel workers and
targets a different problem (SSD IOP parallelism within a single
worker). Complementary to this ADR, not a substitute. Could be a
future optimization for DiskANN specifically.

## References

- ADR-011: Planner cost override (to be superseded)
- ADR-014: Traversal state memory budget
- ADR-016: ef-search control surface
- ADR-026: Live insert backlink lock ordering
- ADR-027: Vacuum graph repair lock ordering
- Postgres documentation: `amcanparallel`, `amestimateparallelscan`,
  `aminitparallelscan`, `amparallelrescan`
- FAISS parallel scan implementation (multi-threaded, shared vs
  independent visited sets)
