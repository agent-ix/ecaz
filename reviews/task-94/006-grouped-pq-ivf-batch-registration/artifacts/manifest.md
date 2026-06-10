# Task 94 Packet 006 Artifacts

- head SHA: `e96ba7eec8af9e389b4c6aef3ad5aa6056a32de7`
- code checkpoint: `e3bc6c621` (`Register grouped-PQ IVF batch scoring`)
- task bucket: `reviews/task-94/006-grouped-pq-ivf-batch-registration/`
- lane: coder-1 LUT lane
- fixture: local Rust unit tests
- storage format / quant: IVF `PqFastScan` / `GroupedPq`
- rerank mode: not applicable
- surface isolation: unit-test in-memory candidate batches, no database table surface
- timestamp: `2026-06-09T10:27:06-07:00`

## Artifacts

### `grouped-pq-batch-tests.log`

- command: `cargo test grouped_pq_batch --lib`
- result: pass
- key result lines:
  - `running 6 tests`
  - `test am::common::candidate_batch::tests::grouped_pq_batch_records_block_and_scalar_tail_counters ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_batch_is_bit_exact_with_scalar ... ok`
  - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2042 filtered out; finished in 0.08s`

## Evidence Notes

- This is a local-only Phase 6 checkpoint. No CI, AWS, Graviton 4, or benchmark run was used.
- The counter evidence is from the unit test that directly inspects `block_kernel_scoring_snapshots()` after a 39-candidate grouped-PQ batch: 32 block candidates and 7 scalar-tail candidates under `(surface=ivf, quant=grouped_pq)`.
- Full AM and benchmark-suite evidence remains pending for later Phase 6 packets.
