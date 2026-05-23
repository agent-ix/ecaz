# Task 52 / 003 — Parallel Build Shared View · Artifact Manifest

Packet path: `reviews/task-52/003-parallel-build-shared-view/`
Task: `plan/tasks/52-common-p8-build-parallel-typed-views.md`
Code commits:
  - `1f983fb0b` — reviewer-absorbed bundle that landed the initial
    slice-003 source files alongside reviewer feedback on slices
    001+002 (Codex co-author).
  - This packet's parent commit applies the anti-pattern B refactor
    requested at the end of
    `reviews/task-52/002-shm-toc-wrappers/feedback/2026-05-23-01-reviewer.md`.

Branch: `task-52`

## Surfaces

- `src/am/ec_hnsw/parallel_build_view.rs` (new file) — the typed view.
- `src/am/ec_hnsw/build_parallel.rs` — visibility opens only
  (`workersdonecv`, `mutex`, `validate`).
- `src/am/ec_hnsw/mod.rs` — module declaration.
- `src/am/common/dsm.rs` — unchanged this slice.

## Per-file before/after `unsafe { ... }` blocks

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/parallel_build_view.rs` | (new) | 6 | +6 |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | 112 | 0 |
| `src/am/common/dsm.rs` | 13 | 13 | 0 |

The +6 wrapper-side blocks all wrap PG-primitive interactions
(SpinLockAcquire via `SpinLockGuard::acquire`, the locked counter
mutate, ConditionVariableSignal via `ConditionVariableRef::from_raw`,
the leader-side `condition_variable_init` + `spinlock_init` pair, and
the `validate` deref-call). Each one is required to remain unsafe at
the wrapper layer; the corresponding consumer-side reductions in
`build_parallel.rs` land in slices 004 and 005.

## Artifacts

This slice's evidence is the diff and the count grep. No standalone
JSON / log artifact is needed.

- Head SHA: the parent commit of this packet's commit.
- Lane / fixture / storage / rerank: N/A (compile-only).
- Isolation: N/A.
- Command (validation, packet-local run):
  - `cargo fmt --all` — clean.
  - `cargo check --no-default-features --features pg18` — `Finished`,
    14.49s incremental, exit 0.
  - `cargo clippy ... -- -D warnings`: not re-run for this slice; the
    only delta is the `validate()` body change (removed an indirection
    through a now-deleted `header()` accessor). Pre-existing rabitq
    backlog unchanged.
- Timestamp: 2026-05-23.
