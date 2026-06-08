# Task 87 Packet 022: Phase 7 real50k/real100k counter suite

## Scope

This packet fills the remaining Phase 7 measurement gap after packet 021. It adds packet-local `ecaz bench suite` evidence for the real50k and real100k surfaces that packet 015 used for SPIRE and IVF, with Task 87 scoring counters enabled.

This is a measurement-only packet. It does not change code and does not merge or depend on Task 91's `QuantCodec` migration.

## Evidence

- Suite config: `phase7-50k-100k-counter-suite.json`
- Artifact manifest: `artifacts/manifest.md`
- Structured suite manifest: `artifacts/run-manifest.json`
- Parsed results: `artifacts/results.jsonl`
- Full run log: `artifacts/run.log`
- Status log: `artifacts/status.log`
- Per-step logs: `artifacts/run/*.log`

Validation:

```text
[suite:task87-phase7-50k-100k-counter-suite] audit passed: 19 steps
[suite:task87-phase7-50k-100k-counter-suite] completed=19 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

| Surface | Recall off/on | Latency off p50/p95/p99 | Latency on p50/p95/p99 | Task 87 counters |
| --- | ---: | ---: | ---: | --- |
| real50k IVF RaBitQ | `0.9300 / 0.9300` | `12.2/13.8/15.3 ms` | `12.3/15.5/18.0 ms` | zero; RaBitQ, not TurboQuant LUT32 |
| real50k SPIRE TurboQuant | `0.9690 / 0.9690` | `21.997/25.240/27.240 ms` | `18.751/21.833/23.164 ms` | `surface=spire flushes=4800 candidates=1739476 elapsed_ms=2006.536739 lut32_flushes=4800 lut32_candidates=1739476` |
| real100k IVF TurboQuant | `1.0000 / 1.0000` | `172.7/183.2/186.5 ms` | `146.2/168.0/179.2 ms` | `surface=ivf flushes=78200 candidates=20000000 elapsed_ms=23574.111606 lut32_flushes=78200 lut32_candidates=20000000` |
| real100k SPIRE TurboQuant | `0.9100 / 0.9100` | `41.179/48.845/51.872 ms` | `35.062/40.653/46.962 ms` | `surface=spire flushes=4800 candidates=3842410 elapsed_ms=4486.740935 lut32_flushes=4800 lut32_candidates=3842410` |

HNSW was probed on the existing real50k and real100k profiles with candidate-batch scoring enabled:

- real50k HNSW latency p50/p95/p99: `5.73/23.7/34.1 ms`; all Task 87 counters zero.
- real100k HNSW latency p50/p95/p99: `7.58/43.2/72.8 ms`; all Task 87 counters zero.

The HNSW probe does not provide a Phase 7 LUT32 route claim. It shows the existing real-corpus HNSW profiles do not exercise the Task 87 common candidate-batch scorer, so HNSW remains on the accepted Phase 5 structural route for this task.

## Notes For Closeout

- Packet 021 already covered real10k after routing SPIRE leaf chunks through the shared candidate-batch scorer.
- Packet 022 adds the missing real50k/real100k counter evidence for the packet 015 surfaces.
- The real50k IVF cell remains the previously documented RaBitQ surface and should be annotated as out of the Task 87 TurboQuant no-QJL LUT32 route in the superseding closeout matrix.
- DiskANN remains handed off to Task 91 via packet 005 and packet 009.

The next packet should be the Phase 7 closeout: a superseding aggregate matrix with counters, updated completion audit, and the final task-status update once the reviewer accepts the Phase 7 evidence.
