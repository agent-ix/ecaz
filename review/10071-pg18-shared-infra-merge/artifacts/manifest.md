# Artifact Manifest

Packet: `10071-pg18-shared-infra-merge`
Head: `01f28d1`

This packet makes no measurement claims.

Validation cited in `request.md` was run directly from the working tree:
- `cargo test` — passed
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — passed
- `cargo pgrx test pg18` — passed
- `cargo test --no-default-features --features pg17` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
