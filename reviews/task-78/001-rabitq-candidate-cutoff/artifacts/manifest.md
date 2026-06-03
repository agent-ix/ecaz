# Artifact Manifest: RaBitQ Bounded Candidate Cutoff

- head SHA: `7a1ff3a3bdc6fcf8289207df7aa4ad5d3625be19`
- task bucket: `reviews/task-78/`
- packet path: `reviews/task-78/001-rabitq-candidate-cutoff/`
- timestamp: `2026-05-31T17:30:51-07:00`
- lane: local PG18 validation
- fixture: unit-test SPIRE assignment scorer and quantized routed candidate fixtures
- storage format: V2 leaf candidate columns for the scan-path coverage
- rerank mode: bounded candidate collection before rerank
- isolated/shared surface: local unit-test object store fixtures, not a shared-table benchmark surface

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --all -- --check`
- result: exit 0
- key lines: rustfmt accepted the workspace; log contains stable-toolchain warnings for ignored unstable rustfmt options.

### `cargo-test-assignment-scorer.log`

- command: `cargo test -p ecaz --no-default-features --features pg18 assignment_scorer -- --nocapture`
- result: exit 0
- key lines: `running 9 tests`; `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1935 filtered out; finished in 0.12s`

### `cargo-clippy-pg18.log`

- command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- result: exit 0
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.20s`
