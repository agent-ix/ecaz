# Task 168: DiskANN Batched-Beam Expansion + Prefetch (rabitq streaming path)

Status: proposed (2026-07-06, P1). Successor to the closed Task 70
(scan-kernel) / Task 32 (M5). Foundation for the ec_distann program:
the batched-beam primitive here is distann's M0 hop-round shape
(FR-081), so landing it in DiskANN first avoids forking the beam loop.
Owner: to be assigned. One coder, one branch.

## Why

Task 70 closed the single-node DiskANN latency gap at **10k** (2.14 ms →
0.64 ms, pgvectorscale parity) and its profile showed **frontier
maintenance = 77 %** of scan time at L=64 (`frontier_us` 366 µs of 475 µs;
`visited_set_ops` ~2.3× the graph reads). Two things that profile did **not**
establish, and this task does:

1. **It measured only 10k, on the `pq_fastscan` lane.** The real default /
   benchmarked codec is **`rabitq`** (`docs/benchmarks.md`: `ec_diskann
   (rabitq)`), which runs the **streaming** `RelationGraphReader`
   (`rabitq → VAMANA_SEARCH_CODEC_RABITQ`, so `uses_grouped_pq = false` →
   no chain materialization). The frontier/read/score split on the rabitq
   streaming path at **50k / 100k** has never been profiled.
2. **The beam loop pops one node per hop** (`scan.rs:451-489`), so with
   default R=32 + frontier dedup each `score_batch` almost always runs
   **under** the 32-wide SIMD block — the full-width rabitq/grouped-PQ
   kernels rarely fire and per-flush LUT/prepared-query setup is amortized
   over a handful of candidates.

The wins below attack the measured 77 % frontier cost and the sub-width
scoring, on the codec that actually ships.

## Non-Goals

- Do not change the DiskANN on-disk format. Wins live inside the existing
  `ec_diskann` page/graph layout.
- Do not lower recall. Any scoring/beam change must preserve recall@10 within
  0.5 pp of the rabitq fixture reference at each L (establish the reference in
  Phase 1).
- Do not do broad SIMD backend work — that is Task 21. Only narrow per-call
  dispatch fixes land here.
- Do not re-benchmark rabitq-vs-pq_fastscan as an end in itself (rabitq
  already wins everywhere; that is not a compelling task). Flipping the stale
  `StorageFormat::DEFAULT` (`options.rs:66`) from `PqFastScan` to `RaBitQ` is
  a trivial included fix, not the point of the task.

## Phase 1 — rabitq-streaming characterization at scale (gating)

Mirror Task 70's split, but on the **rabitq** storage format at
**10k / 50k / 100k** (Task 70 only did 10k / pq_fastscan). One packet:

- Per-query wall-time split (frontier maintenance, streaming page read +
  decode, prefilter score, exact heap rerank, result materialization) at
  L=64 and a high L from the standard sweep.
- Per-hop **flush-width histogram**: how far under 32 the per-hop
  `score_batch` actually runs (the width probe is not currently wired into
  the ec_diskann traversal — wire it).
- Establish the recall@10 reference per scale for the floor.
- Rank the Phase 2+ slices by measured share; skip any projected < ~5 %.

## Phase 2 — Batched-beam (width-W) expansion (headline; distann M0 primitive)

Convert the one-node-per-hop loop to **pop the top-W unexpanded frontier
candidates, gather + dedup their union of neighbors, and issue one prefetched
`score_batch`** over that union — filling the 32-wide block and amortizing
LUT/prepared-query setup. This is exactly the ec_distann FR-081 hop-round
shape; land it here as the shared primitive so Task 162 (distann M0) reuses
it. A/B at 10/50/100k (recall-neutral by construction — confirm; latency
win from full-width SIMD + fewer gathers).

## Phase 3 — Graph-page prefetch + streaming node cache

- Wire the existing `common/stream.rs::graph_prefetch_callback` /
  `ReadStream` (already used by **ec_hnsw**, absent in ec_diskann) into the
  streaming reader so next-frontier neighbor blocks prefetch while the current
  batch scores. (Task 32's reverted prefetch trial was *heap-rerank* prefetch,
  warm-cache — *graph-traversal* prefetch was never tried.)
- Add a scan-lifetime decoded-node cache + block-grouped neighbor reads to the
  `RelationGraphReader` (`scan_state.rs:284-333` pins one buffer per node
  today). A/B — expect the win on cold-cache / larger-than-shared-buffers
  tail at 50/100k.

## Phase 4 — Frontier / allocation cleanups

- Bound the frontier candidate heap to `list_size` and stop storing an owned
  `Vec<ItemPointer>` of neighbors per heap entry (`scan.rs:124-129`; re-fetch
  neighbors at pop time) — mirrors `reader.rs`'s bounded loop.
- Borrowing decode: score `search_code` as `&[u8]` from page bytes, kill the
  3 Vec allocs/node in `tuple.rs:273-290`.
- FxHash (not SipHash) for the `in_frontier` visited set of 6-byte TIDs.
- Each A/B'd against the Phase 1 split; land only measured wins ≥ 5 %.

## Also (cheap, included)

- Flip `StorageFormat::DEFAULT` `PqFastScan → RaBitQ` (`options.rs:66`) so the
  code default matches the benchmarked/recommended codec; update the few tests
  that assert the old default. Optionally characterize the `pq_fastscan`
  per-query full-index materialization cost at 50/100k as evidence for
  de-defaulting/retiring that lane (`scan_state.rs:196-227`) — secondary, not
  the task's purpose.

## Exit Criteria

- Phase 1 characterization packet landed (rabitq, 10/50/100k) with a
  reviewer-approved ranked slice list and per-scale recall references.
- Each landed slice has an A/B measurement packet at 10/50/100k
  (recall + latency) on the rabitq streaming path, recall floor preserved.
- The batched-beam primitive (Phase 2) is landed and documented as the shared
  beam shape ec_distann M0 (Task 162) builds on.
- Cross-engine row in `docs/benchmarks.md` refreshed for rabitq DiskANN.
- `cargo clippy --all-targets --no-default-features --features pg18 -D
  warnings` clean; no new `unsafe` outside `target_feature` kernel bodies with
  paired `# Safety`.

## Coordination

- **Sequence before Task 162 (ec_distann M0)** so distann reuses the
  batched-beam primitive instead of forking `scan.rs`.
- Task 70 / Task 32 are closed; this picks up their deferred scan Phase 2/3
  on the codec that actually ships.
- Task 21 owns broad SIMD backend work; route any SVE/SVE2/AVX-512 need there.
- Local bench host is the Intel desktop (per the SPIRE-era note); confirm
  `data/staged-current/` corpora before claiming env-blocked.

## Stop Conditions

- Stop a slice if Phase 1 ranks it < ~5 % of scan wall time at L=64 on the
  rabitq path.
- Stop if the batched-beam A/B shows a recall change at fixed `list_size` that
  cannot be recovered without exceeding the latency budget — document and
  shelve with evidence rather than trade recall for the SIMD win.
