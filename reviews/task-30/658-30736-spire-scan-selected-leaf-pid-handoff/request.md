# Review Request: SPIRE Scan Selected Leaf PID Handoff

Review the Stage C/C5 scan-routing precursor in `1ff04e6d`:
`Add SPIRE scan selected leaf PID handoff`.

## Change

- Added `collect_scan_plan_selected_leaf_pids(...)` to return the leaf PIDs
  selected by the AM scan plan without reading leaf payload objects.
- Added coordinator-local routing/top-graph loaders that skip non-local
  placements before object-store reads. This keeps remote leaf placements from
  being treated as local object payloads during route selection.
- Added a unit fixture that marks one selected leaf placement as remote and
  invalid for local object-store reads. The new helper still returns both
  selected PIDs, while the existing row-loading path fails on that snapshot.

## Why

Production remote fanout needs the selected remote leaf PIDs before it can hand
them to the executor/receive pipeline. The existing routed-row helpers combine
route selection with local leaf payload reads, which is correct for local scans
but the wrong boundary for multi-instance AM scan integration.

This slice keeps the local row path intact and introduces a routing-only handoff
that can feed C5 without requiring remote leaf payloads to exist in the
coordinator object store.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test collect_scan_plan_selected_leaf_pids --lib`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm the routing-only helper is the right precursor before AM scan invokes
  production remote candidate receive.
- Confirm skipping non-local placements during coordinator routing metadata load
  is the right boundary for now.
- Confirm the fixture proves remote leaf payloads are not read locally while
  preserving the existing routed-row behavior.
