# Review request — Task 105 packet 001: production optimization slices

- Task: 105, Phase 1 (must merge to main before the Phase 2 trip)
- Coder: Task 99/102/103 author lane
- Date: 2026-06-12

The three ADR-077-decided optimizations, implemented and validated
(see `artifacts/manifest.md`):

1. aarch64 NEON-first dispatch (§6 — the measured 27–45% e2e win on
   every G4 TurboQuant cell).
2. `ec_ivf.scratch_soa_batch_decode` default → on (§4), with a
   default-session smoke proving kernel engagement without any GUC.
3. rabitq32 strict-test pair → family envelope (packet 009 1-ULP
   finding).

## Review asks

1. Dispatch flip shape: preference change inside `select_highest_isa`
   with SVE kernels left in-tree and unreachable on real hosts —
   acceptable vs. wanting the SVE arms feature-gated or a preference
   GUC added now (ADR-077 defers the override to a future re-entry
   measurement).
2. The `cfg(test)` getter stub for the IVF GUC stays false — confirm
   that's the right determinism trade.
3. The pre-existing full-`--lib` failure set (11–12 tests, reproduced
   on clean main) — flagging for a separate hygiene task; confirm
   out-of-scope here.
