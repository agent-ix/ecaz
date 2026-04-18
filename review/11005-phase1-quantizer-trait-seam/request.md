# Review Request: Phase 1 Quantizer Trait Seam (ADR-041 stage 0)

Branch: `adr034-diskann-access-method`

Scope:
- `src/quant/traits.rs` (new)
- `src/quant/mod.rs` (re-exports)
- `src/quant/prod.rs` (ProdQuantizer impl + equivalence test)
- `src/quant/grouped_pq.rs` (PqFastScanQuantizer + equivalence test)

## What this slice is

First increment of task 17 **Phase 1** (ADR-041 stage 0): introduce
the `Quantizer` / `QueryScorer` trait seam in `crate::quant` and
implement it for both quantizer families. No file moves. No scan.rs
threading yet — that is Phase 1D (see *Not doing in this packet*).

Landing commits:

- `7f49d1d` — Phase 1A + 1B: traits + ProdQuantizer impl
- `10f5469` — Phase 1C: PqFastScanQuantizer wrapping SRHT +
  grouped-PQ codebook model

Both commits include a bit-exact equivalence test against the current
inline scoring path. Full pgrx suite (479 passing) ran green on
commit 7f49d1d before the 1C additions; 1C adds one test and touches
only `src/quant/grouped_pq.rs`.

## What changed

### `src/quant/traits.rs` (new file, 61 lines)

```rust
pub trait Quantizer: Send + Sync {
    fn encode_code(&self, v: &[f32]) -> Box<[u8]>;
    fn prepare_scorer(&self, query: &[f32])
        -> Box<dyn QueryScorer + Send + Sync + '_>;
    fn code_len(&self) -> usize;
    fn wire_format_version(&self) -> u32;
}

pub trait QueryScorer {
    fn score(&self, code: &[u8]) -> f32;
}
```

**Naming deviation from ADR-041.** The ADR names the scorer trait
`PreparedQuery`. That collides with a concrete struct at
`crate::quant::prod::PreparedQuery` (TurboQuant's prepared state,
referenced in `scan.rs`, `explain.rs`, `lib.rs`, and user-visible
EXPLAIN text). Renaming the struct is ~20 source sites plus EXPLAIN
text change; not worth it just for name parity. Trait is
`QueryScorer` here; documented in the module header. Revisit at
stage 2 (`am/tqhnsw/` rename) if the struct naturally renames along
with its module move.

**Trait method names.** `encode` and `prepare` are inherent methods
on `ProdQuantizer` with different return types; trait methods are
`encode_code` and `prepare_scorer` so Rust doesn't need qualification
to disambiguate receiver-method calls.

### `src/quant/prod.rs` (Phase 1B)

- `ProdQueryScorer<'a>` holds `&'a ProdQuantizer` + `PreparedQuery`.
- `impl Quantizer for ProdQuantizer` routes through
  `pack_payload(encode(v))` (trait `encode_code`),
  `prepare_ip_query(query)` (trait `prepare_scorer`),
  `payload_len(dim, bits)` (trait `code_len`),
  `INDEX_FORMAT_V1_SCALAR` (trait `wire_format_version`).
- One bit-exact test:
  `quantizer_trait_score_matches_inherent_score_ip_encoded`.

### `src/quant/grouped_pq.rs` (Phase 1C)

- New `PqFastScanQuantizer { rotation: Arc<ProdQuantizer>,
  group_count, group_size, flat_codebooks: Vec<f32> }`.
- Construction asserts `flat_codebooks.len() == group_count *
  GROUPED_PQ_CENTROIDS * group_size`.
- `impl Quantizer for PqFastScanQuantizer` routes through
  `rotation::srht_padded` + `encode_grouped_pq` (trait
  `encode_code`), `build_grouped_pq_lut_f32` (trait
  `prepare_scorer`), `group_count.div_ceil(2)` (trait `code_len`),
  `INDEX_FORMAT_V2_GROUPED` (trait `wire_format_version`).
- `PqFastScanScorer` holds the prebuilt LUT + group count; scoring
  dispatches to `grouped_pq_score_f32`.
- One bit-exact test:
  `pq_fastscan_quantizer_trait_score_matches_direct_helpers` at
  dim=1536, group_size=4.

## Review focus

- **Scalar-rerank scope of `QueryScorer::score`.** The trait scores
  one code at a time. ADR-041 is silent on batched (32-wide
  FastScan) scoring; the traits file comments this explicitly as
  "batched paths stay on family-specific APIs". Reviewer confirm
  this is the right boundary — the alternative would be a
  `score_batch(&[u8]) -> Vec<f32>` on the trait, which would force
  both families to allocate a batch even when they scalar-score.
