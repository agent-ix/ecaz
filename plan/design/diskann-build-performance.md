# DiskANN Build Performance Investigation

Status: draft 2026-05-28. Pure code review — no benchmarks yet.

Scope: identify why `ec_diskann ambuild` is "very slow" and propose
fixes, organised in priority order. Compares our implementation in
`src/am/ec_diskann/{ambuild,build,vamana,persist,quantizer}.rs`
against pgvectorscale's Rust DiskANN under
`pgvectorscale/src/access_method/{build.rs,graph/}`.

The build path entry point is `ec_diskann_ambuild`
(`ambuild.rs:158`) → `flush_build_state` (`ambuild.rs:271`) →
`build_and_persist_vamana` (`build.rs:161`) →
`build_vamana_graph_with_stats` (`vamana.rs:462`) →
`greedy_search` + `robust_prune` per pivot × 2 passes →
`persist_vamana_graph` (`persist.rs:77`).

---

## Headline cost model

For N source rows, max degree R, build-list size L:

* Heap-scan O(N²) dedup pass (issue #1) → 5×10⁹ vec-compares at
  N=100k just to detect exact-duplicate vectors.
* O(N²) memory allocation churn during graph build (issue #2) →
  ≈40 GB of `vec![false; N]` allocate-and-zero at N=100k.
* O(visited × L) linear scans inside greedy search (issue #3).
* Two full graph-construction passes (issue #5) — pgvectorscale does
  one.
* Single-threaded everywhere (issue #6) — pgvectorscale uses
  parallel workers.

These compound. Even if each one alone were tolerable, together they
explain order-of-magnitude slowness vs pgvectorscale on the same N.

---

## P0 — algorithmic / asymptotic bugs

These have the highest expected impact. Fix first.

### #1. O(N²) exact-match dedup in heap scan

`src/am/ec_diskann/ambuild.rs:116-147` (`BuildState::push`)

```rust
if let Some(existing) = self
    .heap_tuples
    .iter_mut()
    .find(|existing| source_vectors_match_exactly(&existing.source_vector, &source_vector))
{
    existing.overflow_heap_tids.push(heap_tid);
    return;
}
self.heap_tuples.push(RawHeapTuple { ... });
```

Every incoming vector is bit-compared against every previously
collected vector. At N=100k with d=1536 this is ≈ N²/2 × d float
compares = 7.7×10¹³ comparisons during the heap scan callback alone.
pgvectorscale does **no** input-side dedup (the explore agent grepped
the entire codebase for `dedup` and found nothing).

**Fix options, cheapest first:**
1. **Remove dedup entirely** for V1. Treat each heap TID as a
   distinct node. Exact-duplicate source vectors are rare in real
   corpora and the index already supports overflow heap TIDs through
   `insert::stage_overflow_heap_tids_in_chain` as a post-build path.
2. If we must keep collapse-on-exact-match: hash a short fingerprint
   (e.g. xxhash of the byte-encoded `Vec<f32>` or the first 16
   coordinates' bit pattern) into a `HashMap<u64, SmallVec<usize>>`
   keyed by fingerprint → candidate index list, and only do
   bit-equal compare on hash collisions. That brings dedup to
   expected O(N).

**Recommend option 1.** The dedup feature should justify its cost,
and it currently doesn't.

### #2. O(N²) bitvector churn in `greedy_search`

`src/am/ec_diskann/vamana.rs:212-213`

```rust
let mut in_frontier = vec![false; n];
let mut visited_flag = vec![false; n];
```

These two N-sized `Vec<bool>` allocations happen on every
`greedy_search` call. `greedy_search` is called once per pivot per
pass, i.e. 2N times. Total allocation/zeroing pressure:

    2 × 2N × N bytes = 4 N² bytes

At N=100k that is 40 GB of allocate-and-zero traffic in the malloc
path, not counting drop time. pgvectorscale uses a single
`HashSet<ItemPointer>` per search (`graph/mod.rs:317`), capacity
hinted to expected fill — O(visited) instead of O(N).

**Fix options:**
1. **Reuse buffers.** Thread an `&mut SearchScratch` through
   `greedy_search`. Clear with `.fill(false)` per call — O(N) writes
   instead of O(N) writes + O(N) allocate + O(N) free. Best when
   build runs as a single pass.
2. **Use a bitset** (`bitvec::BitVec` or a `Vec<u64>`) — same
   semantics, 8× less memory traffic, and `fill_with_zeros` is one
   memset.
3. **Use a small `HashSet<u32>`** for sparse expansions, matching
   pgvectorscale. Best when `visited.len() ≪ N`.

**Recommend option 2** for in-frontier (bitset over `Vec<u64>`,
sized to next-pow-2 × `N/64`) plus reused-across-calls scratch held
on the build orchestrator. Option 3 is also fine; the win is "stop
re-allocating N bytes per search".

### #3. Linear-scan frontier inside `greedy_search`

`src/am/ec_diskann/vamana.rs:223-261`

```rust
loop {
    let next = frontier
        .iter()
        .copied()
        .filter(|c| !visited_flag[c.node as usize])
        .min_by(|a, b| a.cmp(b));   // O(L) scan every iteration
    ...
    if frontier.len() > list_size {
        frontier.sort();             // O(F log F) every overflow
        for c in &frontier[list_size..] { in_frontier[c.node as usize] = false; }
        frontier.truncate(list_size);
    }
}
```

Per search this is O((L+E) × L) where E is expansions. Greedy
search ends up dominated by sort and linear scan rather than
distance compute. pgvectorscale's frontier is
`BinaryHeap<Reverse<ListSearchNeighbor>>` (`graph/mod.rs:75`) with a
separate distance-ordered visited list — O(log L) per push, O(1)
peek.

**Fix:** replace the `Vec<Candidate>` with a bounded min-heap, or a
sorted small-vec, or a `BinaryHeap<Reverse<Candidate>>` paired with
a bit "visited" marker. The simplest is:

* A min-heap `BinaryHeap<Reverse<Candidate>>` containing all
  un-expanded entries.
* On overflow, pop the worst end (keep min-heap of size L by also
  tracking max for eviction, or use `bounded-heap`).
* Visited tracked separately as a bitset (issue #2 already covers
  this).

**Recommend** porting pgvectorscale's design directly — it is
well-trodden and correct.

### #4. Per-pivot `vec![false; node_count]` in pool helpers

`src/am/ec_diskann/vamana.rs:338-360` (`candidate_pool_for_prune`),
`vamana.rs:373-389` (`append_candidates_for_prune`)

Both helpers allocate `vec![false; node_count]` to detect duplicate
candidates, even though the candidate pool is only L+R items wide.
These are called once per pivot per pass — same 2N call count as
greedy search, with the same O(N²) churn pattern as issue #2.

**Fix:** the candidate pool is small. Replace the bitset with a
sort-then-dedup or a 64-entry-ish `SmallVec` linear scan. Or share
the build-orchestrator scratch from issue #2.

### #5. Two full passes instead of one growing-alpha pass

`src/am/ec_diskann/vamana.rs:509`

```rust
for (pass_index, &alpha) in [1.0f32, alpha_final].iter().enumerate() {
    // re-shuffle + full N-pivot Vamana pass
}
```

Every pivot is processed twice: once at α=1.0, once at α=alpha_final
(default 1.2). pgvectorscale instead grows α dynamically inside a
single robust-prune call (`graph/mod.rs:413-484`):

```rust
let mut alpha = 1.0;
while alpha <= max_alpha && results.len() < num_neighbors {
    // prune at current alpha
    alpha *= 1.2;
}
```

Same final graph quality target, ~half the distance compute and
~half the pivot iterations.

**Fix:** port the growing-α loop into `robust_prune` and drop the
outer "two-pass" structure. This is the single biggest algorithm
fix.

### #6. No parallelism

`src/am/ec_diskann/{ambuild,vamana}.rs`

Build is single-threaded end-to-end:
* Heap scan callback is sequential (Postgres-imposed; matches
  pgvectorscale).
* The N-pivot loop inside each pass is sequential.
* Codec encode of N payloads is sequential (`ambuild.rs:313`).
* Persistence (placeholder + patch) is sequential.

pgvectorscale uses Postgres `ParallelContext` workers
(`build.rs:352`) and an in-build neighbor cache that periodically
flushes (`build.rs:1146-1151`). Per-worker batches of heap tuples
each carry their own greedy-search state; the shared graph is
written back to a paged store under cache eviction.

**Fix options, by cost:**
1. **Rayon over `encode` step** (`ambuild.rs:313`). Embarrassingly
   parallel, zero shared state. Smallest change, real win on
   high-d/large-N. Probably 30–60% of post-fix wall time.
2. **Rayon over pivots in pass 1** (the α=1.0 pass becomes a
   one-pass under fix #5). Pivots are sequential in the reference
   paper because each insert depends on the graph state, but
   pgvectorscale shows that batched insertion with periodic merging
   is correct in practice. Higher implementation cost; defer until
   #1–#5 land.
3. **Postgres ParallelContext** (matches pgvectorscale exactly).
   Highest implementation cost, biggest payoff at very large N.
   Out-of-scope for the first fix wave.

**Recommend** option 1 immediately, option 2 once the new algorithm
shape is stable.

---

## P1 — quadratic constant-factor waste

### #7. Always-on `Cell` distance counters

`src/am/ec_diskann/vamana.rs:527-642`

Every distance call inside the build loop goes through a wrapping
closure that increments a `Cell<usize>`:

```rust
let greedy_distance_calls = Cell::new(0usize);
...
let result = greedy_search(&graph, medoid, list_size, |n| {
    greedy_distance_calls.set(greedy_distance_calls.get() + 1);
    dist(n, i)
});
```

There are four such counters and the wrapper appears in greedy,
candidate-pool, robust-prune, and backlink paths. At ~2×N×L×R
distance calls per build, the counter writes add a full memory-RMW
to every distance hit. They are also not behind any cfg gate — they
ship in production builds.

**Fix:**
* Move counters into the inner functions (or thread a stats struct
  by `&mut`) only when a `--features build-stats` flag is on.
* In production, the counters disappear and the distance closure is
  a clean `Fn(u32, u32) -> f32`.

### #8. Closure-of-closure inlining wall

The build path passes around `D: Fn(u32, u32) -> f32 + Copy`
generics. `greedy_search` then re-binds via another closure
(`|n| dist(n, i)`). Even with monomorphisation, the LLVM inliner
sometimes fails to flatten this when the call site is large. Worth
checking with `cargo asm` after #7 lands — if the inner distance
call isn't inlined, switch to a small trait object with `#[inline]`
methods, or explicitly extract a `for<'a> fn(&Ctx, u32, u32) -> f32`
function pointer.

### #9. Double clone in `persist_vamana_graph`

`src/am/ec_diskann/persist.rs:145-199`

For every node, the persist pass clones `binary_words` and
`search_code` twice — once for the placeholder tuple (line 152-153)
and again for the patched tuple (line 188-189). It then encodes the
tuple twice and updates the page in place via
`update_raw_tuple`.

For large d / large grouped-PQ codes / large W, those clones are
the dominant persist cost. Total work ≈ N × (W + C) bytes of
allocation and copy × 2.

**Fix options:**
1. Allocate the page slot in pass 1 with the right *size* but bytes
   left zero; overwrite once in pass 2. Page slot allocation is
   already cheap; we just need a "reserve `len` bytes" call on
   `DataPageChain`.
2. Or compute neighbor TIDs in a single forward pass and write the
   tuple once. This requires the persistence order to be a
   topological order of the graph w.r.t. neighbor references, which
   the BFS-from-medoid ordering provides for *reachable* nodes but
   not for back-edges. Trickier; only worth it if option 1 is
   blocked.

### #10. `MetricSummary::from_values` sorts full N

`src/am/ec_diskann/vamana.rs:81-97`

Each pass calls `MetricSummary::from_values` four to seven times
with N-element value vectors, each call sorting a fresh clone. Pure
diagnostic overhead.

**Fix:** behind the same `build-stats` cfg as issue #7. Default
build path produces no metric summaries.

### #11. `approximate_medoid` double-counts symmetric distance

`src/am/ec_diskann/vamana.rs:413-426`

```rust
for &s in &samples {
    let mut sum = 0.0f32;
    for &t in &samples {
        if s != t { sum += dist(s, t); }
    }
    ...
}
```

For S=1000, this is 1M dist calls when 500k would suffice — the
build distance is symmetric. Plus the `dist` closure is the
build-time distance, which currently funnels through the counter
wrapper (#7).

**Fix:** compute the upper triangle once into a flat `Vec<f32>`,
then sum rows. O(S² / 2) compute, O(S²) memory; at S=1000 that's
2 MB — fine.

### #12. Unconditional overflow loop in `flush_build_state`

`src/am/ec_diskann/ambuild.rs:385-394`

```rust
for (node_index, tuple) in state.heap_tuples.iter().enumerate() {
    insert::stage_overflow_heap_tids_in_chain(
        &mut chain,
        ...
        &tuple.overflow_heap_tids,
    )?;
}
```

If issue #1 is fixed (dedup removed), every `overflow_heap_tids`
slice is empty and this loop is pure overhead. Even with dedup, the
loop should `continue` on empty.

**Fix:** `if tuple.overflow_heap_tids.is_empty() { continue; }`
guard. After #1 lands, delete the loop entirely.

### #13. Vec-of-Vec<f32> for source vectors

`src/am/ec_diskann/ambuild.rs:50-54, 250`

Every heap row produces a fresh heap-allocated `Vec<f32>` via
`ecvector_datum_to_vec` (`ambuild.rs:864`). At N=100k and d=1536,
that's 100k separate allocations totalling ≈600 MB of fragmented
heap. The dedup linear scan then iterates all of them — cache-cold
for each comparison.

**Fix:** stage source vectors into a single contiguous
`Vec<f32>` of length `N × d` with parallel `Vec<RawHeapMeta>` for
TIDs. One allocation, dense iteration, much more cache-friendly.
pgvectorscale sidesteps this by streaming straight into paged
tape storage; for us a single big buffer is a simpler intermediate
step.

---

## P2 — minor / cleanup

### #14. Backlink reprune sorts twice

`src/am/ec_diskann/vamana.rs:594-612`

The reprune branch builds a `combined: Vec<Candidate>`, sorts it,
then calls `robust_prune` which sorts again on line 292. Drop the
outer sort or pass a "pre-sorted" hint into `robust_prune`.

### #15. `neighbors_j.contains(&i)` linear scan

`src/am/ec_diskann/vamana.rs:587`

At R=64 this is 64-entry linear scan per backlink, executed up to
R times per pivot. Not catastrophic but a sorted `neighbors_j`
plus binary search would be cheaper, and we already need sorted
neighbors at persist time for stable encoding.

---

## Proposed fix sequence

Phased to keep each step independently shippable and testable
against `tests/integration/diskann_*` and existing recall tests.

### Phase A — single-threaded algorithmic correctness wins (P0)

1. Delete bit-equal dedup in `BuildState::push` (#1).
2. Add `--features build-stats` gate; move all `Cell` counters and
   `MetricSummary::from_values` behind it (#7, #10).
3. Replace `greedy_search` frontier with `BinaryHeap` + bitset
   visited (#2, #3).
4. Replace pool-helper bitsets with reusable scratch or small
   linear scan (#4).
5. Merge two-pass build into a single growing-α pass per
   pgvectorscale's pattern (#5).

After Phase A, run `cargo bench` / the m5 fixture: this is the
"should be 4–10× faster" milestone.

### Phase B — memory layout (P1)

6. Flat `Vec<f32>` source buffer instead of per-row heap allocs
   (#13).
7. Single-write persistence (#9).
8. Symmetric-medoid micro-fix (#11).

### Phase C — parallelism

9. Rayon over codec `encode` step (#6 option 1).
10. Backlink + persist cleanups (#12, #14, #15).
11. Decide later whether to invest in Postgres `ParallelContext`
    (#6 option 3) based on Phase B numbers.

---

## Open questions for the user

* Phase A changes the graph construction order (growing-α vs
  two-pass). Existing recall tests should still pass within the
  current target, but seed-determinism asserts (e.g.
  `BO-007: bo_007_deterministic_for_fixed_seed`) will need updated
  golden values. Acceptable?
* Is dedup-on-exact-match a load-bearing feature for any consumer
  we need to preserve? If yes, fall back to the hash-based version
  in #1 instead of removing.
* The `build-stats` feature flag is invasive. Alternative: keep a
  single atomic counter per pass, accept the cache-line ping, drop
  the per-call closure wrapper. Which do you prefer?
