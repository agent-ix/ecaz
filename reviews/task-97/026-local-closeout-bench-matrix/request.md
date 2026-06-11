# Task 97 Packet 026: Local Closeout Bench Matrix

## Request

Please review the local-only Task 97 closeout bench packet. This responds to the latest local feedback by converting the thorough local PG18 benchmark run into packet-local evidence.

No CI or AWS/G4 work was run. The G4 execution remains approval-gated per packet 022 feedback.

## Feedback Processed

- `reviews/task-97/018-qjl32-octet-batch/feedback/2026-06-10-01-reviewer.md`: local AVX2 ladder approved; G4 lane remains before final closeout.
- `reviews/task-97/020-qjl32-neon-forced-parity-hook/feedback/2026-06-10-01-reviewer.md`: forced NEON parity hook approved.
- `reviews/task-97/022-graviton4-closeout-runbook/feedback/2026-06-10-01-reviewer.md`: runbook approved; approval gate preserved.
- `reviews/task-97/024-post-main-landing-audit/feedback/2026-06-10-01-reviewer.md`: post-main landing readiness approved, pending G4 evidence.
- `reviews/task-97/017-status-through-packet-016/feedback/2026-06-10-01-reviewer.md` and pointer files in packets 019/021/023/025: open-items register acknowledged.

## Evidence

- Manifest: `artifacts/manifest.md`
- Suite config: `artifacts/task97-local-closeout-qjl32-suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Structured results: `artifacts/results.jsonl`
- Summary: `artifacts/local-closeout-summary.md`
- Status log: `artifacts/suite-status-cli.log`
- Run log: `artifacts/suite-run-cli.log`
- Per-surface logs: `artifacts/recall-*.log`, `artifacts/latency-*.log`

## Result

The local Intel/AVX2 suite completed:

```text
[suite:task97-local-closeout-qjl32-suite] completed=34 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Coverage:

- IVF, SPIRE, and HNSW.
- 512-row and 4096-row deterministic local fixtures.
- QJL32 TurboQuant, dim 1024, 64 queries, 150 latency iterations.
- Batch-on and batch-off recall and latency paths.
- `[block-kernel-counters]` rows captured in latency logs and parsed into `results.jsonl`.

Behavior:

- All 14 recall cells matched batch-on versus batch-off exactly.
- Direct SPIRE scoring counters showed 2.48x to 2.97x local AVX2 speedups.
- IVF end-to-end p50 improved across every measured knob.
- HNSW end-to-end p50 improved on the 4096-row fixture and 512/ef32, but 512/ef64 regressed to 0.96x.
- SPIRE end-to-end latency was mostly flat even though direct scoring improved; the 4096/nprobe=32 cell should not be claimed as an end-to-end win.

## Reviewer Focus

Please check that the packet is sufficient as the local closeout matrix input, and that the caveats are accurately framed before the later approval-gated G4 execution packet.
