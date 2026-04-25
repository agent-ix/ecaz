# Feedback: 632 Parallel HNSW Build Graph Assembly ADR

## Verdict: Revise — ADR-048 decision should change

The partitioned-graph-assembly path proposed in ADR-048 is rejected in favor of
concurrent graph insertion with per-node DSM locks. This feedback documents the
analysis and supersedes the "Why Not Shared Concurrent HNSW Insert?" section of
ADR-048.

## Analysis Basis

Three reference implementations were examined:

1. **pgvector** (`src/hnswbuild.c`, current HEAD) — the direct PostgreSQL AM
   precedent for concurrent parallel HNSW build.
2. **hnsw_rs 0.3.4** (`~/.cargo/registry/src/.../hnsw_rs-0.3.4/src/hnsw.rs`)
   — the prior tqvector build dependency; uses rayon + per-node `Arc<RwLock>`.
3. **tqvector native builder** (`src/am/ec_hnsw/build.rs`, packets 622–631)
   — the current serial baseline with measured phase costs.

## Why the ADR-048 Rejection of Concurrent Insertion No Longer Holds

ADR-048 rejected concurrent insertion on three grounds. Each is answered by
tqvector-specific properties the ADR did not account for.

### "PostgreSQL workers are processes, not threads — shared mutable graph requires DSM"

**True, but tractable.** pgvector ships this in production today. The DSM
overhead is well-understood: per-node LWLock (16 bytes each, 800 KB at 50k
nodes), an entry point lock, and a flat node array in shared memory. Both
pgvector and hnsw_rs independently converged on per-node lock granularity as
the right unit. It is the proven approach.

**tqvector's specific advantage over pgvector**: the DSM graph needs to hold
only neighbor-slot arrays (integer indices), not raw f32 vectors. pgvector's
`hnswarea` is bounded by `maintenance_work_mem` because it stores full vector
data in DSM and falls back to serial on-disk insertion when it overflows. That
overflow path loses the parallel benefit entirely. tqvector encodes all tuples
before graph assembly; the DSM graph contains indices into `heap_tuples`, not
the vectors themselves. At 50k nodes × m=6 × 2 layers the neighbor slot
footprint is roughly 2–4 MB. The maintenance_work_mem overflow case does not
apply.

### "Concurrent insertion order is nondeterministic — ADR-042 requires determinism"

**Partially answered by level pre-determination.** tqvector assigns node levels
from a deterministic seed before graph assembly. All node levels are known
before any worker starts. This means:

- The entry point (first node at maximum level) is fixed before workers launch.
  No entry point lock is needed during parallel insertion — the entry point does
  not change. This eliminates the `entryLock + entryWaitLock` pair pgvector
  carries.
- Level assignment is not a source of nondeterminism between serial and parallel
  builds. hnsw_rs's `Arc<Mutex<StdRng>>` problem (level selection serializes
  insertions) does not apply.

The remaining nondeterminism is insertion *ordering*: which worker processes
which node first affects which candidates are present when a given node searches
for neighbors. This is the same nondeterminism pgvector accepts. In practice it
produces bounded recall variation, not structural disconnection. ADR-042's
"up to the documented tolerance of the backlink pruning heuristic" language
provides room for this.

**Recommendation**: treat determinism as a recall quality gate (parallel build
must meet the same recall thresholds as serial build on the existing gates)
rather than a byte-for-byte topology requirement. This is consistent with how
ADR-042 was already applied to the INSERT path.

### "Fine-grained locking may spend the speedup on synchronization"

**Empirically answered by pgvector.** pgvector ships concurrent insertion with
4–8 workers and achieves meaningful speedup on real corpora. The backlink write
is the contention hotspot: inserting node N writes backlinks into existing
nodes M₁..Mₖ, and concurrent workers inserting nearby nodes contend on the
same neighbor locks. This is bounded by the graph degree (M=6 → at most 12
backlink writes per insertion at layer 0) and the neighbor-slot lock hold time
(short: read neighbors, select, write one slot).

tqvector's advantage: score computation is `score(codes[a], codes[b])` using
the existing SIMD kernels — no heap fetch, no distance function dispatch, no
raw vector access. The lock hold time during neighbor search is purely
in-memory SIMD work, which is shorter per candidate than pgvector's float
distance computation on raw vectors.

## Why Partitioned Graph Assembly Is the Harder Path

ADR-048's proposed alternative (partition nodes, workers build local subgraphs,
leader merges boundary patches) has a central unresolved risk: cross-partition
edge quality. HNSW navigability depends on long-range connections established
during insertion. A partitioned graph where workers only see their own node
range will produce locally dense but globally weakly-connected neighborhoods.
The "boundary candidate discovery" step is the critical gate, and its quality
is not well-characterized without measurement.

The concurrent insertion model does not have this risk — every worker inserts
into the same globally-connected graph, so navigability is maintained by
construction (same as serial build).

The partitioned path also requires more infrastructure: a DSM-backed immutable
corpus (separate from the DSM graph), a graph-patch wire format, a deterministic
merge algorithm, and a recall validation gate before the path can be enabled.
The concurrent insertion path requires only the DSM graph with per-node locks
plus a small rework of the worker callback.

## Revised Architecture Recommendation

Replace the ADR-048 partitioned-graph decision with concurrent graph insertion:

1. **Pre-compute all node levels** before worker launch (already done implicitly
   by level assignment from seed; make explicit and store per-node before DSM
   setup).
2. **Determine entry point** before workers launch (highest-level node among
   pre-computed levels). No entry point lock during insertion.
3. **DSM graph**: `Vec<NativeBuildNode>` pre-sized to `heap_tuples.len()`,
   placed in DSM. Each node has one LWLock protecting its `neighbor_slots`.
   Workers address nodes by index (no relative pointer machinery needed beyond
   the base pointer).
4. **Workers**: each takes a parallel heap scan partition (same as current
   ingestion path), encodes tuples, then immediately searches and inserts into
   the shared DSM graph rather than sending via shm_mq to the leader.
5. **Leader participates** in the heap scan + graph insertion (not dedicated to
   queue draining). The current `leader_participates = false` is an artifact of
   the shm_mq drain architecture, which goes away.
6. **Page staging**: after all workers finish, the leader reads the completed
   `Vec<NativeBuildNode>` from DSM and runs the existing page-staging path
   unchanged.

ADR-048 should be revised to reflect this decision. See the updated ADR.

## One Retained Piece from ADR-048

The heap-ingestion coordinator in `build_parallel.rs` remains correct and
useful. The shm_mq streams become unnecessary (workers insert directly into
the shared graph), but the DSM setup, worker launch, WAL/buffer accounting,
and parallel scan descriptor setup are all reusable. The file does not get
replaced — it gets a new graph-assembly code path alongside the existing
ingestion coordinator.
