# Review Request: Task 41 HNSW Shared Debug Index Guard

Code commit: `20eed29accae03c35053c278d68f695adf6bda82`

## Summary

This checkpoint wraps pg_test-only HNSW shared debug index relation opens in
`src/am/ec_hnsw/shared.rs`.

- Adds `DebugAccessShareIndexRelation`.
- Migrates `debug_index_pages`, `debug_planner_tuning_snapshot`,
  `debug_index_metadata`, `debug_update_index_metadata`, and
  `debug_vacuum_stats`.
- Replaces five manual `index_open` / `index_close` pairs with guard-owned
  cleanup.

## Safety Delta

- Baseline entries: `4311` -> `4301`.
- `src/am/ec_hnsw/shared.rs`: `116` -> `106`.
- This keeps debug helper behavior scoped to `AccessShareLock` while preventing
  relation leaks through early error paths.

## Reviewer Focus

- Confirm the guard lifetime covers all raw relation reads and callback calls,
  especially `debug_vacuum_stats` where `info.index` borrows from the guard.
- Confirm preserving `AccessShareLock` matches the previous debug helper
  behavior, including `debug_update_index_metadata`.
- Confirm all returned debug values are owned and do not borrow from the
  relation after the guard drops.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see
`artifacts/manifest.md`.
