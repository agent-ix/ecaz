---
task: 181
packet: 006-decision-correction
role: coder
status: open
date: 2026-07-15
head: 71690b6c4c06e4457c97a3254c6b6b53bea916d4
---

# Review request: correct Task 181 decision to GO

This packet corrects the decision in packet 005 without changing any measured
result. The earlier NO-GO treated the proposed NFR-017 values—0.9990 recall and
the 37.6 ms IVF anchor—as stakeholder-approved hard task gates. They were
planning assumptions authored by the agent and were not approved acceptance
criteria.

On the actual relative A/B, the selected bounded 4,096 training-landmark,
exact-scoring candidate is recall-flat at 10k, improves recall by 0.0140 at 50k
and 0.0350 at 100k, and reduces warm p50 by 6.8 ms and 2.4 ms at 50k and 100k.
It therefore advances to Task 182 for production-path implementation and A/B.

This is not approval to promote a new default. Task 182 must reproduce the
relative gains on the normal production path and account for correctness,
lifecycle, bounded work, latency, storage, and construction costs before its
own promote/iterate/abandon decision.

See `artifacts/manifest.md` for the unchanged source measurements and corrected
decision rationale.