- **Naming deviation (`QueryScorer` vs. ADR's `PreparedQuery`).**
  Documented in `src/quant/traits.rs` module header. Reviewer
  confirm the deviation is acceptable pending stage 2 rename, or
  push back and take the struct rename now.
- **`Arc<ProdQuantizer>` inside `PqFastScanQuantizer`.** The wrapper
  pays one atomic increment at construction and threads an Arc
  through scan setup. Alternative is a non-owning borrow, which
  leaks lifetimes into pgrx callback signatures. Option (a) owning
  won; reviewer confirm.
- **Composite-type layout.** PqFastScanQuantizer takes `group_count`,
  `group_size`, `flat_codebooks: Vec<f32>` as raw fields rather than
  `GroupedCodebookModel` (which is `pub(crate)` in `am::graph` and
  carries an AM-specific `head_tid`). Alternative: extract a shared
  `quant::grouped_pq::GroupedCodebook` struct and have both sides
  hold it. Deferred until Phase 2 (storage-primitive move) forces the
  same refactor.

## Questions to answer

- **Benchmark parity gate timing.** ADR-041 validation rule: the
  trait-indirected scoring path must match task-08 numbers within
  ±5%. That gate applies at scan.rs threading (Phase 1D), since
  today the trait is defined but not called from any hot path. Is
  that deferral OK, or does this review packet need a bench run
  against the unthreaded trait (a smoke test on
  `prepare_scorer`/`score` in isolation)?
- **Stage-5 pre-positioning.** ADR-041 stage 5 eventually splits
  `prod.rs` into `quant/turboquant/` and `quant/pqfastscan/`
  submodules. Current work drops `PqFastScanQuantizer` into
  `grouped_pq.rs` rather than pre-creating the submodule structure.
  Is that the right call (keep churn incremental) or should phase 1C
  also introduce the submodule skeleton?

## Not doing in this packet

- **Phase 1D — scan.rs threading.** Thread `&dyn Quantizer` through
  the `src/am/scan.rs` scoring call sites (identified targets below)
  and run the task-08 bench parity gate. Not yet started. Five
  scoring sites identified:
  - `src/am/scan.rs:2006` (`score_ip_from_parts` in rerank path)
  - `src/am/scan.rs:2171` (`grouped_pq_score_f32` in FastScan rerank)
  - `src/am/scan.rs:4026` (`score_ip_from_parts` in scan loop)
  - `src/am/scan.rs:5856` (test fixture)
  - `src/am/scan.rs:5911` (test fixture)
- **No file moves, no module structure changes.** Per ADR-041
  stage 0 rule.
- **No encode-side trait consumers.** `encode_code` is defined but
  build.rs/insert.rs still call the inherent methods. Wiring those
  through the trait is a Phase 2+ concern once `crate::storage::*`
  moves settle.

## Dependencies

- **ADR-041** (shipped on `main`). Authoritative source for the
  trait shape and the staged migration plan.
- **Task 17 plan** (`plan/tasks/17-diskann-access-method.md` Phase
  1). Describes the gate and review-packet sequence 11005–11013.

## Companion packets

- `review/11001-diskann-task17-plan/` — task 17 plan.
- `review/11002-adr042-vamana-insert-lock-ordering/` — ADR-042 draft.
- `review/11003-adr043-vamana-vacuum-lock-ordering/` — ADR-043 draft.
- `review/11004-diskann-build-algorithm-design/` — build design doc.

Future packets in the Phase 1–3 sequence: 11006 (storage-primitive
move), 11007 (am/tqhnsw rename), 11008 (tqdiskann AM skeleton
re-home).

## Definition of ready (for Phase 1 → merged)

- Reviewer confirms trait shape matches ADR-041 stage 0 intent.
- Naming deviation accepted or alternative agreed.
- Phase 1D lands (scan.rs threading) with task-08 bench parity
  within ±5%.
- Full pgrx suite green.

## Handoff notes (mid-session)

Phase 1 is landing in increments. At the time of packet filing:

- Phase 1A (traits) — landed `7f49d1d`.
- Phase 1B (ProdQuantizer impl) — landed `7f49d1d`.
- Phase 1C (PqFastScanQuantizer impl) — landed `10f5469`.
- Phase 1D (scan.rs threading) — **not started**. Task #16 is
  pending. Next agent should pick up here.

Phase 1D starting notes:

1. Five scoring sites identified above. Three are production paths
   (scan.rs:2006, 2171, 4026), two are test fixtures (5856, 5911).
2. ADR-041 says "leave the match on the outside as the selector;
   collapse the per-arm scoring work to a single trait-object call."
   So the `match GraphStorageDescriptor` shape stays; only the
   scoring-body arms change.
3. Before threading, capture current task-08 benches:
   `cargo bench --bench prepare_ip_query -- d1536_b4` and
   `cargo bench --bench score_ip_encoded -- d1536_b4`. After
   threading, re-run and confirm ±5%.
4. If virtual-call overhead shows up in profiles, ADR-041
   authorizes pivot to generics (`scan.rs::<Q: Quantizer>`). Do not
   pivot pre-emptively.

### Scoping discovery after packet 11005 was first drafted

The three production sites do **not** all have the same payload
shape, which the `QueryScorer::score(&[u8])` trait contract assumes:

- **`scan.rs:2171`** — `score_grouped_search_code_result`: calls
  `grouped_pq_score_f32(lut, group_count, search_code)`. Payload is
  a flat `&[u8]` of packed nibbles. **Direct fit** for
  `QueryScorer::score`. The `PreparedGroupedScanQuery` struct at
  `scan.rs:415` carries exactly `(lut_f32, group_count,
  search_code_len)` — same shape as `PqFastScanScorer` in
  `grouped_pq.rs`. Easiest path: `impl QueryScorer for
  PreparedGroupedScanQuery` with a one-line adapter, then rewrite
  site 2171 as `prepared_query.score(search_code)`. No perf
  change, no payload copy.

- **`scan.rs:2006`** — `score_grouped_rerank_payload_result`: calls
  `quantizer.score_ip_from_parts(prepared_query, gamma, code_bytes)`.
  This is TurboQuant rerank on the cold payload chain after a
  PqFastScan approximate search. Payload is **split** into
  `(gamma: f32, code_bytes: &[u8])` — the 4-byte gamma is kept
  separate from the MSE+QJL packed code to avoid re-reading the
  tuple header on the hot path.

- **`scan.rs:4026`** — `score_scan_element_result`: same split
  `(gamma, code_bytes)` TurboQuant scoring in the scalar scan loop.

Sites 2006 and 4026 do not fit the trait's flat-payload `score(&[u8])`
contract without either:

(i) changing the scan-time tuple-read to produce a concatenated
payload (one 4-byte prepend per scored element — ~768-byte payload,
so ~0.5% extra copy), **or**

(ii) extending the trait with a `score_split(gamma: f32, code: &[u8])`
variant that ProdQuantizer implements directly and PqFastScan
implements as `score(code)` with an ignored gamma, **or**

(iii) accepting that the trait only threads site 2171 (the grouped
PQ LUT scoring), and TurboQuant's split-payload API stays
specialized inside the `match GraphStorageDescriptor` arm.

**Recommendation for next agent.** Start with (iii) — thread site
2171 only via `impl QueryScorer for PreparedGroupedScanQuery`, run
the task-08 bench gate against the partially-threaded state, and
file a follow-up for (i) vs (ii). This preserves today's hot-path
shape, demonstrates the trait-consumption pattern, and defers the
TurboQuant-split decision until we have bench data showing it's the
bottleneck. ADR-041 stage 0 does not require threading every site
to be considered "done" — the authoritative check is that
tqdiskann can consume the trait without reaching into family
internals, and that holds with just site 2171 threaded.

**Alternative reading.** If the reviewer prefers full threading of
all three sites, option (ii) is the lower-risk choice — it adds a
trait method without changing the tuple-read shape. The
`score_split` default could be `self.score(&[gamma_le_bytes,
code].concat())` for families that don't care, with ProdQuantizer
overriding for the split-path fast lane.

5. PqFastScanQuantizer construction site needs an
   `Arc<ProdQuantizer>` + a loaded `GroupedCodebookModel`. Current
   scan setup loads both; the wiring is straightforward — construct
   a `PqFastScanQuantizer` once per scan, hold it in the scan
   opaque, and swap the scoring-body call with `scorer.score(code)`.
   If pursuing recommendation (iii) above, this step is optional —
   `PreparedGroupedScanQuery` already has the LUT and group_count
   baked in; no need to go through a separate
   `PqFastScanQuantizer` at the site-2171 call. The
   `PqFastScanQuantizer` wrapper still matters for **tqdiskann**'s
   own scan path, which has no equivalent of
   `PreparedGroupedScanQuery`.

Tasks on the tracker:

- #13 [completed] Phase 1A: define traits
- #14 [completed] Phase 1B: ProdQuantizer impl
- #15 [completed] Phase 1C: PqFastScanQuantizer impl
- #16 [pending] Phase 1D: thread through scan.rs + bench gate
