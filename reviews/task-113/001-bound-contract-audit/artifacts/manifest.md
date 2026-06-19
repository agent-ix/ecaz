# Manifest — Task 113 / 001 Phase 1 bound contract audit

- Head SHA: `274b02743` (code commit on `task-113-ivf-bound-aware-candidate-pruning`)
- Task bucket: `reviews/task-113/`
- Packet path: `reviews/task-113/001-bound-contract-audit/`
- Type: code review (audit + unit tests). No benchmark.
- Storage format / quant: RaBitQ (all bits); audit is quantizer-level, not lane-specific.
- Rerank mode: n/a (Phase 1 is the scoring-surface audit).
- Isolation: n/a (unit tests, no index).

## Artifacts

- `phase1-cutoff-tests.log` — output of:
  `cargo test --lib --no-default-features --features pg18 try_estimate`
  Timestamp: 2026-06-19. Result line: `test result: ok. 3 passed; 0 failed`.

## Key result lines cited by request.md

- `try_estimate_scalar_cutoff_never_prunes_a_keepable_candidate ... ok`
- `try_estimate_scalar_cutoff_is_monotone_in_threshold ... ok`
- `try_estimate_bound_carrying_cutoff_agrees_with_scalar ... ok`

## Code references (audit findings)

- Sound Cauchy-Schwarz cutoff: `src/quant/rabitq.rs:1160-1168` (scalar),
  `1213-1223` (bound-carrying). Contract doc added at `try_estimate_ip_scalar`.
- Probabilistic ε-envelope (NOT a prune bound): `src/quant/rabitq.rs:4200-4202`,
  `RABITQ_BOUND_CONFIDENCE` at `:144`.
- IVF prune wiring already live: `src/am/ec_ivf/scan.rs:1589-1602`,
  `:1807-1820`, `:1997-2012`; counter `record_posting_pruned_by_bound`
  at `src/am/common/explain.rs:290`.
