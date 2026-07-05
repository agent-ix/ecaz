# Review Request: IVF Reloptions NonNull Relation Handles

Task: 50 unsafe burndown

Commit under review:

- `44eabec1` - `Require NonNull IVF relation options handles`

## Summary

This packet removes the raw-relation API from IVF reloptions lookup.

- Changes `EcIvfReloptionsView::from_relation` and `options::relation_options` to take `NonNull<RelationData>`.
- Updates IVF build, build-empty, admin snapshot, and amrescan callers to validate the index relation before reading reloptions.
- Reuses `IvfScanDescView::index_relation_nonnull(...)` in amrescan so the descriptor view remains the scan callback boundary.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1306` to `1305`.
- IVF reloptions callers now pass checked relation handles.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/ivf-reloptions-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1305` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm `NonNull<RelationData>` is an acceptable boundary for IVF reloptions reads.
- Confirm the updated build/admin/scan call sites preserve the prior null-relation failure behavior.
