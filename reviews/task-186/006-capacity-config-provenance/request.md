---
task: 186
packet: 006-capacity-config-provenance
role: coder
status: open
date: 2026-07-27
head: 34da0e6be
---

# Review request: preserve Task 186 capacity-config provenance

This documentation-only follow-up addresses the second-pass provenance
finding on packet 001. The original two-arm config used for the 4,096 and
8,192 runs was edited in place before the conditional 16,384 run and was not
previously retained. It is now committed separately as
`../001-capacity-control/artifacts/task186-capacity-control-100k-suite.pre16384.json`.

The packet-001 manifest maps the original config SHA to
`run-benchmark-feature/` and the amended three-arm config SHA to
`run-cap16384/`. No benchmark result or conclusion changed; this restores the
config-to-run provenance chain without claiming that the amended config
reproduces the earlier two-arm run.

See `artifacts/manifest.md` for the exact hashes and mapping.
