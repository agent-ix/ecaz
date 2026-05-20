# Review Request: DML Frontdoor Node/Tuple Views

## Summary

Code commit: `55d0ef521fa6` (`Reduce SPIRE DML frontdoor unsafe views`)

This packet starts the direct Task 50 pass on `src/am/ec_spire/dml_frontdoor/mod.rs`, the rank-4 top-15 SPIRE file. It introduces two typed local views:

- `SpireDmlFrontdoorTupleDesc`, which centralizes guarded tuple-descriptor attribute copying for relation catalog helpers.
- `SpireDmlFrontdoorExprNode` plus a range-table-ref view, which centralizes planner `NodeTag` gating before expression/range-table casts.

The slice also clears local clippy noise in this touched file so the remaining clippy failure is the known repo-wide backlog.

## Unsafe Count

| file | before | after | delta | percent | top-15 target status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 160 | 146 | -14 | -8.75% | in progress; 30% target remains 112 or lower |

This is a seed slice, not a ceiling claim. The next DML frontdoor pass should continue with planner-hook/query-context state or a deeper expression-list wrapper to drive the file toward the 30% Task 50 target.

## Risk / Mitigation

Primary risk is a planner/catalog misread if a helper accepts the wrong PG node shape. The new helpers keep the same tag checks at the boundary and return read-only views or copied catalog data only; no planner or catalog pointers escape.

No benchmark was run because this packet does not touch scoring, traversal, cache hot paths, or SPIRE read-efficiency loops.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_spire/dml_frontdoor/mod.rs`: passed, with stable-rustfmt warnings for repo config.
- `git diff --check`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with existing unused-import warnings.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`: failed on the existing repo-wide lint backlog; see `artifacts/clippy-pg18.log`.
- `cargo test dml_frontdoor --lib --no-default-features --features pg18`: built but failed to launch outside PostgreSQL with `undefined symbol: CacheRegisterRelcacheCallback`; see `artifacts/cargo-test-dml-frontdoor.log`.

Artifacts are under `reviews/task-50/010-dml-frontdoor-node-tuple-views/artifacts/`, with artifact metadata in `artifacts/manifest.md`.
