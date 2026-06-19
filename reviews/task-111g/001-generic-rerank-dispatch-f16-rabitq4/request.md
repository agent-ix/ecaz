# Task 111g — Packet 001: Generic rerank_format dispatch + table-side f16/rabitq4

Status: **open** (awaiting reviewer gate).

Covers Task 111g Phases 1 and 2 (the code slices). Phase 3 (benchmark gate) is
a separate packet.

## Summary

The IVF `coarse_rerank` rerank stage is now **pluggable by `rerank_format`**
instead of being hardcoded to heap-f32. The stage still reads the heap source
column **tid-sorted** (unchanged IO shape); only the scoring representation
changes:

- `f32` — exact negative inner product on the fetched source vector.
  **Bit-identical to the pre-111g heap_f32 path** (equivalence test + the
  existing heap_f32 full-probe exact-score fixture both pass).
- `f16` — round the query and each fetched source vector to IEEE-754 binary16
  and back, then exact negative inner product. Hand-rolled f16<->f32 round-trip
  (no `half` crate, no SIMD).
- `rabitq4` — encode each fetched source with the shared 4-bit RaBitQ codec
  (`IvfQuantizer`) and score through the existing `candidate_batch` RaBitQ
  scorers (`score_ip_bits1_batch_from_payloads` → `score_rabitq_bitsn_batch_for`
  / the bits=4 arithmetic batch), negating the IP estimate. **No new SIMD
  kernels.**

`f16` is added to the `RerankFormat` enum (the packet-005 gap). The create-time
rejections for `f16` and `rabitq4` are lifted; `rabitq2` / `rabitq8` /
`turboquant` and `rerank_placement = 'index'` stay rejected. The admin snapshot
reports the new formats via the existing `reloption_name()` path.

## Acceptance criteria coverage

| AC | Status | Evidence |
| --- | --- | --- |
| 1. Dispatch on `rerank_format`; `heap_f32` bit-identical | met | `rerank.rs::tests::f32_scorer_matches_source_reference` (bit-equal vs `negative_inner_product_index_internal`); `pg_test_ec_ivf_heap_f32_rerank_full_probe_matches_exact_scores` still passes |
| 2. `f16` + `rabitq4` creatable; rejections lifted; `f16` in enum; admin reports them | met | options unit tests (accept f16/rabitq4, reject rabitq2/8/tq + index placement); `pg_test_ec_ivf_coarse_rerank_f16_rabitq4_admin_snapshot` |
| 3. Reuse candidate_batch scorers (no new SIMD) | met | `rerank.rs` rabitq4 path routes through `IvfQuantizer::score_ip_bits1_batch_from_payloads`; no new kernel code |
| 4. Table-side reads tid-sorted | met | `rerank_probe_candidates_table_side` sorts by `candidate_heap_tid_cmp` before fetch (unchanged from heap_f32) |
| 5. PG18 fixtures: recall parity (f16 ≈ f32) + rabitq4 correctness | met | `pg_test_ec_ivf_coarse_rerank_f16_matches_f32_ranking`, `pg_test_ec_ivf_coarse_rerank_rabitq4_returns_correct_top_neighbor` |
| 6. Benchmark packet | deferred | Phase 3, separate packet (50k/100k matrix + matched-recall baseline) |

## Design notes for the reviewer

- **No on-disk format change.** The rerank payload is always the heap source
  vector. f16/rabitq4 re-encode the fetched f32 vector on the fly per query and
  score it — this validates the scorer path and recall at the chosen
  representation without adding a sidecar. The packet-005 *byte savings* are a
  storage-side follow-up (a persisted compact sidecar); this slice realizes the
  *scoring representation* lever the 111e contract advertised. Flag if you want
  the persisted-sidecar bytes lever pulled into 111g instead of a follow-up.
- **rabitq4 routing.** `score_ip_bits1_batch_from_payloads` (despite the name)
  dispatches bits 1/2/4/8; bits=4 uses the arithmetic batch estimator per the
  Task 106 M5 routing decision (block kernel measured slower at bits=4). This is
  the existing IVF posting-scan path, reused verbatim — no new SIMD.
- **Index-side placement** stays rejected (out of scope per the spec
  Non-Goals).

## Validation (head aad8c3ccf, PG18)

- `cargo check --no-default-features --features pg18` — clean.
- `cargo clippy --no-default-features --features pg18 -- -D warnings` — clean.
- Unit: `am::ec_ivf::rerank` (7) + `am::ec_ivf::options` (11) — all pass.
- pg_test (PG18): the 3 new coarse_rerank fixtures + the existing contract
  fixture pass (15-test lib run); the 3 heap_f32 rerank fixtures pass
  (regression).

See `artifacts/manifest.md` and the cited logs under `artifacts/`.
