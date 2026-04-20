---
id: US-009
title: EXPLAIN Scan Diagnostics
type: user-story
status: DRAFT
priority: P2-high
traces:
  - StR-004
---
# US-009: EXPLAIN Scan Diagnostics

**As** an application developer debugging query performance,
**I want** `EXPLAIN (ecaz) SELECT ... ORDER BY col <#> $q LIMIT 10` to show Ecaz-specific statistics,
**So that** I can understand whether poor performance is caused by graph topology, I/O, or scoring overhead.

## Acceptance Criteria

1. `EXPLAIN (ecaz)` is recognized as a valid EXPLAIN option when the extension is loaded
2. The output includes: bootstrap candidates expanded, bootstrap pages read, linear scan pages read, elements scored, heap TIDs returned
3. `EXPLAIN (ecaz, ANALYZE)` shows actual runtime values, not just estimates
4. When `ecaz` option is not specified, no additional output is produced (no overhead)
