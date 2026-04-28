# Artifact Manifest

Packet: `review/30074-task28-ivf-typed-score-mode`

Code SHA: `cdbbb26`

Timestamp: `2026-04-27T18:50:37-07:00`

Lane: Task 28 A4 typed score-mode dispatch.

Measurement artifacts: none. This packet records a localized code-quality fix
with focused test validation.

Validation:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- `cargo test --lib quantizer_1536_4bit_disables_unused_qjl_and_lut_state --no-default-features --features pg18`
- `cargo test --lib quant::prod::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `git diff --check`
