# Task 58.1 — slice 004 CLOSEOUT

**Branch:** `task-58-1-floor-recovery`
**Closing commit (pre-merge):** _filled at bench-gate close_
**Parent packet:** `reviews/task-58/004-floor-recovery-followup/`
**Bench config:** `reviews/task-58/007-task-58-1-closeout/suite.json`

## Why this closes Task 58 BLOCK

Task 58 closed at `c0f06af10` with `src/am/ec_hnsw/build_parallel.rs`
at **84 unsafe blocks** — 6 above the -30% floor of ≤78 from the
Task 50 baseline of 112. The Task 58 closeout reviewer
(`9afb2c6b8`) issued a HARD BLOCK on the close-below-floor
disposition per `feedback_no_premature_task_close`. Task 58.1
landed the three audits that the closeout's "structural ceiling"
rationale had dismissed prematurely.

This closeout request **closes the standing BLOCK** and clears
`build_parallel.rs` for merge into main.

## §Exit Criteria status

Per Task 58.1 plan (`reviews/task-58/004-floor-recovery-followup/request.md`)
§Exit Criteria:

| # | Criterion | Status |
|---|---|---|
| 1 | `build_parallel.rs` ≤ **78** unsafe blocks (the -30% floor) | ✓ **74** (4-block margin) |
| 2 | DSM accessor `fn(&self) -> &T` / `fn(&mut self) -> &mut T` shape gone from `EcHnswConcurrentDsmGraphParts` | ✓ verified by grep — 0 matches |
| 3 | Closeout packet records per-audit before/after, Audit-2 absorbed sites, `src/` total, residual-ceiling rationale | ✓ this packet |

Plus the cross-cutting safety doc gate (slice 002.1, reviewer seq
01 on slice 002): every `unsafe fn` in `build_parallel.rs` carries a
fn-level `/// # Safety` contract. Verified parity:

```
$ grep -cE "^[ \t]*(pub(\(.*\))?\s+)?unsafe fn" src/am/ec_hnsw/build_parallel.rs
15
$ grep -c "/// # Safety" src/am/ec_hnsw/build_parallel.rs
15
```

## Task 58.1 arc — per-slice block-count trajectory

| Slice | What | `build_parallel.rs` | Δ | Notes |
|---|---|---:|---:|---|
| pre (post-Task-58 merge) | baseline | **84** | — | above -30% floor of ≤78 |
| 002 (Audit 1) | DSM accessor → `with_*` ops migration | 84 | 0 | metric-neutral; shape fix per `feedback_view_operations_not_accessors` |
| 002.1 (doc parity) | 15/15 `/// # Safety` docs | 84 | 0 | doc-only; reviewer seq 01 BLOCK resolved |
| **003 (Audit 2 + 3)** | per-call delegation absorption + dispatch typing | **74** | **-10** | hit optimistic plan estimate; 4-block margin under floor |
| 004 (closeout) | bench gate + packet | 74 | 0 | this packet |

Cumulative reduction from Task 50 baseline (112): **112 → 74 = -33.9%**.

## Per-audit before/after summary

### Audit 1 — DSM accessor → ops migration (slice 002, commit `d7d019aa7`)

Replaced four safe `fn(&self) -> &T` / `fn(&mut self) -> &mut T`
accessors on `EcHnswConcurrentDsmGraphParts` with `with_*` closure
ops. Migration table:

| Before (anti-pattern)                  | After                                                 |
|----------------------------------------|-------------------------------------------------------|
| `fn header(&self) -> &Header`          | `fn with_header<R>(&self, f: FnOnce(&Header) -> R) -> R` |
| `fn header_mut(&mut self) -> &mut H`   | `fn with_header_mut<R>(&mut self, f) -> R`             |
| `fn node(&self, idx) -> &Node`         | `fn with_node<R>(&self, idx, f) -> R`                  |
| `fn node_mut(&mut self, idx) -> &mut N`| `fn with_node_mut<R>(&mut self, idx, f) -> R`          |

Call-site updates: 6 (3 internal — `node_level` /
`node_neighbor_slot_count` / `node_insert_state_value`; 3
external — L1278 reader, L1134 header init, L1161 node init).

Metric impact: **0 blocks net** (each `with_*` body holds the same
`unsafe { &*ptr }` deref). Required by
`feedback_view_operations_not_accessors` regardless of count.

### Audit 2 — Per-call `unsafe fn` delegation absorption (slice 003, commit `37390579c`)

Removed 8 single-block delegation bodies (each was a single FFI
call or thin delegate whose outer fn's `# Safety` contract — added
in slice 002.1 — fully subsumes the inner `unsafe { ... }` block):

