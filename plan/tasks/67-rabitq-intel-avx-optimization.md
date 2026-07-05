# Task 67: RaBitQ Intel AVX-512 / AVX2 Optimization

Status: **complete** (2026-06-16) — the RaBitQ Intel AVX-512/AVX2 kernels
(bits=1 VPOPCNTDQ + AVX2 popcount fallback, bits=4 nibble FMA, bits=8
arithmetic-dequant, batched paths) are landed on `main`
(`src/quant/rabitq.rs`) with full local + AWS measurement across packets
`reviews/task-67/001`–`041`. Built on Task 66's dispatch shape and
differential-test scaffold.

## Amendment — 2026-05-30 Measured Closeout Scope

Task 67's original SQL headline thresholds were written before the
Intel host measurements decomposed the wall-time budget. The completed
packets show the new kernels meet the kernel-layer targets, but the SQL
headline is dominated by candidate SQL / sidecar I/O rather than RaBitQ
scoring:

- Packet 020 / 023: kernel-layer bits=1/4/8 SIMD speedups meet or
  exceed the per-kernel targets on the Intel `10k-intel` lane.
- Packet 022 / 026: bits=1 SQL headline reaches the 3x threshold only
  at the recall-preserving operating point (`nprobe=64`), with
  `nprobe=16/32` remaining below 3x because the residual bottleneck is
  outside the RaBitQ scoring kernel.
- Packet 027: bits=8 SQL headline evidence covers `rabitq8`,
  `rabitq8ls`, `rabitq8c3`, and `rabitq8c4`; the strict 4x SQL
  threshold is not met, topping out at 1.09x because sidecar scoring is
  roughly 1% of total wall time.
- Packet 029: `rabitq-bf16` preserves recall but slows the SQL lane, so
  the feature gate stays off by default.

For closeout, Task 67 accepts this measured outcome:

1. The kernel implementation work is complete when the AVX-512 and AVX2
   kernels are landed, registered in the differential-test scaffold, and
   the Intel AVX-512 kernel benchmark packet shows the original
   per-kernel targets are met.
2. SQL headline work is complete when packet-local `ecaz bench suite`
   evidence documents the bits=1 and bits=8 pre/post wall time, recall,
   and bottleneck attribution. The original 4x bits=8 and all-nprobe
   3x bits=1 SQL thresholds are no longer closeout blockers after the
   accepted evidence shows further scorer work cannot materially move
   those SQL metrics.
3. The bits=1 SQL gate is interpreted at the recall-preserving
   operating point (`nprobe=64`) for this task's closeout.
4. The AVX2 fallback code remains required and must be covered by the
   differential-test scaffold. Dedicated AVX2-only host benchmark
   evidence is not required for closeout unless an AVX2-only Intel host
   becomes available before closure.
5. The bf16 feature remains optional and disabled by default unless a
   future packet outside Task 67 demonstrates a win.

Owner: coder (to be assigned). One coder, one branch.

## Why

`src/quant/rabitq.rs` has **zero x86 SIMD**. Every `target_feature`
/ `target_arch` block in the file is `aarch64`. On Intel hosts —
both the local benchmark Intel laptop and any Intel cloud lane —
RaBitQ scoring falls through to scalar fallbacks at
`sum_query_dequant_scalar` (`rabitq.rs:1498`),
`estimate_ip_scalar_only_impl` (`rabitq.rs:2014`), and
`estimate_ip_least_squares_scalar_only_impl` (`rabitq.rs:2073`).

This silently disadvantages RaBitQ in every Intel benchmark vs
PqFastScan and TurboQuant (which have AVX2+FMA kernels via the
existing Task 29d + Task 21 work). The Task 60 / Task 63 RaBitQ
benchmark comparisons against PqFastScan on Intel hosts are
running uphill until this lands.

The Task 21 SIMD modernisation umbrella mentions AVX-512 VPOPCNTDQ
"for binary-sidecar scoring (ADR-031 path)" at
`plan/tasks/21-simd-modernization.md:45`, but its scope is the
PqFastScan binary sidecar, not RaBitQ. The popcount kernel shape
is the same (XOR + popcount + reduce); the consumer is different.

## Goal

Close every Intel-side scoring path that the active RaBitQ
deployments exercise, matching the M5 NEON kernel coverage that
Task 66 lands. Use Task 66's dispatch seam — do NOT modify the
shared scoring fan-out beyond registering new per-arch kernels.

## Scope

### In scope

