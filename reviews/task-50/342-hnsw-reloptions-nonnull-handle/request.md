# Review Request: HNSW Reloptions NonNull Handle

Task: 50 unsafe burndown

Commit under review:

- `d3a75515` - `Use non-null HNSW reloptions relation handle`

## Summary

This packet removes the remaining broadened raw-boundary guard hit.

- Changes `ec_hnsw::options::relation_options` to accept `HnswIndexRelation`, a `NonNull<RelationData>` alias, instead of a safe public `pg_sys::Relation` argument.
- Converts HNSW relation-options callers to construct a checked non-null relation handle at the call site.
- Keeps reloptions pointer access centralized inside the HNSW reloptions view.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1315` to `1314`.
- The broadened boundary-signature guard has no hits.

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1314` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm `HnswIndexRelation` avoids the safe `pg_sys::Relation` signature antipattern.
- Confirm all updated callers already operate on live HNSW index relations and only add explicit non-null construction.
