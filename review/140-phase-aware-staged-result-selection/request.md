# Request: Phase-Aware Staged Result Selection

Commit: `ecf2f76`

Summary:
- route staged scan result selection in `src/am/scan.rs` through one phase-aware `select_next_scan_result` helper
- keep bootstrap-specific refill-after-success behavior inside bootstrap selection, while making top-level staged result materialization use the same selection seam for both bootstrap and linear phases
- move the linear exhaustion transition fully into linear selection so the shared materialization path only handles “selected result vs none”

Please review:
- whether the new `select_next_scan_result` seam preserves the intended bootstrap-to-linear fallthrough behavior on the same `amgettuple` call
- whether bootstrap refill-after-success still happens only for the candidate that actually selects/materializes
- whether the linear exhaustion transition still happens at the correct boundary now that exhaustion is owned by `select_next_linear_scan_result`