1. **AVX-512 bits=1 kernel** using `VPOPCNTDQ`
   (`_mm512_popcnt_epi64`) for the sign-popcount inner product.
   Mirrors the NEON `sum_query_dequant_neon_bits1` shape with
   512-bit accumulators and a wider unroll appropriate to Intel
   SIMD pipe count. Runtime gated on `avx512f` + `avx512vpopcntdq`
   feature detection.
2. **AVX2 bits=1 fallback** for Intel hosts without
   `avx512vpopcntdq`. Uses the Mula bit-shuffle popcount idiom
   (`pshufb` 16-entry nibble LUT) or `_mm256_sad_epu8` reduction.
3. **AVX-512 bits=4 kernel** mirroring
   `sum_query_dequant_neon_bits4`. 512-bit FMA accumulators,
   nibble-unpack via `_mm512_srli_epi16` + mask.
4. **AVX2 bits=4 fallback**. 256-bit FMA accumulators with the
   AVX2 nibble-unpack pattern already used by PqFastScan.
5. **AVX-512 bits=8 kernel** using the same arithmetic-dequant
   rewrite Task 66 Slice B introduced — no LUT gather needed,
   pure FMA against the precomputed query-side `query_scale` /
   `query_offset` (Task 66 Slice C). Covers `rabitq8`,
   `rabitq8ls`, `rabitq8c3`, `rabitq8c4`.
6. **AVX2 bits=8 fallback** with the same arithmetic dequant.
7. **AVX-512 / AVX2 batched scoring** matching Task 66 Slice D's
   batched `score_batch` API for both bits=1 and bits=8.
8. **bf16 evaluation on Intel** — AVX-512 BF16 (`avx512bf16`) has
   `_mm512_dpbf16_ps` which is the direct analogue of NEON's bfdot.
   Behind the `rabitq-bf16` feature flag (same gate Task 66 uses)
   and only if measurement shows a win on the Intel host.
9. **Differential tests** — for every new AVX-512 / AVX2 kernel,
   register a paired entry in Task 66 Slice A's `*_for_test`
   scaffold so the existing property tests run NEON, AVX-512,
   AVX2, and scalar variants against each other.
10. **Intel benchmark validation** — run the Task 66 Slice H
    bench harness on the Intel host. Same packet shape, lands
    under `reviews/task-67/`.

### Out of scope

- M5 / NEON kernels — Task 66 owns those.
- ARM SVE / SVE2 — covered by Task 21.
- AVX-512 FastScan / FWHT kernels — those belong to Task 21's
  PqFastScan / TurboQuant scope, not the RaBitQ scoring kernel.
- Changes to Task 66's dispatch shape. If a structural change is
  needed, push back into Task 66 and land it there first.
- Changes to RaBitQ encoding / quantisation math.
- bits ∈ {2, 3, 5, 6, 7} kernels.

## Why a separate task

Decoupling Intel from M5 reflects three realities:

1. **Different runtime targets.** M5 ships first; Intel kernels
   follow on the timeline of the next Intel benchmark cycle. The
   M5 work isn't blocked on Intel hardware availability.
2. **Different microarchitecture tuning.** The AVX-512 4-pipe
   layout, port pressure, and bf16 issue width all differ from
   NEON's 4-6 pipe layout. Trying to land both in one task
   blurs the per-arch measurement story.
3. **Different reviewer attention.** AVX-512 popcount + bit
   tricks need careful unsafe review and a different mental
   model than the NEON arithmetic-dequant rewrite.

## Slice plan

- **Slice A — feature detection plumbing.** Add `avx512f`,
  `avx512vpopcntdq`, `avx512bw`, `avx512bf16` runtime detection
  to the dispatcher registered in Task 66 Slice A. Confirm slots
  exist for every kernel this task will add. No new kernels yet.
- **Slice B — AVX-512 bits=1 kernel.** Land the VPOPCNTDQ
  inner product. Differential test against scalar + NEON
  references via Task 66's scaffold. Intel host single-candidate
  bench: target ≥ 5× scalar.
- **Slice C — AVX2 bits=1 fallback.** Land the Mula or SAD-based
  popcount inner product. Same test surface. Bench: target
  ≥ 3× scalar.
- **Slice D — AVX-512 bits=4 kernel.** Nibble-unpack + FMA.
  Bench target ≥ 5× scalar.
- **Slice E — AVX2 bits=4 fallback.** 256-bit version of Slice D.
- **Slice F — AVX-512 bits=8 kernel** (arithmetic dequant from
  Task 66 Slice B). Bench target ≥ 5× scalar across the four
  `rabitq8*` variants.
- **Slice G — AVX2 bits=8 fallback.** Same pattern at 256-bit.
- **Slice H — batched scoring (bits=1 and bits=8).** AVX-512 +
  AVX2 implementations of Task 66 Slice D's `score_batch` API.
