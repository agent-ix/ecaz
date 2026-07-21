---
task: 191
packet: 001-production-contract
role: coder
date: 2026-07-20
---

# Review request: production lazy-payload contract

## Decision

Make deterministic global-ranked windows of exactly 10 the normal physical
`ec_distann` final-payload policy. The size is fixed internally and is neither
a production GUC nor a reloption. A feature-only benchmark/test override may
select the eager control; installing the prior extension binary restores eager
behavior without rebuilding because the change writes no durable bytes.

## Bound reconciliation

This packet replaces NFR-019's unsatisfiable unconditional `payload reads <= k`
claim. With `W = 10` and the scan-start ceiling
`D = max(initial_search_bar × 64, 1024)`:

- unqualified reads with no tombstone or snapshot-visibility skips are at most
  `min(D, W × ceil(k/W))`; with `t` such skipped ranked slots, the bound is
  `min(D, W × ceil((k+t)/W))`;
- qual-driven reads are at most `D`; and
- a stable-prefix remote vec_id is not re-requested solely because search
  deepening rebuilt the ranked outputs.

The last window can therefore over-read at most nine ranked slots. Later-window
owner failure remains fail-closed: no earlier prefix becomes a completed query.

## Scope

FR-079 now defines executor-triggered owner requests, deterministic windows,
stable-prefix reuse, qual deepening, and later-window failure. FR-081 separates
eager graph/ranking orchestration from lazy final payload work and records the
production/default/rollback surfaces. NFR-019 carries exact read bounds.
TC-040/TC-041 and their option/constraint matrices cover the semantic and
counter evidence required by Task 191.

Implementation and runtime evidence follow in packet 002.
