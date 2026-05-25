# Task 58.1 / slice 002 — Audit 1 (DSM accessor → ops migration)

**Branch:** `task-58-1-floor-recovery`
**Commit:** `d7d019aa7` "Task 58.1/002: Audit 1 — DSM accessor → ops migration (metric-neutral)"
**Parent packet:** `reviews/task-58/004-floor-recovery-followup/` (plan approved at `f64d7a4a7`)

## Scope

Audit 1 of the Task 58.1 plan: remove the safe `fn(&self) -> &T` /
`fn(&mut self) -> &mut T` accessor anti-pattern from
`EcHnswConcurrentDsmGraphParts` per `feedback_view_operations_not_accessors`
(HARD RULE 2026-05-23).

## Migration

| Method (before)                       | Method (after)                                                |
|---------------------------------------|---------------------------------------------------------------|
| `fn header(&self) -> &Header`         | `fn with_header<R>(&self, f: FnOnce(&Header) -> R) -> R`      |
| `fn header_mut(&mut self) -> &mut H`  | `fn with_header_mut<R>(&mut self, f: FnOnce(&mut H) -> R) -> R` |
| `fn node(&self, idx) -> &Node`        | `fn with_node<R>(&self, idx, f: FnOnce(&Node) -> R) -> R`     |
| `fn node_mut(&mut self, idx) -> &mut N` | `fn with_node_mut<R>(&mut self, idx, f: FnOnce(&mut N) -> R) -> R` |

`node_lock()` and `node_insert_state_cell()` left as-is per Task
58.1 plan's Audit 1 table (the former flows into Audit 3; the
latter is already op-shaped via typed `PgLockedDsmInsertStateCell`).

## Call-site updates

- **Internal** (3 sites — `node_level`, `node_neighbor_slot_count`,
  `node_insert_state_value`): `self.node(idx).field` →
  `self.with_node(idx, |n| n.field)`.
- **External** (3 sites):
  - L1278 reader: `parts.header().entry_idx` →
    `parts.with_header(|h| h.entry_idx)`.
  - L1134 header init: `*parts.header_mut() = ...{...}` →
    `parts.with_header_mut(|h| *h = ...{...})`.
  - L1161 node init: `*parts.node_mut(idx) = ...{...}` →
    `parts.with_node_mut(idx, |n| *n = ...{...})`.

Final `grep -nE "(parts|self)\.(header|header_mut|node|node_mut)\("
src/am/ec_hnsw/build_parallel.rs`: **0 matches** — the
anti-pattern shape is gone from this file.

## Metrics

| | before | after |
|---|---:|---:|
| `unsafe { ... }` blocks in `build_parallel.rs` | 84 | 84 |

Metric-neutral by construction — the `unsafe { ... }` block lives
inside each `with_*` body now instead of inside the safe accessor
body. The rule the closeout reviewer cites is about *shape* (no
safe `&'a T` leakage), not count. Audits 2 + 3 will move the
count toward the ≤78 floor in slice 003.

## Validation

- `cargo check --no-default-features --features pg18,bench` — passes.
- `cargo clippy --no-default-features --features pg18 --lib` — 0
  hits in `src/am/ec_hnsw/build_parallel.rs`. Pre-existing
  crate-wide clippy backlog unrelated (Task 52/002 reviewer note).
- Anti-pattern call-site grep: 0 matches.
- macOS `dyld _BufferBlocks` blocker
  (`feedback_dyld_buffer_blocks_known`) applies — compile-gate is
  sufficient; pgrx-runtime tests deferred to Linux per the standard
  Task 58 / 56 / 59 disposition.

## Bench gate

Deferred to slice 004 closeout per Task 58.1 plan §"Performance
gate" — Audit 1 is pure shape moves (closure bodies inline at
optimizer level, no new logic in hot paths). The compound bench
window after Audits 2 + 3 covers Audit 1's exposure.

## Disposition request

Slice 002 disposition request: approve and clear for slice 003
(Audits 2 + 3).

Per the parent packet's reviewer feedback (`f64d7a4a7`):

> Per Task 59 closeout pattern: coder may open the branch and
> slice 002 without waiting for additional reviewer signoff
> between this approval and the slice 002 commit. But wait for
> reviewer signoff between slice 002 commit and slice 003.

Slice 002 commit landed at `d7d019aa7`. Awaiting reviewer signoff
before opening slice 003.

## Update — slice 002.1 follow-up (doc-parity BLOCK resolved)

Reviewer seq 01 (`feedback/2026-05-25-01-reviewer.md`) approved the
Audit 1 API shape on merits but issued a HARD BLOCK on `/// # Safety`
doc parity: 15 `unsafe fn` declarations in `build_parallel.rs`, 0
documented. Per `feedback_dont_defer_safety_fixes` HARD RULE.

Resolved as slice 002.1 follow-up commit `387103152`
("Task 58.1/002.1: /// # Safety doc parity (15/15) for
build_parallel.rs"). Each `unsafe fn` now carries a contract naming
pointer validity, lifetime requirement, and concurrency requirement,
per the Task 56.1 / Task 57 doc-fix template.

Parity verified:

```
$ grep -cE "^[ \t]*(pub(\(.*\))?\s+)?unsafe fn" src/am/ec_hnsw/build_parallel.rs
15
$ grep -c "/// # Safety" src/am/ec_hnsw/build_parallel.rs
15
```

`cargo check` still passes; 0 clippy hits in `build_parallel.rs`;
block count holds at 84 (slice 002.1 is doc-only).

Slice 002 + 002.1 combined disposition request: re-approve and
clear for slice 003.

## Memory rules in play

- `feedback_view_operations_not_accessors` — primary rule (Audit 1
  is mandated by this).
- `feedback_anti_pattern_b_unbounded_lifetime` — the closure-form
  ops respect this by scoping the borrow to `f`.
- `feedback_branch_isolation` — slice touches only
  `src/am/ec_hnsw/build_parallel.rs`.
- `feedback_full_code_review` — per-slice diff is the new
  `with_*` impls + 6 call-site edits; small enough for full read.
- `feedback_dont_defer_safety_fixes` — Audit 1 landed in spite of
  being metric-neutral; not deferred to a "count-payoff" slice.

## Cross-references

- Parent packet: `reviews/task-58/004-floor-recovery-followup/request.md`
- Approving feedback: `reviews/task-58/004-floor-recovery-followup/feedback/2026-05-25-01-reviewer.md`
- Originating BLOCK: `reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md`
- Slice 002 commit: `d7d019aa7`
- Slice 002 metric artifact: `reviews/task-58/005-task-58-1-audit-1/artifacts/manifest.md`
