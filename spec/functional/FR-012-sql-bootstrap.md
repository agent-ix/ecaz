---
id: FR-012
title: SQL Bootstrap — Extension Packaging
type: functional-requirement
status: APPROVED
object_type: configuration
traces:
  - US-004
---
# FR-012: SQL Bootstrap — Extension Packaging

## Requirement

The extension SHALL be installable via standard PostgreSQL extension management.

### SQL Objects Created

`CREATE EXTENSION tqvector` SHALL register:

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
comment = 'TurboQuant compressed vector type with HNSW index'
default_version = '0.1.0'
module_pathname = '$libdir/tqvector'
relocatable = false
superuser = true
```

### PostgreSQL Version Support

The extension SHALL compile and install on PostgreSQL 14, 15, 16, and 17 via pgrx feature flags.

## Acceptance Criteria

### FR-012-AC-1: Clean install
`CREATE EXTENSION tqvector` on a fresh database SHALL succeed without errors.

### FR-012-AC-2: Clean uninstall
`DROP EXTENSION tqvector CASCADE` SHALL remove all objects without orphans in pg_type, pg_operator, or pg_am.

### FR-012-AC-3: Multi-version support
`cargo pgrx test` SHALL pass on pg14, pg15, pg16, and pg17.
