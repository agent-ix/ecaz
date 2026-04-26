# Review Request: IVF/DiskANN Benchmark Pivot

## Summary

This packet records the post-990k decision to stop treating more HNSW
parallel-build tuning as the immediate scale path.

The PG18 HNSW parallel build work remains a win: the real-50k sweep improved
from `07:12.017` at 1 worker to `02:27.948` at 8 launched graph workers after
cluster headroom was fixed. Packet 669 then showed that PG18 HNSW parallel
build launches 8 graph workers correctly on the DBPedia 990k/10k anchor, but
the controlled build still took `01:31:57.326`. That result is useful as a
scale signal, not as a foundation for more threshold tweaking.

The durable plan now says:

- task 26 keeps the HNSW parallel-build result and marks the current path as
  functionally proven but performance-limited at 990k x 1536;
- offline/staged HNSW bulk build remains a later follow-up;
- task 28 becomes the next scale research lane: real Graviton-class IVF and
  DiskANN build/recall/latency baselines, followed by optimization of the
  stronger candidate.

## Files

- `plan/tasks/26-parallel-index-build.md`
- `plan/tasks/28-ivf-diskann-benchmark-optimization.md`
- `plan/tasks/README.md`
- `review/673-c1-ivf-diskann-bench-pivot/artifacts/manifest.md`

## Review Questions

1. Does task 26 now make the HNSW follow-up status clear enough?
2. Is task 28 scoped correctly as benchmark-first, with implementation choices
   gated on measured IVF/DiskANN evidence?
3. Is the Graviton-class benchmark metadata requirement sufficient for future
   product claims?
