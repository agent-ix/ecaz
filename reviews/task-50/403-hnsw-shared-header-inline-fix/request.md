# Task 50/403: Fix packet 401 — inline shared-header borrow (anti-pattern B)

## Why this slice

Packet 401's reviewer feedback at
`reviews/task-50/401-hnsw-shared-header-ref/feedback/2026-05-22-01-reviewer.md`
blocked 401 for reintroducing anti-pattern B: a safe `fn` taking a raw
pointer and returning `&'a T` with unbounded lifetime `'a`. The fix per
reviewer recommendation (Option B): inline the `NonNull::as_ref` borrow at
each of the two worker entrypoints, with `unsafe { ... }` blocks bounded
to the function frame.

The five caller-side `(*shared).field` deletions and the
`participant_count` hoist from 401 are correct in concept and remain
in this packet — only the helper's signature was the problem.

## Scope

- Removed the module-private `shared_header_ref<'a>(*mut T) -> &'a T`
  helper from `src/am/ec_hnsw/build_parallel.rs` (-1 unsafe block in the
  helper body).
- Inlined the borrow at the start of `parallel_build_worker_main` and
  `parallel_graph_build_worker_main` with explicit context-specific
  SAFETY comments. Each block is bounded to its function frame by the
  borrowed `header: &EcHnswParallelBuildSharedHeader` binding (+2 blocks
  total).
- Updated the downstream `participant_count` hoist in
  `parallel_graph_build_worker_main` to read from the already-borrowed
  `header` instead of calling the removed helper.

## Unsafe block counts

| File | Before (after 402) | After 403 | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 116 | 117 | +1 |
| **HNSW subsystem subtotal** | **528** | **529** | **+1** |

Reviewer's projected delta in 401/02 feedback was the same: "Block count
cost: +2 in build_parallel.rs (one per worker entry), but each is bounded
to the function frame syntactically. Net `-3` instead of `-5`." This packet
realizes the +1 vs the slice-401 baseline (because 401 had net -5 from
two effects: -6 caller-side block deletions + +1 helper block; here we get
-6 caller deletions + +2 inline blocks instead, so net -4 vs pre-401, i.e.
+1 vs post-401).

Cumulative HNSW delta across the rotation (pre-399 → 403):

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 399 (read_metadata_page safe facade) | 541 |
| After 400 (IndexInfo split) | 540 |
| After 401 (shared_header_ref) | 535 |
| After 402 (typed shm_toc_lookup_required) | 528 |
| After 403 (this fix) | 529 |

Net rotation delta: **-20 in HNSW**, structurally sound.

## Soundness rationale

The inline pattern (per reviewer feedback 401/02 Option B):

```rust
let header: &EcHnswParallelBuildSharedHeader = unsafe {
    ptr::NonNull::new(shared)
        .unwrap_or_else(|| pgrx::error!("..."))
        .as_ref()
};
```

is sound because:

- the borrow `header` is bound to the local function frame, so the
  lifetime is concretely `'fn-frame` (no unbounded `'a`),
- the explicit `&EcHnswParallelBuildSharedHeader` type annotation prevents
  a future maintainer from accidentally coercing to `'static`,
- the SAFETY comment documents the leader/worker DSM lifetime invariant at
  the use site, where it can be re-evaluated against any nearby code
  changes — not in a helper file that is invisible from the call site.

This is the same fix shape that packets 017 and 304 applied to the
`scan_opaque_ref` and `debug_scan_opaque` regressions.

## Validation

Artifacts under
`reviews/task-50/403-hnsw-shared-header-inline-fix/artifacts/`:

- `manifest.md` — head SHA, pre-slice head, files touched, validation
  mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `diff.patch` — exact diff applied.
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean, no `unused_unsafe` warnings.

## Performance gate

Build hot path. No semantic change vs slice 401 — same borrow, same field
reads, same `validate()` and `participant_count` access. Bench evidence
deferred per `feedback_coder_push_smoke_checks` (2026-05-21).

## Process follow-up

Reviewer feedback 401/02 §Process recommendation flags the need to extend
`scripts/check_unsafe_comments.sh` to catch crate-internal shared-state
types (not just `pg_sys::*`). Suggested addition list:

```
EcHnswParallelBuildSharedHeader
EcParallelScanState
EcParallelCoordinatorState
EcParallelWorkerSlot
EcIvfScanOpaque
TqScanOpaque
```

Plus a structural regex catching `^pub(\(crate\))? fn .*<'a>.*\*(mut|const).*\) -> &'a`.
Captured here as a future packet candidate; not in scope for this fix.
