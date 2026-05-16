# Review Request: SPIRE Degraded Skip Report

Code checkpoint: `f3a6c651` (`Report SPIRE degraded skipped nodes`)

## Scope

- Adds `ec_spire_remote_search_degraded_skip_report(...)`.
- The report is dry: it reuses production libpq dispatch planning and does not
  resolve conninfo secrets or open sockets.
- In degraded mode, pre-dispatch blocked remotes are reported one row per
  skipped dispatch with:
  - `node_id`;
  - `skipped_pid_count`;
  - `first_skip_category`;
  - `status`.
- Documents the new diagnostic in `docs/SPIRE_DIAGNOSTICS.md`.
- Marks the Phase 12.7 degraded skipped/stale node reporting row complete.

## Validation

- `git diff --check f3a6c651^ f3a6c651`
- `cargo fmt --check`
- `cargo test degraded_skip_report_lists_each_skipped_node --lib`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and result lines.

## Review Focus

- Confirm the report shape satisfies the Phase 12.7 requirement for node
  identity, count, and first skip category.
- Confirm it is acceptable that this dry diagnostic covers pre-dispatch
  degraded skips, including stale epoch and incompatible version, while live
  transport/candidate/heap skip rows continue to be summarized by the existing
  production executor state surfaces.
- Confirm the row should remain scoped to degraded-skipped dispatches rather
  than returning all remote dispatch rows.
