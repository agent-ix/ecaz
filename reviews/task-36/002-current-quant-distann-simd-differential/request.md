---
task: 36
agent: codex
role: coder
model: GPT-5
date: 2026-07-24
head: c373eb51f61654226f81b489a4be4a40e6e45025
---

# Review request: current quant, DistANN, and SIMD differential coverage

Please review checkpoint `c373eb51f61654226f81b489a4be4a40e6e45025`.
This is a review request, not Task 36 closeout.

## What changed

- Made `make simd-diff` the explicit local inventory lane for current product
  scoring/FWHT hooks, RaBitQ arithmetic and `rabitq32`, QJL, LUT, grouped PQ,
  int8/SDOT, Hamming, HNSW/DiskANN source scoring, and DistANN composition.
- Added forced host-backend assertions so locally available NEON paths cannot
  silently skip.
- Expanded block/tail coverage over candidate widths
  `1,7,8,9,16,17,31,32,33`, dimension tails, and production dimensions up to
  1536.
- Added DistANN codec-composition tests for grouped PQ, RaBitQ, and TurboQuant,
  including prepared-vs-direct scoring, `count * stride` slicing, poison
  payload, and exact-distance sign composition.
- Documented the kernel-to-access-method inventory, equality/tolerance policy,
  and the boundary between local Apple evidence and future Intel/Graviton
  hardware evidence.

All Rust source additions are test-only or test/bench hooks; production scoring
behavior is unchanged.

## Evidence

See [artifacts/manifest.md](artifacts/manifest.md) for provenance and exact
commands.

- [make-simd-diff.log](artifacts/make-simd-diff.log): full post-revert lane
  passed on AArch64 NEON + dot-product/SDOT.
- [distann-simd-diff.log](artifacts/distann-simd-diff.log): focused DistANN
  composition passed (2 tests).
- [mutation-control-failure.log](artifacts/mutation-control-failure.log):
  deliberate `+1.0` SDOT scoring defect was caught by four comparisons.
- [cargo-fmt-check.log](artifacts/cargo-fmt-check.log): required global check
  captured; it fails on inherited repository-wide formatting drift. No broad
  formatting rewrite is included here.

## Review focus

1. Does the make lane accurately enumerate the current implemented kernel
   families without presenting placeholders as real SIMD?
2. Do the DistANN tests validate access-method composition rather than merely
   retesting isolated quantizer kernels?
3. Are exact equality and `1e-5` tolerance used at the correct boundaries?
4. Is the local-only hardware claim sufficiently clear? Intel AVX2/AVX-512 and
   Graviton SVE/SVE2 execution are intentionally left for later host runs.

Task 36 should remain open until an outside reviewer responds and the deferred
hardware lanes are handled.
