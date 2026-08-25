---
task: 226
packet: 004-main-integration
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 226 clean-main evidence integration and final review

This packet requests final outside review of Task 226 on branch
`integrate/task226-bw8-evidence`, based on exact current `main`
`de28655a42d254c2ac7f181569f07b92de5f3fae`.

This PR intentionally contains no production code or default change. Current
main already carries the stronger Task 167 retry-attribution and snapshot
lifetime fixes that were needed to run the original Task 226 branch, so their
older branch-local forms are not replayed. The PR lands only the canonical task
definition, task-scoped suite evidence, review requests, and narrow roadmap /
README status rows.

The measured disposition remains `USEFUL NON-DEFAULT CONFIGURATION`:

- 10k: recall 0.9990 / 0.9990; mean 14.80 -> 14.20 ms;
- 50k: recall 0.9540 -> 0.9690; mean 16.90 -> 16.80 ms;
- 100k: recall 0.9285 -> 0.9450; mean 16.40 -> 16.20 ms.

The registered gate passes at every scale, but p99 regresses 7.14% at 50k and
5.08% at 100k. Task 219 therefore continues to own the shipped BW4 default;
BW8 cannot become default without an explicit product-policy ruling. Please
review the same-generation provenance, gate arithmetic, tail caveat, and the
non-default disposition in packets 002 and 003.

No tests were rerun because this integration PR changes documentation and
immutable evidence only. No formatter was run.
