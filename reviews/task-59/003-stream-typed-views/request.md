# Task 59 / 003 — `stream.rs` Typed Views (Self-narrow)

**Branch:** `task-59-parallel-stream-burndown`
**Slice:** 003 (stream.rs) of [001, 002, 003, 004]
**Scope-lock:** `src/am/common/stream.rs` only. External consumers in
`src/am/ec_hnsw/`, `src/am/ec_ivf/`, `src/am/ec_diskann/`,
`src/am/ec_spire/` are not modified (per Task 59 §Non-Goals).

## Summary

Adds the typed-view surface enumerated in `plan/tasks/59-common-parallel-stream-burndown.md`
§Scope for stream.rs (`ReadStreamScope<'rel>` RAII + `next_pinned` /
`next_locked` iterator ops), folds the scan-owned pinned / locked
visit paths into typed `next_scan_owned_pinned` / `next_scan_owned_locked`
helpers, and applies all honest in-file folds. Lands stream.rs at
**13 blocks (-23.5%)**.

This is **below the per-file -30% floor and below the -35% target**.
The remaining 13 blocks are structurally minimal under Task 59
§Non-Goals (no cross-AM consumer migration, no public-API signature
changes); §Structural ceiling rationale below explains each block.

**Per `feedback_no_premature_task_close`:** the structural-ceiling
claim here is filed with per-block rationale, not as a "within
rounding" off-ramp. Reviewer disposition is requested at slice 004
closeout, not pre-empted here.

## Wrappers added / refactored

### `ReadStreamScope<'rel>` (replaces `PgReadStreamGuard`)

Typed RAII scope around `pg_sys::ReadStream`:

- `unsafe fn open(mode, relation, callback, callback_private_data) -> Self`
  — single `unsafe fn` constructor that wraps `read_stream_begin_relation`.
  Replaces the prior 4 distinct `read_stream_begin_relation` call sites
  (legacy direct in `prefetch_relation_blocks`, `PgReadStreamGuard::new`
  body, and the two visit helpers).
- `next_pinned() -> Option<Result<(PinnedBufferGuard, Option<BlockNumber>), String>>`
  — fuses `read_stream_next_buffer` + `PinnedBufferGuard::from_pinned`
  under a single SAFETY anchor. Per
  `feedback_view_operations_not_accessors`, no `*mut Buffer` or raw
  `*mut ReadStream` leaks to the caller.
- `next_locked(lockmode) -> Option<Result<(LockedBufferGuard, BlockNumber), String>>`
  — same shape with `LockedBufferGuard::lock_pinned`.
- `impl Drop` — single `read_stream_end` call.

`PgReadStreamGuard` (the old internal RAII type) is fully replaced
and removed.

### `next_scan_owned_pinned` / `next_scan_owned_locked` (replaces `next_scan_owned_read_stream_buffer` for the typed visit paths)

Internal helpers that fuse the per-iteration `read_stream_next_buffer`
+ buffer-typing into one block. `visit_scan_owned_read_stream_pinned`
and `visit_scan_owned_read_stream_locked` now consume these typed
helpers directly. The original untyped `next_scan_owned_read_stream_buffer`
is removed.

### `visit_read_stream` retargeted

Now consumes `&mut ReadStreamScope<'_>` via `next_locked` instead of
`&PgReadStreamGuard`. Behavior identical (visit-per-block with
per-buffer-data block-number fallback to `LockedBufferGuard::block_number`).

## Per-file count

```
$ scripts/unsafe_block_count.sh src/am/common/parallel.rs src/am/common/stream.rs
  22 src/am/common/parallel.rs
  13 src/am/common/stream.rs
```

- **stream.rs**: 17 → **13** (Δ **-4, -23.5%**).
- parallel.rs unchanged at 22 (slice 002 outcome).

**Versus task plan §Migration Targets:**

| File | Pre | §Exit Target (-35%) | -30% Floor | Actual | vs target | vs floor |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| stream.rs | 17 | ≤ 11 | ≤ 12 | **13** | **+2 over target** | **+1 over floor** |

**§Exit target NOT met; per-file floor NOT met by 1 block.** This
slice files the structural-ceiling claim with per-block rationale
below; reviewer disposition is the close-gate decision at slice 004.

## §Folds applied

