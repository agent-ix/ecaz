---
task: 179
packet: 068-transport-latency-isolation
role: coder
status: review-requested
head: 519177225a7e8a7ab4d8b85de5edc4a508477d2e
date: 2026-07-14
---

# Review request: exact packet-036 transport latency isolation

Please review the exact historical parent-versus-checkpoint A/B in
`comparison.md` as the response to packet 060's finding that packet 036 lacked
its own latency isolation.

The before extension is `9a0f21f0824c675d06e9e87747eb36a70859611f`;
the after extension is packet 036's
`ceb15f73ac69fcd98896457c9578fadae2ff0c09`. Both use the same current suite
runner and the same 10k/50k/100k recall, warm latency, storage, topology, and
engagement shape. Historical extensions are labeled neutrally as
`pre-provenance`; packet 067 requests review of that runner compatibility.

Requested decisions:

1. Is the historical commit isolation exact and its provenance sufficient?
2. Do all three scales demonstrate the checkpoint's measured recall, latency,
   and storage effect without inferring neutrality?
3. Do the final 3/3 manifests, audits, thresholds, compact source logs, and
   checksums form decision-grade evidence after raw operational logs are
   pruned?

Final measured result: recall is unchanged at all scales; storage is unchanged
apart from 32 KiB at 10k; warm p95 is 4.1-8.3% lower; and the broader
recall-workload mean is mixed (-21.3%, +5.5%, +14.7%). The packet makes no
speedup or statistical-neutrality claim from one sequential run per arm.
