---
task: 226
packet: 002-current-head-100k
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 226 current-head 100k screen

This packet now contains the completed fixed-4096 current-head BW4/H100 versus
BW8/H100 transfer screen. The production run satisfies preregistered ADVANCE
branch (b): paired recall improves by +0.016500 with 95% CI
`[+0.008000, +0.026500]`; mean latency improves 1.22%, and p95 regresses
4.21%, inside the 5% envelope. The disposition is to run the already
preregistered fresh 10k/50k confirmation matrix, not to change the default.

The production step runs `aa-control`, `aa-candidate`, `bw4-control`, and
`bw8-candidate` on one immutable generation. The A/A pair must be byte
identical; the exact BW4/BW8 names activate paired per-query recall. Only beam
width changes in the A/B. A separate fresh full-metrics fixture captures stage
and work attribution so instrumented latency is not the production decision
row.

The numerical ADVANCE/TRADE/STOP rule was recorded in the Task 226 file at
`d42d01e32`, before measurement. The checked-in SuiteConfig is
`artifacts/task226-current-head-bw8-100k.json`; audit and expanded-command
evidence also predate the run.

The first launch exposed a pre-measurement baseline defect after generation
publication: production retry code assumed the Task 167-only diagnostic table
`ec_distann_retry_attribution` existed. The already-landed upstream guard
`c9c9628eb` was cherry-picked unchanged as `c51e74c5e`; the failed fixture was
deleted and no test-only table workaround was introduced. See
`artifacts/pre-guard-failure.md`. No BW8 result is claimed from that launch.

The next fresh launch reached valid published topology and passed the serving
smoke, then PostgreSQL aborted the first benchmark-table ANN query on the
`subtrans.c:169` visibility assertion. Stack mapping identified an existing
snapshot-lifetime defect: traversal retained a raw refreshed-snapshot pointer
after its registration guard dropped. Existing upstream correction
`15f7fcf5f` was cherry-picked unchanged as `c85196ce8`; it keeps retry-refreshed
snapshots owned across traversal and does not replace the normal query
snapshot on an ordinary successful hop. See
`artifacts/pre-snapshot-guard-failure.md`. This launch also produced no arm
measurement or gate value, and its stopped 6.6 GB fixture was removed.

The successful release-profile execution head is `a1f158496`. Production A/A
predictions are byte-identical and recall is 0.9285 in both arms. Production
BW4 recall/mean/p95 is 0.9285/16.40/19.00 ms; BW8 is
0.9450/16.20/19.80 ms. Storage is arm-invariant and the published topology has
exactly 100,000 owned rows, zero non-owned rows, and zero orphans.

The separate full-metrics fixture independently reproduces the recall
direction (+0.018500, 95% CI `[+0.009500, +0.029000]`) and attributes the
candidate primarily to lower traversal transport wait (3.259058 to
2.795992 ms), partially offset by greater total scan work. Its instrumented
end-to-end latency is diagnostic and is not substituted for the production
gate. `artifacts/decision-summary.md` is the compact decision record;
normalized source rows remain in the two `results.jsonl` artifacts.

Review request: verify the preregistered branch-(b) arithmetic, same-generation
A/A/B provenance, separation of production and instrumented latency, and the
ADVANCE-to-confirmation disposition. The default remains unchanged pending
the full-scale packet and outside review.
