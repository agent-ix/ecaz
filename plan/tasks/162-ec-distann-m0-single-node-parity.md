# Task 162: ec_distann M0 — Single-Node Parity

Status: **done — banded M0 exit** (2026-07-07). Reviewer signoff:
`reviews/task-162/004-parity-remeasure/feedback/2026-07-07-01-reviewer.md`;
operator acceptance: `.../2026-07-07-02-coder.md`. Outcome: parity vs
rabitq ec_diskann holds through the ~0.988 recall band at 10k/50k
(≤1.16×); D7 default measured and flipped to rabitq; D1 (TQ exceeds page
capacity at R=32; buildable formats ≤1.9× raw, no fallback layout needed)
and D3 (C=4096) measured; Gate G0 kill-check GO
(`reviews/task-162/003-g0-killcheck/`). Known miss carried to the M4 risk
list: the 50k ≥0.995 single-node tail is 2.03× (D11 exact-per-expansion
cost; remediation options recorded in packet 004). Evidence:
`reviews/task-162/001..004`. Branch `task-162-ec-distann-m0`, not merged.
Original text follows.

Depends on: Task 161 (specs merged).
Prefer landing Task 168 (DiskANN batched-beam primitive) first and reusing its
width-W beam shape for the FR-081 local-expansion loop, rather than forking
`ec_diskann/scan.rs`.
Owner: coder (to be assigned). One coder, one branch off main after 161.
Priority: P0 — first implementation milestone; carries the program
kill-check spike.

## Why

De-risk record format, head index, and the scan-loop shape with zero network
variables, against the sibling `ec_diskann` as the control. ADR-085 D1's own
arithmetic says the R=32 defaults likely breach NFR-018's 4× budget — M0's
storage measurement decides the format posture before anything distributed
is built.

## Goal

`ec_distann` builds, scans, and matches `ec_diskann` recall single-node;
D1/D3/D7 measured; kill-check spike projects multinode viability.

## Scope

- New AM dir `src/am/ec_distann/` (mod/routine/options/build/scan/insert/
  vacuum/cost/page/tuple) per FR-075; register in `src/am/mod.rs`; handler +
  `CREATE ACCESS METHOD` in `sql/bootstrap.sql`; GUCs (`beam_width`,
  `hop_rounds`) via `register_gucs`.
- FR-076 record format (reuse `ec_diskann/tuple.rs`+`page.rs` patterns,
  add neighbor-code block via `QuantCodec`); vec_id per ADR-085 D6.
- FR-080 head index (single-shard degenerate case; entry-region BFS sample;
  reuse `ec_spire/build/top_graph.rs` in-memory Vamana builder).
- FR-081 loop with LOCAL expansion function (signature identical to the
  future remote fn — the interface seam is the milestone's design output).
- Monolithic build via `build_vamana_graph_with_stats`/`robust_prune`
  (`src/am/ec_diskann/vamana.rs:890,:575`).
- Interim DML posture per FR-083 early slices (delta buffer, tombstone flag).
- Tests: TC-037 (`src/tests/ec_distann_basic.rs`) + M0 bench cells.
- **Kill-check spike** (ADR-085 D2): single-node recall-vs-H curve × the
  measured per-round transport cost of the existing SPIRE pipeline →
  projected multinode p50, in the packet.

## Required Evidence

`ecaz bench suite`, release build, A/B vs ec_diskann at 10k/50k: latency
≤1.3×, distinct_recall@10 within 0.002 (FR-075-AC-4); NFR-018 storage ratio
per codec (D7 comparison); head-cap C sensitivity (FR-080-AC-4); kill-check
spike table.

## Non-Goals

Sharded build/stitch (163), any remote path (164+).

## Acceptance Criteria

1. TC-037 pg_tests green; index create/drop/reindex/options solid.
2. Bench A/B evidence at 10k/50k in the packet; D1 format decision recorded
   (keep defaults / lower R / smaller codes / fallback layout).
3. Kill-check spike published with a go/no-go note against NFR-017.

## References

- FR-075, FR-076, FR-080, FR-081 (local slice), FR-083 (early slices)
- `plan/design/distann-global-graph-architecture.md` (M0)
