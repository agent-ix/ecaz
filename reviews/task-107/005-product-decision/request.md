# Task 107 Packet 005 Product Decision

## Summary

This packet closes the Phase 3 product decision requested by Task 107. It uses
the completed Task 107 benchmark matrix plus existing comparator artifacts; it
does not rerun HNSW, IVF, DiskANN, vchord, or single-node/single-disk SPIRE
benchmarks.

## Recommendation

- Drop local multi-disk / multi-store SPIRE as a product surface for now.
- Keep multinode SPIRE only as a narrow research or internal-validation surface.
- Do not promote SPIRE RaBitQ or SPIRE TurboQuant as product-relevant
  alternatives to the current comparator frontier without a separate follow-up
  that materially reduces latency and resolves the storage observability gap.

## Key Reasons

- Single-node 2-disk SPIRE did not improve the 1m product-scale result.
  RaBitQ 1m with 2 stores had identical recall to the available same-host
  single-store reference but worse k10 c1 latency at every measured nprobe
  through 32.
- Multinode SPIRE is the only measured shape with a real performance win over
  single-node SPIRE. RaBitQ 1m distributed reached k10 c1 mean
  `89.3 / 103.6 / 111.8 / 121.3 ms` at nprobe `8 / 16 / 24 / 32`, versus the
  same-host single-store reference at `187.9 / 336.2 / 487.4 / 620.9 ms`.
- The win is still not enough to justify productizing the distributed SPIRE
  surface. At 1m, existing non-SPIRE comparators include pgvectorscale DiskANN
  at `6.5 ms p50 / 0.980 recall@10` and vchord RaBitQ at
  `90.3 ms p50 / 0.9995 recall@10`.
- The 2-disk storage rows are not decision-grade: `ecaz bench storage` reports
  only the catalog `ec_spire` relation (`168.0 KiB`, `0.2 B/row`) and misses
  the per-store tablespace payload. The packet treats that as an explicit
  observability gap, not as a storage win.

## Evidence

- Decision analysis:
  `reviews/task-107/005-product-decision/artifacts/decision.md`
- Decision manifest:
  `reviews/task-107/005-product-decision/artifacts/manifest.md`
- Task 107 completed matrix:
  `reviews/task-107/004-distributed-completion/artifacts/manifest.md`
- Existing comparator baselines:
  `benchmarks/comparators-50k-100k-1m/manifest.md`
- Existing Task 106 single-node SPIRE / IVF / HNSW AWS evidence:
  `reviews/task-106/004-aws-targeted-bench/artifacts/manifest.md`
  and
  `reviews/task-106/004-aws-targeted-bench/artifacts/aws-intel/results.jsonl`

## Feedback Addressed

- B1 / AC7: added this Phase 3 keep/drop/narrow decision packet.
- B2 / AC5: explicitly marked multi-store storage as a known observability gap.
- B3 / AC6: cited existing HNSW/IVF/comparator baselines without reruns.
- Housekeeping: transient untracked SSM tunnel state and abandoned L4 scratch
  files were removed locally; committed L4 drift artifacts are quarantined in
  the packet-004 manifest as out-of-scope.