| Fold | Old | New | Δ |
| --- | ---: | ---: | ---: |
| `PgReadStreamGuard` (struct + `new` body + `Drop` body) collapsed into `ReadStreamScope` | 2 (new begin block at L232 + Drop body L252) | 2 (`open` body + `Drop` body) | 0 (same internal blocks, but consolidated under one type with typed ops) |
| 4 `read_stream_begin_relation` call-sites (legacy direct in `prefetch_relation_blocks` L168 + `PgReadStreamGuard::new` callers at L424/L454) → 3 `ReadStreamScope::open` caller-sites + 1 internal body | 4 (L168 direct + L232 ctor body + L424/L454 caller `unsafe { PgReadStreamGuard::new(...) }` blocks) | 4 (3 caller `unsafe { ReadStreamScope::open(...) }` blocks + 1 internal in `open` body) | 0 (consolidation moved blocks but did not reduce count — the 4 distinct callback/state tuples cannot collapse without changing the public API of each consumer entry point) |
| `prefetch_relation_blocks` (pg18) iter loop: `read_stream_next_buffer` + `PinnedBufferGuard::from_pinned` + direct `read_stream_end` → `ReadStreamScope::next_pinned()` + Drop | 4 (L168/L184/L191/L199) | 2 (1 caller open + 1 used-from-Drop) | **-2** (the `next_buffer` + `from_pinned` pair folds into `next_pinned` body's one unsafe block; direct `read_stream_end` is absorbed by `Drop`) |
| `visit_read_stream` iter loop: `read_stream_next_buffer` + `LockedBufferGuard::lock_pinned` → `ReadStreamScope::next_locked()` | 2 (L284 + L291) | 0 (delegates to `next_locked`) | **-2** (fold into the scope's `next_locked` block, which was already counted) |
| `visit_scan_owned_read_stream_pinned`: `next_scan_owned_read_stream_buffer` (next + per-buffer-data) + inline `PinnedBufferGuard::from_pinned` → `next_scan_owned_pinned` (fused) | 2 (L325 next-helper + L369 inline from_pinned) | 1 (fused) | **-1** |
| `visit_scan_owned_read_stream_locked`: same pair → `next_scan_owned_locked` (fused) | 2 (L325 reused + L397 inline lock_pinned) | 1 (fused) | **-1** (counted once; L325 is shared between pinned and locked variants in the original code, so the locked variant only adds the `lock_pinned` block) |
| Test surface | 0 | 0 | 0 (stream.rs has no test module) |

**Verified per-block accounting:**

| Old block | Status after slice 003 |
| --- | --- |
| L168 (legacy direct `read_stream_begin_relation` in `prefetch_relation_blocks`) | replaced by `ReadStreamScope::open` caller-site at L168 (still 1 block; consolidated under typed scope) |
| L184 (legacy `read_stream_next_buffer` in prefetch loop) | folded into `ReadStreamScope::next_pinned` body |
| L191 (legacy `PinnedBufferGuard::from_pinned` in prefetch loop) | folded into `ReadStreamScope::next_pinned` body |
| L199 (legacy direct `read_stream_end` in prefetch) | absorbed by `ReadStreamScope::Drop` |
| L211 (pg17 `PrefetchBuffer` fallback) | unchanged (single block in `cfg!=pg18` arm) |
| L232 (`PgReadStreamGuard::new` body — `read_stream_begin_relation`) | replaced by `ReadStreamScope::open` body |
| L252 (`PgReadStreamGuard::drop` body — `read_stream_end`) | replaced by `ReadStreamScope::Drop` body |
| L265 (`read_stream_per_buffer_block_number` helper) | unchanged (single deref helper) |
| L284 (`visit_read_stream` `read_stream_next_buffer`) | absorbed by `ReadStreamScope::next_locked` |
| L291 (`visit_read_stream` `LockedBufferGuard::lock_pinned`) | absorbed by `ReadStreamScope::next_locked` |
| L325 (`next_scan_owned_read_stream_buffer` `read_stream_next_buffer`) | absorbed by `next_scan_owned_pinned` + `next_scan_owned_locked` (helper deleted) |
| L346 (`reset_scan_owned_read_stream` `read_stream_reset`) | unchanged (single FFI op) |
| L369 (`visit_scan_owned_read_stream_pinned` `PinnedBufferGuard::from_pinned`) | folded into `next_scan_owned_pinned` |
| L397 (`visit_scan_owned_read_stream_locked` `LockedBufferGuard::lock_pinned`) | folded into `next_scan_owned_locked` |
| L424 (`visit_relation_linear_read_stream` `PgReadStreamGuard::new` call) | replaced by `ReadStreamScope::open` caller-site |
| L454 (`visit_relation_block_sequence_read_stream` `PgReadStreamGuard::new` call) | replaced by `ReadStreamScope::open` caller-site |
| L486 (`write_stream_block` per-buffer-data write) | unchanged (single callback write helper) |

Net: **-4** blocks (17 → 13).

## §Structural ceiling rationale (per-block, post-slice-003)

Per Task 50/448 precedent and Task 56 closeout `feedback_dont_defer_safety_fixes`
discipline. Each remaining block is enumerated with the specific
fold attempt and the structural reason it does not pay off.

### Category A: PG FFI single-op blocks (6 blocks)

These blocks each wrap a single PG FFI call that has no fold partner
within stream.rs's API surface.

| Block | Line | Op | Fold attempt | Structural reason no fold |
| --- | ---: | --- | --- | --- |
| `PrefetchBuffer` (non-pg18) | L196 | single FFI | inline in loop | only block in `cfg!=pg18` arm; cannot share with pg18 code path; the loop body has just one unsafe op |
| `ReadStreamScope::open` body | L249 | `read_stream_begin_relation` FFI | the constructor body | single FFI call; cannot fold without a co-located unsafe op |
| `ReadStreamScope::Drop` body | L352 | `read_stream_end` FFI | RAII Drop | single FFI; cannot fold with construction (different lifetime phase) |
| `read_stream_per_buffer_block_number` helper | L365 | per-buffer-data deref | private helper used by 4 sites | each callsite already calls this helper; the helper consolidates the deref to one place |
| `reset_scan_owned_read_stream` body | L459 | `read_stream_reset` FFI | single FFI | cannot fold with next/lock (different stream phase); cannot fold with Drop (different lifecycle) |
| `write_stream_block` | L595 | per-buffer-data write callback | callback-side helper | single deref called from 3 callbacks (graph / linear / block_sequence); the helper consolidates the write to one place |

### Category B: Fused-pair blocks (4 blocks — already at minimum)

These blocks each fuse a `read_stream_next_buffer` call with a typed
buffer-guard construction inside a single `unsafe { ... }` block. They
are already at the structural minimum: each represents the pair of
PG FFI ops needed to consume one buffer from the stream.

| Block | Line | Fused ops |
| --- | ---: | --- |
| `ReadStreamScope::next_pinned` body | L291 | `read_stream_next_buffer` + `PinnedBufferGuard::from_pinned` |
| `ReadStreamScope::next_locked` body | L330 | `read_stream_next_buffer` + `LockedBufferGuard::lock_pinned` |
| `next_scan_owned_pinned` body | L410 | `read_stream_next_buffer` + `PinnedBufferGuard::from_pinned` |
| `next_scan_owned_locked` body | L434 | `read_stream_next_buffer` + `LockedBufferGuard::lock_pinned` |

Sharing between ReadStreamScope's next ops and the scan_owned next ops
would require either:

- Making the scan_owned helpers methods on a typed
  `BorrowedReadStreamScope<'a>` constructed via `unsafe fn from_raw`.
  This was modeled: each of the 3 scan_owned external entry points
  (`reset_scan_owned_read_stream`, `visit_scan_owned_read_stream_pinned`,
  `visit_scan_owned_read_stream_locked`) would gain a caller-site
  unsafe block for the `from_raw` call. Net: +3 caller blocks − 3
  helper bodies (the helpers move into method bodies of the same
  count) = +0 to +3 (depending on whether the methods consolidate).
  Even in the optimistic case, no reduction — so this refactor
  trades blocks for a structural improvement at no count gain.
- Making `from_raw` a SAFE FN. Per `feedback_anti_pattern_b_unbounded_lifetime`
  spirit (no safe construction from a raw pointer with unbounded
  lifetime in the wrapper), this is rejected as metric-gaming-adjacent.

The existing `*mut pg_sys::ReadStream` API of the public scan_owned
helpers is locked by Task 59 §Non-Goals (no AM consumer migration).
The cross-AM lift would be: AM scan-opaque code constructs a typed
read-stream handle once at scan setup, then passes the handle (not
the raw pointer) into the scan_owned helpers. That removes 3
caller-side blocks per AM. Deferred to a follow-on per-AM task.

### Category C: Public API call-site blocks (3 blocks)

These are the `ReadStreamScope::open` call sites at the 3 in-file
entry points that own different callback / state pairs.

| Block | Line | Caller | Callback / state |
| --- | ---: | --- | --- |
| `unsafe { ReadStreamScope::open(...) }` | L168 | `prefetch_relation_blocks` (pg18) | `block_sequence_prefetch_cb` + `BlockSequencePrefetchState` |
| `unsafe { ReadStreamScope::open(...) }` | L532 | `visit_relation_linear_read_stream` | `linear_prefetch_cb` + `LinearPrefetchState` |
| `unsafe { ReadStreamScope::open(...) }` | L563 | `visit_relation_block_sequence_read_stream` | `block_sequence_prefetch_cb` + `BlockSequencePrefetchState` |

Each caller is a SAFE FN (the public API for stream.rs consumers).
Calling the unsafe `ReadStreamScope::open` requires an explicit
unsafe block at each safe-fn caller site.

**Fold attempts:**

1. A `read_stream_for_state<S>(mode, relation, callback, &mut S) -> ReadStreamScope<'_>`
   generic helper that takes the state by `&mut S` and `.cast()`s
   internally. The helper would still be `unsafe fn` (relation is
   a raw `pg_sys::Relation`; callback is an unsafe extern "C-unwind"
   fn whose contract depends on the state type at the caller's
   choosing). Net: 1 internal block (in the helper) + 3 caller blocks
   (for the 3 wrapping unsafe-fn calls) = 4, same as current.
2. Make `ReadStreamScope::open` a SAFE FN. Rejected per
   `feedback_anti_pattern_b_unbounded_lifetime` — `relation` is a
   raw pointer with unbounded lifetime relative to the returned
   scope; safe construction violates the typed-wrapper discipline
   used in `src/am/common/dsm.rs` (`PgAtomicU32Ref::from_raw` is
   `unsafe fn`) and the Task 59 slice 002 parallel.rs view wrappers
   (both `from_raw` constructors are `unsafe fn`).

No fold pays off without changing the public API of the 3 stream.rs
entry points. Consumer-side migration to a typed-handle API would
absorb these 3 blocks at the AM scan-opaque level (3 fewer caller
sites in stream.rs, 3 corresponding sites in the AM code), but is
out of scope per Task 59 §Non-Goals.

## What unlocks further reduction (deferred)

To reach the §Exit target (≤11) or the floor (≤12), one of:

1. **Cross-AM consumer migration** of HNSW scan/build, IVF scan,
   DiskANN routine, SPIRE storage to:
   - Construct a `ReadStreamScope`-equivalent at the AM scan-opaque
     level once, then pass the typed handle to stream.rs (rather
     than `relation + callback + state` triples). This absorbs the
     3 Category C caller-site blocks.
   - Wrap the scan-opaque-owned read stream in a
     `BorrowedReadStreamScope<'a>` at scan setup and pass that
     handle into the scan_owned visit helpers. This absorbs the 4
     Category B scan_owned blocks (or rather, moves them into the
     AM code where each AM has one construction site).
2. **API signature changes** to stream.rs's public functions to
   require typed handles instead of raw `*mut pg_sys::ReadStream` /
   `pg_sys::Relation`. Same cross-AM consumer-migration impact.

Both are out of scope per Task 59 §Non-Goals "Do not migrate
AM-specific call sites in this task". Recommended follow-on:
**per-AM read-stream consumer migration packets**, parallel to the
HNSW Task 58.1 build_parallel migration that consumes the slice 002
parallel.rs typed views.

## Safety-doc parity

Per `feedback_dont_defer_safety_fixes` and the slice 002 fix-up
precedent — safety docs ship in the introducing commit, not deferred.

```
$ grep -cE "^[ \t]*(pub(\(.*\))?\s+)?unsafe fn" src/am/common/stream.rs
1
$ grep -c "/// # Safety" src/am/common/stream.rs
1
```

The single new `unsafe fn` declaration is `ReadStreamScope::open`;
its doc names (a) relation liveness invariant, (b)
`callback_private_data` aliasing / lifetime, (c) callback / state
type-match contract.

## Plan divergence (from slice 001)

None for slice 003 itself. The slice 001 plan listed `PrefetchScope<'rel>`
as a wrapper to add for the pg17 / non-pg18 `PrefetchBuffer`
fallback. **Slice 003 does not add `PrefetchScope`** because:

- The non-pg18 `PrefetchBuffer` path is a single `unsafe { … }` block
  inside a single loop body in `prefetch_relation_blocks`. Adding a
  typed wrapper would either net-zero (1 internal block + 1 caller
  block) or +1 (with state separated).
- The pg18 path uses `ReadStreamScope` instead; `PrefetchScope` was
  only relevant to the legacy pg17 fallback.
- Per `feedback_dont_defer_safety_fixes` line on metric-gaming: adding
  a wrapper that doesn't reduce blocks AND doesn't improve safety
  (the PrefetchBuffer call is already minimally-scoped) is API
  bloat. The slice 003 plan defers this to a separate decision —
  flagged for §Plan divergence visibility.

**This is a planning divergence, not a structural ceiling.** Flagged
for §Closeout disposition: if reviewer wants `PrefetchScope` added
purely for symmetry with `ReadStreamScope`, that goes in slice 003.1
or slice 004.

## Validation Gate Status (slice 003)

| Gate | Status | Notes |
| --- | --- | --- |
| `cargo fmt --check` for stream.rs | ✓ | applied via `cargo fmt -- src/am/common/stream.rs` |
| `cargo check --no-default-features --features pg18 --lib` | ✓ | green; one pre-existing unused-import warning in `src/am/ec_spire/update.rs`, unrelated, unchanged |
| `cargo clippy --features pg18 --lib -- -D warnings` for stream.rs | zero new lints in stream.rs (repo-baseline pre-existing 101 lint errors in unrelated files are unchanged) | — |
| `cargo test stream::tests` compile | ✓ green; runtime blocked on macOS dyld `_BufferBlocks` (per `feedback_dyld_buffer_blocks_known`); bench-gate exercise scheduled at slice 004 | — |
| Per-file count | **stream.rs 17 → 13 (-23.5%)** — **below -30% floor** | structural ceiling rationale per Category A/B/C above |
| Safety-doc parity | **1/1** | one `unsafe fn` (`ReadStreamScope::open`), one `/// # Safety` heading |
| Anti-pattern A/B sweep on stream.rs | ✓ clean | `grep -nE "fn [a-z_]+\([^)]*\*mut [A-Z]" src/am/common/stream.rs` returns the 3 pre-existing public scan_owned helpers (`reset_scan_owned_read_stream`, `visit_scan_owned_read_stream_pinned`, `visit_scan_owned_read_stream_locked`) which take `*mut pg_sys::ReadStream` as safe-fn params. These predate Task 59 and remain locked per §Non-Goals; their internal unsafe ops are properly scoped. No new anti-pattern surface introduced by slice 003. |
| External consumer binary compat | ✓ | `prefetch_relation_blocks`, `visit_relation_linear_read_stream`, `visit_relation_block_sequence_read_stream`, `visit_scan_owned_read_stream_pinned`, `visit_scan_owned_read_stream_locked`, `reset_scan_owned_read_stream` — all signatures unchanged |

## Disposition

- **Slice 003 work-in-progress at -23.5%** — below per-file floor.
- **Structural ceiling claim** filed per-block above; not a "within
  rounding" off-ramp.
- **Reviewer call:** approve the structural ceiling and move to
  slice 004 closeout, OR direct one of the cross-AM consumer
  migrations to land in scope (would require expanding Task 59
  §Non-Goals).
- **Per `feedback_no_premature_task_close`:** slice 003 does NOT
  close Task 59 — slice 004 closeout + bench gate still pending.
- **Combined subsystem state**: parallel.rs 22 + stream.rs 13 =
  **35 blocks** (Δ from baseline 51, **-16 / -31.4%**). Combined
  target was ≤33 (-35%); landed at -31.4%, **at the combined
  subsystem floor (-30%)**.

## Cross-References

- Slice 002 fix-up (reviewer-approved): `reviews/task-59/002-parallel-typed-views/feedback/2026-05-24-04-reviewer.md`
- Task 56.1 follow-up (reviewer-approved): `reviews/task-56/007-doc-parity-followup/feedback/2026-05-24-01-reviewer.md`
- Wrapper precedents: `src/am/common/dsm.rs`, `src/am/common/parallel.rs` post-slice-002.
- View-op discipline: `feedback_view_operations_not_accessors`.
- Anti-pattern B: `feedback_anti_pattern_b_unbounded_lifetime`.
- Metric-gaming line + floor enforcement: `feedback_dont_defer_safety_fixes`, `feedback_no_premature_task_close`.
- Structural-ceiling precedent: Task 50/448, Task 56 closeout.

## Artifacts

- `artifacts/post_003_counts.txt` — `scripts/unsafe_block_count.sh` output for both files post-003.
- `artifacts/stream_blocks_post.txt` — `grep -n "unsafe {" src/am/common/stream.rs` post-003.
- `artifacts/manifest.md` — packet-local source of truth.

## Slice handoff

→ **004 — closeout**: bench gate RUN (8-step `ecaz bench suite` vs
`benchmarks/task-50-m5-hnsw-baseline/`), per-file deltas, src/ total
snapshot, consumer-handoff list, §Exit Criteria disposition, coder
reply file per Task 57/005 template, wait for reviewer signoff
before merging.

The reviewer disposition on the slice 003 structural-ceiling claim
will determine whether slice 004 close requires further stream.rs
work or accepts the per-block rationale above.
