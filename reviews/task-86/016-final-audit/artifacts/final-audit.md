# Task 86 Final Coder Audit

Head SHA: `bf819567c3f00d6944a40d9cfa4bdd8f6cb1ae9f`

Date: 2026-06-07

## Verdict

Coder-side Task 86 requirements are satisfied, with outside reviewer acceptance still pending for the reopened TQ+ and cleanup packets.

This audit does not self-close reviewer workflow. It records that the previously missing optimization evidence now exists and that the no-new-unsafe-blocks exit criterion has been corrected.

## Requirement Check

| Requirement | Evidence | Coder status |
| --- | --- | --- |
| Investigate TurboVec TurboQuant against our TurboQuant, not RaBitQ or another quantizer. | Packet 001 report and packet 012 audit. Packet 011 compares only IVF `turboquant` vs IVF `turboquant_tqplus`. | Satisfied. |
| Answer README/query-same-space question. | Packet 001 explains TurboVec rotates/calibrates query into the same transformed coordinate space and builds query LUTs; database vectors stay packed. | Satisfied. |
| Identify whether query-time comparisons are faster because of encoding/query transform/scorer layout. | Packet 001 identifies per-coordinate calibration, query LUTs, no database-vector decompression, blocked flat-scan layout, and fused top-k as the relevant TurboVec choices. Packet 008 and packet 011 measure two transferable TQ-family candidates. | Satisfied. |
| Compare vector size. | Packet 001 byte accounting; packet 008 storage unchanged for SPIRE; packet 011 storage changes only by calibration metadata noise (`951.1 -> 952.7`, `925.2 -> 925.5`, `925.5 -> 925.7` B/row). | Satisfied. |
| Compare SIMD/kernel strategy. | Packet 001 and packet 006 compare TurboVec blocked/LUT/fused kernels to our TQ dim-LUT/SIMD surfaces. Packet 004 records byte-pair LUT as a negative result. Packet 008 measures SPIRE LUT routing. | Satisfied. |
| Identify TurboVec index type. | Packet 001: TurboVec is a flat exhaustive compressed-vector scanner, not HNSW, DiskANN, IVF, or SPIRE. | Satisfied. |
| Consider HNSW, DiskANN, IVF, and SPIRE transferability without overclaiming cross-AM support. | Packet 001/006 transferability sections; packet 011 scopes TQ+ to IVF and names SPIRE/HNSW/DiskANN as follow-up measurement/mapping work. | Satisfied. |
| Use real benchmarks, not only synthetic probes. | Packet 008 SPIRE real10k/50k/100k baseline-vs-change; packet 011 IVF real10k/50k/100k TurboQuant-vs-TQ+. | Satisfied. |
| Use 10/50/100 spread. | Packet 008 and packet 011 both cover real10k, real50k, real100k. | Satisfied. |
| Include recall, latency, and storage. | Packet 008 `benchmark-delta.md`; packet 011 manifest tables include recall@10, p50/p95/p99 latency, and index B/row. | Satisfied. |
| Use `ecaz bench suite`, not ad hoc sweepers. | Packet 008 `suite-lutoff.json`/`suite-luton.json`; packet 011 `suite-baseline.json`/`suite-tqplus.json`. | Satisfied. |
| At least one candidate improvement is prototyped and measured or explicitly shelved. | SPIRE LUT measured and accepted in packet 008/010; IVF TQ+ measured in packet 011/012; byte-pair LUT and renorm-only paths shelved with reasons. | Satisfied. |
| Any accepted code change has packet-backed TurboQuant baseline evidence. | SPIRE LUT: packet 008. IVF TQ+: packet 011. | Satisfied. |
| Any rejected idea explains blocker. | Packet 006 and packet 010 list latency, quality, bytes, AM transferability, or implementation complexity blockers for byte LUT, renorm, blocked slabs, dense rotation, fused top-k, and DiskANN adapter work. | Satisfied. |
| No unrelated quantizer work included. | Measurements and reports stay TurboQuant/TQ+ scoped. Mentions of other storage formats in code are dispatch compatibility arms or error text, not comparison evidence. | Satisfied. |
| No new unsafe blocks. | Commit `d58ff8716670d721edc1b6ca90c9418ee9a23970` removes the two added `unsafe { ... }` blocks from `src/am/ec_ivf/insert.rs`; `artifacts/no-added-unsafe-blocks.log` records no added unsafe blocks remain at this packet's head. | Satisfied. |
| PG18-focused validation for code slices. | Packets 008, 011, 013, 014, 015; packet 016 adds `cargo check -p ecaz --lib --no-default-features --features pg18` after unsafe-block cleanup. | Satisfied. |
| Format-changing slice has an ADR or task-local format-version plan. | Packet 011 `artifacts/tqplus-format-plan.md` documents tag `4`, calibration chain, compatibility, insert/scan/vacuum semantics, and promotion requirements. | Satisfied. |

## Benchmark Summary

### SPIRE TurboQuant LUT

Packet 008 measures SPIRE `storage_format=turboquant` LUT-off vs LUT-on on real10k/50k/100k:

- recall unchanged at all nine nprobe cells;
- storage unchanged at all three fixture sizes;
- SQL mean and p50/p95/p99 pipeline latency improve at all nine cells.

### IVF TQ+

Packet 011 measures IVF `storage_format=turboquant` vs `storage_format=turboquant_tqplus`, rerank disabled:

- recall improves at all nine nprobe cells;
- p50/p95/p99 latency improves at all nine nprobe cells;
- hot posting bytes stay the same, with only small index B/row metadata deltas.

The current suite runner does not expose separate scorer-only and query-prep timing for these index runs. The task evidence therefore cites production scan-path latency and does not claim separate scorer/prep timing beyond what is available.

## Remaining External Workflow

The PR has no submitted reviews, no review comments, and no unresolved review threads at the time this packet was prepared. Packets 011, 012, 015, and 016 still need outside reviewer acceptance under the repository workflow before the task file should be marked complete rather than coder-complete.

## Scope Cleanup

Task 87/88 task-definition commits were inspired by the TurboVec research, but they are follow-up planning work rather than Task 86 deliverables. They were reverted from the Task 86 branch after this audit so PR scope remains Task 86 only.