- **Slice I — bf16 evaluation.** Under `rabitq-bf16` feature,
  measure AVX-512 BF16 on the Intel host. Document, flip the
  gate only if a win.
- **Slice J — Intel measurement packet.** Re-run Task 66 Slice
  H's bench harness on the Intel host with these kernels. Same
  four bits=8 variants plus the bits=1 batched path. Per-kernel
  throughput, recall delta, headline wall time pre/post.

## Validation gate

1. **Functional.** All `cargo test -p ecaz quant::rabitq` tests
   pass on Intel. Differential tests in Task 66's scaffold pass
   with NEON + AVX-512 + AVX2 + scalar agreeing within documented
   tolerance.
2. **Recall.** No regression on real-10k / Intel benchmark
   fixtures. Within 0.5 pp at all list sizes.
3. **Performance — per-kernel.** Each slice meets its individual
   bench target. Reported in the slice's packet.
4. **Performance — headline.** On the Intel benchmark host,
   end-to-end RaBitQ scoring throughput is at least 4× current
   scalar across the four bits=8 variants, and at least 3× on
   the bits=1 batched path. Numbers published in Slice J's
   measurement packet.
5. **Safety.** Same rules as Task 66:
   - Every new `unsafe fn` carries a `# Safety` doc.
   - Runtime feature detection paired with SAFETY comments at
     every call site.
   - No anti-pattern B (`fn(*mut T) -> &'a T`).
   - No new unsafe outside `target_feature` kernel bodies.
6. **Cross-arch agreement.** The Task 66 differential-test
   scaffold catches any drift between NEON / AVX-512 / AVX2 /
   scalar implementations.

## Dispatch constraint

This task MUST consume the dispatch shape Task 66 Slice A
defines. Specifically:

- New kernels register as additional entries in the per-arch
  fn-pointer / trait table.
- `sum_query_dequant_with_bf16` and `estimate_ip_*_impl` should
  need **zero edits** in this task — only kernel files and the
  per-arch detection block.
- If a structural change is required, this task blocks: open a
  follow-up on Task 66 to land the structural change, then
  resume here.

If reviewer feedback finds this task editing
`sum_query_dequant_with_bf16` or `estimate_ip_*_impl` bodies, the
slice is rejected and Task 66 is reopened.

## Coder workflow notes

- **Branch off the Task 66 close commit.** Do not start before
  Task 66's Slice G ("Intel readiness gate") explicitly confirms
  the dispatch seam is ready.
- **Each slice = one commit + one packet** under
  `reviews/task-67/`.
- **No `unsafe` outside `target_feature` kernel bodies.** Same
  rule as Task 66.
- **Intel benchmark host access** — Slice J needs the Intel
  laptop or Intel cloud lane. Confirm host availability before
  Slice B; if the only Intel access is the cloud lane, plan
  for the cloud-bench workflow described in
  `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/manifest.md`'s
  failure mode (do NOT replay that failure — the workflow was
  patched after).
- **No `/tmp` benchmark logs.** All packet evidence under
  `reviews/task-67/{NNN}-{slug}/artifacts/`.

## References

- Quantizer source: `src/quant/rabitq.rs`
- Predecessor task (mandatory): `plan/tasks/66-rabitq-m5-neon-optimization.md`
- SIMD modernisation umbrella: `plan/tasks/21-simd-modernization.md`
- IVF RaBitQ8 history (motivation for the four bits=8 variants):
  `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/manifest.md`,
  `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/manifest.md`,
  `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/manifest.md`,
  `benchmarks/task51-local-rabitq8ls-sidecar/manifest.md`
- AVX2+FMA precedent (the kernel pattern to mirror):
  `src/am/ec_diskann/ambuild.rs:548-693`
  (`source_inner_product_avx2_fma`) plus the equivalent
  PqFastScan AVX2 kernels under `src/quant/`.

## Acceptance criteria

1. Slices A–J all landed, each with its own commit + packet.
2. Validation gate passes on Intel.
3. Slice J measurement packet documents pre/post throughput,
   per-kernel breakdown, and recall delta across at least
   `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`, and the
   bits=1 batched path.
4. No regression in DiskANN, HNSW, or IVF scan tests on any arch.
5. Task 66's dispatch seam unmodified outside the slot mechanism.

## Estimated size

Medium-large. Six new kernel variants (bits=1 ×2, bits=4 ×2,
bits=8 ×2) plus the batched path and Intel-host measurement.
Expect 3–4 weeks for a single coder including measurement and
the Intel host coordination.
