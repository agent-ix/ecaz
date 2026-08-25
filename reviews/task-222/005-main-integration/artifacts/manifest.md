# Task 222 packet 005 artifact manifest

- Base SHA: `de28655a42d254c2ac7f181569f07b92de5f3fae`
- Pre-packet integration head: `4d06c1ee5`
- Branch: `integrate/task222-payload-projection`
- Task bucket / packet: `reviews/task-222/005-main-integration/`
- Lane: current-main PG18 focused integration
- Timestamp: 2026-08-24 PDT (America/Los_Angeles)
- Fixture: focused pgrx test database; no corpus, index, shared-table surface,
  storage format, or rerank-mode change in this packet

Artifacts:

- `cargo-check-pg18.log`
  - Command: `cargo check --lib --no-default-features --features pg18`
  - Result: exit 0; dev profile completed.
- `pgrx-payload-projection-pg18.log`
  - Command: `cargo pgrx test pg18 test_distann_payload_projection_contract
    --no-default-features --features pg18`
  - Result: exit 0; 1 passed, 0 failed, 2,582 filtered out.

The first sandboxed compile attempt could not resolve crates.io and was
immediately rerun with normal network access; the captured check log is the
successful replacement run. No formatter output is part of this packet.
