# Review Request: HNSW Debug AM Scan Lifecycle Guard

## Summary

This slice continues the HNSW debug unsafe burndown after packet 283.

Code commit: `2805f8b215045e4b088e6cf3e80230935ba1b470`

Changes:

- Added `DebugAmScan`, a local RAII guard for HNSW debug AM scan descriptors.
- Converted `debug_begin_end_scan` to use the guard for AM cleanup and descriptor release.
- Converted `debug_end_scan_twice` to use the guard while preserving the deliberate second AM cleanup call for the idempotence probe.
- Moved opaque-presence checks behind guard methods so callers no longer own raw descriptor checks.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2145 -> 2142`.
- Removed caller-local raw opaque checks from the begin/end probes.
- Removed explicit caller-owned `debug_am_end_scan` / `debug_index_scan_end` cleanup from those probes.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/284-hnsw-debug-am-scan-lifecycle-guard/artifacts/manifest.md`

