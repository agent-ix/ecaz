---
task: 237
packet: 001-plan-taxonomy-and-counter-map
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 237 protocol errors and EXPLAIN observability plan

This packet requests review of Task 237 at planning checkpoint `dd3e37078`.

FR-081's Task-214 F8 gap records the missing `ExplainCustomScan` surface, while
FR-079 requires missing owned graph, vector, and row-tier payload state in a
Published generation to fail as distinct structural faults. The implementation
currently has mixed behavior, including a path that can silently drop a
missing remote payload and shorten the result.

Task 237 freezes typed, sanitized, retriable/non-retriable protocol categories;
removes silent missing-owned-data filtering; and adds bounded normal-release
text/JSON EXPLAIN counters for head, traversal, gateway, materialization,
pooling, retries, timeouts, cancels, and failures. It also extends `ecaz bench
suite` parsing/assertions so EXPLAIN and structured Task-228 metrics reconcile.

Please review the taxonomy, tombstone/exhaustion versus corruption boundary,
counter cardinality, secret/payload non-exposure, stable labels, and PG18
differential/fault coverage. Tasks 234 and 236 supply the cancellation and TLS
categories before the final vocabulary is implemented.

This is planning-only. No tests were run.


