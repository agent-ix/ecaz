---
id: US-009
title: EXPLAIN Scan Diagnostics
type: user-story
artifact_type: US
status: DRAFT
priority: P2-high
relationships:
  - target: "ix://agent-ix/ecaz/StR-004"
    type: "derives_from"
    cardinality: "N:1"
---
# US-009: EXPLAIN Scan Diagnostics

**As** an application developer debugging query performance,
**I want** `EXPLAIN (ecaz) SELECT ... ORDER BY col <#> $q LIMIT 10` to show Ecaz-specific statistics,
**So that** I can understand whether poor performance is caused by graph topology, I/O, or scoring overhead.

## Acceptance Criteria

### US-009-AC-1

`EXPLAIN (ecaz)` is recognized as a valid EXPLAIN option when the extension is
loaded.

### US-009-AC-2

The output includes bootstrap candidates expanded, bootstrap pages read, linear
scan pages read, elements scored, and heap TIDs returned.

### US-009-AC-3

`EXPLAIN (ecaz, ANALYZE)` shows actual runtime values, not just estimates.

### US-009-AC-4

When the `ecaz` option is not specified, no additional output is produced.
