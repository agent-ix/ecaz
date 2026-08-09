# Task 219 packet 002 — default-policy decision

Decision: retain the shipped BW4/H100/L32 interactive default and retain
recall-equivalence for future shipped-default changes. Do not promote the
BW64/H8/L64-effective candidate.

The candidate is a measured higher-recall/higher-latency trade, not a Pareto
improvement: mean latency is +20.2% / +39.4% / +47.7% at 10k / 50k / 100k,
while storage is effectively unchanged. The interactive operating regime has
no accepted budget for that trade. A recall-sensitive operating point would
need an explicit future product/productionization task.

Please review `artifacts/decision.md`, with the paired frontier in
`../001-frontier-assembly/artifacts/frontier.md` and the source
`reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2/results.jsonl`.
