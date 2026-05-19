# Task 50 Consolidated Candidates

Ranking formula:

```text
(unsafe blocks absorbed * callsite count * cross-AM applicability)
/ (implementation risk + performance risk)
```

Product priority modifies ties: RaBitQ, SPIRE, and IVF outrank cleanup-only
HNSW/DiskANN applications unless the HNSW/DiskANN work is required to prove a
shared helper.

## 1. Cross-AM AM Callback Wrapper Helper

Source evidence:

- Task 35 closeouts: SPIRE 083, HNSW 104, DiskANN 107, IVF 122.
- Task 35 test sweep: packets 108-118 prototyped the macro-consolidation
  pattern ten times.
- Current direct count: dozens of `pgrx_extern_c_guard` sites across
  `src/am/ec_ivf`, `src/am/ec_spire`, `src/am/ec_diskann`, `src/am/ec_hnsw`,
  and `src/am/common`.

Plan:

Create a small callback helper or macro, likely in `src/am/common`, that
centralizes `pgrx::pgrx_extern_c_guard` and the callback-duration pointer
contract. It should support both closure callbacks and zero-argument callback
function pointers used by tree-height/parallel-descriptor paths. The first
production rollout should hit IVF, SPIRE, and RaBitQ-adjacent shared callback
surfaces before broad HNSW cleanup.

Expected payoff:

High. Many repeated unsafe blocks disappear into one helper contract, with low
runtime risk if the helper is `#[inline]` and preserves the existing closure
shape.

## 2. IVF Page Tuple And Posting-Range Visitors

Source evidence:

- IVF closeout 122 names the page tuple line-pointer chain and posting-list
  block-range traversal as separate structural candidates.
- Current direct count: `src/am/ec_ivf/page.rs` has 134 unsafe blocks and
  `src/am/ec_ivf/scan.rs` has 102.

Plan:

Introduce typed tuple/read visitors that own the line-pointer validation and
yield borrowed tuple views scoped to the locked buffer. For IVF, include the
posting-list block range as a second helper only after the tuple visitor lands.
The first target should be RaBitQ-relevant IVF scan/build paths because those
will feed optimization profiling.

Expected payoff:

High for IVF and RaBitQ readiness. Also establishes a pattern HNSW and SPIRE
page code can consume later.

## 3. SPIRE Active Epoch Anchor

Source evidence:

- SPIRE closeout 083 names `ActiveEpochAnchor` as the recurring invariant:
  live index relation, root control page, active epoch, manifest set, placement
  directory, and local store config.
- Current direct count: SPIRE production hotspots include
  `src/am/ec_spire/dml_frontdoor/mod.rs` at 160,
  `coordinator/hierarchy_snapshots.rs` at 71,
  `coordinator/snapshots.rs` at 62, and `page.rs` at 58.

Plan:

Create a typed context that proves the active epoch chain once and hands safe
borrows/owned snapshots to read-efficiency, coordinator, and diagnostic paths.
Start with production read paths that are closest to Task 30 phase 13d
observability, not DML frontdoor, because SPIRE read performance is the
product-differentiating target.

Expected payoff:

High for SPIRE safety and future optimization work. Implementation risk is
moderate because the context crosses several modules and should be sliced
narrowly.

## 4. Cross-AM Heap Source Scorer Helper

Source evidence:

- HNSW closeout 104 and IVF closeout 122 name the shared
  heap-relation + snapshot + reusable-slot scorer chain.
- SPIRE closeout 083 includes heap-rerank relation fallback and relation
  guards in the same invariant family.
- DiskANN vacuum/backlink planning also repeatedly loads heap source vectors.

Plan:

Create an owned scorer object that holds heap relation, snapshot, reusable slot,
attribute resolution, and score scratch. First apply it to IVF insert/vacuum
and SPIRE heap-rerank/read paths; then consume it from HNSW and DiskANN.

Expected payoff:

High cross-AM safety payoff, but performance risk is higher than the callback
wrapper because accidental allocation or extra vector copying would directly
hit scan/rerank paths.

## 5. Reloption Offset And NUL-Terminated String Wrapper

Source evidence:

- IVF 122 names the reloption offset + NUL-terminated string chain.
- SPIRE 083, DiskANN 107, and HNSW 104 reference the same reloption/read
  boundary family.

Plan:

Create typed reloption views per AM over the PostgreSQL `rd_options` blob, with
all offset and C-string decoding localized. Start with IVF/SPIRE reloptions
that govern RaBitQ and production SPIRE behavior.

Expected payoff:

Medium-high. Cross-AM but less urgent than scan/page/read-path helpers.

## 6. Exclusive Buffer And WAL Transaction Pair

Source evidence:

- IVF 122 names the append range + exclusive-lock + WAL chain.
- HNSW 104 names metadata/data page rewrites through `GenericXLogTxn`.
- DiskANN 107 names page/WAL writes.

Plan:

Add a closure-style helper tying exclusive buffer lock, generic WAL
registration, mutable page access, and `finish()` together. Apply first to IVF
posting append/mutation paths, then HNSW/DiskANN writes.

Expected payoff:

Medium-high with moderate correctness risk. This is valuable but should follow
the read-side visitor work.

## 7. Vector Datum Detoast/Slice Wrapper

Source evidence:

- DiskANN 107 names vector Datum detoasting, varlena layout, and `f32` slice
  construction.
- IVF build/insert and RaBitQ input paths depend on the same vector source
  contract.

Plan:

Create a safe `EcVectorDatum`/slice wrapper that validates layout and exposes
dimensioned `&[f32]`. Apply to IVF/RaBitQ build and insert paths first, then
DiskANN.

Expected payoff:

Medium. Important for RaBitQ safety, but the surface is narrower than callbacks
or page visitors.

## 8. SIMD Load/Store Newtypes

Source evidence:

- DiskANN 107 and quant packets 022-023 document target-feature dispatch and
  lane load/store bounds.
- `src/quant/hadamard.rs` remains a top-15 unsafe-block file at 62 blocks.

Plan:

Wrap lane-sized loads/stores and scalar tails in small target-feature-specific
types. Treat this as a RaBitQ/quant profiling prerequisite only after the IVF
page/callback groundwork lands.

Expected payoff:

Medium. Perf risk is nonzero; requires benchmark evidence from quant and AM
scan lanes.

## 9. DSM Atomic Field Wrapper

Source evidence:

- HNSW closeout 104 names PostgreSQL atomic load/store/exchange calls in
  parallel build DSM state.
- `src/am/ec_hnsw/build_parallel.rs` has 203 unsafe blocks.

Plan:

Introduce typed atomic fields for DSM layout. Defer until after priority
IVF/SPIRE/RaBitQ safety work unless Task 39/47 work exposes parallel build as
a gating issue.

Expected payoff:

High for HNSW, low for immediate product priority.
