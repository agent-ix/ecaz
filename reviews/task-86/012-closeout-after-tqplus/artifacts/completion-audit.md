# Task 86 Completion Audit After TQ+

Head SHA: `c7e85e8ac542a20c3934d8c24c0a875d5a935fc2`

Task bucket: `reviews/task-86/012-closeout-after-tqplus/`

Date: 2026-06-07

## Scope

This audit re-checks Task 86 after packet 011 added real-corpus TQ+ evidence.
The previous closeout packet accepted the SPIRE TurboQuant LUT slice but
explicitly left TQ+ unmeasured. Packet 011 closes that missing measurement.

This audit does not self-approve the task. It records coder-complete evidence
and requests reviewer acceptance.

## Requirement Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| Analyze TurboVec TurboQuant against our TurboQuant only. | `reviews/task-86/001-turbovec-tq-analysis/artifacts/turbovec-tq-analysis.md` and packet 001 request/feedback. | Satisfied. |
| Explain "query encoded in same space." | Packet 001 report: TurboVec rotates/calibrates query into the same transformed coordinate space and builds query LUTs; it does not store queries as database codes. | Satisfied. |
| Determine whether TurboVec has a faster query-time comparison approach. | Packet 001 report plus packets 002/003/004/005/008/011. Transferable winners are no-QJL dim-LUT scoring and TQ+ calibration; byte-pair LUT and renorm-only paths are not promoted. | Satisfied. |
| Compare vector size. | Packet 001 byte accounting; packet 008 SPIRE storage rows; packet 011 IVF baseline-vs-TQ+ storage rows. | Satisfied. |
| Compare SIMD/kernel strategy. | Packet 001 report and packet 006 options report compare TurboVec blocked/LUT/fused flat scan strategy with our current TQ dim-LUT/SIMD surfaces. Packet 004 records byte-pair LUT negative evidence. | Satisfied. |
| Identify TurboVec index type. | Packet 001 identifies TurboVec as a flat exhaustive compressed-vector scanner, not HNSW, DiskANN, IVF, or SPIRE. | Satisfied. |
| Keep comparison scoped to TurboQuant, not RaBitQ or other quantizers. | Task file non-goals; packet 001/006 framing; packet 011 compares only IVF TurboQuant baseline to IVF TQ+. | Satisfied. |
| Do not ship optimization code without real benchmark evidence. | Packet 008 measured SPIRE LUT before/after on real10k/50k/100k. Packet 011 measured IVF TQ+ before/after on real10k/50k/100k. | Satisfied. |
| Use `ecaz bench suite` for benchmark matrices. | Packet 008 suite configs; packet 011 `suite-baseline.json` and `suite-tqplus.json`; manifests cite suite commands and structured results. | Satisfied. |
| Include 10/50/100 spread. | Packet 008 SPIRE real10k/50k/100k; packet 011 IVF TQ+ real10k/50k/100k. | Satisfied. |
| Include recall, latency, and storage. | Packet 008 `benchmark-delta.md`; packet 011 `artifacts/manifest.md` tables. | Satisfied. |
| TQ+ real-corpus measurement. | Packet 011: TQ+ improves recall and p50/p95/p99 latency at every measured IVF point with unchanged hot posting bytes. | Satisfied. |
| Account for AM transferability across HNSW, DiskANN, IVF, SPIRE. | Packet 001/006 discuss transferability; packet 011 explicitly scopes TQ+ to IVF and lists SPIRE/HNSW/DISKANN follow-up measurement, avoiding cross-AM overclaim. | Satisfied. |
| Any accepted code change has packet-backed TurboQuant baseline evidence. | SPIRE LUT has packet 008; IVF TQ+ has packet 011. | Satisfied. |
| Any rejected/shelved idea explains blocker. | Packet 006 options report and packet 010 audit list byte-LUT latency, renorm quality/storage, blocked slabs AM fit, dense rotation query cost, fused top-k API/workload fit, and DiskANN adapter mapping blockers. | Satisfied. |
| No unrelated quantizer work included. | Code changes are TurboQuant LUT/TQ+ scoped. Packet 011 baseline compares only TurboQuant vs TQ+. | Satisfied. |
| No new unsafe blocks. | TQ+ changes add no new unsafe blocks; IVF scan/insert call existing unsafe model loaders only through existing unsafe functions. | Satisfied by source inspection; reviewer should confirm. |
| PG18-focused validation for scan/storage/SQL-visible behavior. | Packet 008/011 PG18 suite runs; packet 011 logged `cargo check`, IVF quantizer tests, and metadata storage-format test before and after cleanup. | Satisfied. |
| Format-changing slice has ADR or task-local format-version plan. | Packet 011 `artifacts/tqplus-format-plan.md` documents storage-format tag 4, calibration chain, compatibility, insert/scan/vacuum semantics, and promotion requirements. | Satisfied. |

## TQ+ Evidence Summary

Packet 011 compares `storage_format=turboquant_tqplus` against
`storage_format=turboquant` on IVF with rerank disabled.

| fixture | recall baseline -> TQ+ | p50 baseline -> TQ+ | index B/row baseline -> TQ+ |
| --- | --- | --- | --- |
| real10k | `0.9740/0.9745/0.9745` -> `0.9860/0.9870/0.9870` | `2.90/7.02/8.96 ms` -> `2.68/6.48/8.30 ms` | `951.1` -> `952.7` |
| real50k | `0.9265/0.9450/0.9470` -> `0.9400/0.9665/0.9685` | `10.8/31.5/45.2 ms` -> `10.0/28.9/41.1 ms` | `925.2` -> `925.5` |
| real100k | `0.9225/0.9505/0.9525` -> `0.9300/0.9605/0.9620` | `22.8/70.7/91.5 ms` -> `21.2/64.5/83.5 ms` | `925.5` -> `925.7` |

This satisfies the reviewer escalation that synthetic TQ+ MAE was not enough
for a production-bound task.

## Current Code State

The branch now has three post-reopen commits:

- `e0ae9fe7d` - IVF `turboquant_tqplus` measurement profile.
- `16f1e6104` - packet 011 real-corpus benchmark evidence.
- `c7e85e8ac` - format plan, production TQ+ API naming cleanup, quantile cache, persisted calibration validation, and current-state logs.

The TQ+ production path no longer calls `*_for_test` helpers. The remaining
TQ+ `*_for_test` functions are byte-LUT probe helpers gated to test/bench.

## Remaining Follow-Ups

These are not blockers for Task 86 closure because the task asked for
investigation and measured candidate improvements, not full cross-AM TQ+
rollout:

- Decide whether to replace the reused IVF PQ codebook tuple chain with a
  dedicated calibration tuple before broader production promotion.
- Add negative/corruption fixtures for invalid TQ+ calibration metadata.
- Measure TQ+ on SPIRE TurboQuant, then map HNSW/DISKANN codec surfaces before
  making cross-AM support claims.
- Re-run the real spread if the tuple kind or scoring behavior changes.

## Coder Verdict

Coder-complete pending reviewer acceptance.

The missing TQ+ real-corpus benchmark has been supplied, the format-change plan
is packet-local, and the current code has PG18-focused validation logs. An
outside reviewer should now review packet 011 and this closeout audit before
the task is marked complete.
