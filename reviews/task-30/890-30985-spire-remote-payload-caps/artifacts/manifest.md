# Artifact Manifest: SPIRE Remote Payload Caps

- head SHA: `54d286584e391952fa197e5f046d911ef906ecdf`
- packet/topic: `30985-spire-remote-payload-caps`
- timestamp: `2026-05-13`
- lane / fixture / storage format / rerank mode: code and docs hardening for
  remote payload caps; no new SQL benchmark fixture was run in this packet.
- isolated/shared surface: n/a for this code-focused checkpoint.

## Evidence

### Existing measurement source

- packet/topic: `30975-spire-tuple-transport-measurement`
- committed source artifacts:
  - `review/30975-spire-tuple-transport-measurement/artifacts/manifest.md`
  - `review/30975-spire-tuple-transport-measurement/request.md`
- key cited result:
  - `payload materialized by each run: 200 rows and 31,510 text bytes`
  - row-cap rationale: 31,510 / 200 = about 158 bytes per row; default
    `ec_spire.max_remote_payload_bytes_per_row = 1024` rounds the requested 4x
    safety margin up to 1 KiB.
  - batch-cap rationale: default
    `ec_spire.max_remote_payload_rows_per_batch = 64` matches the Phase 12
    local capacity target for selected PIDs per remote node.

### Validation Commands

- command: `cargo test remote_payload --lib`
  - key result: `2 passed; 0 failed`
- command: `cargo test tuple_transport --lib`
  - key result: `4 passed; 0 failed`
- command: `cargo test production_fault_matrix_covers_required_categories --lib`
  - key result: `1 passed; 0 failed`
- command: `cargo fmt --check`
  - key result: exited successfully after stable-rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- command: `git diff --check`
  - key result: exited successfully.
