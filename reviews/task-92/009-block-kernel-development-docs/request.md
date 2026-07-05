# Task 92 Packet 009: Block Kernel Development Docs

## Summary

This packet implements Task 92 Phase 4's reference skeleton documentation.

Code checkpoint:
`200884d8e19e56d991a5f198329633b34db55b51`

Changes:

- Added `docs/block-kernel-development.md`.
- Documents the ADR-076 module layout:
  `src/quant/<kernel>/{mod.rs,scalar.rs,neon.rs,sve.rs,avx2.rs}`.
- Uses Task 87/92 `src/quant/lut32/` as the worked reference example.
- Documents width gating at 32 candidates, scalar tail handling, and
  backend-returned ISA counter attribution.
- Carries forward the Packet 008 rule that fallback backend stubs return
  `Isa::Scalar` until real ISA kernels replace them.
- Documents the Graviton 4 / SVE2 evidence contract:
  target `Isa::Sve2` when available, report measured vector length for
  width-specific claims, and keep scalar tails under `isa=scalar`.
- Captures minimum unit-test, counter, and packet-evidence expectations for
  Tasks 93-98.

No runtime code changed in this packet.

## Validation

- `git diff --check`
  - passed with no output
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

- Confirm the documentation is sufficient as the Task 92 Phase 4 reference
  skeleton.
- Confirm the Graviton 4/SVE2 wording matches ADR-076 and Packet 008.
- Confirm future kernel packets have enough concrete test and evidence rules to
  avoid drifting from the shared infrastructure contract.
