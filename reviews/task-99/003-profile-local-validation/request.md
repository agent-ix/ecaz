# Review request — Task 99 packet 003: profile local validation run

- Task: 99, item 9 (local validation gate before the AWS lanes)
- Coder: Task 102/103 author lane
- Date: 2026-06-12
- Backend: release, sha `6ec46732…` (suite preflight, `suite-manifest.json`)

## Summary

The full packet-002 profile executed on the local Intel desktop:
**91/91 steps green** (85 main + 6 retagged no-kernel baselines),
**34/34 recall on/off pairs byte-equal**, `scalar_candidates=0` on
every kernel row, and per-family kernel rates reproducing the
per-family closeouts on one shared fixture set (table in
`artifacts/manifest.md`). One flagged cell (diskann binary batch-on)
was contamination; the isolated recheck is clean and supersedes it.

## Findings the reviewer should weigh in on

1. **DiskANN grouped-PQ prefilter arm is ungated** (kernel rows in
   "off" cells; deltas ~0 by construction). Recorded as an ADR-077 §4
   nuance — confirm that's sufficient, vs. wanting the arm gated for a
   real A/B before the trip.
2. **SPIRE×rabitq has no batch counter attribution on Intel either**
   (M5 finding reproduced) — stays "e2e only" in the matrix.
3. **kernel_status tags are skip directives**: runnable no-kernel
   baselines retagged to plain `no_kernel_*` tags (generator comment
   documents the convention). Sanity-check the convention.
4. **First negative-net batch cell**: IVF QJL @1024 at nprobe 8/16
   (+8.3%/+3.0%) — carried into packet 007's decoupling map and the
   ADR-077 §4 IVF default decision.

## Artifacts

`manifest.md` (source of truth), `suite-manifest*.json`,
`results*.jsonl`, `suite-run*.log`, `suite-status.log`, per-cell
recall/latency/storage/load logs, fixture SQL logs, truth caches.

## Gate

With this packet green, the profile config is validated for the AWS
lanes. Next: packet 004 (`ecaz.isa_cap`) validation evidence, branch →
main, then the packet 006 runbook executes.
