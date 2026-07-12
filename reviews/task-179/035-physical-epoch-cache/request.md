---
task: 179
packet: 035-physical-epoch-cache
role: coder
status: review-requested
head: 83a40f8fdaaf77efe2aa5ce6d735bf34292098bf
date: 2026-07-12
---

# Review request: validated physical epoch cache and 10k/50k/100k A/B

Please review commits `d4d38a865` and `83a40f8fd`, the suite evidence under
`artifacts/`, and the scoped decision in `verdict.md`.

The requested decisions are:

1. Is the two-entry backend-local cache correctly keyed and restricted to
   already validated immutable epoch-head state?
2. Does the cache preserve lifecycle safety by excluding conninfo, relation
   handles, active-pointer state, and scan tokens?
3. Does the isolated 10k/50k/100k cache-off/cache-on suite establish a
   recall-neutral, storage-neutral, material latency improvement?
4. Does commit `83a40f8fd` correctly remediate the real fixture's Param-versus-
   Const plan-proof bug without weakening its input validation?

Key results:

- all six suite steps succeeded and all 18 thresholds passed;
- physical recall is identical off/on: 1.0000, 0.9800, and 0.9500;
- physical p50 falls 99.38%, 99.16%, and 99.54% at 10k/50k/100k;
- mean and p95 fall by more than 94% at every scale, including one cold sample;
- storage deltas are -16 KiB, +8 KiB, and -16 KiB;
- every arm has exact Ready/Published row coverage, zero residue/orphans, and
  two remote-owner CustomScan plus exact-row probes; and
- the initial Param-based proof failure is preserved and the exact-SHA rerun
  passes after the constant-query correction.

This request is intentionally scoped to the cache and its real-fixture proof.
Task 179 remains open for the outstanding transport, fault-window, sensitivity,
and outside-review work listed in `verdict.md`.
