---
id: ADR-017
title: "Keep a single tqvector extension identity through the PostgreSQL 18 upgrade"
status: DECIDED
impact: MEDIUM for FR-012, US-004
date: 2026-04-06
---
# ADR-017: Keep a single tqvector extension identity through the PostgreSQL 18 upgrade

## Context

The current packaged surface is a single PostgreSQL extension:

- control file: `tqvector.control`
- module pathname: `$libdir/tqvector`
- SQL identity: `CREATE EXTENSION tqvector`

Productization planning now includes PostgreSQL 18 support, and the repository has since added the
`pg18` Cargo feature, PG17/PG18 validation lanes, and PG18 module-identity coverage. The project
still needed an explicit upgrade direction before implementation work started so the live landing
would not fork the packaged extension identity.

## Decision

PostgreSQL 18 support SHALL preserve the existing extension identity:

- keep the extension name as `tqvector`
- keep `module_pathname = '$libdir/tqvector'`
- keep one control file and one SQL extension lineage

The PG18 effort is an in-place compatibility upgrade, not a forked module identity. There will be
no separate `tqvector_pg18` library, control file, or SQL extension name.

Implementation status:

1. the `pg18` Cargo feature is live and is now the default target
2. CI and local validation now include PostgreSQL 18 alongside the PG17 fallback lane
3. PG18 module identity is verified through `pg_get_loaded_modules()` while preserving the same
   `CREATE EXTENSION tqvector` surface and `$libdir/tqvector` library identity

## Consequences

### Benefits

- Users keep one extension name across supported PostgreSQL majors.
- Upgrade, packaging, and documentation stay aligned with normal PostgreSQL extension workflows.
- Planner/productization work can target one stable identity while PG18 support is being added.

### Tradeoffs

- Compatibility issues in pgrx or PostgreSQL 18 still need to be resolved without the escape hatch
  of a second extension identity.
- The current `pgrx 0.17` PG18 lane requires an explicit-field `pg_module_magic!` workaround until
  the shorthand behavior is fixed upstream.

## Follow-Up

1. Keep migration and upgrade testing under the same `tqvector` extension name.
2. Keep PG17 fallback working while PG18 remains the primary target.
3. Consider upstreaming the observed `pg_module_magic!(name, version)` PG18 shorthand issue to
   `pgrx`.
