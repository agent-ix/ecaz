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

Productization planning now includes PostgreSQL 18 support, but the repository has not yet added a
`pg18` Cargo feature or validation matrix. The project therefore needs an upgrade direction before
implementation work starts.

## Decision

PostgreSQL 18 support SHALL preserve the existing extension identity:

- keep the extension name as `tqvector`
- keep `module_pathname = '$libdir/tqvector'`
- keep one control file and one SQL extension lineage

The PG18 effort is an in-place compatibility upgrade, not a forked module identity. There will be
no separate `tqvector_pg18` library, control file, or SQL extension name.

Near-term upgrade plan:

1. add the `pg18` Cargo feature once the pgrx/toolchain lane is ready
2. extend CI and local validation to include PostgreSQL 18
3. verify `CREATE EXTENSION tqvector` and upgrade/install flows still work under the unchanged
   extension identity

## Consequences

### Benefits

- Users keep one extension name across supported PostgreSQL majors.
- Upgrade, packaging, and documentation stay aligned with normal PostgreSQL extension workflows.
- Planner/productization work can target one stable identity while PG18 support is being added.

### Tradeoffs

- PG18 support remains explicitly pending until the toolchain and test matrix are in place.
- Compatibility issues in pgrx or PostgreSQL 18 must be resolved without the escape hatch of a
  second extension identity.

## Follow-Up

1. Add `pg18` feature support in Cargo and validation scripts.
2. Update US-004 and packaging docs when PG18 is actually supported, not before.
3. Keep migration and upgrade testing under the same `tqvector` extension name.
