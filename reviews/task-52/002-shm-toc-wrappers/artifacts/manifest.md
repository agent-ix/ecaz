# Task 52 / 002 — ShmToc Wrappers · Artifact Manifest

Packet path: `reviews/task-52/002-shm-toc-wrappers/`
Task: `plan/tasks/52-common-p8-build-parallel-typed-views.md`
Head SHA: `e2bade4e9` (code commit), this packet commits on top.
Code commit: `e2bade4e9 Task 52/002: ShmTocBuilder + ShmTocReader wrappers`
Branch: `task-52`

## Surfaces

- `src/am/common/dsm.rs` only. No consumer migration in this slice.

## Per-file before/after `unsafe { ... }` blocks

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/common/dsm.rs` | 9 | 13 | +4 |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | 112 | 0 |

Pre-state source: `reviews/task-52/001-execution-planning/artifacts/baseline-unsafe-density.txt`.

The +4 are PG FFI calls inside the wrapper bodies (`shm_toc_allocate`,
`shm_toc_insert`, `shm_toc_lookup` × 2). They must remain unsafe; the
consumer-side reduction lands in slice 004 when call sites are routed
through the wrappers and shed their own `unsafe { ... }` blocks.

## Artifacts

This slice's evidence is static: command + result lines, captured in
`request.md`. No standalone JSON / log file is needed.

- Head SHA: `e2bade4e9`
- Lane / fixture / storage / rerank: N/A (compile-only).
- Isolation: N/A.
- Command (validation, from packet-local run):
  - `cargo fmt --all` — clean.
  - `cargo check --no-default-features --features pg18` — `Finished` exit 0,
    11m 27s from-scratch debug build (background id `bnfbmkhm5`).
  - `cargo clippy --no-default-features --features pg18 -p ecaz --lib -- -D warnings`
    — 103 pre-existing `-D warnings` failures, **none in
    `src/am/common/dsm.rs`** (the touched module).
    `grep -E "src/am/common/dsm.rs" /tmp/task52-clippy-full.log` returns
    zero matches. The 103 failures live in `src/quant/rabitq.rs` and
    other files; they were introduced by the post-main-merge IVF RaBitQ
    optimization landing (per memory
    `feedback_main_priority_in_conflicts`). Slice 002 introduces zero
    new clippy warnings on its touched module.
- Timestamp: 2026-05-23.
