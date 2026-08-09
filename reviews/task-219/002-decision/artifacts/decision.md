# Task 219 default-policy decision

## Decision

Retain BW4/H100/L32 as the shipped `top_k=10` interactive default, and retain
recall-equivalence as the default-change acceptance clause. Do not promote the
BW64/H8/L64-effective point from Task 215.

## Rationale

The candidate improves recall by 0.0005 / 0.0355 / 0.0535 at 10k / 50k /
100k, but increases mean latency by 20.2% / 39.4% / 47.7%. Storage is
effectively unchanged. It therefore does not dominate the shipped point; it is
a higher-recall/higher-latency trade.

The default serves interactive retrieval with a bounded latency budget. No
accepted product requirement or task evidence authorizes spending 3.8 / 8.2 /
10.2 ms of mean latency for the candidate's recall increase. Recall-sensitive
retrieval remains a valid future operating regime, but selecting it requires
an explicit product decision and a separately scoped productionization task.

## Contract decision

Recall-equivalence remains the acceptance clause for changing the shipped
default. A candidate that changes recall must be evaluated as a deliberate
Pareto/product-policy decision rather than being promoted as a latency
optimization. This records why the question should not be reopened merely
from the existing BW64/H8 measurements.

## Evidence

The frontier is in `../001-frontier-assembly/artifacts/frontier.md`; every row
traces to the accepted Task 215 `artifacts/run-r2/results.jsonl`. The source
Task 215 decision and both reviewer feedback files record the same STOP and
restore BW4/H100.
