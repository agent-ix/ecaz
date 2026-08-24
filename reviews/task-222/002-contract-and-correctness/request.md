---
task: 222
packet: 002-contract-and-correctness
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 222 payload projection contract checkpoint

This packet requests delta review of implementation checkpoint `c9f79be4a`,
which resolves every blocking finding in reviewer feedback sequences 02 and
03. It builds on the typed payload-mask API introduced by `f088021ea` and the
two independently attributable fixes `54802d299` (query expression evaluation
context) and `010a0accc` (refreshed snapshot lifetime across hop rounds).

The checkpoint:

- exports `Exact(attnums)` versus `AllColumns(FallbackReason)` and retains the
  typed result in executor state;
- proves the narrow ordering exemption from the original Query and exact
  pathkeys at plan time, then derives the mask from the final PG18 executor
  target list, quals, and query-value expression during `BeginCustomScan`;
- makes an executor-local shallow `CustomScan` copy and deep target-list copy,
  replaces exactly one proven-unused ordering projection in that private tree
  with a typed NULL, and rebuilds projection state without mutating the shared
  or cached plan;
- retains the vector in the exact mask whenever copying or matching fails, so
  the projection and mask invariant fails closed instead of raising or
  evaluating an unshipped attribute;
- retains every attribute used by a visible expression or qual, falls back for
  whole-row/unproved shapes, and preserves the existing system-column error;
- supports both `PARAM_EXTERN` generic cached plans and `PARAM_EXEC` correlated
  LATERAL rescans, including repeated execution with changed parameter values;
- makes the benchmark control force all-column shipping with projection
  elision disabled, and emits honest mask/projection observability.

The focused three-owner PG18 regression passes at `c9f79be4a` (1 passed, 2578
filtered out). It executes and pins id-only attnum `1`, retention of attnums
`1,2` for visible distance and qual use, `SELECT *`, whole-row fallback, no
upper Sort, null/toasted payloads, mixed local/remote execution, multi-window
qual rejection, repeated cached execution, forced generic plans, changed
external Params, correlated `PARAM_EXEC` rescans, EPQ/concurrent update,
post-first-batch remote failure, result identity, and the existing `ctid`/
`xmin` hard errors.

During this expansion, GDB identified a separate crash as a stale refreshed
MVCC snapshot retained only by raw pointer between graph hop rounds. Commit
`010a0accc` keeps the `RegisteredSnapshotGuard` alive in the expander; the
passing rerun covers the formerly crashing path. The decisive failed run and
backtrace are retained as root-cause evidence rather than committing every
intermediate failed log.

This remains the correctness checkpoint, not Task 222 closeout. The
pre-registered same-generation 100k A/B and, if useful, full 10k/50k/100k
decision matrix live in packets 003 and 004. No benchmark result is claimed by
this packet.

Validation and provenance are in `artifacts/manifest.md`. Strict repository
clippy remains blocked by four pre-existing warnings outside the changed
files; the same all-target command passes when only those four named baseline
lints are allowed.
