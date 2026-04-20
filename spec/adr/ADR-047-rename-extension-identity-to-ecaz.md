---
id: ADR-047
title: Rename extension identity from tqvector to ecaz
status: DECIDED
supersedes: ADR-017
impact: MEDIUM for FR-012, FR-024, FR-025, FR-026, US-004, US-009, US-011
date: 2026-04-20
---
# ADR-047: Rename extension identity from tqvector to ecaz

## Context

The original extension name, `tqvector`, came from the initial TurboQuant-only product shape.
That no longer matches the current direction:

- the extension now exposes the canonical `ecvector(dim)` row type for general vector storage
- `tqvector` remains as the explicit TurboQuant family-specific artifact/debugging surface
- the project is expected to grow beyond a single quantized family

Keeping `tqvector` as the extension, module, and diagnostics identity would keep the whole product
named after one family-specific artifact.

The PG18 shared-infrastructure landing already concentrated the user-visible identity seams in one
place: extension packaging, module magic, `pg_get_loaded_modules()`, custom EXPLAIN option naming,
custom pgstat registration, and the SQL statistics/admin surface. That makes a clean rename
possible without carrying compatibility aliases.

## Decision

Rename the extension identity from `tqvector` to `ecaz`.

The live extension/admin identity SHALL be:

- extension name: `ecaz`
- control file: `ecaz.control`
- module pathname: `$libdir/ecaz`
- module identity in `pg_get_loaded_modules()`: `ecaz`
- EXPLAIN option: `ecaz`
- SQL statistics/admin surface: `ecaz_stats()`
- preload activation name: `shared_preload_libraries = 'ecaz'`

The existing `tqvector` datum, functions, operators, and operator class remain the TurboQuant
family-specific SQL surface for now. This ADR does not rename the TurboQuant artifact type.

No deprecated compatibility alias SHALL be kept for the old extension identity. Operators either
use `ecaz` or migrate explicitly.

## Consequences

### Benefits

- Product identity matches the broader platform direction instead of one family-specific artifact.
- Current PG18 diagnostics and module-identity surfaces line up under one consistent name.
- Future quantized families can coexist without the extension being named after TurboQuant.

### Tradeoffs

- Existing databases using `CREATE EXTENSION tqvector` need an explicit migration path rather than
  an in-place alias.
- Scripts, docs, validation harnesses, and operator runbooks must all cut over to `ecaz` together.
- The mixed world remains temporarily visible in SQL because the extension is `ecaz` while the
  TurboQuant family surface is still `tqvector`.

## Follow-Up

1. Keep the rename limited to extension/module/admin identity for this slice.
2. Preserve the `tqvector` datum and TurboQuant-specific SQL surface until there is a separate
   family-surface rename or retirement decision.
3. Keep the explicit `pg_module_magic!` name/version workaround until the upstream `pgrx` PG18
   shorthand issue is fixed.
