---
id: ADR-010
title: "Match duplicate tqvector entries on gamma plus code bytes"
status: DECIDED
impact: HIGH for FR-001, FR-009, FR-010
date: 2026-04-05
---
# ADR-010: Match duplicate tqvector entries on gamma plus code bytes

## Context

`tqvector` stores `(dimensions, bits, seed, gamma, code_bytes)`.

The build path and live `aminsert` path previously treated two values as duplicates when their
stored `code_bytes` matched. That was too weak.

The SQL-facing raw-query scorer uses the persisted `gamma` term. Two `tqvector` values with the
same code bytes but different `gamma` values can therefore score differently against the same
query. Coalescing those rows into one index element would collapse distinct candidate state.

The index page layout still stores only `code_bytes` inside `TqElementTuple`, not `gamma`.

## Decision

Duplicate matching SHALL use `(gamma, code_bytes)`, not code bytes alone.

Specifically:

- Build-time duplicate coalescing matches only when both `gamma` and `code_bytes` are equal.
- Live `aminsert` duplicate coalescing, when it finds a same-code candidate element, reads the
  representative heap row for that element and compares its persisted `gamma` against the new
  tuple before coalescing.

This keeps the current on-disk element layout unchanged while restoring correct duplicate
semantics for query scoring.

## Consequences

### Benefits

- Distinct persisted candidates no longer collapse just because their quantized code bytes match.
- Existing live duplicate coalescing remains available for truly identical `(gamma, code_bytes)`
  pairs.
- No immediate page-layout change is required.

### Tradeoffs

- Live duplicate detection is now more expensive on code matches because it must read a
  representative heap row to recover `gamma`.
- The element tuple still does not carry enough state for exact query scoring inside future
  ordered index traversal.

## Follow-Up

Later scan and storage slices should revisit this boundary:

1. consider persisting `gamma` in element tuples to remove representative heap fetches
2. make candidate scoring consume `(gamma, code_bytes)` directly without reconstructing temporary
   payload buffers
3. only enable planner-visible ordered search after candidate-side scoring state is fully
   available inside the index
