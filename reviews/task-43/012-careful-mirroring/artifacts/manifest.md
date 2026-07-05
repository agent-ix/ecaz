# Task 43 Packet 012 Artifact Manifest

- Head SHA: `95baf211e0dbd71cf237d3205741c8e34f0b4561`
- Task bucket: `reviews/task-43/`
- Packet: `reviews/task-43/012-careful-mirroring/`
- Timestamp: `2026-05-18T20:33:17Z`
- Lane: cargo-careful mirroring / blocker audit

## Artifacts

### `careful-harness-cargo-test.log`

- Command:
  `script -q -c 'cargo test --manifest-path hardening/careful/Cargo.toml --lib' reviews/task-43/012-careful-mirroring/artifacts/careful-harness-cargo-test.log`
- Fixture / surface:
  `hardening/careful` path-lifted storage page, DiskANN tuple, DiskANN
  vacuum, DiskANN Vamana graph, and HNSW search modules.
- Storage format / rerank mode:
  Pure Rust harness only; no PostgreSQL storage callbacks; no rerank mode.
- Shared-table surface:
  Not applicable.
- Key result:
  `test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s`
- Note:
  Emits an existing `align_up` dead-code warning from the path-lifted storage
  module.

### `make-careful.log`

- Command:
  `script -q -c 'make careful' reviews/task-43/012-careful-mirroring/artifacts/make-careful.log`
- Fixture / surface:
  cargo-careful execution of the same `hardening/careful` path-lifted harness.
- Storage format / rerank mode:
  Pure Rust harness only; no PostgreSQL storage callbacks; no rerank mode.
- Shared-table surface:
  Not applicable.
- Key result:
  `test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s`
- Key result:
  `Doc-tests ecaz_careful_hardening`
  `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- Note:
  cargo-careful prepared a sysroot for `x86_64-unknown-linux-gnu`; the run
  completed successfully.

### `cargo-fmt-check.log`

- Command:
  `script -q -c 'cargo fmt --all -- --check' reviews/task-43/012-careful-mirroring/artifacts/cargo-fmt-check.log`
- Key result:
  `COMMAND_EXIT_CODE="0"`
- Note:
  The stable formatter reports existing warnings for unstable
  `imports_granularity` and `group_imports` settings, but exits cleanly.

### `git-diff-check.log`

- Command:
  `script -q -c 'git diff --check' reviews/task-43/012-careful-mirroring/artifacts/git-diff-check.log`
- Key result:
  `COMMAND_EXIT_CODE="0"`

## Blocker Audit

The cargo-careful harness currently path-lifts every module that is
path-liftable without pulling in pgrx/PostgreSQL callback context:

- `src/storage/page.rs`
- `src/am/ec_diskann/tuple.rs`
- `src/am/ec_diskann/vacuum.rs`
- `src/am/ec_diskann/vamana.rs`
- `src/am/ec_hnsw/search.rs`

The remaining Miri-covered SPIRE surfaces are not mirrored in cargo-careful in
this packet because they live behind SPIRE include trees that bind pure logic to
pgrx-facing `ItemPointer`/OID context, object-reader contracts, storage/meta
types, or coordinator payload structs. The tracker records concrete extraction
paths for top-k, routing, vacuum/delete-delta, remote typed payloads, and SPIRE
serialization.
