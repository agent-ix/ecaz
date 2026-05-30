# Task 67 Packet 034 Artifact Manifest

- head SHA: f6bad4800a745b329130651f8c18005d42b765e6
- task bucket: `reviews/task-67/034-ivf-adaptive-test-fixture/`
- timestamp: 2026-05-30T15:39:49Z
- lane: AC4 scan validation support
- fixture / storage format / rerank mode: not applicable; unit-test fixture correction
- isolated one-index-per-table or shared-table surfaces: not applicable

## Artifacts

### `artifacts/local/cargo-test-ecaz-lib-ivf-scan.log`

- command: `script -q -c "cargo test -p ecaz --lib am::ec_ivf::scan::tests" reviews/task-67/034-ivf-adaptive-test-fixture/artifacts/local/cargo-test-ecaz-lib-ivf-scan.log`
- result: failed before the source fix
- key lines: `adaptive_nprobe_keeps_requested_width_when_gap_is_small` failed with `left: [1, 2, 3] right: [1, 2, 3, 4, 5, 6]`

### `artifacts/local/cargo-test-ecaz-lib-ivf-adaptive-nprobe-single.log`

- command: `script -q -c "cargo test -p ecaz --lib am::ec_ivf::scan::tests::adaptive_nprobe_keeps_requested_width_when_gap_is_small -- --exact" reviews/task-67/034-ivf-adaptive-test-fixture/artifacts/local/cargo-test-ecaz-lib-ivf-adaptive-nprobe-single.log`
- result: reproduced the failure before the source fix
- key lines: same failed assertion as the full IVF scan run

### `artifacts/local/cargo-test-ecaz-lib-ivf-scan-rerun.log`

- command: `script -q -c "cargo test -p ecaz --lib am::ec_ivf::scan::tests" reviews/task-67/034-ivf-adaptive-test-fixture/artifacts/local/cargo-test-ecaz-lib-ivf-scan-rerun.log`
- result: passed after the source fix
- key lines: `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1917 filtered out`

### Other Validation Logs

These were captured while preparing the closeout audit and are retained here for continuity:

- `artifacts/local/cargo-test-ecaz-lib-quant-rabitq.log`: `46 passed; 0 failed`
- `artifacts/local/cargo-test-ecaz-lib-diskann-scan.log`: `18 passed; 0 failed`
- `artifacts/local/cargo-test-ecaz-lib-hnsw-scan.log`: `73 passed; 0 failed`
