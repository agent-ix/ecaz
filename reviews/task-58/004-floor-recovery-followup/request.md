# Task 58.1 — `build_parallel.rs` floor-recovery follow-up

**Status:** open — Task 58 closed at 84 unsafe blocks (above the
-30% floor of ≤78 by 6 blocks). Closeout reviewer
(`reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md`)
issued a HARD BLOCK on the close-below-floor disposition per
`feedback_no_premature_task_close` (HARD RULE, 2026-05-23) and
named the follow-up scope: land three concrete audits that the
structural-ceiling rationale dismissed prematurely.

This packet opens that follow-up.

**Branch:** new feature branch off main, e.g., `task-58-1-floor-recovery`.
Do not reuse `task-58` (it carries the pre-block snapshot;
the closeout has already merged to main at `c0f06af10`).

**Scope-lock:** `src/am/ec_hnsw/build_parallel.rs` only. No other
HNSW file, no `src/am/common/` extensions unless the LWLock dispatch
typing genuinely requires a generic primitive lift (in which case
that lift lands as its own commit per the standing slice rule).

## Why this opens (and not "Task 58 reopens")

- Task 58 closeout commit `c0f06af10` already merged to main.
  Per `feedback_branch_isolation`, we don't rewrite landed
  history; we land the corrective work as a successor task.
- Precedent: Task 56.1 followed the same pattern
  (`reviews/task-56/007-doc-parity-followup/`) — open a new packet
  under the same task bucket, name it `58.1`, address the standing
  block items, then re-write the close disposition.
- The closeout reviewer feedback explicitly names this path:
  > Given Task 58 is already merged to main, the cleanest path
  > forward is a **Task 58.1 follow-up** to land the three audits
  > above.

## Scope — the three standing audits

Quoting the BLOCK feedback verbatim for traceability:

> 1. **Land the DSM accessor → ops migration** (6 blocks;
>    metric-neutral but mandated by
>    `feedback_view_operations_not_accessors`).
> 2. **Audit the per-call `unsafe fn` delegation wraps** (~12
>    blocks; pessimistic -3 to -5).
> 3. **Type the LWLock dispatch table** (~6 blocks; net -5).

### Audit 1 — DSM accessor → ops migration

Anti-pattern under `feedback_view_operations_not_accessors`
(2026-05-23, HARD RULE — tightens
`feedback_anti_pattern_b_unbounded_lifetime`):

> typed `*View<'a>` wrappers expose operations
> (validate, record_workers_done, with_locked_mut, signal) — never
> safe `fn(&self) -> &'a T`.

The `EcHnswConcurrentDsmGraphParts` impl currently exposes the
forbidden shape at multiple sites. Sample from current
`build_parallel.rs` (HEAD = main):

| Line | Method | Returns | Disposition |
| ---: | --- | --- | --- |
| 247 | `header(&self)` | `&EcHnswConcurrentDsmGraphHeader` | safe accessor — convert to `with_header(&self, |h| {...})` op |
| 253 | `header_mut(&mut self)` | `&mut EcHnswConcurrentDsmGraphHeader` | safe mut accessor — convert to `with_header_mut(&mut self, |h| {...})` op |
| 264 | `node(&self, idx)` | `&EcHnswConcurrentDsmNode` | safe accessor — convert to `with_node(&self, idx, |n| {...})` op |
| 270 | `node_mut(&mut self, idx)` | `&mut EcHnswConcurrentDsmNode` | safe mut accessor — convert to `with_node_mut(&mut self, idx, |n| {...})` op |
| 276 | `node_lock(&self, idx)` | `*mut pg_sys::LWLock` | unchanged — the raw `*mut LWLock` flows into the LWLock dispatch (Audit 3) |
| 282 | `node_insert_state_cell(&self, idx)` | `PgLockedDsmInsertStateCell` | already op-shaped via typed cell — leave as-is |

Migration shape:

```rust
// before (current):
//   let h = self.header();
//   if h.workers_done >= h.workers_expected { ... }
//
// after:
//   self.with_header(|h| {
//       if h.workers_done >= h.workers_expected { ... }
//   })
```

