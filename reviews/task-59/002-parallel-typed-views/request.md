# Task 59 / 002 — `parallel.rs` Typed Views (Self-narrow)

**Branch:** `task-59-parallel-stream-burndown`
**Slice:** 002 (parallel.rs) of [001, 002, 003, 004]
**Scope-lock:** `src/am/common/parallel.rs` only. External consumer
sites (HNSW `scan.rs`, `build_parallel.rs`) untouched per Task 59
§Non-Goals — they remain HNSW / Task 58.1 scope.

## Summary

Adds the typed-view surface enumerated in `plan/tasks/59-common-parallel-stream-burndown.md`
§Scope, refactors `parallel.rs` internal call sites + tests to consume
the new views, and applies legitimate folds:

- `EcParallelCoordinatorView<'state>` — value-returning ops via a
  private `with_coordinator(|coord| …)` closure helper; no
  `&'state EcParallelCoordinatorState` accessor.
- `EcParallelWorkerSlotsView<'state>` — `with_slot(slot_index, |slot| …)`
  closure form encapsulating the stride-deref; no
  `&'state EcParallelWorkerSlot` accessor.
- Safe attachment methods: `coordinator_view()`, `worker_slots_view()`,
  `claim_worker_slot()`, `release_worker_slot()`,
  `publish_worker_slot_runtime_snapshot()`,
  `read_worker_slot_snapshot()` operating through the typed views.
- Existing PG-callback-facing `unsafe fn` entry points
  (`claim_parallel_scan_worker_slot`,
  `release_parallel_scan_worker_slot`,
  `publish_parallel_scan_worker_slot_runtime_snapshot`,
  `read_parallel_scan_worker_slot_snapshot`) retained at their
  current signatures and now delegate to the safe attachment methods.
  HNSW `scan.rs` is binary-compatible.
- The `ParallelScanAttachment` struct shape is unchanged — `state`,
  `coordinator`, `descriptor_bytes`, `worker_slot_count`,
  `rescan_epoch` remain `pub(crate)` for HNSW `scan.rs` field access.
## Plan divergence (from slice 001)

**`EcParallelWorkerSlotGuard` not delivered in slice 002.** Slice 001
listed an RAII guard type so callers could express `try_claim()
-> Option<EcParallelWorkerSlotGuard<'_>>` and have `release` happen on
drop. Slice 002 instead delivers safe attachment methods
(`claim_worker_slot`, `release_worker_slot`) directly on
`ParallelScanAttachment`.

**Rationale:** the in-`parallel.rs` consumers (release/publish/read
unsafe-fn entry points and the test surface) are paired with explicit
`release` calls today; an RAII guard at the in-file level would shift
ownership semantics without changing the block count or the surface
shape of those callers. The HNSW `scan.rs` consumer that *would*
benefit from a guard (because its `release_parallel_scan_worker_slot`
call sits on a separate code path from the claim) is out of scope
per Task 59 §Non-Goals "Do not migrate AM-specific call sites in this
task".

**Disposition:** the guard belongs to the Task 58.1 HNSW
build_parallel consumer migration (Task 59 §Coordination: "Task 58.1
follow-up"). That task knows the call-site shape it needs (claim in
one block, release in another, optional rescan in between) and can
introduce the guard tied to that shape rather than speculatively
designing it here.

**This is a planning divergence, not a structural ceiling.** Flagged
for Task 58.1 planner.

## Folds applied

| Site | Old | New | Δ |
| --- | ---: | ---: | ---: |
| `worker_slot` / `coordinator` accessor methods on `ParallelScanAttachment` (anti-pattern B safe-`fn(*mut T)->&'a T`) | 2 | 0 (replaced by `coordinator_view` / `worker_slots_view` returning typed views) | -2 |
| `EcParallelCoordinatorView` ops (claimed_worker_slots, record_claimed, record_released, store_for_test) | 0 (new) | 1 (single `with_coordinator` closure helper) | +1 |
| `EcParallelWorkerSlotsView::with_slot` | 0 (new) | 1 | +1 |
| `coordinator_view`, `worker_slots_view` attach methods | 0 (new) | 2 | +2 |
| `reset_parallel_scan_layout` (coord init + slot loop) | 3 | 1 (single outer block) | -2 |
| `reset_parallel_scan_state` (`&mut *state` double-deref) | 2 | 1 | -1 |
| `release` / `publish` / `read` bodies (`worker_slot` / `coordinator` method-internal blocks no longer reached) | (counted under accessor methods) | (delegated via views) | (covered above) |
| Test helper `set_test_parallel_scan_ps_offset` (extract shared cfg-arm ps_offset writes; cfg arms then collapsed under one `unsafe { … }` block during the reviewer-driven re-roll) | 4 | 1 | -3 |
| Test `test_parallel_scan_desc_and_target` now reuses `test_parallel_scan_target` | 1 (duplicate `base.add(OFFSET)`) | 0 | -1 |
| Test helper `raw_test_parallel_scan_attachment` (extract shared `parallel_scan_attachment` call) | 2 | 1 | -1 |
| Test `claim/release/publish/read` helpers — drop inline `unsafe { unsafe_fn_entry }` and call safe attachment methods | 5 | 0 | -5 |
| Test `worker_slot_for_test` (anti-pattern B method removed) — `worker_slot_error_for_test` + `worker_slot_header_snapshot_for_test` rewritten through `worker_slots_view().with_slot(...)` | 0 (no inline test unsafes were here) | 0 | 0 |
| Production `parallel_scan_state_ptr` cfg-arm consolidation (reviewer-driven re-roll): pg17 + pg18 arms collapsed into a single function with one `unsafe { … }` block carrying both `ps_offset` and `ps_offset_am` reads under `#[cfg(...)]` | 2 | 1 | -1 |

