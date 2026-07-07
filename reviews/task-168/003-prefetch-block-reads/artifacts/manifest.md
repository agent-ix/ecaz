# Task 168 Packet 003 — graph prefetch + block-grouped reads (Phase 3, SHELVED)

- Task: `plan/tasks/168-diskann-batched-beam-and-prefetch.md`; work branch
  `task-168-diskann-batched-beam`.
- Code under test: `9e0a09617` (prefetch + block-grouped `read_nodes`), a
  working-tree grouping-only variant (prefetch call removed, otherwise
  identical — reproducible by deleting the `prefetch_relation_blocks` call
  from that commit's `scan_state.rs` override), and baseline `1737ad5be`
  (packet-002 W=4 state). **Outcome: shelved by `685e81f0c`, which restores
  `src/` byte-identical to the packet-002 state.**
- Host / backend: Intel desktop, PG18 pgrx tree (port 28818), db
  `tqvector_bench`; release backend verified per arm (`build-profile.log`).
- Fixture: packet-001 indexes reused (`t168_p1_real{10k,50k,100k}_diskann`,
  rabitq); all arms at the W=4 beam default. Truth caches from packet 001.
- Commands:
  - after-arm: `ecaz --host /home/peter/.pgrx --port 28818 bench suite run
    --config <pkt>/suite.json --artifact-dir <pkt>` (recall+latency × 3
    scales + per-L OS-evict cold steps at 100k; `results.jsonl`).
  - grouping-only arm: `--config <pkt>/suite-grouponly.json
    --results-output <pkt>/results-grouponly.jsonl` (first pass) and
    `--results-output <pkt>/results-grouponly-warm.jsonl` (second pass —
    **use the warm pass**; the first pass's 100k rows are contaminated by
    the preceding suite's OS-cache eviction steps: 11.1 ms at L=64 warming
    to normal through the sweep).
  - baseline arm: packet 002 `results-w4.jsonl` (same day, same host, same
    fixture, same GUCs).
- Bespoke SuiteConfig justification: commit-level A/B packet over the
  packet-001 fixture, not the standard lane sweep.

## Key results (mean warm latency at W=4; recall identical in all arms)

| scale | L | baseline | prefetch+group | group-only (warm pass) |
|---|---|---|---|---|
| 10k | 64 | 3.27 ms | 3.35 ms | 3.35 ms (+2.4%) |
| 10k | 800 | 5.85 ms | 6.77 ms | 5.69 ms (−2.7%) |
| 50k | 64 | 3.86 ms | 3.93 ms | 4.28 ms (+10.9%) |
| 50k | 400 | 6.77 ms | **8.75 ms (+29%)** | 6.62 ms (−2.2%) |
| 50k | 800 | 10.1 ms | 12.2 ms | 10.3 ms (+2.0%) |
| 100k | 64 | 4.04 ms | 4.29 ms | 4.72 ms (+16.8%) |
| 100k | 200 | 5.72 ms | 6.34 ms | 6.67 ms (+16.6%) |
| 100k | 800 | 12.3 ms | 14.4 ms | 13.2 ms (+7.3%) |

Full tables in `results.jsonl` / `results-grouponly-warm.jsonl`;
recall@10 was bit-identical to baseline at every cell in both variants
(read-order-only change), `recall-*-after.log`.

Cold arm (informational, after-variant only): OS-evict then 50 iterations —
100k L=64 5.44 ms, L=800 14.4 ms (`latency-100k-after-cold-l{64,800}.log`).
No baseline cold arm was run since the warm regression already killed the
slice.

## Findings

1. **Prefetch+grouping loses everywhere on warm cache** (up to +29%). The
   read-stream prefetch helper pins and releases every block synchronously,
   then the grouped read re-pins it — double buffer traffic with zero I/O
   to hide on a cached index.
2. **Grouping alone is neutral-to-negative**: no cell reaches the task's
   ≥5% landing bar; 100k is consistently worse (sort + placeholder buffer
   overhead exceeds the saved pin/unpin on warm pages).
3. The task file's scan-lifetime decoded-node cache was not built: the
   `in_frontier` dedup already guarantees at most one read per node per
   scan (packet 001 NOTICE rows: `graph_read_count` == unique frontier
   inserts), so an intra-scan cache cannot hit.
4. Verdict per the task's Stop Conditions: **shelve Phase 3 with
   evidence** (`685e81f0c` restores the packet-002 code). Cold-tail
   prefetch remains plausible only for indexes that exceed memory — that
   regime is out of this task's 10/50/100k envelope.
