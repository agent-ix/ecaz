# Task 86 Packet 009 Artifact Manifest

- Head SHA: `7dfb7765d41711be1ac26a65658d403661b20dcd`
- Task bucket: `reviews/task-86/009-spire-lut-parity`
- Timestamp: `2026-06-07T15:38:42Z`
- Lane: local PG18-focused unit coverage
- Fixture / storage format / rerank mode: not applicable; focused Rust unit test for SPIRE TurboQuant scorer parity.
- Isolated one-index-per-table or shared-table surface: not applicable.

## Artifacts

### `focused-test.log`

- Command:
  `cargo test -p ecaz --lib --no-default-features --features pg18 turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path -- > reviews/task-86/009-spire-lut-parity/artifacts/focused-test.log 2>&1`
- Key result lines:
  - `running 1 test`
  - `test am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1978 filtered out; finished in 0.03s`
