# Task 67 Packet 032 Artifact Manifest

- head SHA: 650fb11c83a26812fa58884911cfbf45dc3dc7cc
- task bucket: `reviews/task-67/032-cloud-install-pre-git-clean/`
- timestamp: 2026-05-30T15:20:10Z
- lane: cloud install wrapper support for Slice I bf16 AWS validation
- fixture / storage format / rerank mode: not applicable; no benchmark data in this packet
- isolated one-index-per-table or shared-table surfaces: not applicable

## Artifacts

### `artifacts/local/cargo-fmt-check.log`

- command: `script -q -c "cargo fmt --check" reviews/task-67/032-cloud-install-pre-git-clean/artifacts/local/cargo-fmt-check.log`
- result: passed
- key lines: rustfmt exited 0; log contains only existing stable-channel warnings for unstable rustfmt options.

### `artifacts/local/cargo-test-ecaz-cloud-install-script.log`

- command: `script -q -c "cargo test -p ecaz-cloud install_script_ --lib" reviews/task-67/032-cloud-install-pre-git-clean/artifacts/local/cargo-test-ecaz-cloud-install-script.log`
- result: passed
- key lines: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out`

### `artifacts/local/cargo-build-ecaz-cli.log`

- command: `script -q -c "cargo build -p ecaz-cli" reviews/task-67/032-cloud-install-pre-git-clean/artifacts/local/cargo-build-ecaz-cli.log`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo]`; log also records the existing `LoadedDistributedPlacementConfig.path` warning.
