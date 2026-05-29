# Task 66: RaBitQ M5 NEON Optimization (+ Intel dispatch seam)

Status: **proposed** 2026-05-28.

Owner: coder (to be assigned). One coder, one branch.

## Why

`src/quant/rabitq.rs` is the scoring hot path for DiskANN-RaBitQ,
HNSW-RaBitQ, and IVF-RaBitQ. On M5 / Apple Silicon (aarch64,
NEON), the current NEON coverage is partial:

| bits/dim | M5 NEON kernel? | M5 fallback |
|---:|---|---|
| 1 | ✅ `sum_query_dequant_neon_bits1` (`rabitq.rs:1705`) | — |
| 4 | ✅ `sum_query_dequant_neon_bits4` (`rabitq.rs:1789`) | — |
| 2, 3, 5, 6, 7 | ❌ | `sum_query_dequant_scalar` (`rabitq.rs:1498`) |
| **8** | ❌ | `sum_query_dequant_scalar` |

The bits=8 gap is load-bearing: the active IVF-RaBitQ variants under
benchmark on M5 are **`rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`**
(per the `task51-aws-rabitq8*`, `task51-local-rabitq8*` benchmark
packets and `plan/tasks/51-ivf-rabitq-second-optimization-round.md`
exp 7). All four share the same bits=8 code layout and route through
the same `sum_query_dequant_with_bf16` dispatch, so a single NEON
bits=8 kernel benefits every active 8-bit variant.

Additional NEON gaps observed during the Task 65 close-out review:

1. **`estimate_ip_bits1_batch` is fake-batched** (`rabitq.rs:933-973`).
   It loops calling `estimate_ip_scalar_only_impl` per code, re-reading
   the prepared query state on every call instead of hoisting it.
2. **No software prefetch** in any NEON kernel. RaBitQ codes are
   tiny (~200 bytes at d=1536 bits=1, ~1.5 KB at bits=8) and the
   DiskANN/IVF scan access pattern is indirect, so a `prfm pldl1keep`
   ahead of the next code's address would hide L2 misses.
3. **bf16 path is feature-gated and untested on M5.** The
   `rabitq-bf16` Cargo feature gates `sum_query_dequant_neon_bf16_bits4`
   (`rabitq.rs:1462-1475`), with a comment noting the kernel measured
   "neutral-to-slightly-slower vs f32 NEON" on **Neoverse-V2**. M5's
   bf16 microarchitecture differs — this needs an M5-specific
   measurement before assuming the gate-off applies.

This task also lays the dispatch-and-test groundwork that Task 67
(Intel AVX-512/AVX2) will plug into, so the Intel task is purely
about kernel implementation rather than dispatch plumbing.

## Goal

Close every NEON-side scoring path that the active M5 deployments
exercise, in priority order of measured impact. Establish the
shared dispatch + differential-test seam that Task 67 will reuse.

## Scope

### In scope

