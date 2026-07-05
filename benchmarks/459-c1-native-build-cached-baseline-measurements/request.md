# Review Request: C1 Native Build Cached Baseline Measurements

Current head at execution: `b1bba4f`

## Context

Before merge, this branch needed one more measurement packet taken from the
stable cached surfaces on the long-lived pg17 scratch cluster:

- cached real-corpus surface: `tqhnsw_real_50k_corpus`
- cached query subset: `tqhnsw_real_50k_queries_50`
- cached source-fixture surface from packet `454`

The intent here is to record the actual merge-baseline reruns on the persisted
measurement surfaces, not just confirm that the caches exist.

## What ran

Cached real-corpus gate reruns on `/home/peter/.pgrx`, port `28817`:

```bash
./scripts/run_real_corpus_recall_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --port 28817 \
  gate \
  --prefix tqhnsw_real_50k \
  --storage-format turboquant \
  --queries-table tqhnsw_real_50k_queries_50
```

```bash
./scripts/run_real_corpus_recall_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --port 28817 \
  gate \
  --prefix tqhnsw_real_50k \
  --storage-format pq_fastscan \
  --queries-table tqhnsw_real_50k_queries_50
```

## Result

Cached `50k` baseline reruns remained strong:

- `turboquant`: `m=8, ef=40 -> 0.886`; `m=8, ef=128 -> 0.930`; `m=8, ef=200 -> 0.930`; `m=16, ef=200 -> 0.964`
- `pq_fastscan`: `m=8, ef=40 -> 0.886`; `m=8, ef=128 -> 0.930`; `m=8, ef=200 -> 0.930`; `m=16, ef=200 -> 0.968`

These are the packet-local merge-baseline measurements from the stable cached
real-corpus surfaces.

The separate 25-query source-fixture lane from packet `454` still matters, but
it remains a determinism / reuse check rather than the branch's primary recall
gate. This packet is only recording the cached real-corpus baseline reruns.

## Artifacts

See [artifacts/manifest.md](artifacts/manifest.md) plus:

- [artifacts/turboquant-gate.sql](artifacts/turboquant-gate.sql)
- [artifacts/turboquant-gate.tsv](artifacts/turboquant-gate.tsv)
- [artifacts/pq-fastscan-gate.sql](artifacts/pq-fastscan-gate.sql)
- [artifacts/pq-fastscan-gate.tsv](artifacts/pq-fastscan-gate.tsv)

## Review focus

1. Is this sufficient as the final cached-surface baseline packet before merge?
2. Is the separation clear between the cached real-corpus recall baseline here
   and the source-fixture determinism lane already documented in packet `454`?
