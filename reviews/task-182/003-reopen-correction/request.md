---
task: 182
packet: 003-reopen-correction
role: coder
status: open
date: 2026-07-15
head: 71690b6c4c06e4457c97a3254c6b6b53bea916d4
---

# Review request: reopen Task 182

Task 182 is restored to `proposed / unblocked`. Packet 002's `won't pursue`
disposition depended entirely on Task 181's superseded hard-gate decision.

The frozen input candidate is the bounded 4,096 training-landmark policy with
exact head scoring and 32 returned seeds measured in Task 181. Task 182 may now
translate that benchmark-only policy into a deterministic production contract,
implement it without benchmark-only dependencies, and run the required
10k/50k/100k production-path A/B.

This correction does not promote the candidate or change production behavior.
See `artifacts/manifest.md` for the entry evidence and remaining decision
boundary.
