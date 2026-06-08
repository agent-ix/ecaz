# Task 87 Packet 024: Phase 7 SPIRE real10k p99 rerun

## Scope

Packet 023 documented one weak closeout cell: real10k SPIRE had p50/p95 improvements but p99 was effectively flat at `+0.4%`. This packet reruns only that cell with a packet-local `ecaz bench suite` config.

No code changed.

## Evidence

- Suite config: `phase7-spire-real10k-p99-rerun-suite.json`
- Manifest: `artifacts/manifest.md`
- Results: `artifacts/results.jsonl`
- Status: `artifacts/status.log`
- Raw logs: `artifacts/run/`

Validation:

```text
[suite:task87-phase7-spire-real10k-p99-rerun-suite] audit passed: 3 steps
[suite:task87-phase7-spire-real10k-p99-rerun-suite] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

| Cell | Recall | p50 | p95 | p99 | Counters |
| --- | ---: | ---: | ---: | ---: | --- |
| real10k SPIRE off | `1.0000` | `18.809 ms` | `21.837 ms` | `23.966 ms` | zero |
| real10k SPIRE on | `1.0000` | `15.400 ms` | `16.354 ms` | `17.879 ms` | `surface=spire flushes=4800 candidates=1551640 elapsed_ms=1783.173537 lut32_flushes=4800 lut32_candidates=1551640` |

Deltas: p50 `-18.1%`, p95 `-25.1%`, p99 `-25.4%`.

## Closeout Impact

This packet strengthens packet 023's closeout matrix by replacing the prior real10k SPIRE flat p99 note with packet-local evidence that p50, p95, and p99 all improve while recall remains unchanged and all SPIRE candidate batches reach the LUT32 path.
