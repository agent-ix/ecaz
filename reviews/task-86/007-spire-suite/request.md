# Review Request: SPIRE TurboQuant Suite Evidence

## Summary

This packet adds index-level PG18 evidence for the Task 86 SPIRE TurboQuant LUT scoring change from packet `005-spire-tq-lut`.

The suite is intentionally small and synthetic because the standard real10k SPIRE corpus was not present in this checkout. It uses a packet-local deterministic 1536-d fixture and the canonical `ecaz bench suite` runner rather than a one-off benchmark script.

## Evidence

Artifact manifest: `reviews/task-86/007-spire-suite/artifacts/manifest.md`

Successful suite artifacts:

- Config: `reviews/task-86/007-spire-suite/suite.json`
- Manifest: `reviews/task-86/007-spire-suite/artifacts/suite-manifest-host-rerun.json`
- Results: `reviews/task-86/007-spire-suite/artifacts/results-host-rerun.jsonl`
- Report: `reviews/task-86/007-spire-suite/artifacts/suite-report.md`

Key result:

```text
steps: completed 4, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0
ec_spire index task86_spire_synth256_tq_idx: 328.0 KiB, 1312.0 B/row
nprobe=4: latency p50 0.526 ms, p95 0.645 ms, recall@k 0.5813
nprobe=8: latency p50 0.625 ms, p95 0.634 ms, recall@k 0.9187
```

## Interpretation

The SPIRE TurboQuant path builds and scans under PG18 after the LUT-scoring change. This does not prove an end-to-end speedup by itself; the earlier micro packets established that our existing dim-LUT scorer is the best current query-time kernel among the tested no-format-change options, and this packet verifies the SPIRE index surface still works through the accepted path.

The remote-serving SPIRE export status reports `requires_rabitq_storage_format`, which is expected for a local TurboQuant suite. Query metrics and production read profile still complete with `result_source=local_heap_candidates`.

## Review Focus

- Whether this suite is sufficient index-level evidence for the no-format-change SPIRE LUT scoring checkpoint.
- Whether follow-up Task 86 work should prioritize a real10k current-lane rerun once the canonical corpus is restored, or move directly to the TQ+ calibration-only profile investigation.