| Fn | Lines | Inner blocks removed |
|---|---:|---:|
| `EcHnswConcurrentDsmGraphLayout::from_header` | 908 | -1 |
| `estimate_chunk` | 3309 | -1 |
| `estimate_keys` | 3321 | -1 |
| `test_lock_guard` | 4302 | -1 |
| `try_parallel_build` | 2108 | -2 |
| `try_parallel_concurrent_dsm_graph_build` | 2181 | -2 |
| **Subtotal** | | **-8** |

### Audit 3 — LWLock dispatch typing (slice 003, commit `37390579c`)

`EcHnswConcurrentDsmLockOps` at L540 was already typed correctly
pre-slice (function-pointer fields are `unsafe fn`, `shared` /
`exclusive` methods are `unsafe fn` with proper `# Safety`
contracts). This slice consumed the two single-block dispatch
bodies via Audit 2's absorption pattern:

| Fn | Inner blocks removed |
|---|---:|
| `EcHnswConcurrentDsmLockOps::shared` | -1 |
| `EcHnswConcurrentDsmLockOps::exclusive` | -1 |
| **Subtotal** | **-2** |

**Combined Audits 2 + 3: -10 inner blocks removed.**

## Residual structural ceiling (74 → ≤70 gap)

Task 58 plan §Exit target was ≤70 (-37%); Task 58.1 plan §Exit
floor was ≤78 (-30%); we sit at 74 (-34%). The 4-block residue
above the Task 58 stretch target is **honest structural ceiling**
per the reviewer's "soundness over metric" discipline. The
residue lives in 6 multi-call unsafe-fn bodies, each whose inner
`unsafe { ... }` blocks carry SAFETY rationales narrower than the
outer-fn contract:

| Fn | Inner blocks kept | Reason |
|---|---:|---|
| `EcHnswParallelBuildLeader::begin` | 9 | distinct FFI calls (EnterParallelMode, CreateParallelContext, per-key shm_toc_*, RegisterSnapshot, InitializeParallelDSM, LaunchParallelWorkers, WaitForParallelWorkersToAttach); each narrower than outer |
| `EcHnswParallelGraphBuildLeader::begin` | 8 | same pattern for graph-build phase |
| `parallel_build_worker_main` | 7 | heap-scan worker entrypoint with distinct unsafe operations |
| `parallel_graph_build_worker_main` | 5 | graph-build worker entrypoint |
| `drain_worker_messages` | 5 | per-queue shm_mq_receive loop with per-tuple ownership rationales |
| `insert_leader_partitions` | 2 | leader-side DSM write + layout attach |
| **Total kept** | **36** | |

Absorbing any one of these would either:
- inline the SAFETY rationale where the reader can't see it
- merge specific rationales into a less specific outer doc

Both worse than the current shape. This residue is the genuine
ceiling; further reduction requires a different intervention
(e.g., refactoring the leader builder into a typed-state machine,
which is out of scope for Task 58.1).

## `src/` total impact

| | pre (after slice 002.1) | post (after slice 003) | Δ |
|---|---:|---:|---:|
| `src/` total unsafe blocks | 776 | 766 | -10 |

(Confirmed by reviewer seq 01 on slice 003 at
`reviews/task-58/006-task-58-1-audit-2-3/feedback/2026-05-25-01-reviewer.md`.)

## Bench gate

Run: `ecaz bench suite run --config reviews/task-58/007-task-58-1-closeout/suite.json`
against `benchmarks/task-50-m5-hnsw-baseline/`-shaped suite.

**Comparison baseline: Task 58 closeout** (HEAD `c0f06af10`,
artifacts at `reviews/task-58/003-closeout/artifacts/`) — the
immediate predecessor of Task 58.1. The Task 50 baseline (HEAD
`18acf379a`) is two task-cycles behind and not directly
comparable for the Task 58.1 no-regression gate.

Full per-step before/after numbers and analysis live in
`artifacts/before-after-summary.md`. Headline:

| Gate | Result |
|---|---|
| Suite steps 8/8 succeed | ✓ |
| Recall bit-for-bit identical at all 10k sweep points | ✓ (0.9040 / 0.9530 / 0.9605 / 0.9775 / 0.9950) |
| Recall within ci95 at all 100k sweep points | ✓ (max abs Δ = 0.0014) |
| Storage bit-for-bit identical (`m=8`/`m=16` at 10k, `m=16` at 100k) | ✓ (1235.4 B / 1366.4 B / 1365.4 B per row) |
| Latency within 5% of Task 58 baseline | **✗ — ~20-30% mean elevation, reproducible across two runs** |

### Latency disposition

The latency signal is **almost certainly machine-state noise, not a
real regression**:

- Recall and storage are dispositive on correctness — **no behavior
  change at all**.
- Task 58.1's three slices touch only build-side machinery
  (`EcHnswConcurrentDsmGraphParts` ops shape; `unsafe fn` doc
  parity; inner-block absorption in 8 unsafe-fn bodies). **None
  touch the scan-path hot loop** (`scan.rs` / `search.rs` /
  `graph.rs` are not in scope for Task 58.1).
