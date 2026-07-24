# Task 198 packet 001 artifact manifest

- Head SHA: `d42c12fc2e2ebd0a8a5c5464902e2f6a0cb46504`
- Task / packet: `reviews/task-198/001-contract-and-format/`
- Timestamp: `2026-07-23T19:12:42-07:00`
- Lane: local Intel desktop, PG18 compile/unit lane
- Fixture: no database fixture; canonical format/state-machine unit tests
- Storage format: FR-084 traversal-replica format v1
- Rerank mode: exact-vector bytes; no scan executed in this packet
- Isolation: no index/table benchmark surface; not a measurement packet

## `pg18-unit.log`

- Command:
  `env PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test traversal_replica --no-default-features --features pg18`
- SHA-256:
  `fba1cc4dfb1ae55692abed9b7c644c71d1484e8f9153f7aae2fbd0f4028aaed2`
- Bytes: `3351`
- Result: command exit `0`; `3 passed; 0 failed`; 2,520 unrelated library
  tests filtered out.
- Covered:
  deterministic identity-bound content digest; duplicate/order/owner/vector
  shape/cardinality rejection; allowed state transitions.

## Static checks

- `git diff --check`: pass before the code checkpoint.
- Repository-wide `cargo fmt --all -- --check`: pre-existing unrelated
  formatting drift on `main`; the new module was formatted directly with
  `rustfmt --edition 2021 src/am/ec_distann/traversal_replica.rs`.
