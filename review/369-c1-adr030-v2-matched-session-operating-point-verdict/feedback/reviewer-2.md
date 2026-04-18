## Feedback 2: ADR-030 v2 Matched-Session Operating-Point Verdict

Follow-up to `reviewer-1.md` on the same packet. That first
pass flagged the recall reconciliation and the truncated
`ef_search` sweep but did not name an architectural lever that
would change the verdict. This second pass does.

Read `src/am/scan.rs` around the grouped rerank seam
(1663-1671, `grouped_candidate_rerank_comparison_score`), the
cold rerank payload scorer at 1567-1574
(`score_grouped_rerank_payload_result` — note the
`score_ip_from_parts` call: this is the 4-bit scalar scorer,
not an exact f32 path), the live rerank window at 2440-2489
(`prefetch_next_grouped_windowed_graph_result`), the heap-side
infrastructure at `src/am/scan_debug.rs:846`
(`debug_profile_ordered_scan_with_heap_fetch` — generic
executor-API heap fetch, already benchmarked in packet 366),
`spec/adr/ADR-018-hnsw-quantized-graph-quality.md:16-20` and
`:136`, and the active plan at
`plan/tasks/14-adr030-v2-grouped-index.md:30-36`.

### The verdict is right about the ceiling; wrong about what caused it

The packet frames the recall gap as "tqvector does not reach
pgvector's measured recall floor on this lane" and leaves the
next step as a product choice: push recall higher vs. frame
grouped-v2 as latency-first only. That framing treats the
recall ceiling as a property of tqvector.

It is not. The ceiling is a property of the **payload stack**,
not of the system:

- hot traversal: grouped PQ4, 48 B/vec, 4 bits per subvector
- binary sidecar: 1 bit per dim
- "cold rerank": **still quantized** — `scan.rs:1573` routes
  through `quantizer.score_ip_from_parts(prepared_query,
  rerank_gamma, &rerank_code)`, i.e. the existing scalar 4-bit
  tqvector code path

Every stage the scan touches discards information. The
highest-fidelity representation tqvector holds anywhere in the
scan path is 4 bits/dim. pgvector holds raw f32. That's a
~5 bits/dim gap per coordinate that is structurally impossible
to recover from inside the current layout. No amount of
`ef_search` increase, no OPQ swap, no better beam scheduler,
and no PQ8 cold payload closes 5 pp of recall — they each buy
1-2 pp at best. The curve in packet 363 is already flattening
(`ef=128 -> 320` gained 0.006 of recall) precisely because the
quantized scorer is running out of signal to spend `ef_search`
on.

So the packet's framing — and coder-1's "nearing the end of
the line" read — is correct as a statement about the current
architecture but not as a statement about the system's
potential. There is one underexploited architectural lever
sitting in the tree.

### The lever

**Exact f32 rerank from the heap on the final survivors**,
instead of (or in addition to) the stored quantized cold
rerank payload.

Concretely:

1. Run the existing pipeline (`binary prefilter -> grouped
   FastScan -> live rerank window`) with a slightly larger
   window, e.g. window = 3x the SQL `LIMIT`
2. For those final ~30 survivors, fetch the f32 source column
   from the heap via the already-wired executor-API path used
   by `debug_profile_ordered_scan_with_heap_fetch`
3. Compute the exact inner product on f32 for those ~30 rows
4. Emit top-k ordered by the exact score

This is not a novel pattern. `ADR-018:16-20` already lists it
as one of three legitimate graph-compression/search shapes:

> **Build compressed, rescore top-k** | Quantized distances |
> Over-fetch compressed → rescore with raw | Elasticsearch BBQ,
> Weaviate v1.21+, DiskANN search

