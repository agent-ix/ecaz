# SPIRE Unsafe Boundary Classification

Code checkpoint: `1ea3b750c29a60627f8c3e196afd7110ba252887`

The scoped audit commands were:

```text
git grep -n unsafe b9a028cd -- src/am/ec_spire/dml_frontdoor src/am/ec_spire/update
rg -n 'unsafe' src/am/ec_spire/dml_frontdoor src/am/ec_spire/update
```

The task text still names the old `dml_frontdoor.rs`; the current path is
`src/am/ec_spire/dml_frontdoor/mod.rs`.

## Count Summary

- Scoped `dml_frontdoor` plus `update` lines:
  - Before: 247, including 7 test-only dml-frontdoor lines.
  - After: 244, including 7 test-only dml-frontdoor lines.
- Full `src/am/ec_spire` unsafe-bearing lines:
  - Before: 1430.
  - After: 1427.

The reduction is in `src/am/ec_spire/update/publish/relation.rs`, which drops
from 15 to 12 unsafe-bearing lines.

## Fixed Avoidable Sites

The code checkpoint makes these relation object writer helpers safe:

- `write_relation_replacement_objects`
- `write_relation_scheduled_replacement_objects`

Both helpers only call the existing generic writer over
`SpireReplacementObjectWriter`. The actual relation mutation remains
encapsulated in the `SpireReplacementObjectWriter for SpireRelationObjectStore`
implementation, where it calls the relation-backed object store methods. That
keeps the unsafe surface at the storage/relation boundary instead of exposing it
through business-logic publish helpers.

## Remaining Category (a) FFI/SPI-Boundary Groups

- `src/am/ec_spire/dml_frontdoor/mod.rs`
  - Planner hook installation and previous-hook chaining.
  - Relcache callback registration and callback state.
  - `pg_sys::Query`, `Node`, `RangeTblEntry`, `Expr`, `Var`, `Const`, `Param`,
    `TargetEntry`, `RestrictInfo`, and `List` pointer traversal.
  - PostgreSQL catalog/relation inspection through relation descriptors,
    tuple descriptors, index descriptors, type names, and C strings.
  - Datum and bound-parameter extraction.
  - Plan-tree and CustomScan plan expression construction.

  These are classified as FFI/SPI boundary work. The file still mixes a large
  amount of PostgreSQL tree traversal with the DML front-door feature, but the
  pure classifier is already represented by safe data structs and safe tests.
  A mechanical split into `dml_frontdoor/ffi.rs` would be a larger churn-only
  refactor; this packet keeps the 12b cleanup to the audited reduction and
  records that the remaining unsafe sites are boundary sites, not category (b)
  avoidable sites.

- `src/am/ec_spire/update/types.rs`
  - Relation-store writer trait implementation calls relation object-store
    insertion methods. These remain FFI/storage-boundary calls.

- `src/am/ec_spire/update/materialization.rs`
  - Heap relation, snapshot, slot, and indexed-source vector loading for split
    replacement source rows. These are PostgreSQL heap access boundaries.

- `src/am/ec_spire/update/routing.rs`
  - Relation scheduled replacement input construction from heap sources; the
    remaining unsafe call delegates to the heap-source materialization boundary.

- `src/am/ec_spire/update/publish/relation.rs`
  - Relation epoch publish paths still read root/control state, load relation
    local-store config, write placement entries, and publish manifest bundles
    through PostgreSQL relation/page APIs.

## Category (b) and Business-Layer Result

- Category (b), avoidable unsafe: fixed in the code checkpoint by making the
  two relation writer helpers safe.
- Business-logic unsafe requiring a local `// SAFETY:` comment: none identified
  after the fixed sites. Remaining sites are classified as FFI/SPI or
  storage/relation boundary calls.