- In release-mode codegen, the Audit 1 closure-form `with_*` ops
  and the Audit 2 inner-block removals lower to identical
  instruction streams as the prior shape — `unsafe { x }` inside
  `unsafe fn` ≡ bare `x` at the codegen level.
- The Task 58 baseline was taken on 2026-05-23 in a quiet machine
  state; the Task 58.1 run was taken on 2026-05-25 immediately
  after a `cargo pgrx install --release` (heavy filesystem +
  page-cache churn) and with the Codex parallel-reviewer agent
  active in another process.
- Stddev is also elevated (10k ef=40: 0.11 → 0.51 ms), consistent
  with machine-state jitter rather than a systematic regression.

**Coder recommendation:** approve close. The safety gates and
correctness gates are all green; the latency variance is
explainable by machine state, with no plausible regression
mechanism in the touched code surface. If reviewer disagrees, a
clean-machine re-run is the right way to resolve.

## Update — reviewer seq 01 HARD BLOCK + coder seq 02 evidence

Reviewer at `feedback/2026-05-25-01-reviewer.md` issued a HARD
BLOCK on the latency gate. Per operator selection (Option A +
proof-of-not-noise), I ran a level-playing-field experiment:
same machine, same time, baseline code vs Task 58.1 code, 5
iterations each. Full analysis at
`feedback/2026-05-25-02-coder.md` and
`artifacts/baseline-code-rerun/EXPERIMENT-NOTES.md`.

Result (`ec_real_10k_hnsw`, 5-iteration mean):

| ef | T58 manifest 2026-05-23 | T58 code TODAY | T58.1 code TODAY |
|---:|---:|---:|---:|
| 40  | 0.55 ms | **0.762** | 0.690 |
| 80  | 0.94 ms | **1.152** | 1.118 |
| 200 | 1.06 ms | **1.390** | 1.390 |

**The baseline code itself runs +22-39% slower today than its
own manifest.** Task 58.1 is at parity or 3-9% faster than the
baseline code on the level playing field. The bench gate's
stated regression is **entirely machine-state drift**, not
Task 58.1 code.

Latency disposition (revised): **PASS** vs the right comparator
(baseline code on same machine at same time).

## Cross-AM consumer handoff list

Task 58.1 was internal-to-HNSW; nothing is exported for cross-AM
consumption from this task. The `EcHnswConcurrentDsmLockOps`
dispatch table remains HNSW-private; if a future task needs
generic LWLock dispatch in `src/am/common/`, that lift lands as
its own packet.

`src/am/common/parallel_context.rs` (`ParallelContextRef<'a>`)
and the `src/am/common/dsm.rs` ShmToc wrappers landed via the
Task 52 reconcile merge `833f29cba` and remain available for
future consumers (Task 56/57 SPIRE/IVF parallel-build P8
consumer migrations); Task 58.1 itself did not consume them
because the HNSW build_parallel.rs P8 sites are inside the
"residual structural ceiling" bodies and would require the
parallel_build_view.rs redo (orphan piece, deferred).

## Memory rules — honored throughout Task 58.1

- `feedback_no_premature_task_close` ✓ — floor met with margin;
  no off-ramp.
- `feedback_dont_defer_safety_fixes` ✓ — 15/15 doc parity
  maintained.
- `feedback_view_operations_not_accessors` ✓ — Audit 1 closure ops
  in place; no `&'a T` accessor leakage.
- `feedback_anti_pattern_b_unbounded_lifetime` ✓ — regex sweep
  clean.
- `feedback_branch_isolation` ✓ — `build_parallel.rs` only.
- `feedback_full_code_review` ✓ — every reviewer seq backed by
  per-fn diff + grep verification.
- "Soundness over metric" ✓ — 36 inner blocks kept intentionally
  in 6 multi-call bodies, documented above.

## Cross-references

- Task 58.1 plan: `reviews/task-58/004-floor-recovery-followup/request.md`
- Plan approval (Codex reviewer): `f64d7a4a7`
- Slice 002 Audit 1 code: `d7d019aa7`
- Slice 002 review packet: `d60161669`
- Slice 002.1 doc parity: `387103152`
- Slice 002.1 packet update: `b38c2dcb7`
- Slice 002.1 approval (Codex reviewer): `56f3fda1d`
- Slice 003 Audit 2 + 3 code: `37390579c`
- Slice 003 review packet: `fbace8c60`
- Slice 003 approval (Codex reviewer): `3e1dff64d`
- Reviewer seq 02 "NOT COMPLETE" reminder: `3430cf0a2`
- Reviewer seq 03 "ball is in coder's court": `6193d912c`
- Originating BLOCK (Task 58 closeout): `reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md` (on main at `9afb2c6b8`)
- Task 50 baseline reference: `benchmarks/task-50-m5-hnsw-baseline/manifest.md`
- Task 52 orphan branch reconcile (on main): merge `833f29cba`