1. **NEON bits=8 kernel** with arithmetic-dequant rewrite (see
   below). Covers `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
   and any future bits=8 variant.
2. **True batched scoring kernel** for bits=1 and bits=8 — a
   `score_batch(codes: &[u8], code_len: usize, out: &mut [f32])`
   path that hoists the prepared query state once per call and
   loops over candidates inside the NEON kernel. Replaces the
   current `estimate_ip_bits1_batch` loop-of-scalar implementation.
3. **Software prefetch** added to the bits=1 and bits=8 NEON
   kernels — `vprfm` (`std::arch::aarch64::vprfm_*`) issued one or
   two cache lines ahead of the next code-byte chunk.
4. **bf16 path re-measurement on M5** — under the `rabitq-bf16`
   feature, measure on M5 specifically. If neutral or worse,
   document the M5 finding next to the existing Neoverse-V2
   comment. If a win, flip the default gate condition.
5. **Shared dispatch shape** for the cross-arch fan-out. Today
   `sum_query_dequant_with_bf16` (`rabitq.rs:1428`) has
   `target_arch = "aarch64"` blocks inline; refactor so each
   per-arch kernel is a small fn behind a uniform internal trait
   or fn pointer, and the cross-arch dispatcher picks one at
   runtime. The shape MUST accommodate AVX-512 / AVX2 variants
   that Task 67 will add without further plumbing changes.
6. **Differential-test scaffold** — extend the existing
   `*_for_test` helpers (`sum_query_dequant_*_neon_for_test`,
   etc.) so every kernel has a paired scalar reference and a
   property test that hammers random query + code inputs for
   bit-exact agreement modulo documented tolerance. Task 67
   will add Intel kernels to this same scaffold.
7. **Bench harness extension** — a focused `cargo bench` (or
   ecaz CLI sweep) over the four 8-bit variants on M5, comparing
   pre- and post-task wall time, recall delta, and per-kernel
   throughput. Output lands in a packet under `reviews/task-66/`.

### Out of scope

- Intel AVX-512 / AVX2 kernels — Task 67.
- ARM SVE / SVE2 — M5 doesn't have SVE.
- Changes to the RaBitQ encode/quantize math — scoring kernel only.
- bits ∈ {2, 3, 5, 6, 7} kernels — none of the active deployments
  use these bit depths. Add only if measurement evidence shows a
  consumer that needs them.
- Changes to the prepared-query allocation path (`PreparedEstimator`
  construction) beyond the bits1_byte_lut allocation moved into
  it as part of #2.

## Arithmetic-dequant rewrite — why bits=8 is small

Reading `dequant_level` (`rabitq.rs:1282-1295`):

```rust
fn dequant_level(level: u32, bits: usize, sqrt_d: f32, quant_clip: f32) -> f32 {
    let levels = 1_u32 << bits;
    let center_scaled = (level as f32 + 0.5) / levels as f32 * (2.0 * c) - c;
    center_scaled / sqrt_d
}
```

This factors to `dequant(level) = level * scale + offset` for
constants `scale`, `offset` that depend only on `(bits, sqrt_d,
quant_clip)`. The 256-entry `dequant_lut[256]` lookup at
`sum_query_dequant_scalar:1508` is unnecessary — for bits=8 the
dequant is a pure arithmetic function of the code byte.

So the bits=8 NEON kernel is purely arithmetic; no NEON gather
needed (NEON has no scatter/gather instruction):

1. `vld1q_u8(code_ptr)` — load 16 code bytes per outer iteration.
2. Widen via `vmovl_u8` → `uint16x8_t × 2`, then
   `vmovl_u16` + `vcvtq_f32_u32` → `float32x4_t × 4`.
3. `vfmaq_f32(offset_vec, code_f32, scale_vec)` — produce 16
   dequant values arithmetically.
4. `vfmaq_f32(acc, dequant, query)` — accumulate inner product.

Four-way unroll mirrors the existing bits=1 and bits=4 kernels
(`rabitq.rs:1697`-style 4-pipe layout).

Further optimisation worth measuring: precompute query-side
`query_scale[i] = query[i] * scale` and `query_offset[i] = query[i]
* offset` once per query (d=1536 vectors stored in the
`PreparedEstimator`). The per-candidate kernel collapses to:

```
acc += code_byte_as_f32 * query_scale + query_offset
```

One FMA + one add per dim. Composes naturally with the batched
scoring kernel from item #2.

## Slice plan

- **Slice A — dispatch refactor + test scaffold.** Reshape
  `sum_query_dequant_with_bf16` and `estimate_ip_*_impl` so each
  per-arch kernel sits behind a uniform internal seam. Extend
  the `*_for_test` differential-test helpers. No new kernels yet;
  byte-equal output vs current head.
- **Slice B — NEON bits=8 arithmetic-dequant kernel.** Land
  `sum_query_dequant_neon_bits8` with the rewrite above. Wire
  into the dispatch from Slice A. Differential test vs scalar
  for random inputs, dim 64/256/1536, levels 0..256.
- **Slice C — query-side precompute on `PreparedEstimator`.**
  Add `query_scale: Vec<f32>` / `query_offset: Vec<f32>` to
  the prepared state for bits=8 paths. Confirm `bits1_byte_lut`
  per-query allocation can move here too. Single-candidate
  benchmark on M5: bits=8 should drop substantially vs Slice B.
- **Slice D — true batched scoring.** Replace
  `estimate_ip_bits1_batch` with a NEON-batched kernel that
  iterates candidates inside the SIMD loop. Add an equivalent
  bits=8 batched path. Public API matches the existing batch
  signature (slab in, scores out).
- **Slice E — prefetch.** Add `vprfm pldl1keep` to the bits=1
  and bits=8 NEON kernels, one cache line ahead. Measure on M5
  vs no-prefetch.
- **Slice F — bf16 M5 re-measurement.** Under the `rabitq-bf16`
  feature, run the bits=4 bf16 NEON kernel on M5 and compare
  vs f32 NEON. Document the finding. Flip the default gate
  only if a measured win.
- **Slice G — Intel readiness gate.** Confirm the dispatch
  shape from Slice A accepts a hypothetical
  `sum_query_dequant_avx512_bits8` slot without further
  plumbing. Land any final tweaks. Bench harness ready to
  measure pre/post Task 67 on the Intel host.
- **Slice H — measurement packet.** M5 release-mode wall time
  + recall delta + per-kernel throughput across all four bits=8
  variants. Static unsafe audit on the new NEON kernels.

## Validation gate

1. **Functional.** All existing `cargo test -p ecaz quant::rabitq`
   tests pass. New differential tests for every NEON kernel pass
   with documented tolerance.
2. **Recall.** No regression on real-10k or whatever benchmark
   fixtures exercise the bits=8 IVF-RaBitQ scoring path. Within
   0.5 pp at all list sizes.
3. **Performance.** On M5: bits=8 single-candidate score throughput
   ≥ 3× current scalar fallback. Batched bits=1 score throughput
   ≥ 2× current per-candidate loop. Headline gates published in
   the packet.
4. **Safety.** Per memory `feedback_dont_defer_safety_fixes` and
   `feedback_anti_pattern_b_unbounded_lifetime`:
   - Every new `unsafe fn` has a `# Safety` doc.
   - No `fn(*mut T) -> &'a T` lifetime-laundering.
   - Each new NEON kernel ships behind runtime
     `is_aarch64_feature_detected!` plus a paired SAFETY comment
     citing the feature gate.
