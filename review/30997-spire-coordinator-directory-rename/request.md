# Review Request: SPIRE Coordinator Directory Rename

- Code commit: `317beec3` (`Rename SPIRE root directory to coordinator`)
- Task: Task 30 Phase 12b.5, `root/` -> `coordinator/`
- Scope: mechanical source-layout rename; no SQL/operator identifier rename

## Summary

This checkpoint renames the internal SPIRE coordinator implementation directory:

- from `src/am/ec_spire/root/`
- to `src/am/ec_spire/coordinator/`

The old `root/` directory was not a Rust `root` module; `ec_spire/mod.rs`
included those files directly into the `ec_spire` module. The code change is
therefore the directory move plus include-path updates in `src/am/ec_spire/mod.rs`.

## Operator-Visible Naming Decision

No SQL functions, catalog identifiers, diagnostics, or `root/control` wording are
renamed in this checkpoint. Those names are operator-visible and describe the
root/control relation state rather than the Rust implementation directory. This
matches the Phase 12b default decision.

## Validation

- PG18 compile check:
  `review/30997-spire-coordinator-directory-rename/artifacts/cargo-check-pg18.log`
- Format check:
  `review/30997-spire-coordinator-directory-rename/artifacts/cargo-fmt-check.log`
- Rust module-path sanity:
  `review/30997-spire-coordinator-directory-rename/artifacts/git-grep-root-module-paths.log`
- Stale source include/path sanity:
  `review/30997-spire-coordinator-directory-rename/artifacts/src-stale-root-path-search.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm this is a pure layout rename with no behavioral surface change.
2. Confirm leaving SQL/operator `root` names stable is the right call.
3. Confirm there are no stale Rust source include paths under `src/am/ec_spire`.
