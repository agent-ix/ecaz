---
id: StR-002
title: MIT-Licensed Extension Owned by Agent-IX
type: stakeholder-requirement
status: APPROVED
derived_usecases:
  - US-004
---
# StR-002: MIT-Licensed Extension Owned by Agent-IX

## Need

The agent memory system is a product component distributed to external users. All database extensions must have licensing compatible with commercial distribution.

## Expectation

The `tqvector` extension SHALL be MIT licensed and fully owned by Agent-IX. It SHALL depend only on crates with permissive licenses (MIT, Apache-2.0, BSD).

## Rationale

- pgvecto.rs: deprecated
- VectorChord: AGPLv3 / ELv2 — blocks commercial redistribution
- Building our own ensures license control and the ability to ship as part of the product

## Success Criteria

- LICENSE file declares MIT
- `cargo deny check licenses` passes with no copyleft violations