Net: **-12** blocks (34 → 22).

## Per-file count

```
$ scripts/unsafe_block_count.sh src/am/common/parallel.rs src/am/common/stream.rs
  22 src/am/common/parallel.rs
  17 src/am/common/stream.rs
```

- **parallel.rs**: 34 → **22** (Δ -12, **-35.3%**).
- stream.rs unchanged at 17 (slice 003 handles that).

**Versus task plan §Migration Targets:** target was ≤22 (-35%).
Actual: **22 (-35.3%), target met**.

History note: the initial slice 002 commit landed at 24 (-29.4%) and
attempted to frame that as "at the per-file floor within rounding".
Reviewer feedback at
`reviews/task-59/002-parallel-typed-views/feedback/2026-05-24-02-reviewer.md`
HARD BLOCKED that framing — `-29.4%` is not `-30%`, and the floor rule
is `≥ -30%`. This packet revision documents the corrected outcome
after a fix-up pass that landed two additional honest folds:

- **Test-side cfg-arm fold** in `set_test_parallel_scan_ps_offset`:
  both pg17 and pg18 writes now sit inside a single `unsafe { … }`
  block guarded by `#[cfg(feature = "…")]` inside the block, dropping
  one block of static text.
- **Production cfg-arm fold** in `parallel_scan_state_ptr`: the
  duplicate pg17 / pg18 functions are now one function whose single
  `unsafe { … }` block reads `ps_offset` or `ps_offset_am` under
  `#[cfg(...)]`, dropping one block.

Neither fold removes any actual `unsafe` operation; each pulls two
identical operations under one shared SAFETY comment that
characterizes them both. The reviewer-flagged
"metric-gaming-by-stripping-explicit-blocks-from-unsafe-fn-bodies"
pattern is **not** invoked — every block still anchors a real deref.

## Compile / smoke

```
$ cargo fmt -- src/am/common/parallel.rs            # clean
$ cargo check --no-default-features --features pg18 --lib
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.87s
    (one pre-existing warning in src/am/ec_spire/update.rs — unrelated)
```

```
$ cargo test --no-default-features --features pg18 --lib -- am::common::parallel::
    Finished `test` profile [unoptimized + debuginfo] target(s) in 32.68s
     Running unittests src/lib.rs (target/debug/deps/ecaz-…)
dyld[14989]: symbol not found in flat namespace '_BufferBlocks'
error: test failed, to rerun pass `--lib`

  Caused by: process didn't exit successfully (signal: 6, SIGABRT)
```

Runtime test fails at dyld load on macOS, per the documented
`feedback_dyld_buffer_blocks_known` blocker (now extended to lib
tests at HEAD). Validation falls back to compile-time gates:

- `cargo check --no-default-features --features pg18 --lib` — green.
- `cargo fmt --check` for parallel.rs — clean after `cargo fmt`.
- `cargo clippy --no-default-features --features pg18 --lib` — pre-existing
  101 lint errors in unrelated files (lib-wide `-D warnings` baseline
  is already broken at HEAD; not regressed by this slice). Filtered to
  parallel.rs / stream.rs: **zero warnings introduced by 002**.

Per the §Validation rule in the task plan and the
`feedback_coder_push_smoke_checks` rule for parallel build/scan paths,
runtime exercise is the bench gate at slice 004, not pytest-style
unit-test runs.

## Diff overview

`src/am/common/parallel.rs`:

- L1–L18: added `use std::marker::PhantomData;`.
- L76–L196: NEW typed-view types (`EcParallelCoordinatorView`,
  `EcParallelWorkerSlotsView`) with closure-form ops.
