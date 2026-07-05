# Review Request: C1 Warm Steady-State Optimization Survey

## Context

Packet `262` removed the temporary full-tuple byte copy in `src/am/graph.rs`
and produced a small but consistent warm win. Packet `263` plans direct graph
tuple decoding to eliminate the intermediate page tuple struct round-trip.

Current verified warm steady-state on real `10k`, `m=8`, `ef_search=40`,
`warm-after-prime3`, `per-cell`:

- `p50=14.315ms`
- `p95=16.350ms`
- `p99=17.613ms`
- `mean=14.194ms`

NFR-001 targets:

- `p50 < 5ms`
- `p99 < 15ms`

The gap is roughly 3x on p50. No single optimization will close it. This
packet surveys the remaining hot-path surface and proposes a prioritized
sequence of experiments.

## Hot-path evidence

Perf on repeated warm backend queries:

- `31.78%` in `tqvector::quant::prod::ProdQuantizer::score_ip_from_split_parts`
- `4.11%` in `Vec::extend_from_slice`
- `1.20%` in `tqvector::am::graph::read_page_tuple_bytes`

The remaining ~63% is spread across many functions with no single dominant
entry.

## Working assumptions

- dim=1536, bits=4, m=8, ef_search=40, LIMIT 10
- `qjl_enabled(1536, 4)` = false (tile_dim(1536) = Some(512) and bits == 4)
- `mse_bits` = 4 (16 centroids), code = 768 bytes, codebook = 16 entries (64 bytes)
- Hot scoring path: `score_ip_from_split_parts_no_qjl_4bit` -- scalar nibble-by-nibble
  loop over 768 bytes (prod.rs:234-263)
- `PreparedQuery.lut` is built (96KB, 1536 x 16 entries) but never read by the
  no_qjl_4bit path
- Each query does ~300-800 score evaluations (beam search expansion + frontier refills)
- The element cache (`graph_element_cache`) is per-scan -- every scan starts cold, so
  every element requires at least one PG buffer read

## Top 5 ideas (highest signal first)

### 1. AVX2/NEON SIMD for `score_ip_from_split_parts_no_qjl_4bit`

**What:** `src/quant/prod.rs:234-263`. Replace the scalar nibble-by-nibble loop
with a vectorized kernel. The existing AVX2 kernel (`score_ip_from_split_parts_avx2`,
line 576) already proves the pattern for the 3-bit+QJL path. The 4-bit no-QJL path
has no SIMD variant.

**AVX2 approach (32 dims/iteration):**

- Load 16 bytes of `mse_packed`, broadcast each 4-byte group to all 8 i32 lanes
- Variable-shift (`_mm256_srlv_epi32`) by `[0,4,8,12,16,20,24,28]` + mask `& 0x0F`
  to extract 8 nibble indices in dimension order
- 16-entry codebook lookup: split codebook into two `__m256` (entries 0-7, entries
  8-15), use two `_mm256_permutevar8x32_ps` + `_mm256_cmpgt_epi32` +
  `_mm256_blendv_ps`
- `_mm256_fmadd_ps` with `_mm256_loadu_ps(&rotated[dim])` into 4-way accumulator
- 1536 dims / 32 per iteration = 48 iterations (vs. 768 scalar iterations)

Working set is L1-friendly: codebook (64B) + rotated (6KB) + mse_packed (768B) =
~7KB total. No LUT needed.

**Why:** This is the 31.78% perf hotspot. Even a conservative 4x speedup on scoring
cuts ~3-4ms off the 14ms total. With 4-way unrolling, 6-8x is achievable.

**Risk:** Medium. The 3-bit AVX2 path is already proven; this follows the same
architecture. The nibble-unpacking via srlv+mask differs from the 3-bit word decode
but is well-understood. Needs a NEON counterpart.

**Measure:** Run the warm benchmark. Compare `score_ip_from_split_parts` perf
percentage. Expected: drops from ~32% to ~5-8% of total.

### 2. Zero-copy element decoding -- eliminate `read_page_tuple_bytes` `.to_vec()`

**What:** `src/am/graph.rs:618-665`. Currently, every PG buffer page read copies
raw tuple bytes into a fresh `Vec<u8>`, then `TqElementTuple::decode` (page.rs:199)
copies the `code` bytes into another `Vec<u8>` and copies all 10 heaptid slots into
a `Vec<ItemPointer>` before truncating to the actual count.

For a single element tuple (772 bytes), this is two heap allocations + two memcpys
per element load. With hundreds of element loads per query, this is significant.

