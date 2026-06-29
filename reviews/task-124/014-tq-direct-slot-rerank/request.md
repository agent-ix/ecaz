# Task 124 Packet 014: TQ Direct Slot Rerank

## Summary

This packet tests a Phase 2 structural materialization slice for TurboQuant
stage-2: encode a direct group slot in posting-carried `rerank_tid` values so
scan can avoid group-local heap-TID scans when selecting TQ sidecar payloads.

Result: **do not land**. Recall stayed unchanged, storage stayed unchanged
(`1057.2 B/row`, `100.8 MiB`), and latency movement was mixed/noisy rather than
a clear win. The temporary code was reverted and preserved only as
`artifacts/discarded-direct-slot-rerank.diff`.

## Evidence

- Manifest: `artifacts/manifest.md`
- Suite config: `artifacts/task124-tq-direct-slot-rerank-100k-suite.json`
- Suite manifest: `artifacts/direct-slot-100k/suite-manifest.json`
- Results: `artifacts/direct-slot-100k/results.jsonl`
- Run log: `artifacts/suite-run.log`
- Discarded diff: `artifacts/discarded-direct-slot-rerank.diff`

## Outcome

This rules out another local materialization micro-optimization as the source of
TQ's gap. The remaining Task 124 fork is a larger payload/storage redesign, or
the Shelve path backed by Phase 6 cold/IO evidence.
