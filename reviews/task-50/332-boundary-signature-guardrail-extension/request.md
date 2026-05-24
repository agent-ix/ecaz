# Review Request: Boundary Signature Guardrail Extension

Task: 50 unsafe burndown

Commit under review:

- `727127da` - `Broaden unsafe boundary signature warning`

## Summary

This packet addresses the reviewer note on packet 328 that the pre-commit guardrail should cover more than `pg_sys::Relation`.

The warning in `scripts/check_unsafe_comments.sh` now scans safe public helper signatures for a broader set of raw PostgreSQL boundary types:

- `Relation`
- `IndexScanDesc`
- `StringInfo`
- `ParamListInfo`
- `Query`
- `PlannerInfo`
- `RelOptInfo`
- `Node`
- `Expr`
- `List`
- `TupleTableSlot`
- `ScanKey`
- `IndexBuildHeapScan`
- `IndexVacuumInfo`
- `IndexBulkDeleteResult`

The check remains warning-only. It currently identifies four existing cleanup targets, including IVF and HNSW debug/vacuum surfaces.

## Validation

- `bash -n scripts/check_unsafe_comments.sh` passed. See `artifacts/check-unsafe-comments-bash-n.log`.
- The broadened grep reports the current raw-boundary signatures. See `artifacts/boundary-signature-guard.log`.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.

`bash scripts/check_unsafe_comments.sh` emits the new warning and existing unsafe-comment baseline drift; see `artifacts/check-unsafe-comments.log`.

## Reviewer Focus

- Confirm the expanded boundary-type list is broad enough for the repeated safe-raw-pointer-signature regression class.
- Confirm keeping this warning-only is acceptable while the existing signatures are burned down in follow-up packets.