5. **Determinism.** Within documented float-rounding tolerance,
   NEON and scalar kernels agree on random inputs. The
   differential test scaffold catches drift.

## Cross-task interface (for Task 67)

Task 67 (Intel AVX-512 / AVX2 RaBitQ) depends on this task's:

- **Dispatch shape** — a Slice-A-defined internal fn-pointer or
  per-arch trait that Intel kernels register against without
  modifying `sum_query_dequant_with_bf16` or
  `estimate_ip_*_impl`.
- **Differential-test scaffold** — Slice A's
  `*_for_test`-style helpers; Intel kernels will add
  `*_avx512_for_test`, `*_avx2_for_test` variants.
- **Bench harness** — Slice G's harness re-runs on the Intel
  benchmark host with the Task 67 kernels swapped in.

Task 67 MUST NOT need to touch `rabitq.rs` outside the slot
mechanism Slice A defines. If it does, Slice A is incomplete and
this task is not done.

## Coder workflow notes

- **Branch off main** after Task 65 closes.
- **Each slice = one commit + one packet.** Differential test
  evidence and the focused measurement go into the packet's
  `artifacts/`.
- **No new `unsafe` outside `target_feature` kernel bodies.**
- **macOS pgrx-test note** — RaBitQ scoring tests are pure
  `cargo test` (no pgrx callbacks), so they run cleanly on M5
  per memory `feedback_dyld_buffer_blocks_known`.
- **No `/tmp` benchmark logs.** All packet evidence under
  `reviews/task-66/{NNN}-{slug}/artifacts/` per
  `spec/non-functional/NFR-007-benchmark-provenance.md`.

## References

- Quantizer source: `src/quant/rabitq.rs`
- Task 65 closure review (carries the gap analysis):
  `reviews/task-65/002-vamana-core-measurement/feedback/2026-05-28-03-reviewer.md`
- IVF RaBitQ optimisation context:
  `plan/tasks/51-ivf-rabitq-second-optimization-round.md`
- IVF RaBitQ8 benchmark history:
  `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/manifest.md`,
  `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/manifest.md`,
  `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/manifest.md`,
  `benchmarks/task51-local-rabitq8ls-sidecar/manifest.md`
- SIMD modernisation umbrella: `plan/tasks/21-simd-modernization.md`
  (this task does not block on 21; the AVX-512 line item there is
  fulfilled by Task 67).
- Follow-on: `plan/tasks/67-rabitq-intel-avx-optimization.md`

## Acceptance criteria

1. Slices A–H all landed, each with its own commit + packet.
2. Validation gate passes on M5.
3. Measurement packet (Slice H) documents pre/post throughput
   and recall delta on at least `rabitq8`, `rabitq8ls`,
   `rabitq8c3`, `rabitq8c4`, and the bits=1 batched path.
4. Dispatch shape from Slice A is ready for Task 67 to consume.
5. No regression in DiskANN, HNSW, or IVF scan tests.

## Estimated size

Medium. NEON kernel work is well-trodden; the dispatch refactor
+ test scaffold is the load-bearing piece. Expect 2–3 weeks for
a single coder including measurement.
