---
task: 36
agent: codex
role: coder
model: GPT-5
date: 2026-07-25
head: a915b062bac167532f961ee77dd184905db58d90
---

# Review request: resolve packet 002 current-head flags

Please review head `a915b062bac167532f961ee77dd184905db58d90`
against all five flags in packet 002.

## Corrections

- Every filtered `make simd-diff` stage now declares its expected test count.
  Cargo's zero-match success can no longer make the lane green.
- The tiled-LUT prod safety stage from packet 003 is included in that counted
  inventory.
- All x86 public forced hooks now fail if AVX2/FMA execution is unavailable:
  HNSW, DiskANN, Prod direct scoring, Prod code-to-code scoring, and FWHT.
- QJL scalar candidates and tails remain bit-exact. The 4-ULP/relative
  `1e-6` allowance applies only to candidates processed by an observed
  non-scalar 32-wide block, whose vector reduction changes accumulation order.
- QJL output now reports the ISA returned by the block scorer.
- The manual CI SIMD matrix calls `make simd-diff`, preventing workflow/local
  lane drift. CI triggers remain manual-only.

Intel and Graviton hardware execution are still later host runs, as agreed;
this packet makes no claim that Apple compilation proves those paths.

## Evidence

See [artifacts/manifest.md](artifacts/manifest.md) for the exact
finding-by-finding mapping.

- [make-simd-diff.log](artifacts/make-simd-diff.log): all ten counted stages
  passed on NEON/SDOT.
- [empty-filter-negative-control.log](artifacts/empty-filter-negative-control.log):
  a zero-match Cargo success is converted into lane failure.
- [quant-lib-tests.log](artifacts/quant-lib-tests.log): 188 passed, 0 failed,
  3 ignored.
- [cargo-fmt-check.log](artifacts/cargo-fmt-check.log): inherited global drift
  remains; neither changed Rust file appears in the format diff.

## Review focus

1. Does the counted wrapper preserve Cargo failures while rejecting missing or
   unexpected test counts?
2. Are QJL exactness/tolerance boundaries now as narrow as the execution path
   permits?
3. Do the x86 assertions match the already-enforced aarch64 “not a skip”
   contract?
4. Does the manual workflow now execute exactly the authoritative local lane?

Task 36 remains review-requested pending this response.
