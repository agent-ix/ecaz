# Artifact Manifest

Packet: `10071-pg18-shared-infra-merge`
Head: `501f422`

This packet makes no measurement claims.

Validation cited in `request.md` was run directly from the working tree:
- `cargo test` — passed
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — passed
- `cargo pgrx test pg18` — passed
- `cargo test --no-default-features --features pg17` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed

Reviewer follow-up incorporated at this head:
- preload-on shared pgstat writes now flush one aggregate per scan instead of locking once per hot-path event
- the shim now uses a tqvector-owned custom pgstat kind (`PGSTAT_KIND_CUSTOM_MIN + 1`) instead of `PGSTAT_KIND_EXPERIMENTAL`
- the EXPLAIN hook now exits before access-method-name allocation when the `tqvector` option is disabled
