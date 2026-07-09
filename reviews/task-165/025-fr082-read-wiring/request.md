# Task 165 — packet 025: FR-082 published-epoch read wiring

Coder review request. Closes the packet-020/021 **P1 "FR-082 active epoch
publication is still not wired into reads."** Reads now consume the published
`metadata.active_epoch` instead of the `ec_distann.epoch` session GUC, proven on
the real 3× PG18 fixture.

## Summary

- Every epoch-fingerprint site (scan loop, expand / materialize /
  apply_record_writes / epoch_fingerprint endpoints, CustomScan owner
  materialization, debug expander) now derives the epoch from the persisted
  manifest via `scan_epoch(metadata)` + `placement_directory_for_epoch(epoch)`.
- The scan loop re-reads metadata per attempt, so a scan runs wholly under one
  epoch and a concurrent republish is an FR-082-AC-2 retriable mismatch, not a
  torn result.

## Evidence (`reviews/task-165/025-fr082-read-wiring/`)

- `artifacts/manifest.md` — full change list, command, cited result lines.
- `artifacts/distann-multinode-summary.log`, `artifacts/fixture-run.log`.

Key lines:
- `fr082_published_epoch base_ok=true coord_only_publish_errored=true all_publish_swap_ok=true pass=true`
  — a coordinator-only publish breaks the scan (only possible if reads consume
  `active_epoch`); an all-node publish swaps the epoch losslessly.
- `concurrency_scan_insert_epochswap pass=true` — coordinated all-node epoch
  swaps under load; one-epoch-per-scan (complete or fail-closed, never torn).
- `RECALL_RESULT … mismatched_ids=0`; 12 fault drills pass; GATE PASS.
- 70 ec_distann unit tests pass; clippy clean.

## Status of the packet-021 P1s

- ✅ NFR-020/TC-042 fault taxonomy complete (packet 024, 12/12).
- ✅ FR-082 published-epoch read consumption (this packet).
- ⏳ Full `ecaz bench suite` 10k/50k/100k recall+latency+storage matrix — the
  remaining M3 closeout gate; next.
