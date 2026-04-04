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

1. `CREATE EXTENSION tqvector` registers the type, operators, and access method
2. `DROP EXTENSION tqvector CASCADE` cleanly removes all objects
3. The extension supports PostgreSQL 14, 15, 16, and 17
4. `cargo pgrx install --release` produces a valid extension package
