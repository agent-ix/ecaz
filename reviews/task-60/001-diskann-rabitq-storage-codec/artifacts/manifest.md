# Task 60 / Packet 001 Artifact Manifest

- head SHA: `6a5ed394e46d9ba6fc0e54c69d1789ed5df9840b`
- task bucket: `reviews/task-60/`
- packet path: `reviews/task-60/001-diskann-rabitq-storage-codec/`
- lane: DiskANN RaBitQ storage-format code checkpoint
- fixture: compile-only local validation; no corpus benchmark in this packet
- storage format: `ec_diskann` `pq_fastscan` default plus new `rabitq` codec wiring
- rerank mode: unchanged heap exact rerank
- isolated/shared surface: code checkpoint only; no benchmark table surface
- timestamp: 2026-05-25

## Artifacts

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- result: pass
- key result line: `Finished dev profile [unoptimized + debuginfo]`

### `cargo-check-pg18-pg-test.log`

- command: `cargo check --no-default-features --features "pg18 pg_test"`
- result: pass
- key result line: `Finished dev profile [unoptimized + debuginfo]`
- notes: existing HNSW `unused_unsafe` warnings remain unrelated to this packet.

## Deferred Evidence

- `cargo test --no-default-features --features pg18 rabitq` compiled the test binary but could not run outside the pgrx/PostgreSQL harness because the binary failed to resolve `CacheRegisterRelcacheCallback`.
- Required Task 60 benchmark evidence is not included here: 100k and 1M `pq_fastscan` vs `rabitq` recall/latency/storage still need a dedicated `ecaz bench suite` packet.
