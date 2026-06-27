---
id: FR-012
title: SQL Bootstrap — Extension Packaging
type: FR
status: APPROVED
object: configuration
traces:
  - US-004
---
# FR-012: SQL Bootstrap — Extension Packaging

## Description

The extension SHALL be installable via standard PostgreSQL extension management.

### SQL Objects Created

`CREATE EXTENSION ecaz` SHALL register:

1. **Type**: `tqvector` (with in/out/send/recv functions)
2. **Functions**:
   - `encode_to_tqvector(float4[], int, bigint) → tqvector`
   - `tqvector_inner_product(tqvector, tqvector) → float4`
   - `tqvector_negative_inner_product(tqvector, tqvector) → float4`
   - `tqvector_query_inner_product(tqvector, float4[]) → float4`
   - `tqvector_negative_query_inner_product(tqvector, float4[]) → float4`
3. **Operators**:
   - `<#>` (tqvector, tqvector) → float4
   - `<#>` (tqvector, float4[]) → float4
4. **Access Method**: `ec_hnsw`
5. **Operator Class**: `tqvector_ip_ops` DEFAULT FOR TYPE tqvector USING ec_hnsw

### Implementation

- Use `extension_sql_file!` macros in pgrx pointing to `sql/bootstrap.sql`
- The bootstrap SQL file SHALL be version-controlled and auditable

### Extension Control File

```
comment = 'Ecaz compressed vector extension with HNSW index'
default_version = '0.1.1'
module_pathname = '$libdir/ecaz'
relocatable = false
superuser = true
```

### PostgreSQL Version Support

The extension SHALL compile and install on PostgreSQL 17 and 18 via pgrx feature flags, with PG18 as the default build target.

## Configuration

The extension control file (`ecaz.control`) declares the following settings, all fixed at extension-creation time.

| Name | Scope | Type | Default | Description |
|---|---|---|---|---|
| comment | creation | string | `Ecaz compressed vector extension with HNSW index` | Human-readable extension comment shown in catalogs. |
| default_version | creation | string | `0.1.1` | Version installed by `CREATE EXTENSION ecaz` when no version is specified. |
| module_pathname | creation | string | `$libdir/ecaz` | Path used to resolve the shared library for `LANGUAGE c` functions. |
| relocatable | creation | boolean | `false` | Extension objects cannot be moved to another schema after install. |
| superuser | creation | boolean | `true` | Only a superuser may run `CREATE EXTENSION ecaz`. |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-012-AC-1 | `CREATE EXTENSION ecaz` succeeds on a fresh database | Test |
| FR-012-AC-2 | `DROP EXTENSION ecaz CASCADE` removes all objects without orphans in pg_type, pg_operator, or pg_am | Test |
| FR-012-AC-3 | `cargo pgrx test pg17` and `cargo pgrx test pg18` both pass | Test |

### FR-012-AC-1: Clean install
`CREATE EXTENSION ecaz` on a fresh database SHALL succeed without errors.

### FR-012-AC-2: Clean uninstall
`DROP EXTENSION ecaz CASCADE` SHALL remove all objects without orphans in pg_type, pg_operator, or pg_am.

### FR-012-AC-3: Multi-version support
`cargo pgrx test pg17` and `cargo pgrx test pg18` SHALL both pass.

## Dependencies

- **Upstream**: US-004 (traces)
- **Downstream**: none identified
