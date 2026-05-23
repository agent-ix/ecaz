# Task 52 / 001 — Execution Planning · Artifact Manifest

Packet path: `reviews/task-52/001-execution-planning/`
Task: `plan/tasks/52-common-p8-build-parallel-typed-views.md`
Head SHA: `0d01a77c0`
Branch: `task-52`
Phase: Phase 1 — common wrappers, HNSW-only consumer migration.

## Surfaces

This is a planning packet — no code change under review. Pure measurement
and survey evidence to anchor the slice plan and the closeout's
before/after delta calculation.

## Artifacts

### `baseline-unsafe-density.txt`
Top-25 file-level `unsafe { ... }` block density across `src/` plus the
`src/am/common/dsm.rs` and `src/` totals. The closing summary packet
will cite these as the pre-state.

- Command:
  `find src -name '*.rs' -print0 | xargs -0 grep -c "unsafe\s*{" | awk -F: '$2 > 0' | sort -t: -k2,2 -rn | head -25`
- Timestamp: 2026-05-23
- Head SHA: `0d01a77c0`
- Lane / fixture / storage / rerank: N/A (static count).
- Isolation: N/A (no DB run).
- Key result lines cited by `request.md`:
  - `src/am/ec_hnsw/build_parallel.rs:112` — Task 52 target file
  - `src/am/common/dsm.rs:  9` — slice-447 wrapper module
  - `src/ total unsafe blocks: 960` — Task 52 closeout will compute Δ

### `build-parallel-consumer-sites.txt`
Survey of every shm_toc / SpinLock / CV / `(*shared)` / `(*pcxt)` touch
site in `src/am/ec_hnsw/build_parallel.rs`, grouped by phase and
function. Used to size the slice-005+ migration and to verify the four
proposed wrappers cover the real surface.

- Command:
  `grep -nE "EcHnswParallelBuildShared|shm_toc_(allocate|insert|lookup|estimate|attach)|SpinLockAcquire|SpinLockRelease|ConditionVariableSignal|SpinLockInit|ConditionVariableInit" src/am/ec_hnsw/build_parallel.rs`
- Timestamp: 2026-05-23
- Head SHA: `0d01a77c0`
- Key result lines cited by `request.md`:
  - Lines 2828–2833 / 2897–2902 — twin SpinLock+CV compounds
    targeted by `EcHnswParallelBuildSharedView::record_workers_done`.
  - Lines 2184–2302 / 2489–2562 — leader-side shm_toc allocate/insert
    chains targeted by `ShmTocBuilder`.
  - Lines 2744 / 2763 / 2836 / 2838 / 2852 / 2872 / 2905 / 2907 —
    worker-side shm_toc_lookup_required sites targeted by `ShmTocReader`.
  - Line 190 — the **single** `EcHnswParallelBuildSharedHeader`
    used by both build phases (correction to task-spec §Scope wording
    that names two headers).

## What this packet does not include

- No benchmark or perf evidence. Per Task 52 §Performance Gate, the bench
  window opens once at task close, comparing against
  `benchmarks/task-50-m5-hnsw-baseline/`.
- No code change or test log. Subsequent slice packets (002+) carry those.
