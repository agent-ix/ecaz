# Review Request: PG18 Upgrade Boundary Snapshot

Scope:
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/adr/ADR-017-pg18-module-identity-and-upgrade-direction.md`
- `spec/functional/FR-026-pg18-module-identity.md`
- `spec/functional/FR-027-pgrx-pg18-upgrade.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a read-only SQL/admin surface, `tqhnsw_pg18_upgrade_snapshot()`, that reports the current
  extension identity (`tqvector`, `$libdir/tqvector`), Cargo package version, current default
  feature, and explicit readiness flags for `pg18` Cargo-feature support, PG18 default-build
  promotion, and `PG_MODULE_MAGIC_EXT`.
- Kept all PG18 readiness flags false, because the repository still has `default = ["pg17"]`,
  no `pg18` feature, and no live module-magic-ext wiring.
- Added pg coverage that asserts the stable single-extension identity while making the pending
  toolchain boundary explicit.
- Updated ADR-017, FR-026, FR-027, the test matrix, and Task 11 notes so this snapshot is recorded
  as productization/planner scaffolding rather than actual PG18 support.

Review focus:
- Whether `tqhnsw_pg18_upgrade_snapshot()` is a useful planner/productization seam for reconciling
  the expanded PG18 spec surface with current implementation reality
- Whether surfacing the current default feature and pending `pg18` readiness flags is explicit
  enough without overcommitting the eventual upgrade implementation
- Whether the snapshot and docs stay aligned with ADR-017’s single-extension-identity decision

Questions to answer:
- Is `tqhnsw_pg18_upgrade_snapshot()` the right near-term boundary, or should this information stay
  purely in ADR/spec text until PG18 toolchain work lands?
- Are the three readiness flags the right productization-level signals, or should another boundary
  be surfaced now for the eventual upgrade lane?
- Does this make the FR-026 / FR-027 current-vs-target story clearer for future implementation and
  review work?
