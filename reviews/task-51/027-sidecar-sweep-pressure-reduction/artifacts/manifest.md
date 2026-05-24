# Task 51 Packet 027 Artifact Manifest

- Head SHA: `0429af2ab72fcb6577daf22e67f520c6b3f08230`
- Task bucket: `reviews/task-51/027-sidecar-sweep-pressure-reduction`
- Timestamp: `2026-05-24T02:52:33Z`
- Scope: IVF/RaBitQ sidecar sweep pressure reduction
- Lane / fixture / storage format / rerank mode: AWS 1M IVF/RaBitQ sidecar harness, `real_1m_ivf_rabitq1_rerank`, new RaBitQ8 sidecar variants, `tid-sorted`
- Surface isolation: shared benchmark packet config; sidecar measurement tables are per-variant tables derived from the preserved AWS corpus table

## Artifacts

### `cargo-build-release-ecaz-cli.log`

- Command: `script -q -e -c "cargo build -p ecaz-cli --release --no-default-features" reviews/task-51/027-sidecar-sweep-pressure-reduction/artifacts/cargo-build-release-ecaz-cli.log`
- Timestamp: `2026-05-24T02:51:xxZ`
- Result: pass
- Key line: `Finished release profile [optimized] target(s) in 0.48s`

### `cargo-test-ecaz-cli-sidecar.log`

- Command: `script -q -e -c "cargo test -p ecaz-cli sidecar --no-default-features" reviews/task-51/027-sidecar-sweep-pressure-reduction/artifacts/cargo-test-ecaz-cli-sidecar.log`
- Timestamp: `2026-05-24T02:51:xxZ`
- Result: pass
- Key line: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 354 filtered out`