- L198–L297: replaced `ParallelScanAttachment::worker_slot` and
  `ParallelScanAttachment::coordinator` (anti-pattern B accessor
  methods) with `coordinator_view`, `worker_slots_view`,
  `claim_worker_slot`, `release_worker_slot`,
  `publish_worker_slot_runtime_snapshot`,
  `read_worker_slot_snapshot`.
- L335–L386: folded `reset_parallel_scan_layout` coord init + slot
  loop into one outer `unsafe { … }` block.
- L549–L597: existing `unsafe fn` entry points now delegate to safe
  attachment methods.
- L597–L619: folded `reset_parallel_scan_state` `&mut *state`
  double-deref into one borrow.
- L692–L824 (tests): extracted `set_test_parallel_scan_ps_offset`,
  `raw_test_parallel_scan_attachment`; rewrote
  `claim_test_worker_slot`, `try_claim_test_worker_slot`,
  `release_test_worker_slot`,
  `publish_test_worker_slot_runtime_snapshot`,
  `read_test_worker_slot_snapshot` through the safe attachment
  surface; rewrote `worker_slot_error_for_test`,
  `worker_slot_header_snapshot_for_test`,
  `coordinator_claimed_worker_slots`,
  `stage_claimed_state_for_rescan_test` through the typed views.

## Validation Gate Status (slice 002)

| Gate | Status | Notes |
| --- | --- | --- |
| `cargo fmt --all` | ✓ | applied to parallel.rs |
| `cargo check --features pg18 --lib` | ✓ | one pre-existing warning unrelated |
| `cargo clippy --features pg18 --lib -- -D warnings` | ⚠ pre-existing repo-baseline failures unrelated to 002 | parallel.rs / stream.rs contribute **zero new lints** |
| `cargo test parallel::tests` | ⚠ macOS dyld blocker | compile-time exercise green; runtime deferred to bench gate at 004 |
| Per-file count | **parallel.rs 34 → 22 (-35.3%)** | ≤ 22 §Exit target met |
| `/// # Safety` doc parity | **10/10** | one per `unsafe fn` in parallel.rs (10 `unsafe fn`, 10 `/// # Safety` headings) |
| Anti-pattern B sweep | ✓ | `grep -nE "fn [a-z_]+\([^)]*\*mut [A-Z]" src/am/common/parallel.rs` empty |
| HNSW scan binary compat | ✓ | no field / signature changes consumed by `src/am/ec_hnsw/scan.rs` |

## Cross-References

- Slice 001 baseline: `reviews/task-59/001-execution-plan/artifacts/baseline_counts.txt`.
- Wrapper precedents: `src/am/common/dsm.rs` (`PgAtomicU32Ref::from_raw` + `load_acquire/store_release` op pattern); Task 54 P3 `WalTxnScope` / `RegisteredBufferPage`.
- View-op discipline: `feedback_view_operations_not_accessors`.
- Anti-pattern B: `feedback_anti_pattern_b_unbounded_lifetime`. `worker_slot` and `coordinator` accessor methods on `ParallelScanAttachment` were textbook anti-pattern B (safe `fn(&self) -> &'a T` where the `*mut T` lifetime is unbounded); their removal in this slice closes that violation.
- Metric-gaming line: `feedback_dont_defer_safety_fixes`. The -35.3% landing comes from honest folds (two cfg-arm consolidations under one `unsafe { … }` block each) — every block still anchors a real deref / FFI op.
- Premature-close rule: `feedback_no_premature_task_close`. This slice does NOT close Task 59 — slice 003 (stream.rs) and slice 004 (closeout + bench gate) still pending. The earlier `request.md` revision (committed at `4961b99cf`) attempted a "within rounding" floor framing that the reviewer correctly HARD BLOCKED; this revision documents the corrected -35.3% outcome.
- macOS dyld: `feedback_dyld_buffer_blocks_known`.

## Artifacts

- `artifacts/post_002_counts.txt` — `scripts/unsafe_block_count.sh` output post-002.
- `artifacts/parallel_blocks_post.txt` — `grep -n "unsafe {" src/am/common/parallel.rs` post-002.
- `artifacts/manifest.md` — packet-local source-of-truth.

## Slice handoff

→ **003 — stream.rs typed views**: `ReadStreamScope::open`,
`ReadStreamScope::next_pinned` / `next_locked`,
`ReadStreamScope::reset`, `PrefetchScope` per slice 001 enumeration.
Target: stream.rs 17 → ≤ 11 (-35%) per task plan.

→ **004 — closeout**: bench gate, per-file deltas, src/ total
snapshot, consumer-handoff list, §Exit Criteria disposition.
