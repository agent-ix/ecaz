# Review Request: HNSW Debug AM Scan Guard Reuse

## Summary

This slice continues the HNSW debug unsafe burndown after packet 284.

Code commit: `4a09f35d767a6f4080e3407d23d28a4236ad2cf5`

Changes:

- Extended `DebugAmScan` with scoped `rescan`, `gettuple`, and `with_opaque` helpers.
- Converted normal cleanup probes to use the guard:
  - `debug_rescan_query_dimensions`
  - `debug_rescan_overwrites_query_dimensions`
  - `debug_rescan_with_unused_key_buffer`
  - `debug_gettuple_after_rescan_result`
- Deleted the raw `debug_scan_has_opaque` and `debug_scan_opaque_is_null` helpers; opaque presence reads now live on the descriptor-owning guard.
- Left deliberate error-path probes unchanged because PostgreSQL error control flow is the behavior under test there.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2142 -> 2138`.
- Replaced repeated caller-owned AM cleanup in normal probes with RAII drop.
- Moved opaque pointer presence checks into the guard boundary.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/285-hnsw-debug-am-scan-rescan-guard-reuse/artifacts/manifest.md`

