# Task 98 Packet 002: Phase A Measurement — Instrumentation Finding

Measurement packet for the packet-001 kernels and routing (commits through
`96c6e3476`). Outcome is a **finding, not a clean matrix**: the full cell
grid ran (24 cells, 3 corpora × 2 modes × kernel-on/off), recall is
byte-identical between cells, the mode GUC demonstrably works — but zero
`surface=hnsw` block-kernel rows were recorded anywhere, meaning the
routed batch arm is not on the executed scan path even with the binary
prefilter disabled.

Two concrete findings for review (details in `artifacts/manifest.md`):

1. Default HNSW TurboQuant scans shadow the exact modes behind the binary
   prefilter entirely; Phase A cells must (and now do) disable it.
2. An open instrumentation gap remains between
   `live_loaded_state_from_exact_payload` / the `TurboQuantHotCold`
   loaded-state path and the widened batching arm. The next slice
   root-causes it with runtime tracing; the width-histogram scope-down
   decision is deferred until counters fire.

This matches the task's own Phase A contingency ("if counters still do not
fire on that surface, Phase A first resolves instrumentation rather than
treating Task 87's FullLut-oriented closeout as sufficient evidence") —
the same gap evidently affects the original FullLut batching on this
surface, which is itself a finding the Task 87 closeout record should
eventually absorb.

## Review request

Please review the two findings and the cell evidence. Next slice:
runtime-trace the loaded-state path on a TurboQuant fixture, fix the
batching arm placement, rerun this suite unchanged.