**Approach:** Hold the PG buffer pin + shared lock for the duration of the decode.
Decode `TqElementTuple` fields directly from the page buffer slice. Store the `code`
as a copied-to-reusable-buffer or borrowed slice. For `heaptids`, use the inline
`[ItemPointer; HEAPTID_INLINE_CAPACITY]` array pattern that `SelectedScanResult`
already uses (scan.rs:1930).

**Why:** `Vec::extend_from_slice` is 4.11% in perf. Eliminating per-element allocation
removes allocator pressure and reduces cache pollution.

**Risk:** High. Changes the lifetime model for `GraphElement`. The element cache
(`HashMap<..., Arc<GraphElement>>`) stores owned data; switching to borrows requires
rethinking the cache. A staged approach: first inline heaptids (idea #4), then tackle
code ownership.

**Measure:** DHAT allocation profiling before/after. Watch for `Vec::extend_from_slice`
disappearing from the profile.

### 3. Lazy deletion in `BeamSearch::forget_queued` to eliminate O(n^2) heap drain

**What:** `src/am/search.rs:409-433`. `forget_queued` drains the entire `BinaryHeap`,
filters one node out, collects to `Vec`, and rebuilds the heap. It is called from
`take_best_matching` (line 393) which loops until it finds a matching node --
potentially calling `forget_queued` multiple times. With ef_search=40, each
drain+rebuild is O(40 log 40). Chained calls make `consume_best` worst-case O(n^2).

**Approach:** Add a `HashSet<NodeId>` of logically deleted nodes to `BeamSearch`.
`forget_queued` inserts into the deleted set (O(1)). `peek_best` / `pop_best` skip
deleted nodes at the top (O(log n) amortized). Alternatively, replace `BinaryHeap`
with an indexed priority queue supporting O(log n) removal.

**Why:** During frontier consumption, `consume_best` calls
`expansion.take_best_matching(|node| self.contains_node(node))` which pops
non-matching nodes via `forget_queued`. Each non-match triggers a full heap drain.
At ef_search=40 this is small in absolute terms but adds up across 10+ result
emissions. At ef_search=200 this becomes material.

**Risk:** Low-medium. Lazy-deletion is well-understood. Main concern: keeping the
`visited` set consistent with the logical heap contents.

**Measure:** Instrument `forget_queued` call count and total drain size. Before/after
comparison on p50 at ef_search=40 and ef_search=200.

### 4. Inline heaptids + avoid per-decode code Vec in `GraphElement` / `TqElementTuple`

**What:**

- `src/am/page.rs:219-247` (`TqElementTuple::decode`): replace `Vec<ItemPointer>`
  heaptids with `[ItemPointer; HEAPTID_INLINE_CAPACITY]` + count. Replace
  `code: input[cursor..].to_vec()` with `Box<[u8]>` or arena-backed slice.
- `src/am/graph.rs:11-18` (`GraphElement`): same inline pattern for `heaptids`.

**Why:** Every `TqElementTuple::decode` currently:

1. Allocates a `Vec<ItemPointer>` with capacity 10, pushes 10 items, then truncates
   via `.take(count).collect()` -- allocating a second Vec
2. Allocates a `Vec<u8>` of 768 bytes for code

With inline heaptids (60 bytes on stack), both heaptid Vec allocations become zero.

**Risk:** Low for heaptids (mechanical change). Medium for code -- needs API changes
anywhere `GraphElement.code` is passed to `score_ip_from_parts`.

**Measure:** DHAT allocation profiling. Look for reduction in total heap bytes
allocated per query.

### 5. Pre-allocated scratch buffers for per-expansion neighbor Vec allocations

**What:**

- `src/am/graph.rs:345-362` (`valid_neighbor_tids_for_layer`): returns
  `Vec<ItemPointer>`. Allocates every call.
- `src/am/graph.rs:110-134` (`load_layer0_successor_candidates`): returns
  `Vec<BeamCandidate<...>>`. Allocates every call.
- `src/am/graph.rs:592-616` (`layer0_successor_candidates_from_elements`): returns
  `Vec<BeamCandidate<...>>`. Allocates every call.

Each frontier expansion step chains these, producing 3+ temporary Vecs. With 40+
expansions per query, that is 120+ Vec allocations.

**Approach:** Add a `ScratchBuffers` struct to `TqScanOpaque` containing pre-allocated
`Vec`s that get `.clear()`'d and reused. Pass `&mut scratch` through the loading
functions.

**Why:** Eliminates ~120+ heap allocations per query. Reused buffers stay warm in L1.

**Risk:** Low. Mechanical plumbing of scratch buffers through function signatures.

**Measure:** Benchmark p50/p99 before/after. DHAT allocation count should drop.

## Additional ideas

### 6. Skip building the `PreparedQuery` LUT when QJL is disabled

`prepare_ip_query` (prod.rs:164-170) builds a 96KB LUT for 4-bit, but
`score_ip_from_split_parts_no_qjl_4bit` never reads it. When
`!qjl_enabled && bits == 4`, skip the LUT entirely. Saves ~microseconds per query
and avoids L2 cache pollution.

### 7. Remove Arc wrapping for single-threaded caches

`graph_element_cache` and `graph_neighbor_cache` (scan.rs:2108-2109) wrap entries in
`Arc`. The caches are accessed only through `TqScanOpaque` (single-threaded PG
backend). `Arc` refcount operations are unnecessary. Using owned values or `Rc` saves
atomic operations on every cache hit/insert.

### 8. VisibleFrontier: replace linear-scan Vec with a sorted structure

`VisibleFrontier::best_candidate_by_score` (search.rs:303) does O(n) linear scan.
`remove_node` (search.rs:310) does O(n) position search + O(n) shift.
`contains_node` (search.rs:100) is O(n). With `MAX_BOOTSTRAP_FRONTIER_CANDIDATES = 3`
(scan.rs:17), n is very small, so this is only relevant if frontier size grows.

### 9. Eliminate redundant `mse_code_len` / `qjl_code_len_for_bits` recomputation

`split_code_bytes` (prod.rs:532) recomputes `mse_code_len(self.original_dim, self.bits)`
and `qjl_code_len_for_bits(self.original_dim, self.bits)` on every call. These are
pure functions of `self` and could be precomputed as struct fields. Minor but called
hundreds of times per query.

### 10. Batch PG buffer reads for co-located element + neighbor tuples

When an element and its neighbor tuple are on the same page, the current code does two
separate `ReadBufferExtended` + lock/unlock cycles (graph.rs:618-665 called twice via
`load_graph_adjacency`). A combined read path that pins the buffer once and decodes both
tuples would halve the PG buffer manager overhead for co-located pairs.

### 11. Use FxHash/identity hash for ItemPointer keys

The visited/expanded/emitted HashSets and graph/score caches hash `ItemPointer`
(6 bytes). hashbrown uses AHash. A specialized identity hash
(`block_number * 65536 + offset_number`) could be zero-cost. Likely marginal.

## Assessment of in-progress heap-tid Vec removal

The existing work to remove a temporary Vec/copy in `SelectedScanResult`
materialization (scan.rs) is correct and clean but low-impact. `SelectedScanResult`
already uses an inline `[ItemPointer; 10]` array (line 1933).
`ScanResultState::store_pending` copies from this inline array to another inline
`pending_heaptids` array -- that is a ~60 byte memcpy at most, happening once per
result emission (10 times for LIMIT 10). Negligible relative to the ~4.5ms in
scoring or the ~0.6ms in allocations.

Recommendation: finish it as cleanup but do not prioritize it over SIMD scoring (#1).

## Recommended experiment sequence

1. **SIMD scoring (#1)** -- largest single-function win. Implement the AVX2
   `no_qjl_4bit` kernel first, benchmark. Expected: p50 drops by 3-4ms.
2. **Skip LUT build (#6)** -- 5-minute change, eliminates 96KB of wasted work.
   Do alongside #1.
3. **Inline heaptids in TqElementTuple/GraphElement (#4)** -- mechanical, low risk.
   Eliminates 2 Vec allocs per element decode.
4. **Lazy deletion in BeamSearch (#3)** -- fix the O(n^2) drain. Important for
   correctness-under-scale even if marginal at ef_search=40.
5. **Scratch buffers (#5)** -- wire through pre-allocated Vecs for neighbor loading.
   Eliminates ~120 allocs/query.
6. **Zero-copy page decode (#2)** -- highest complexity, tackle once #4 is done and
   the GraphElement struct is already changing.

After steps 1-2, re-profile. The perf distribution will shift: scoring shrinks, and
the remaining allocation/cache/buffer-manager costs dominate. Steps 3-6 address that
new profile.

## Checkpoint

- Code checkpoint: analysis-only, no code changes
- Baseline: `p50=14.315ms`, `p99=17.613ms` at `m=8`, `ef_search=40`, `warm-after-prime3`
- Perf evidence: `31.78%` scoring, `4.11%` extend_from_slice, `1.20%` read_page_tuple_bytes
- Packet status: open

## Exit criteria

- the survey is recorded as a durable reference for the next C1 implementation packets
- each idea includes file/function references, rationale, risk, and measurement method
- the recommended sequence is ordered by expected impact for warm steady-state latency
