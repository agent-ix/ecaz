---
id: Task-001
title: "M0 — ec_distann single-node parity + kill-check spike"
type: Task
status: completed
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/FR-075
    type: references
  - target: ix://agent-ix/ecaz/FR-076
    type: references
  - target: ix://agent-ix/ecaz/FR-080
    type: references
  - target: ix://agent-ix/ecaz/FR-081
    type: references
  - target: ix://agent-ix/ecaz/NFR-018
    type: references
  - target: ix://agent-ix/ecaz/TC-037
    type: verifies
---
# Task-001: M0 — ec_distann single-node parity + kill-check spike

## Scope

Repo task `plan/tasks/162-ec-distann-m0-single-node-parity.md` (normative).
New AM `src/am/ec_distann/` with the FR-076 lean record (NO inline
full-precision vector — ADR-085 D11), degenerate single-shard co-placed heap
tier (FR-078 pattern, local), FR-080 head index (single-shard), FR-081 local
loop consuming the Task-168 batched-beam, monolithic seed-deterministic
build; D1/D3/D7 measurements and the ADR-085 D2 kill-check spike (Gate G0).

## Subtasks

- [ ] **AM scaffold.** mod/routine/options/build/scan/insert/vacuum/cost/
      page/tuple per FR-075; register in `src/am/mod.rs`; handler +
      `CREATE ACCESS METHOD` in `sql/bootstrap.sql`; GUCs `beam_width`,
      `hop_rounds` via `register_gucs`. Pattern: `src/am/ec_diskann/routine.rs`,
      `sql/bootstrap.sql:689-739`.
- [ ] **FR-076 lean record.** vec_id = hash64(source_identity) (D6; collision
      = build error), heap_tid, flags, search_code, neighbor block via
      `QuantCodec` (`src/am/common/quant_codec.rs`); extend
      `ec_diskann/{tuple,page}.rs` patterns.
- [ ] **Co-placed heap + exact rerank (local).** FR-079's exact_dist from the
      local heap read; mirror `ec_diskann/routine.rs exact_heap_rerank_distance`.
- [ ] **FR-080 head index.** Entry-region BFS sample; reuse
      `ec_spire/build/top_graph.rs:114-150` in-memory Vamana.
- [ ] **FR-081 local loop.** Consume `greedy_descent_beam_with`; design and
      FREEZE the local-expansion signature (== future remote fn; the
      milestone's design output).
- [ ] **Monolithic build.** `build_vamana_graph_with_stats` / `robust_prune`
      (`src/am/ec_diskann/vamana.rs:890/:575`), seed-deterministic (FR-077
      determinism contract starts here).
- [ ] **Interim DML posture.** Delta buffer + tombstone flag (FR-083 early
      slices).
- [ ] **TC-037 tests** in `src/tests/ec_distann_basic.rs`.
- [ ] **M0 bench cells.** `ecaz bench suite`, release backend: parity A/B vs
      rabitq ec_diskann at 10k/50k (latency ≤1.3×, distinct_recall@10 within
      0.002, FR-075-AC-4); NFR-018 storage ratio per codec (D7); head-cap C
      sensitivity (FR-080-AC-4).
- [ ] **Gate G0 kill-check spike.** Recall-vs-H × measured SPIRE per-round
      transport cost → projected multinode p50 vs the 37.6 ms anchor;
      go/no-go note in the packet.

## Deliverables

- `src/am/ec_distann/` scaffold + tests; review packet
  `reviews/task-162/00N-*` with manifest, suite config, results.jsonl.
- D1 format decision recorded (keep defaults / lower R / smaller codes /
  D1 fallback layout).
- Frozen expansion-seam signature documented in the packet.

## Notes

- Branch `task-162-ec-distann-m0`. Do NOT re-introduce an inline vector or a
  per-neighbor stored rerank tier (D11).
- NFR-018 arithmetic sits AT 4.0× at R=32 defaults (D1 refreshed by D11) —
  the storage measurement decides, don't skip it.
- Unblocks: Task-002 (build substrate), Task-003 (frozen seam).
