---
id: FR-026
title: PG18 Module Identity and Version Reporting
type: functional-requirement
status: DRAFT
object_type: configuration
traces:
  - US-004
  - StR-004
---
# FR-026: PG18 Module Identity and Version Reporting

## Requirement

On PG18, the extension SHALL use `PG_MODULE_MAGIC_EXT` to declare its name and version, making this information available via `pg_get_loaded_modules()` for diagnostics and version tracking.

### Implementation

```rust
#[cfg(feature = "pg18")]
PG_MODULE_MAGIC_EXT(
    .name = "tqvector",
    .version = env!("CARGO_PKG_VERSION"),  // reads from Cargo.toml
);

#[cfg(not(feature = "pg18"))]
PG_MODULE_MAGIC;
```

### Observable Behavior

```sql
SELECT * FROM pg_get_loaded_modules() WHERE name = 'tqvector';
-- name     | version
-- tqvector | 0.1.0
```

### PG Version Compatibility

On PG17, the standard `PG_MODULE_MAGIC` macro is used. The extension name and version are not available via `pg_get_loaded_modules()`.

## Acceptance Criteria

### FR-026-AC-1: Module visible
On PG18, `SELECT name, version FROM pg_get_loaded_modules() WHERE name = 'tqvector'` SHALL return one row with the correct version.

### FR-026-AC-2: Version matches Cargo.toml
The reported version SHALL match the `version` field in `Cargo.toml`.

## References

- PG source: `src/include/fmgr.h` — `Pg_magic_struct` definition (`.name`, `.version` fields), `PG_MODULE_MAGIC_EXT` macro, `PG_MODULE_MAGIC_DATA` initializer
- PG source: `src/backend/utils/fmgr/dfmgr.c` — `pg_get_loaded_modules()` function that exposes module name/version
