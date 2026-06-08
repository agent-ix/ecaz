# Task 86 Packet 005 Artifact Manifest

- Head SHA: `d86cccf42e5644bc2be436d0ffe2df7710bfecd8`
- Task bucket: `reviews/task-86/005-spire-tq-lut`
- Timestamp: `2026-06-07T06:54:53Z`
- Lane: focused unit validation
- Fixture: deterministic 1536-dimensional SPIRE TurboQuant assignment scorer test
- Storage format: SPIRE assignment payload format `TurboQuant`, existing no-QJL 4-bit packed TQ payload
- Rerank mode: none
- Index surface: SPIRE assignment scorer
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `spire-tq-lut-test.log`

Command:

```sh
cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path -- --nocapture > reviews/task-86/005-spire-tq-lut/artifacts/spire-tq-lut-test.log 2>&1
```

Key result:

```text
test am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1978 filtered out; finished in 0.03s
```
