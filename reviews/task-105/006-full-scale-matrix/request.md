# Review request: Task 105 Phase 3 — full-scale matrix, baseline comparison, handoff

- Task: `plan/tasks/105-production-optimization-full-scale-sweep.md`
- Packet: `reviews/task-105/006-full-scale-matrix/`
- Kind: aggregation/analysis only (no new measurements, no code change)

## Summary

This packet publishes the three Phase 3 deliverables and completes the
remaining acceptance criteria (AC4; AC2/AC3 evidence is consolidated
here; AC5 teardown is now fully executed):

1. **`artifacts/full-scale-matrix.md`** — the
   scale × AM × quant × option × lane matrix (latency on/off, recall,
   ISA attribution, storage, kernel scoring share), generated from the
   committed lane artifacts by `artifacts/gen_matrix.py`. Headlines:
   SPIRE TQ −23..−57% at every scale/lane; DiskANN TQ consistent wins;
   zero foreign-ISA attribution; recall on/off parity at every cell;
   G4 100k confirm matches Task 99 NEON-cap exactly.
2. **`artifacts/baseline-comparison.md`** — ivf-rabitq1 @1M is −18%
   p50 vs the May final gate at matched ~25% scan fraction and clears
   the pinned vchord bar at 1M (5.0×/1.4× at the two operating
   points); TQ cross-era comparison is geometry-confounded and
   explicitly left without a verdict. No comparator re-runs.
3. **`artifacts/handoff-release-readiness.md`** — the measured
   foundation + known-gaps contract for the safety/cleanup/release
   track (Task 106 gaps, IVF off-arm note, variance-flagged cells,
   snapshot inventory).

## Material caveat surfaced during aggregation

The sweep config's IVF off arm omitted `--ivf-scratch-soa-batch-decode`
instead of forcing the GUC off; after the Phase 1 default flip both
arms run batch decode, so the IVF on/off pairs in packets 004/005 are
same-config noise pairs, not a kernel A/B. The lane manifests and
request files were amended accordingly; the IVF differential evidence
remains Task 99's explicit 100k A/B. Flagged in the matrix doc's
honest markers and the handoff note (one snapshot-restore away if a
fresh differential is wanted).

## Status

With this packet, all five acceptance criteria have evidence on the
branch. Coder does not close the task: review of packets 004/005/006
(and the 2026-06-12-01 feedback response in 004) is open for the
outside reviewer.
