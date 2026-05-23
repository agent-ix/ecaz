# Review Request: SPIRE DML Frontdoor Expression Boundaries

## Summary

This checkpoint clears the remaining main soundness-audit grep hits for safe helpers accepting raw PostgreSQL expression/list pointers.

Code commit: `7ac915eff4fcb8a319e26f936f665e9f84d146ff`

The reviewer was correct: DML frontdoor expression walkers and list readers consume planner-owned `Expr` and `List` pointers and should not be safe APIs. This slice marks those helpers unsafe and adds call-site acknowledgments in the planner analysis path and unit tests.

## Scope

- Marked DML frontdoor expression-node readers unsafe.
- Marked predicate-value, var-column, coercion-wrapper, and single-list-argument helpers unsafe.
- Marked DML frontdoor `PgList`/raw-ref helper views unsafe.
- Added explicit safety blocks at immediate planner-expression/list call sites and tests.
- The primary raw-pointer helper grep used for this pass now returns no hits.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed; count increase is expected for this explicit-boundary pass.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
