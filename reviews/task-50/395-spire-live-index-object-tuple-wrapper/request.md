# Review Request: SPIRE Live Index Object Tuple Wrapper

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/395-spire-live-index-object-tuple-wrapper`
- code commit: `edccd563b38b96cfd8b55a0a29853b42f4286249`
- scope:
  - `src/am/ec_spire/coordinator/snapshots.rs`
  - `src/am/ec_spire/coordinator/diagnostics.rs`
  - `src/am/ec_spire/custom_scan/planner.rs`

## Summary

This slice adds `SpireLiveIndexRelation::object_tuple`, a scoped safe wrapper
for reading owned object-tuple bytes from a live SPIRE index relation.

Two call sites now use that relation-owned wrapper instead of opening their own
caller-side unsafe blocks around `page::read_object_tuple`:

- CustomScan planner placement-directory eligibility loading.
- Boundary-placement diagnostics manifest loading.

The low-level page helper remains the unsafe boundary; callers now express the
relation lifetime through `SpireLiveIndexRelation`.

## Unsafe Count

- before this slice: `1121`
- after this slice: `1120`
- movement: two caller-side unsafe blocks removed, one relation-owned object
  tuple boundary added.

## Plan Coverage

- Program: P2/P4, PostgreSQL Handle Views and page tuple views.
- Wave/tranche: Wave 2 SPIRE production fanout, live relation object-tuple
  wrapper reuse.
- Disposition: direct object-tuple page reads are absorbed into the existing
  live-index relation contract.

## Validation

Artifacts are packet-local under
`reviews/task-50/395-spire-live-index-object-tuple-wrapper/artifacts/`.

- `rustfmt-check.log`: passed; only existing stable-toolchain warnings for
  unstable rustfmt settings.
- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML
  re-export warning in `src/am/mod.rs`.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: no matches.
- `src-unsafe-count.log`: `1120`.
- `unsafe-ledger-generate.log`: generated `1120` current `src` ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers `1120` current unsafe rows.
