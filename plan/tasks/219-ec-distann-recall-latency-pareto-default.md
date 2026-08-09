# Task 219: ec_distann Recall/Latency Pareto Default Decision

Status: **implementation complete; review-open** (2026-08-09, packet
`reviews/task-219/002-decision/`). The measured decision retains the shipped
BW4/H100/L32 default; outside reviewer disposition is pending. Priority: P1
default policy.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.
Origin: Task 215 carry-in (reviewer seq-01/seq-02 Pareto observation).

This is a **new task**, not a reopening of Task 215. Task 215 is review-closed
STOP and its verdict stands *under its own contract*. This task asks the
question that contract could not: whether the contract itself is the right
default policy.

## Why

Task 215's release A/B is the authoritative measurement of the shipped
`top_k=10` default. Its two arms at 100k:

| Arm | Recall | Mean latency |
| --- | ---: | ---: |
| BW4/H100/L32 (shipped) | 0.9280 | 21.40 ms |
| BW64/H8, 128 seeds | 0.9815 | 31.60 ms |

The candidate was rejected because Task 215's contract required **recall
equivalence**, and recall was not equivalent — it *rose* by 0.0535. The
rejection was arithmetically correct and the reviewer accepted it on those
terms.

But that means the program is currently declining the single largest measured
recall improvement available to it, on a policy clause rather than on evidence
that the trade is bad. No task owns re-examining that clause, and three
independent facts make it worth examining now:

1. **The recall lanes are exhausted.** Task 207 closed head construction (union
   improves membership +5.3 pts and does not move recall). Task 185 closed head
   selection (set-cover chose a Jaccard-1.0 identical member set; diversity arms
   tied recall at ~3x latency). Task 188's BW8 result was a search-budget win.
   Search budget is the only lever measured to move end-to-end recall.
2. **`NFR-017`'s 0.999 / 37.6 ms figures are aspirational, not gates** (recorded
   in the roadmap). At 31.60 ms the candidate is inside the aspirational latency
   reference while materially closer to the aspirational recall.
3. **The same-graph owner oracle reaches 0.9970.** The shipped 0.9280 leaves
   ~0.07 recall unclaimed; BW64/H8 claims most of the first half of it.

## Goal

A recorded, reviewed **default-policy decision**: does ec_distann ship the
lower-latency/lower-recall point, the higher-recall/higher-latency point, or a
third point on the measured frontier — and is "recall equivalence" the right
acceptance clause for future default changes.

This task decides policy from **existing measured evidence** wherever possible.
It is not a licence to re-run the 206/215 matrices.

## Scope

- Assemble the measured frontier at 10k/50k/100k from the accepted Task 215
  release rows, annotated with the Task 206 work-surface caveat (`top_k=200`/L200
  there versus `top_k=10`/effective L64 in 215 — 206 rows are **not** a release
  forecast and must not be plotted as one).
- Fill **at most** the intermediate points needed to make the frontier a
  decision surface rather than two dots. Any new arm runs on the normal release
  build under the Task 215 protocol, with matching query SHA-256s.
- State the operating regime the default serves, since that is what the answer
  turns on: interactive p95 budget versus recall-sensitive retrieval.
- Produce an ADR if the default changes, and a separate productionization task
  if a new point is selected — a benchmark winner is not a release decision.

## Non-goals

- Re-litigating Task 215's verdict under its own contract. It stands.
- Any code change to traversal, head, or materialization.
- Adding a new candidate mechanism. This task chooses among measured points.

## Acceptance

1. The frontier table exists with every row traced to a committed
   `results.jsonl`, at 10k/50k/100k, on release builds, with provenance.
2. An explicit recommendation with the regime assumption stated.
3. A recorded decision on whether recall-equivalence remains the acceptance
   clause for default changes, or is replaced by a stated Pareto rule.
4. If the default changes: an ADR plus a numbered productionization task. If it
   does not: the reason recorded so the question is not re-asked by default.

## Required review packets

1. `reviews/task-219/001-frontier-assembly/`
2. `reviews/task-219/002-decision/`

## References

- `reviews/task-215/003-release-matrix-and-decision/` and its two feedback files
- `reviews/task-215/003-release-matrix-and-decision/artifacts/reconciliation-206.md`
- `reviews/task-207/007-membership-diagnostic/`, `reviews/task-185/003-fixed-cap-screen/`
- `spec/non-functional/NFR-017-*`