Expected metric impact: **0 blocks net** (the unsafe block moves
from the accessor body into the op body). The rule is about shape,
not count. Per the closeout feedback:

> The "metric-neutral" framing dodges the rule by saying it won't
> help the count — but the rule's about shape, not count.

**This must land regardless of the count outcome.**

### Audit 2 — Per-call `unsafe fn` delegation-wrap audit

Per the standing block: ~12 candidate blocks. Pessimistic estimate
-3 to -5 net removal.

Pattern: when the enclosing fn is itself `unsafe fn` and its
`# Safety` doc already covers the call's preconditions, the inner
`unsafe { ... }` block is redundant and the call can sit directly
in the function body.

```rust
// before:
//   unsafe fn outer(p: *mut T) {
//       // # Safety: ... p is valid ...
//       unsafe { inner_unsafe_fn(p); }   // <-- redundant block
//   }
//
// after:
//   unsafe fn outer(p: *mut T) {
//       // # Safety: ... p is valid ...
//       inner_unsafe_fn(p);              // covered by outer's # Safety
//   }
```

Action: grep `build_parallel.rs` for `unsafe fn` outer wrappers;
identify call sites whose inner block adds no soundness boundary
the outer fn doesn't already document. Remove redundant inner
blocks. Where the inner block expresses a tighter SAFETY than the
outer (e.g., an outer-fn `# Safety` that is genuinely broader than
one specific call), **keep** the inner block — soundness over
metric.

### Audit 3 — LWLock dispatch table typing

Current (HEAD):

```rust
// src/am/ec_hnsw/build_parallel.rs lines 535-555
struct EcHnswConcurrentDsmLockOps {
    acquire_shared: fn(*mut pg_sys::LWLock) -> LwLockGuard,
    acquire_exclusive: fn(*mut pg_sys::LWLock) -> LwLockGuard,
}

impl EcHnswConcurrentDsmLockOps {
    unsafe fn shared(self, lock: *mut pg_sys::LWLock) -> LwLockGuard {
        unsafe { (self.acquire_shared)(lock) }
    }
    unsafe fn exclusive(self, lock: *mut pg_sys::LWLock) -> LwLockGuard {
        unsafe { (self.acquire_exclusive)(lock) }
    }
}
```

Each consumer call site adds its own `unsafe { ops.shared(lock) }`
block. Reviewer estimate: ~6 such call sites; absorbing them into a
typed `*View<'a>`-shaped wrapper net -5.

Target shape:

```rust
struct DsmLockOps {
    acquire_shared: unsafe fn(*mut pg_sys::LWLock) -> LwLockGuard,
    acquire_exclusive: unsafe fn(*mut pg_sys::LWLock) -> LwLockGuard,
}

impl DsmLockOps {
    /// Acquire a shared guard over `lock`.
    ///
    /// # Safety
    /// `lock` must point at an LWLock living in the DSM graph image
    /// these ops dispatch into.
    unsafe fn shared(&self, lock: *mut pg_sys::LWLock) -> LwLockGuard {
        // SAFETY: contract delegated to the caller's outer fn.
        unsafe { (self.acquire_shared)(lock) }
    }
    // ditto exclusive
}
```

Consumer sites collapse `unsafe { (ops.acquire_shared)(lock) }` to
`unsafe { ops.shared(lock) }` and the outer-fn-already-unsafe sites
absorb that block via Audit 2.

If the dispatch can stay a regular method without the outer-fn `unsafe`,
even better: the consumer site's block count drops directly.

**Anti-pattern check**: do NOT add a safe
`fn(&self, lock) -> SharedGuard` here — the function-pointer
dispatch is unconditionally `unsafe fn` because the `*mut LWLock`
contract belongs to the caller. The op wrapper is shape, not
soundness.

## §Exit Criteria for Task 58.1

Task 58.1 closes when **all three** are true:

1. `src/am/ec_hnsw/build_parallel.rs` ≤ **78** unsafe blocks
   (the -30% floor from 112). Margin appreciated.
2. The DSM accessor `fn(&self) -> &T` / `fn(&mut self) -> &mut T`
   shape is gone from `EcHnswConcurrentDsmGraphParts` (Audit 1
   landed regardless of count).
3. A closeout summary packet records:
   - per-audit before/after block count
   - explicit list of Audit-2 sites where the inner block was
     removed and which outer-fn `# Safety` doc covers each
   - the `src/` total block count change
   - if the count still cannot reach ≤78 **after all three audits
     land in good faith**, an honest structural-ceiling rationale
     for the remaining gap (acceptable per the closeout
     feedback's "after-the-audits" clause).

## Performance gate

No new bench window required if Audit 1/2 are pure structural
shape moves (block-shuffling, no new logic) and Audit 3 is a
trivial dispatch-table typing.

If any audit changes call semantics (e.g., closure shape
introduces an extra borrow that changes hot-loop register
allocation), then the same bench gate as Task 58 closeout applies:
HNSW 100k parallel-build wall-clock at `workers ∈ {0, 2, 4}` vs
the post-Task-54 baseline at
`benchmarks/task-50-m5-hnsw-baseline/`, 5% noise band.

The coder should default to **assume the bench gate is required**
and produce evidence; the reviewer can waive if the diff is
inspection-trivial.

## Validation per slice

Per `feedback_coder_push_smoke_checks`:

- `cargo fmt --all`
- `cargo check --no-default-features --features pg18,bench`
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
- direct unsafe-block count for `build_parallel.rs` + `src/` total
- focused tests **only** if the slice changes worker-loop behavior

The macOS `dyld _BufferBlocks` blocker
(`feedback_dyld_buffer_blocks_known`) applies — compile-gate is
sufficient on macOS; runtime tests deferred to Linux.

## Memory rules in play

- `feedback_no_premature_task_close` — primary driver. The floor
  must be met (or a post-audit honest residual-ceiling rationale
  documented).
- `feedback_view_operations_not_accessors` — drives Audit 1
  regardless of count.
- `feedback_anti_pattern_b_unbounded_lifetime` — applies to any
  new wrapper added under Audit 3.
- `feedback_dont_defer_safety_fixes` — no audit may be deferred
  to a hypothetical Task 58.2.
- `feedback_main_priority_in_conflicts` — if `build_parallel.rs`
  conflicts with main mid-flight, main wins on substance.

## Cross-references

- Standing BLOCK:
  `reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md`
- Task 58 closeout commit on main: `c0f06af10`
- Reviewer BLOCK commit on main: `9afb2c6b8`
- Task 58 plan: `plan/tasks/58-hnsw-build-parallel-p8-consumer-migration.md`
- Precedent for follow-up packet shape:
  `reviews/task-56/007-doc-parity-followup/`
- Bench baseline: `benchmarks/task-50-m5-hnsw-baseline/manifest.md`

## Note on the user's "Task 59 typed views" hypothesis

The orchestrator-side speculation that Task 59's typed views (now
merged at `14f535fce`) would help push `build_parallel.rs` from
84 → ≤78 should be tempered:

- Task 59 added `EcParallelCoordinatorView` and
  `EcParallelWorkerSlotsView` in `src/am/common/parallel.rs` and
  `ReadStreamScope<'rel>` etc. in `src/am/common/stream.rs`. These
  are **scan-side** typed views (the post-init scan worker
  coordination surface).
- `build_parallel.rs` is **build-side** P8 (DSM-resident shared
  header for the build coordinator + workers). The Task 52 P8
  wrappers (`EcHnswParallelBuildSharedView`, `ShmTocBuilder/Reader`,
  `ParallelContextRef`) are the right consumer surface for
  build-side work — and (see Task 52 reconciliation note below)
  they do not exist on main yet.
- For Task 58.1, the three standing audits stand on their own and
  do not depend on Task 59. Land them per the closeout feedback's
  shape, not the Task 59 analogy.
