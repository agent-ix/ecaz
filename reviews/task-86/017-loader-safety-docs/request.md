# Task 86 Packet 017: Loader Safety Docs

## Summary

This packet resolves the final merge-blocking reviewer flag from packet 016.

Commit `8f36f02fec9ca35bc74f9df0824d056dd006d3fa` adds `/// # Safety` contracts to the IVF model loaders in `src/am/ec_ivf/quantizer.rs`:

- `load_pq_fastscan_model` (pre-existing unsafe function, same loader shape)
- `load_tqplus_model` (Task 86 TQ+ unsafe function)

No scoring, storage, format, or benchmark behavior changed.

## Validation

- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - passed; see `artifacts/cargo-check-pg18.log`
- `git diff --unified=0 origin/main...HEAD -- src hardening | rg -n '^\\+.*unsafe \\{'`
  - no matches; see `artifacts/no-added-unsafe-blocks.log`

## Review Focus

- Confirm F-1 from `reviews/task-86/016-final-audit/feedback/2026-06-07-01-reviewer.md` is resolved.
- Confirm the task status flip to complete is appropriate after packet 016 reviewer acceptance plus this F-1 fix.
