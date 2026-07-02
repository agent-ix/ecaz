# Task 139: SPIRE Routing Selectivity Pareto

Status: proposed (2026-07-02; filed from the Task 131 closeout research
synthesis).
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 — the highest-leverage measured latency direction after Task 131
shelved scan-time pruning. Depends on Task 138's distinct-recall metric for
all recall claims.

## Why

Task 131 measured that the representative distributed read shape
(`n128/b4/nprobe96`) scans **93-95% of all corpus row-instances per query**
(packet 027: 37.3k of 40k at 10k; 189.3k of 200k at 50k). No downstream
optimization survives that: recall-safe pruning ceilings collapsed to 0.010%
of rows at 50k (Task 131), heap-side reduction was latency-flat (Task 131
Phase 1), and transport bytes were not the driver (Task 123).

Meanwhile the fine-list shape already measured 4-7x faster at matched
(duplicate-tolerant) recall in existing packets:

- 50k: `n1024/b2` p50 `663.809 ms` vs `n128/b4` p50 `2582.977 ms`
  (Task 131 packet 024 Phase 1 table, before-arms).
- 100k: `n1024/b2/nprobe64` p50 `0.73-0.78 s` vs `n128/b4/nprobe96` p50
  `5.1-5.2 s` (Task 123 packets 019/020, cited as the Task 131 baseline).

That is routing selectivity working: probing 64/1024 lists (~6%) instead of
96/128 (75%). But the space was never mapped: nlists has only ever been tested
at {32, 128, 1024}; recall-vs-nprobe was never normalized to fraction of
corpus scanned; `top_graph_search_list_size` was clipped at 96 with recall
still rising (Task 121); `training_sample_rows` saturation beyond 50k is
unexplored; and no controlled single-vs-distributed attribution run exists
(cross-packet comparisons are confounded by shape/host differences).

## Goal

Map the routing-config Pareto frontier at matched distinct-recall and produce
a promote / iterate / shelve decision for a new default distributed read
shape. Target to beat: recall(distinct) >= 0.999 at <= 10-15% of corpus
row-instances scanned per query, with p50 at or under the current `n1024/b2`
numbers and storage accounted.

## Scope

### Phase 0 - Fixture Normalization

- Report fraction-of-corpus-scanned (row-instances available per query /
  total row-instances) alongside raw nprobe in every cell, using the Task 131
  packet 010/011 scan-profile counters.
- All recall claims use `distinct_recall@k` from Task 138 (hard dependency).

### Phase 1 - nlists x boundary Sweep

- Grid: nlists {128, 316, 512, 1024, 2048} x boundary_replica_count {0, 1, 2}
  at 50k and 100k, `rabitq`, standard sweep normalized per Phase 0.
- Measure distinct-recall, latency p50/p95/p99, index storage, build time,
  corpus-fraction scanned.
- A/B per change; use `ecaz bench suite` with configs in the packet.

### Phase 2 - Router Quality Levers On The Frontier Shape

- `top_graph_search_list_size` saturation: 96 / 128 / 200 / 400 (Task 121
  clipped at 96 with recall still rising).
- `training_sample_rows` saturation: 50k / 100k / full-scale sample.
- Only on the Phase 1 frontier shape(s); one variable at a time.

### Phase 3 - Controlled Distribution Attribution

- Same index shape, same host, same corpus: single-instance vs 3-worker
  multi-instance lane, both at the frontier config and at the current
  representative config. This closes the long-standing confound (no
  apples-to-apples run exists in any packet).

### Phase 4 - Scale Confirmation And Decision

- 1m confirmation of the winning shape if 50k/100k show promise (repo
  closeout rule: 10k/50k/100k minimum for any promotion claim).
- Closeout: promote / iterate / shelve for a new default shape, with the
  full evidence matrix.

## Required Evidence

- `ecaz bench suite` for every matrix; configs checked into the owning packet.
- distinct_recall@k, latency p50/p95/p99, storage, build time,
  corpus-fraction scanned, selected PID counts per cell.
- 10k/50k/100k minimum before any promotion claim; 1m encouraged.

## Non-Goals

- No scan-time pruning protocols (Task 131 shelved them with scale evidence).
- No transport/payload work (Task 121/125 territory).
- No dedupe fix (Task 137) and no metric work (Task 138) — consume their
  outputs.
- No AWS product-claim matrix without explicit user approval (Task 120 rule).

## Acceptance Criteria

1. Every cell reports corpus-fraction scanned alongside nprobe.
2. Phase 1 grid complete at 50k/100k with distinct-recall.
3. Frontier shape identified with router-lever saturation measured.
4. Single-vs-distributed attribution numbers exist for frontier + current
   representative shapes.
5. Final packet gives promote / iterate / shelve for a new default shape,
   citing immutable packets.

## References

- `plan/tasks/138-spire-distinct-recall-metric-audit.md` (dependency)
- `plan/tasks/131-spire-streaming-global-topk-pruning.md` (why pruning lost)
- `plan/tasks/121-spire-coarse-routing-recall-doe.md` (prior DOE, levers)
- `plan/tasks/120-spire-coarse-rerank-measurement-program.md` (loss attribution)
- `reviews/task-131/027-phase3-increment-a-ab/` (93-95% scan measurement)
- `reviews/task-123/004-phase-b-100k-nlists-spotcheck/` (nlists=1024 data)
- `crates/ecaz-cli/src/profiles.rs` (default sweeps)
