# Artifact Manifest: Task 85 Packet 020

- head SHA at validation start: `c54d869075b464e08c7b943c2be2cd647525be60`
- task bucket: `reviews/task-85/020-v5-selected-segment-locators/`
- lane: local PG18/Rust validation
- fixture: unit tests and CLI parser/bench-surface tests
- storage format: SPIRE leaf V5 summary segment locators, legacy V2/V3/V4 fallback retained
- rerank mode: unchanged
- isolated/shared surface: local unit tests; not an AWS benchmark packet

## Artifacts

### `cargo-test-leaf-partition-object-v.log`

- command:
  `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline leaf_partition_object_v -- --nocapture`
- timestamp: 2026-06-07
- key result:
  `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1964 filtered out`

### `cargo-test-leaf-block-row-ranges.log`

- command:
  `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline leaf_block_row_ranges -- --nocapture`
- timestamp: 2026-06-07
- key result:
  `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1967 filtered out`

### `cargo-test-ecaz-cli-spire.log`

- command:
  `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test --manifest-path crates/ecaz-cli/Cargo.toml spire --locked --offline`
- timestamp: 2026-06-07
- key result:
  `test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 350 filtered out`

## Notes

Cargo validation in this checkout needs `CARGO_DISABLE_GIT_DISCOVERY=1` because the repository currently contains roughly one million review/benchmark artifact files under the working tree. `Cargo.toml` now constrains the root crate package input set to source/build/test/bench files, but the explicit environment override was still used for deterministic validation during this packet.
