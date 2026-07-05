# Artifact Manifest

Packet: `462-c1-pg18-shared-infra-merge`
Head: `6d383a4`

This packet makes no measurement claims.

Validation cited in `request.md` was run directly from the working tree:
- `cargo test` — passed
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — passed
- `cargo pgrx test pg18` — passed
- `cargo test --no-default-features --features pg17` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo +nightly miri test --lib -- miri_score_scan_element_result_via_raw_opaque_ptr_updates_stats_delta` — passed

Reviewer follow-up incorporated at this head:
- preload-on shared pgstat writes now flush one aggregate per torn-down or rescanned scan instead
  of locking once per hot-path event
- the shim now uses a tqvector-owned custom pgstat kind (`PGSTAT_KIND_CUSTOM_MIN + 1`) instead of `PGSTAT_KIND_EXPERIMENTAL`
- the EXPLAIN hook now exits before access-method-name allocation when the `tqvector` option is disabled
- `docs/pg18.md` now documents that the shared snapshot can lag backend-local counters for in-flight scans
- the PG18 EXPLAIN hook registration state now uses `OnceLock` / `AtomicBool` instead of `static mut`
- the focused scan-side raw-pointer Miri regression test passed on this head
