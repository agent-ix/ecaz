# Review Request: SPIRE Local Store Relation Name Plan

- Branch: `task30-spire-partition-object-spec`
- Code commit: `db0b4b7b7c0ffbd57fe268ed366332177dd996ed`
- Scope: Phase 4 local-store relation planning before auxiliary store DDL

## Summary

This checkpoint adds a deterministic planning layer for future SPIRE auxiliary
local store relations.

It:

- adds `SpireLocalStoreRelationPlanEntry` with `(local_store_id,
  relation_name, tablespace_oid)`;
- derives store relation names as `ec_spire_store_<index_oid>_<store_id>`;
- validates that an index relid is available and that the generated name fits
  PostgreSQL's identifier limit;
- sorts planned stores by `local_store_id` and rejects duplicate store IDs;
- preserves repeated tablespace OIDs so same-device baseline runs remain
  representable;
- wires the ambuild preflight path from resolved tablespace OIDs into relation
  planning while keeping `local_store_count > 1` blocked until store relation
  creation/opening lands.

This is a DDL precursor only. It does not create auxiliary store relations,
record dependencies, or persist a new active store config generation.

## Files

- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

- Whether deterministic names based on `index_oid` and `store_id` are the right
  first relation-discovery surface before `ChooseRelationName`/collision
  handling lands in the DDL helper.
- Whether preserving repeated tablespace OIDs in the relation plan is sufficient
  for same-NVMe baseline testing.
- Whether ambuild should derive the relation plan before or after the current
  `local_store_count > 1` guard. This patch derives it before the guard so name
  and tablespace errors fail early and are testable independently.

## Validation

- `cargo test local_store_relation --lib`
- `cargo test local_store_tablespace_plan --lib`
- `cargo pgrx test pg18 test_ec_spire_options_snapshot_sql`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
