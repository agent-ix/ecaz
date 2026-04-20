# Handoff: Task 17 isolated work (next session)

**Branch:** `adr034-diskann-access-method`
**As of:** 2026-04-19, after commit `79bf72d` (ADR review packets)
**Owner:** coder-2

## TL;DR

All pure-Rust isolated slices inside `src/am/diskann/` have shipped
except Phase 5C-3 / 8B / 3 (blocked by native-build lane merge). The
two PROPOSED ADRs (046 insert, 047 vacuum) have gap-list packets
filed — waiting on reviewer resolution before ACCEPTED.

If the native-build lane is still blocked and you want to keep
pushing on this branch, the remaining options are narrow: polishing
existing slices, writing additional design docs, or reviewing other
branches' packets. The heavy lifting inside `src/am/diskann/` is
done.

## What just shipped (this session)

Recent commits on this branch, oldest → newest:

- `f33503d` Phase 8A: tuple-level vacuum primitives
- `b81fa72` Phase 5D: persisted-graph reader
- `476ea6b` Phase 6A: scan algorithm shell
- `79bf72d` ADR-046 + ADR-047 gap lists

Prior-session commits still in play (see commit messages for scope):

- `764e2b8` ADR-045: page-layout discipline
- `611be2e` Phase 5A: in-memory Vamana algorithm core
- `a314308` Phase 5B: slim VamanaNodeTuple
- `eb6e326` Phase 5C-1: persistence sequencer
- `a266927` Phase 5C-2: build orchestrator
- `dad43f1` defer record for Phase 5C-3 / 8B / 3

Review packets filed (review/):

- `11014-adr045-page-layout-discipline/`
- `11015-phase5a-vamana-algorithm-core/`
- `11016-phase5b-slim-tuple/`
- `11017-phase5c1-persist-sequencer/`
- `11018-phase5c2-build-orchestrator/`
- `11021-phase8a-vacuum-primitives/`
- `11022-phase5d-persisted-graph-reader/`
- `11023-phase6a-scan-algorithm-shell/`
- `11024-adr046-review-prep/`
- `11025-adr047-review-prep/`

Test count: 74 `am::diskann::*` tests passing.

## What's blocked

- **Phase 3** (`am/tqhnsw/` rename) — native-build lane conflict.
- **Phase 5C-3** (pgrx ambuild + quantizer training) — native-build
  lane conflict.
- **Phase 7** (pgrx live insert) — needs ADR-046 ACCEPTED (packet
  11024 open) + native-build lane.
- **Phase 8B** (pgrx vacuum callback) — needs ADR-047 ACCEPTED
  (packet 11025 open) + native-build lane.
- **Phase 6B** (pgrx amgettuple) — needs native-build lane merge.

## What's actionable next (thin options)

Pure-Rust work inside `src/am/diskann/` is largely done. Remaining
options are narrow:

### Option A — Overflow heaptid primitive (pure-Rust)

ADR-046 G1 (packet 11024) asks about overflow heaptid handling on
insert. Once ADR-046 resolves, a small primitive module
(`src/am/diskann/overflow.rs`?) implementing the overflow chain's
add / lookup / strip operations at the tuple layer would follow the
same pattern as Phase 8A's vacuum primitives. Good candidate for the
next thin commit.

### Option B — Visited-set reuse refactor

Packet 11022 Q2 and 11023 Q2 both flag `HashSet<ItemPointer>`
allocation per scan. Phase 6B will want a reusable visited buffer.
Small refactor: introduce `VisitedSet` (wrapping `HashSet<ItemPointer>`
with `clear`), thread `&mut self` through `greedy_search_persisted`
and `greedy_descent`. ~40 lines of code, 1 test to prove clear
works.

### Option C — Design doc for Phase 6B pgrx wiring

Write `plan/design/diskann-scan-pgrx.md` sketching the Phase 6B
callback shape: buffer-pin the metadata page once, open the data
chain, construct `PersistedGraphReader`, bind `Quantizer::prepare_
scorer` to `prefilter`, bind `ecvector::exact_distance` to `rerank`,
iterate `ScanResult`s across `amgettuple` cursor state. This
crystallizes the Phase 6B work so it drops in cleanly post-merge.

### Option D — Review another branch's packets

If packets from coder-1 or others are open for review, reviewing
them is net-positive branch work. Check `review/` for unreviewed
packets.

## Conventions to keep

- **Author:** coder-2.
- **Review packets:** numbered 11000s, one directory per packet
  under `review/`, `request.md` is the single deliverable. Next
  free: `11026`.
- **Commits:** one logical unit per phase sub-slice. Trailer:
  `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`.
- **Tests:** every pure-Rust slice ships with unit tests in the
  same file. Prefix per module (`PE-` persist, `BO-` build, `VC-`
  vacuum, `RD-` reader, `SC-` scan). Confirm `cargo check --lib`
  clean and `cargo test --lib am::diskann` green before commit.
- **Don't edit:** `src/am/build.rs`, `src/am/insert.rs`,
  `src/am/scan.rs`, `src/am/search.rs`, `src/am/graph.rs`,
  `src/am/page.rs`, `src/am/source.rs`, `src/am/shared.rs`,
  `src/am/vacuum.rs`. Read them freely for reference; do not edit
  until the native-build merge.
- **Read but don't edit (yet):** `src/quant/*`, `src/storage/page`,
  `src/storage/wal`, `src/transform/*`. Confirm with the user
  before any edit here.

## When the native-build lane merges

1. Rebase `adr034-diskann-access-method` onto merged `main`.
2. Re-evaluate the shared training surface: where SRHT signs +
   grouped-PQ codebook fit live, and the public API.
3. Resume Phase 5C-3 (pgrx ambuild + quantizer wiring).
4. Resume Phase 3 (`am/tqhnsw/` rename + `am/common/` extraction)
   per ADR-041.
5. Resume Phase 7 (pgrx aminsert) once ADR-046 is ACCEPTED.
6. Resume Phase 8B (pgrx vacuum callback) once ADR-047 is ACCEPTED.
7. Resume Phase 6B (pgrx amgettuple). Thin over `src/am/diskann/
   scan.rs::vamana_scan`.

## Pointers to context

- `plan/tasks/17-diskann-access-method.md` — canonical task plan.
- `spec/adr/ADR-045-page-layout-discipline-for-graph-access-methods.md`
  (ACCEPTED)
- `spec/adr/ADR-046-vamana-insert-lock-ordering.md` (PROPOSED,
  gap list in packet 11024)
- `spec/adr/ADR-047-vamana-vacuum-lock-ordering.md` (PROPOSED,
  gap list in packet 11025)
- Memory: `project_diskann_baseline_for_future_ams.md`,
  `project_native_build_conflict_surface.md`.
