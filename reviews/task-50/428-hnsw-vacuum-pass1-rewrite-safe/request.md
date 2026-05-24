# Task 50/428: HNSW vacuum.rs — pass-1 rewrite + update applier safe-fn lifts

## Why this slice

`vacuum::apply_page_pass1_updates` and `vacuum::rewrite_page_pass1` are
the next two cascading lifts after slice 426's `plan_page_pass1`. Both
now compose only safe operations after slices 424 / 425 / 426; the
`unsafe fn` declarations are legacy.

## Scope

- `vacuum::apply_page_pass1_updates` lifted from `unsafe fn` to safe
  `fn`. Body uses only the now-safe `shared::with_writable_page_tuple_bytes`.
- `vacuum::rewrite_page_pass1` lifted from `unsafe fn` to safe `fn`.
  Body uses only the now-safe `plan_page_pass1` +
  `index.begin_page_rewrite` + the lifted `apply_page_pass1_updates`.
- 3 caller-side `unsafe { ... }` wraps stripped: 1 in
  `rewrite_page_pass1` (around `apply_page_pass1_updates`), 1 in
  `bulkdelete_apply_pass2_updates` (line ~1985, also around
  `apply_page_pass1_updates`), and 1 in `run_bulkdelete_with_adapter`
  (around `rewrite_page_pass1`).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/vacuum.rs` | 37 | 34 | -3 |
| **HNSW subsystem subtotal** | **393** | **390** | **-3** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 427 | 393 |
| After 428 | 390 |

Net rotation delta: **-159 in HNSW** (-29.0%).

## Soundness rationale

Each lifted function had zero internal `unsafe { ... }` blocks after
slices 424-426; the lift is signature-only. No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/428-hnsw-vacuum-pass1-rewrite-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Vacuum hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

Net -159 (-29.0%); within 6 unsafe blocks of crossing the Task 50
§Exit Criteria -30% per-module target on HNSW.
