# Review Request: SPIRE DML Plan-Tree ADR Limits

## Scope

Doc commit: `c446598668c4b1d1d77d517537129af011089953`

This packet updates ADR-069 after the transparent UPDATE/DELETE CustomScan
executor slices landed.

Changes:

- Replaces stale ModifyTable/view-hook wording for transparent UPDATE with the
  accepted planner-hook plan-tree replacement contract.
- Adds the matching transparent DELETE plan-tree replacement wording.
- Documents the v1 limitations implied by bypassing PostgreSQL `ModifyTable`:
  no `RETURNING`, no coordinator table row-level triggers, and no statement
  transition tables for transparent distributed UPDATE/DELETE.
- Updates the Phase 11 task file to mark this documentation item complete.

## Validation

- `git diff --check c4465986^ c4465986 -- spec/adr/ADR-069-spire-distributed-write-path-scope.md`
  - passed for the ADR commit
  - artifact: `artifacts/git-diff-check.log`

No Rust or PG18 tests were run for this doc-only packet.

## Review Focus

1. Confirm ADR-069 now matches the accepted top-level CustomScan plan-tree
   replacement approach.
2. Confirm the documented v1 limitations are clear and do not overpromise
   trigger or `RETURNING` semantics.
