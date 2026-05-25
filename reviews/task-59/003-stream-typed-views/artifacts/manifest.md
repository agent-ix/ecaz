# Task 59 / 003-stream-typed-views — Artifact Manifest

- **Branch:** `task-59-parallel-stream-burndown`.
- **Task bucket / packet path:** `reviews/task-59/003-stream-typed-views/`.
- **Lane / fixture / storage format / rerank mode:** N/A — code refactor
  with compile-time validation. Runtime exercise scheduled for bench
  gate at slice 004.
- **Shared / isolated surface:** N/A — module-internal refactor.

## Artifacts

| File | Source | Command | Timestamp | Notes |
| --- | --- | --- | --- | --- |
| `post_003_counts.txt` | post-refactor `src/am/common/{parallel,stream}.rs` | `scripts/unsafe_block_count.sh src/am/common/parallel.rs src/am/common/stream.rs` | 2026-05-25 | parallel.rs 22, stream.rs 13. |
| `stream_blocks_post.txt` | post-refactor `src/am/common/stream.rs` | `grep -n "unsafe {" src/am/common/stream.rs` | 2026-05-25 | 13 lines; per-block line enumeration cited in request.md §Structural ceiling rationale. |

## Key Result Lines Cited

- `22 src/am/common/parallel.rs` — unchanged from slice 002 fix-up
  (`post_002_counts.txt`). §Exit target ≤22 still met.
- `13 src/am/common/stream.rs` — `post_003_counts.txt:2`. Compared to
  baseline `17` from
  `reviews/task-59/001-execution-plan/artifacts/baseline_counts.txt`,
  Δ = **-4 (-23.5%)**. **Below the per-file -30% floor by 1 block.**
- Combined subsystem: 22 + 13 = **35** vs baseline 51, Δ = **-16
  (-31.4%)**. **At the -30% combined floor; -3.6 short of the -35%
  combined target.**

## Structural-ceiling note

The 13 stream.rs blocks are documented per-block in `request.md`
§Structural ceiling rationale under three categories:

- Category A (6 blocks): single PG FFI ops with no in-file fold
  partner.
- Category B (4 blocks): fused-pair `read_stream_next_buffer` +
  buffer-typing in single unsafe block per op type per stream-owner
  variant (ReadStreamScope-owned + scan-opaque-owned × pinned + locked).
- Category C (3 blocks): `ReadStreamScope::open` caller-site blocks
  at the 3 public stream.rs entry points (`prefetch_relation_blocks`,
  `visit_relation_linear_read_stream`,
  `visit_relation_block_sequence_read_stream`), each owning a
  distinct callback / state pair that cannot collapse without
  cross-AM consumer migration.

Cross-AM migration to typed read-stream handles is the route to
absorb Category B and Category C blocks; it is explicitly out of
scope per Task 59 §Non-Goals "Do not migrate AM-specific call
sites".

## Reviewer pre-conditions met

- Task 59 slice 002 fix-up reviewer seq 04 disposition step 3
  ("Slice 003: stream.rs typed views, 17 → ≤11") — this packet
  files the slice 003 work and the structural-ceiling claim for the
  -23.5% landing.
- Task 56.1 reviewer seq 01 ("Slice 003 can now open per coder's
  sequencing") — sequencing observed; Task 56.1 lands before slice
  003.
- Safety-doc parity from the introducing commit: 1 `unsafe fn`,
  1 `/// # Safety` heading.
