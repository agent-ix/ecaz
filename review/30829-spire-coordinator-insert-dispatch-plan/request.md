# Review Request: SPIRE Coordinator Insert Dispatch Plan

## Scope

Narrow ADR-069 coordinator-routed INSERT slice after packet `30828`.

This packet adds the side-effect-free remote dispatch readiness primitive that
the later mutating 2PC INSERT executor will consume after
`ec_spire_plan_coordinator_insert(...)` classifies a row.

This slice:

- Adds `ec_spire_plan_coordinator_insert_dispatch(index_oid, node_id,
  served_epoch)`.
- Reuses the Stage C `ec_spire_remote_node_descriptor` lookup path for the
  classified remote `node_id`.
- Reuses the external conninfo-secret resolution status surface and returns only
  the provider lookup key, never raw conninfo.
- Checks the remote descriptor served-epoch window before reporting a ready
  dispatch action.
- Returns the libpq transport, 2PC protocol name, remote index regclass,
  descriptor generation, identity byte count, dispatch action, status, and next
  step.
- Documents the primitive in ADR-069 and marks the Phase 11 tracker sub-slice.

This intentionally does not open a libpq connection, forward INSERT payloads,
issue `PREPARE TRANSACTION`, mutate `ec_spire_placement`, or claim the final
transparent `INSERT INTO tbl ...` path.

## Validation

- `cargo test insert_dispatch --lib`
  - Passed: 4 PG18 tests.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm the dispatch readiness primitive is the right boundary before the
  mutating remote INSERT/2PC executor.
- Confirm the status model matches Stage C behavior:
  `requires_remote_node_descriptor`, `requires_conninfo_secret_resolution`,
  `stale_epoch`, and `ready`.
- Confirm the SQL surface exposes enough descriptor/secret information for
  operator diagnostics without exposing raw conninfo or implying a row has been
  sent.
- Confirm the Phase 11 tracker still leaves the true remote INSERT, remote
  `PREPARE TRANSACTION`, placement-directory mutation, and end-to-end PG18
  fixture open.

## Artifacts

- `review/30829-spire-coordinator-insert-dispatch-plan/artifacts/manifest.md`
- `review/30829-spire-coordinator-insert-dispatch-plan/artifacts/cargo-test-insert-dispatch-lib.log`
- `review/30829-spire-coordinator-insert-dispatch-plan/artifacts/cargo-fmt-check.log`
- `review/30829-spire-coordinator-insert-dispatch-plan/artifacts/git-diff-check.log`
- `review/30829-spire-coordinator-insert-dispatch-plan/artifacts/git-diff-cached-check.log`
