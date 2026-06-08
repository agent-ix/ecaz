# Review Request: SPIRE TurboQuant LUT Parity Guard

## Scope

This checkpoint addresses reviewer feedback on packet 005 before the full real-corpus benchmark rerun.

Changes:
- Clarified in `src/am/ec_spire/quantizer/mod.rs` that SPIRE drops `gamma` only on the no-QJL 4-bit TurboQuant lane, where there is no QJL residual sign payload and the exact score has no gamma term.
- Strengthened `turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path` so it compares the SPIRE LUT score against the generic `ProdQuantizer::score_ip_from_parts` path with the encoded gamma supplied, not only against the LUT helper.

## Validation

- `cargo test -p ecaz --lib --no-default-features --features pg18 turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path --`
  - Artifact: `reviews/task-86/009-spire-lut-parity/artifacts/focused-test.log`
  - Result: `1 passed; 0 failed`

## Follow-Up

The production benchmark gap remains owned by `reviews/task-86/008-spire-real-spread/`, which is running the 10k/50k/100k SPIRE TurboQuant before/after suite.
