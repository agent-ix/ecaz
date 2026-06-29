# Task 124 Closeout Manifest

- head SHA before closeout packet: `2cbbfc066d1c9f2bd7af5d9ad6e3ee7d575b67c0`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/016-closeout-shelve`
- lane: closeout decision
- decision: Shelve
- date: 2026-06-29
- run surface: local PG18 `ec_ivf`, staged real-corpus 10k/50k/100k suites, packet-local artifacts

## Evidence Sources

- `reviews/task-124/001-tq-stage2-engine-slice/`: Phase 0/1 audit and disabled engine path.
- `reviews/task-124/002-tq-stage2-attribution-counters/`: Phase 3 attribution counters.
- `reviews/task-124/003-tq-stage2-ab-suite/`: first required 10k/50k/100k in-engine A/B matrix and partial payload loader.
- `reviews/task-124/005-tq-final-width-sweep/`: final-width recall/latency sweep; final10 rejected, final15 selected for later work.
- `reviews/task-124/007-tq-binary-stage2-suite/` and `reviews/task-124/008-tq2-stage2-suite/`: byte-reduction format attempts rejected on recall.
- `reviews/task-124/011-tq-selected-payload-slab/`: kept measured materialization locality improvement; reviewer directed Phase 2 structural slice or Shelve after Phase 6.
- `reviews/task-124/012-tq-stage2-topk-fusion/`: score/top-k/materialization fusion attempt rejected and reverted.
- `reviews/task-124/013-tq-compact-rerank-groups/`: compact header structural storage slice rejected and reverted.
- `reviews/task-124/014-tq-direct-slot-rerank/`: direct slot materialization slice rejected and reverted.
- `reviews/task-124/015-tq-phase6-local-cache-evict/`: required Phase 6 local IO-sensitive validation.

## Closeout Criteria Check

Task 124 allowed three closeout outcomes:

- Promote: not satisfied. Recall matched, but p50/p95/p99 latency was not lower across product-relevant cells, and storage/IO tradeoff was not justified.
- Iterate: no longer the right task-local outcome after the reviewer-directed fork. The cheap and medium structural TQ levers were tested, the scorer was proven SIMD, and Phase 6 did not reveal the IO win needed to justify the 4.5x index footprint.
- Shelve: satisfied. In-engine TQ stage-2 cannot beat current RaBitQ + f32 at the measured product matrix.

## Key Facts

- Hot path is not scalar: TQ stage-2 reports `isa=neon`, `scalar_candidates=0`, `width_ge32=100` in the measured suites.
- Required 10k/50k/100k A/B evidence exists in packet 003; recall matched the f32 baseline after the partial payload loader.
- Final width 10 was explicitly rejected because it broke recall; final15 was the best measured narrower final exact width but still did not close the task.
- Storage remained the blocker: 100k f32/source ec_ivf index `22.5 MiB` vs TQ ec_ivf index `100.8 MiB`.
- Phase 6 local macOS relation `F_NOCACHE` validation at 100k did not produce a product latency win:
  - nprobe32 f32 `p50=5.74 ms`, `p95=9.53 ms`, `p99=13.8 ms`;
  - nprobe32 TQ `p50=6.76 ms`, `p95=10.9 ms`, `p99=14.3 ms`;
  - nprobe64 f32 `p50=9.01 ms`, `p95=11.5 ms`, `p99=11.8 ms`;
  - nprobe64 TQ `p50=9.24 ms`, `p95=9.98 ms`, `p99=12.8 ms`.

## Validation For This Packet

This packet is documentation/status-only. No code tests were run for the closeout packet itself. The cited code and benchmark validation remains in packet-local artifacts 001-015.
