# Artifact Manifest: SPIRE Remote Candidates Test Layout

Packet: `30993-spire-remote-candidates-test-layout`
Head SHA at run time: `36247454` plus working-tree checkpoint changes
Timestamp: 2026-05-13 America/Los_Angeles

This packet is a test-layout cleanup checkpoint for Phase 12b.1. Lane,
fixture, storage format, and rerank mode are not applicable except where a
command explicitly names a test filter.

| Artifact | Command | Result |
|---|---|---|
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | exit 0; one pre-existing unused-import warning in `src/am/mod.rs` |
| `cargo-fmt-check.log` | `cargo fmt --check` | exit 0; rustfmt emitted stable-toolchain warnings for unstable import-group config |
| `git-diff-check.log` | `git diff --check -- src/am/ec_spire/root/remote_candidates/endpoint_identity.rs src/am/ec_spire/root/remote_candidates/mod.rs src/am/ec_spire/root/remote_candidates/tests/endpoint_identity.rs plan/tasks/task30-phase12b-spire-cleanup.md` | exit 0 |
| `cargo-test-remote-tuple-transport.log` | `cargo test --no-default-features --features pg18 remote_tuple_transport` | exit 0; 3 passed, 0 failed, 1709 filtered out |

