---
task: 36
agent: codex
role: coder
model: GPT-5
date: 2026-07-25
head: 15e3831c13b65b488fea1c0f1ac1da8d46e321f1
---

# Review request: Task 36 current-head corrections

Please review the actual Task 36 current head
`15e3831c13b65b488fea1c0f1ac1da8d46e321f1`.

The feedback in packet 001 named reviewed head `48fc8ee21`, which predates the
Task 36 implementation checkpoint `c373eb51f`. Consequently, its lane-
inventory, tolerance, and SVE-accounting findings were already corrected on
this branch. The feedback still exposed two valid regressions, both corrected
here.

## Corrections

- Restored the tiled-LUT query length and no-QJL 4-bit lane guards.
- Promoted the LUT codebook/centroid shape invariant from `debug_assert_eq!`
  to release-enforced `assert_eq!`.
- Added regression tests for all three failure modes and made them part of
  `make simd-diff`.
- Pinned every int8 block/partial differential dispatch to the uncapped
  host-preferred ISA, so an environment cap cannot produce a vacuous
  scalar-vs-scalar pass.
- Removed the remaining stale “caught at PR time” language. The task and
  hardening guide now accurately describe manual-dispatch CI and packet-backed
  pre-merge evidence.

No CI workflow was changed. Intel and Graviton execution remain explicit later
hardware runs, not claims made by this Apple packet.

## Evidence

See [artifacts/manifest.md](artifacts/manifest.md) for the complete mapping of
all five findings.

- [make-simd-diff.log](artifacts/make-simd-diff.log): authoritative lane
  passed, including the six current block-kernel families, int8 ISA pinning,
  and three tiled-LUT safety tests.
- [quant-lib-tests.log](artifacts/quant-lib-tests.log): reviewer discovery
  slice passed, 188 passed / 0 failed / 3 ignored.
- [int8-scalar-cap-negative-control.log](artifacts/int8-scalar-cap-negative-control.log):
  forcing `ECAZ_ISA_CAP=scalar` fails with actual `Scalar` versus expected
  `Neon`, proving the test cannot pass vacuously.
- [quality-checks.md](artifacts/quality-checks.md): whitespace passed; global
  format and clippy baseline failures are identified by exact unrelated scope.

## Review focus

1. Do the restored guards eliminate the release-mode silent wrong-score path?
2. Does uncapped host selection make the int8 tests fail loudly under an ISA
   cap or dispatch fallback?
3. Does the current packet now distinguish current-head evidence from the
   older head reviewed in packet 001?

Task 36 remains review-requested; the coder is not closing the request.