`ADR-018:136` references DiskANN directly for this approach.
So the pattern is documented and used in production systems.
It just has not been picked up as an active packet for
tqvector — the current plan at
`plan/tasks/14-adr030-v2-grouped-index.md:30-32` commits to a
**stored quantized** rerank tier ("existing scalar 4-bit
payload ... with room for a later PQ8/residual rerank
payload"), which keeps every stage quantized and inherits the
same ceiling.

### Why this lever is viable now

Three pieces of the groundwork are already in place:

1. **Heap-fetch seam exists.** `scan_debug.rs:846` is a
   working generic-executor-API path that opens the heap
   relation, pushes a registered snapshot, and fetches tuple
   slots through `index_getnext_slot`. Packet 366 already
   measured slot-fetch totals in the 0.008-0.034 ms range and
   executor-like totals tracking internal AM totals within
   noise. The infrastructure is proven; only the wiring
   differs.
2. **Cold rerank seam exists and is query-scoped.**
   `grouped_candidate_rerank_comparison_score` at
   `scan.rs:1663-1671` is already the single dispatch point
   the live rerank buffer consults
   (`prefetch_next_grouped_windowed_graph_result:2508-2514`).
   Replacing the scorer behind that seam with a heap-fetch
   path does not require restructuring the live rerank window,
   the beam scheduler, the visible frontier, or any of the
   scan state machine in `scan.rs`.
3. **Source column is already persisted.** `build.rs:33`
   (`source_vector: Option<Vec<f32>>`) and `build.rs:310-338`
   establish that the build path expects the f32 source column
   in the heap. Both pgvector and tqvector users keep it there
   for graph build; pgvector additionally duplicates it into
   the index, tqvector does not. The lever does not add
   storage — it uses storage the user is already paying for in
   the heap.

### The expected shape

Order-of-magnitude, before measurement:

| dimension | current grouped-v2 | with heap-fetch rerank | pgvector m=16 |
|---|---|---|---|
| index size | 65 MB | 65 MB | 391 MB |
| total disk (heap + index) | ~365 MB | ~365 MB | ~690 MB |
| SQL mean ms (ef=128) | 2.163 | ~2.5-4 (estimate) | 3.101 |
| top-10 recall | 0.9400 | ≈ pgvector (hypothesis) | 0.9980 |

The product claim shifts from "narrow sub-1.6 ms latency
pocket" to **"pgvector-quality recall at pgvector-range latency
with 6x smaller index and ~2x smaller total disk."** That is a
categorically different pitch than packet 369's current verdict
and worth the feasibility spike before the branch commits to
latency-first framing.

### Why coder-1 probably missed this

Coder-1's "find a different storage/search composition that
preserves the low-latency advantage while narrowing the recall
gap" is pointing in roughly this direction but stops short of
naming the mechanism. That omission is consistent with the
active plan's commitment to stored-quantized rerank as the
"cold" tier — if you are anchored to the
`scalar-4-bit-now-PQ8-later` trajectory in
`plan/tasks/14-adr030-v2-grouped-index.md:31-32`, the exit
from the quantization-bound ceiling is not obvious because
every tier in the planned pipeline is inside the quantized
budget. The heap-fetch alternative only becomes visible once
you notice (a) `ADR-018:16-20` already lists it as a known
pattern and (b) the f32 data is already in the heap because
the build path needs it there.

### Concrete next packet proposal

One focused feasibility spike, narrower than a full pipeline
change:

1. add a heap-fetch variant of
   `grouped_candidate_rerank_comparison_score` behind a gate
   (env var or opt-in reloption, same shape as the existing
   `TQVECTOR_EXPERIMENTAL_ADR030_V2_*` family in
   `scan.rs:22-33`)
2. inside that variant, reuse the executor-API pattern from
   `scan_debug.rs:846` to fetch the f32 source attnum for each
   survivor in the live rerank window
3. score `-dot(query_f32, heap_f32)` and let the existing
   `pop_best_buffered_grouped_scan_result` at `scan.rs:2349`
   rank by the exact score
4. measure on the same isolated grouped `m=16` lane + same
   `tqhnsw_real_50k_queries_50` subset as packets 362/363/368:
   - Recall@10 per `ef_search`
   - SQL mean and p95 in `per-cell plain-server` mode
   - rerank window sweep ({10, 20, 30, 50})
5. cross-table the result against packet 368's matched-session
   pgvector numbers and packet 369's cross-tables, using the
   reconciled recall basis from concern #1 of reviewer-1

Gate criteria:

- Recall@10 at tqvector `ef=128, window=30` should land within
  0.01 of pgvector `ef=64` (0.9920). If it does not, the
  survivor pool is not comprehensive enough and needs a wider
  ef sweep or a stronger first-stage.
- SQL mean at the same cell should land under ~4 ms.
  Otherwise the heap-fetch tier costs more than the recall
  lift is worth at that `ef_search` point.

If both gates hold, the operating-point verdict in this packet
changes qualitatively — not just quantitatively — and the
active plan at `plan/tasks/14-adr030-v2-grouped-index.md:30-36`
needs its rerank-payload framing updated to treat stored
quantized rerank as one option among two rather than the only
one.

### Assumptions and failure modes

- **Source column retained in heap.** The pitch assumes the
  user keeps the f32 column. Some vector-index workloads drop
  the source column post-build to save heap space. For those
  callers the lever does not apply and quantized rerank is the
  only path — so the framing should be "optional exact tier
  when source column is present" rather than "default rerank
  tier."
- **Cold cache is worse than packet 366 measured.** Packet
  366's heap-fetch numbers were warm-cache. A cold-cache top-30
  fetch is 30 random 8 KB I/Os — that could add milliseconds,
  not microseconds. The spike must measure both warm and cold.
- **Survivor pool quality.** Heap rerank only helps if the
  quantized pipeline's window-of-30 actually contains the true
  top-10. At small `ef_search` that may fail. The existing
  binary+grouped capture study in `src/bin/approx_score_study.rs`
  already has the capture-fraction metric
  (`capture_fraction`, `src/bin/approx_score_study.rs:1312`) —
  reuse it to confirm capture ≥ 0.99 before claiming the
  recall lift is real.
- **TID stability across concurrent writes.** Fetching via
  TID that was valid at index-read time but now points at a
  HOT-updated tuple needs the same snapshot discipline as
  `debug_profile_ordered_scan_with_heap_fetch` — register a
  snapshot once per scan, not per candidate. The debug helper
  already demonstrates the right pattern; the runtime variant
  has to preserve it.

### Final framing

This is not a recommendation to land the heap-fetch rerank
path today. It is a recommendation that packet 369 should not
be read as "tqvector has hit its recall ceiling" — it has hit
*this payload stack's* recall ceiling, which is a much weaker
claim. Before the branch commits to the "latency-first only"
product framing at `369:199-205`, the feasibility spike above
should run. The cost is one packet; the upside is a
categorically different product claim that neither the plan
nor coder-1's framing currently has on the table.
