# Task 159: graph-AM RaBitQ flush-width profile (HNSW/DiskANN small-batch regime)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P3

## Why

HNSW and DiskANN flush the RaBitQ batch scorer **once per expanded node**
(HNSW `flush_rabitq_search_code_batch`, `src/am/ec_hnsw/scan.rs:2624/3951`;
DiskANN `score_batch` after `read_node` of the neighbor set,
`src/am/ec_diskann/scan.rs:458-475`), so batch width equals live neighbor
degree (~16-64) and a large share of flushes land in the `<32` partial path
(`score_rabitq_bits1_partial`). The width-bucket counters exist precisely to
quantify this (`src/am/common/candidate_batch/counters.rs:111-114,131-143`)
but nobody has published the histogram for the graph AMs on the RaBitQ lane.
Task 134 measured a small-batch scorer **negative — for TurboQuant**; the
RaBitQ graph lanes were never measured, and the payload layout differs
(scattered `Arc<CachedGraphElement>` / owned `VamanaNodeTuple` refs, not a
contiguous slab).

## Scope

- Publish the width-bucket histogram + kernel-vs-partial time split for
  ec_hnsw and ec_diskann RaBitQ (storage_format='rabitq') at 10k/50k/100k,
  using the existing counters (enable via the Task 150 instrumentation gate
  if that has landed).
- From the data, size the ceiling of (a) cross-node flush accumulation (defer
  scoring until ≥32 pending, respecting beam-order correctness constraints)
  and (b) a partial-width kernel improvement — as an Amdahl bound, not an
  implementation.
- Recommendation: file the follow-up implementation task only if the bound
  clears ~5% end-to-end; otherwise record the negative next to Task 134's TQ
  datum and stop.

## Out of Scope (hard)

- No traversal-order or beam-semantics changes in this task; measurement +
  direction only. Cross-node accumulation, if recommended, is its own task.

## Gate / Exit Criteria

- Committed histograms + time splits for both graph AMs at 10k/50k/100k and a
  data-backed go/no-go on the follow-up. Closes when those land.
