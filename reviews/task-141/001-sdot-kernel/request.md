# Review request: Task 141 — SDOT (dotprod) kernel for int8_approx32

- Task: `plan/tasks/141-tq-int8-approx-sdot-kernel.md`
- Branch: `task-141-sdot-kernel` (off main `e5ef96109`)
- Code commit: `2d98ec5b7`
- Evidence: `artifacts/manifest.md`

## What changed

`src/quant/int8_approx32/neon.rs` gains a `dotprod`-dispatched fast path:
each `vmull_s8`/`vmull_high_s8` + `vpadalq_s16` pair in the 32-dim step
collapses into one `sdot` instruction. `vdotq_s32` is still unstable in
stable Rust (`stdarch_neon_dotprod`, rust#117224), so the instruction is
emitted through a stable inline-asm wrapper `sdot_asm` — the exact pattern
of the existing `bfdot_asm` precedent in `quant::rabitq`. Nibble-split,
`vqtbl1q` codebook dequant, de-interleaved `vld2q` loads, scalar dim-tail,
and the legacy NEON fallback are unchanged. Runtime dispatch:
`dotprod` → SDOT kernel; NEON-only → legacy kernel; else scalar.

Bit-exactness: SDOT accumulates exact widened i8×i8 products into i32
lanes; integer addition is order-independent, so all existing `.to_bits()`
parity tests pass through the new path unchanged. A new test additionally
pins BOTH the legacy and dotprod candidate kernels against the scalar
reference directly, so the legacy path keeps coverage on dotprod test
hosts (M-series/Graviton), across dims [7, 64, 100, 191, 1536].

No new `unsafe fn` beyond the kernel-internal `#[target_feature]` fns that
mirror the existing pattern; no dispatch-surface or prepared-query change.

## A/B result (before/after commit, same session/tables, int8_approx scorer)

| scale | recall@10 pre → post | latency mean pre → post | scorer_batch pre → post |
|---|---|---|---|
| 10k | 0.9719 → 0.9719 | 0.76 → 0.67 ms (−11.8%) | 15.9 → 9.0 ms (−43.6%) |
| 50k | 0.9521 → 0.9521 | 1.55 → 1.29 ms (−16.8%) | 26.6 → 14.2 ms (−46.7%) |
| 100k | 0.8938 → 0.8938 | 2.34 → 1.95 ms (−16.7%) | 25.8 → 13.5 ms (−47.5%) |

Recall is byte-identical (bit-exact kernel swap). The kernel came in at
~1.9× — above the task's 1.3–1.8× estimate. Storage unchanged by
construction.

Stacked against the ORIGINAL i16-LUT baseline (Task 136 packet): 100k mean
2.79 ms (lut) → 2.34 (int8) → 1.95 (int8+SDOT) — the scorer axis has now
delivered ~−30% e2e in two bit-gated steps.

## Validation

- `cargo test --lib int8_approx32`: 5 passed (4 pre-existing parity tests
  now exercising the SDOT path on this host, plus the new dual-path pin).
- clippy pg18 gate: only the pre-existing manual-checked-division finding.
- pgrx runtime tests skipped per the macOS policy; behavior validated
  end-to-end by the A/B suite.

## Asks

1. Review the `sdot_asm` wrapper against the `bfdot_asm` precedent
   (operand classes, `pure, nomem, nostack` options) and the dispatch
   ordering (dotprod before neon).
2. Confirm this satisfies the Task 141 gate; the HNSW surface uses the
   same kernel and inherits the win (no HNSW-side change needed).
3. Task 143's 1m matrix will take this as the int8 candidate; Graviton
   (also dotprod-capable) remains the standing cross-lane follow-up.
