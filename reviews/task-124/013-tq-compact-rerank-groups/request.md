# Task 124 Packet 013: TQ Compact Rerank Groups

## Summary

This packet tests a Phase 2 structural storage slice for TurboQuant stage-2:
compact rerank group headers for TQ index-side sidecars.

Result: **do not land**. The slice is structurally targeted, but measured storage
barely moved (`1057.2 B/row` -> `1056.8 B/row` at 100k, still `100.8 MiB`) and
latency movement was mixed/tiny. The temporary code was reverted and preserved
only as `artifacts/discarded-compact-rerank-groups.diff`.

## Evidence

- Manifest: `artifacts/manifest.md`
- Suite config: `artifacts/task124-tq-compact-rerank-groups-100k-suite.json`
- Suite manifest: `artifacts/compact-groups-100k/suite-manifest.json`
- Results: `artifacts/compact-groups-100k/results.jsonl`
- Run log: `artifacts/suite-run.log`
- Discarded diff: `artifacts/discarded-compact-rerank-groups.diff`

## Outcome

The review steering from packet 011 was correct: small metadata compaction does
not solve the storage wall. This packet narrows the remaining viable path to a
larger structural change, such as direct payload addressing/deduplicated payload
storage/fused materialization, or the Shelve path with Phase 6 cold/IO evidence.
