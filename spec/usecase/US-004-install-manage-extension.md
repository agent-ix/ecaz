---
id: US-004
title: Install and Manage Extension Lifecycle
type: user-story
status: APPROVED
traces:
  - StR-002
---
# US-004: Install and Manage Extension Lifecycle

**As** a platform engineer,
**I want** to install, upgrade, and remove the extension using standard PostgreSQL commands,
**So that** it integrates with existing database management tooling and workflows.

## Acceptance Criteria

1. `CREATE EXTENSION ecaz` registers the type, operators, and access method
2. `DROP EXTENSION ecaz CASCADE` cleanly removes all objects
3. The current packaged extension supports PostgreSQL 17 and 18
4. `cargo pgrx install --release` produces a valid extension package

Current productization direction:

- PostgreSQL 18 support now ships under the `ecaz` extension identity, while the original
  `tqvector` datum and operator surfaces remain in place as the TurboQuant-specific family surface,
  per ADR-047.
