# Review Request: Task 41 Invariant #2 C string/catalog borrow audit

Audit head: `ae3e516d1c1444ccc68d705738c985a2327327d3`

## Summary

This packet covers Phase E from the invariant #2 strategy: PostgreSQL-owned C
strings and catalog/type-name borrows.

The reviewed patterns are either:

- converted to owned `String` before `pfree` of PostgreSQL-allocated strings;
- used synchronously while the owning relation, tuple descriptor, parse node,
  or reloption memory remains live;
- converted to owned reloption strings before returning from the helper.

No C-string backed `&str` was found escaping past the owner free/drop boundary.

## Scope

- Audit-only packet; no code change.
- Covered `src/lib.rs`, AM option parsing, type-name helpers, common explain,
  SPIRE DML/custom-scan helpers, and representative catalog attribute-name
  reads.
- Did not cover page tuple byte views; those remain Phase D buffer/page work.

## Evidence

- `artifacts/c-string-inventory.log` is the full C-string/catalog inventory.
- `artifacts/hnsw-format-type-excerpt.log` shows `format_type_be` converted to
  owned `String` before `pfree`.
- `artifacts/spire-dml-format-type-excerpt.log` shows SPIRE DML type-name and
  attribute-name helpers returning owned `String`.
- `artifacts/custom-scan-attr-name-excerpt.log` shows synchronous tuple
  descriptor attribute-name use during output slot population.
- `artifacts/options-owned-string-excerpt.log` shows reloption strings copied
  to owned `String` before returning.
- `artifacts/git-status.log` records audit worktree context.

## Validation

No tests were run because this is an audit-only packet with no code change.

## Reviewer Focus

- Confirm C-string uses either copy to owned data or stay synchronous under the
  owning PostgreSQL object.
- Confirm no `format_type_be` result is borrowed after its `pfree`.
- Confirm Phase E can be marked complete for invariant #2.
