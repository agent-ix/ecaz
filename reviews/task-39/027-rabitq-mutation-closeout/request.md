# Task 39 RaBitQ mutation closeout

## Summary

Drives the residual RaBitQ mutation survivors carried out of packet 026 to
0 missed / 0 timeouts.

Packet 026 left 9 missed and 2 timeouts. This closeout splits into:

1. Boundary-assertion strengthening for the `O_DOT_FLOOR` / `QR_FLOOR`
   guards in `CenteredScorer::score_at` and `estimate_ip_impl`. The
   existing equal-to-floor tests asserted `estimate.is_finite()`, but the
   `< → <=` mutant returns `estimate = 0.0` (which is finite), so the
   assertion missed the mutation. The closeout asserts
   `bound.is_finite()` on the equal-to-floor case, where the mutant
   returns `f32::INFINITY`.
2. A small refactor of `quantize_level` and `dequant_level` to route the
   bin-width `2 * c` through a new module-level `RABITQ_QUANT_RANGE = 4.0`
   const. With `RABITQ_QUANT_CLIP == 2.0`, the original `(2.0 * c)` form
   produced an equivalent mutant under `* → +` (both evaluate to 4.0);
   pulling the magnitude out as a literal removes the binop from that
   expression.

## Code under review

- Commit: `2522248b6985d4edcb2f3ce81af766e4fe05014c`
- Changed file: `src/quant/rabitq.rs`

## Mutation results

### Targeted rerun (former survivors)

- Command: `cargo mutants --in-place --package ecaz-careful-hardening
  --file hardening/careful/src/../../../src/quant/rabitq.rs
  --re 'rabitq\.rs:(763|765|1006|1140):'
  --output reviews/task-39/027-rabitq-mutation-closeout/artifacts/targeted-rerun/rabitq.rs.mutants`
- Result: 14 mutants tested in 7m: **14 caught, 0 missed, 0 timeouts**.
- Lines `763` / `765` / `1006` / `1140` correspond to the post-commit
  positions of the former 755 / 757 / 998 / 1132 survivors (line shifts
  come from the new `RABITQ_QUANT_RANGE` const and added doc comments).

### Full-file sweep

- Command: `cargo mutants --package ecaz-careful-hardening
  --file hardening/careful/src/../../../src/quant/rabitq.rs
  --output reviews/task-39/027-rabitq-mutation-closeout/artifacts/full/rabitq.rs.mutants
  -j 4`
- Result: **447 mutants tested in 8m: 426 caught, 21 unviable,
  0 missed, 0 timeouts.** Evidence:
  `artifacts/full/rabitq-full-mutants.log` and
  `artifacts/full/rabitq.rs.mutants/mutants.out/{missed,timeout,caught,unviable}.txt`.

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib rabitq
  -- --nocapture` passed: 30 RaBitQ tests (artifact:
  `artifacts/rabitq-focused-tests.log`).
- `git diff --check -- src/quant/rabitq.rs` passed (artifact:
  `artifacts/diff-check.log`).

## Notes

- Per the handoff, timeout follow-ups use the packet 017 pattern of
  asserting fixed expected edges directly rather than bumping the
  cargo-mutants timeout. The `< → <=` mutants on the floor checks at
  former lines 1127 / 1132 are caught via the same boundary-assertion
  strengthening that closes the missed `< → <=` mutants.
- The full sweep uses scratch-copy (no `--in-place`) so it can run with
  `-j 4`; cargo-mutants 27.0.0 refuses `--in-place` combined with
  `--jobs`.
