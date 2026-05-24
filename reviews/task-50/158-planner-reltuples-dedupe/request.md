# Review Request: Planner Reltuples Helper Dedupe

## Summary

This checkpoint addresses the soundness-audit finding that `common/cost.rs::relation_reltuples` duplicated the canonical storage relation facade.

Code commit: `54376500487187cece4ec08079c480ce2788d0f2`

The reviewer was correct: the duplicate helper added another raw-relation contract surface. This slice removes that facade and routes HNSW, IVF, SPIRE, DiskANN, and common planner cost code to `crate::storage::relation::relation_reltuples`.

## Scope

- Deleted `src/am/common/cost.rs::relation_reltuples`.
- Updated AM cost callers to use the storage relation helper directly.
- Kept caller-side unsafe acknowledgments at the planner/cost relation boundary.

## Completion Audit Note

This packet closes one listed audit finding. It does not close Task 50. The comprehensive plan still requires every direct unsafe row to be removed or residual-registered, and current `make unsafe-block-count` output still contains many unsafe rows.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
