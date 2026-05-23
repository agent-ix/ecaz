# Task 53 / 001 — Execution Planning

Branch: `task-53` (off `origin/main` `5c0e9e2bd`)
Task source: `plan/tasks/53-common-p6-datum-wrappers.md`

## Summary

Open Task 53 — the second Phase-1 lane in the post-Task-50 hardening
sequence (Task 52 is the first; Task 54 follows). Goal: lift the four
typed datum wrappers named in the task spec into
`src/am/common/datum.rs` and migrate `src/am/ec_hnsw/source.rs` 29 →
≤ 14 unsafe blocks.

No code change in this packet. Pre-state baseline + consumer-site
survey + slice plan only.

## Provenance + branch isolation

This branch was opened off `origin/main` per the Task 52/004 reviewer's
direction (`reviews/task-52/004-build-parallel-shm-toc-migration/feedback/2026-05-23-02-reviewer.md`
§"Direction to coder"):

> Branch: open a fresh `task-53` branch off main rather than continuing
> `task-52` — keeps the burndown lanes cleanly isolatable when they
> merge back to main.

Task 52 lives separately on `task-52` (HEAD `9878aa158`, closeout
pushed, awaiting reviewer ack on the full 8-step bench). Task 52's
new modules (`src/am/common/dsm.rs` additions,
`src/am/common/parallel_context.rs`) are NOT in this branch's
working tree at planning time — they'll arrive when both branches
merge to main.

## Pre-state baseline

From `artifacts/baseline-unsafe-density.txt`:

| Surface | Pre-state `unsafe { ... }` blocks |
| --- | ---: |
| `src/am/ec_hnsw/source.rs` (total) | 29 |
| ├─ SIMD intrinsics (lines 84-252) | 10 (out of scope) |
| └─ Datum-handling (lines 269-744) | **19 (in scope)** |
| `src/am/common/detoast.rs` (existing wrapper) | 8 |
| `src/` total | 960 |

Task 50 closed HNSW at 549 → 327; this branch starts from the
post-Task-50 main snapshot. Task 53 chips the documented `source.rs`
ceiling.

**Target**: `source.rs` 29 → ≤ 14 (-52% or better).

The 10 SIMD blocks (`inner_product_avx2_fma`, `_mm256_loadu_ps`,
`_mm256_storeu_ps`, NEON `vld1q_f32` / `vst1q_f32` and their unrolled
variants) are explicit §Non-Goals — they stay `unsafe fn` per Rust's
`#[target_feature]` language requirement. That leaves 19 datum-handling
blocks; the ≤ 14 target requires shedding 15 of those 19. Tractable
given the four wrappers above.

## Consumer-site survey

`artifacts/source-rs-consumer-survey.txt` enumerates each in-scope
unsafe block, the pattern it belongs to, and the wrapper that
absorbs it. Three observations:

1. **`DetoastedVarlena` already exists** in `src/am/common/detoast.rs`
   (lifted under prior P6 prep). 1 source.rs consumer site (line 499)
   already uses it. Task 53's contribution to this wrapper is adding
   the `'a` lifetime + typed-slice accessor methods (per task spec
   §Scope #1).
2. **`DetoastedFloat4Datum`, `FlatFloat4ArrayRef`, `FlatFloat4VarlenaRef`,
   `FlatFloat4SourceRef`** live HNSW-local in `source.rs` today
   (lines 487-700). Task 53 lifts them to `src/am/common/datum.rs`
   with the spec's renames + lifetime additions. Most of the
   wrapper-internal unsafe blocks travel with them; source.rs sheds
   them.
3. **`AttnumLookup`** (per task spec §Scope #4) is a new wrapper.
   Single consumer site: line 269
   (`pg_sys::get_attnum((*heap_relation).rd_id, source_column.as_ptr())`).

## Slice plan

Each slice is a separate code commit + a matching review-request
commit, both pushed before the next slice begins.

| Slice | Packet | Scope | Exit condition |
| --- | --- | --- | --- |
| 002 | `002-datum-wrappers` | Add `src/am/common/datum.rs` (new module) with the four wrappers (`DetoastedVarlena<'a>` enhancements OR new module that re-exports the existing one; `FlatFloat4Source<'a>`; `EcVectorDatum<'a>`; `AttnumLookup`). Wrapper-only. | `cargo check + clippy` clean on `pg18,bench`. |
| 003 | `003-source-rs-consumer-migration` | Migrate `src/am/ec_hnsw/source.rs` 29 → ≤ 14 by routing call sites through the new wrappers. Retire local HNSW wrappers if their bodies fully move to common. | Per-file before/after counts in packet; clippy clean on touched modules. |
| 004 | `004-closeout` | Full 8-step `ecaz bench suite` against `benchmarks/task-50-m5-hnsw-baseline/` (matching that suite's shape — load + recall + latency + storage at both 10k and 100k). Closing summary with per-file deltas, full common wrapper surface, `src/` total, and the explicit SPIRE/IVF/DiskANN handoff list. | All four §Exit Criteria satisfied. |

(Note: scope may split slice 003 into 003a/003b if the source.rs
diff is too large for a single review.)

## Non-goals (restated from task spec)

- No touch to IVF, DiskANN, SPIRE consumer sites of the new wrappers
  — those land in Tasks 55/56/57. This task names the consumer call
  sites in the closeout's handoff list per §Exit Criteria #4.
- No SIMD intrinsic refactor — out of scope.
- No on-disk format change — wrappers are read-side ABI shims.
- No RaBitQ math refactor — Task 51's domain.

## Coordination

- Phase-1 lane #2, after Task 52 (P8 build_parallel typed views) and
  before Task 54 (P3 page/WAL wrappers).
- Branch isolation: off main, per Task 52 reviewer direction.
- The `src/am/common/datum.rs` module pattern mirrors Task 52's
  `src/am/common/dsm.rs` + `parallel_context.rs` pattern — same
  anti-pattern B / view-operations discipline applies.
- Reviewer scope-lock: HNSW-only consumer migration on this branch.

## Memory rules applied

- `feedback_anti_pattern_b_unbounded_lifetime`: wrapper constructors
  are `unsafe fn` returning `Self`/`Option<Self>`, not `&T`.
- `feedback_view_operations_not_accessors`: wrapper methods are
  operations (`as_bytes()`, `as_slice<T>()`, `dims()`, `len()`) — no
  safe `fn(&self) -> &'a Field` accessors over wrapped pointer
  fields.
- `feedback_no_premature_task_close`: Task 53 closes only when all
  four §Exit Criteria are met with full bench evidence.
- `feedback_coder_push_smoke_checks`: push after every slice; smoke
  checks between slices; bench window opens once at task close.

## Artifacts (in this packet)

- `artifacts/manifest.md` — packet-local manifest.
- `artifacts/baseline-unsafe-density.txt` — pre-state line-by-line
  census.
- `artifacts/source-rs-consumer-survey.txt` — per-line wrapper
  mapping + estimated reductions.

## Cross-references

- Supersedes: `reviews/task-50/030-comprehensive-unsafe-burndown-plan`
  §P6 disposition.
- Closes the ceiling documented in:
  `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md`
  §"source.rs ceiling".
- Reviewer direction to branch off main:
  `reviews/task-52/004-build-parallel-shm-toc-migration/feedback/2026-05-23-02-reviewer.md`.
- Existing wrapper precedent (slice 447 / Task 50):
  `src/am/common/detoast.rs`.
