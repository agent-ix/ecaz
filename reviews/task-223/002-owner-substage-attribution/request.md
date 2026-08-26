---
task: 223
packet: 002-owner-substage-attribution
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 223 whole-bucket ceiling and STOP request

This packet requests outside review of a Task 223 STOP before substage
instrumentation or direct-tuple implementation.

The plan was written before Task 222's production result existed. It required
feature-only substage counters, then authorized a direct row-tier tuple
candidate only if the addressable 100k residual reached either 1 ms/scan or 5%
of warm mean. Task 222 has since been review-closed and merged as the production
default, providing a stronger result than a substage estimate: its entire
`materialize_owner_payload_sql_work` bucket is 0.514999 ms/scan against an
11.60 ms warm mean.

The registered thresholds are therefore 1.000000 ms and 0.580000 ms. Even an
impossible zero-cost replacement of the whole payload-SQL bucket can save only
4.439647%. A real direct path must still fetch/deform tuples, call binary send
functions, and assemble the response, so its attainable saving is strictly
smaller.

The whole-bucket timer encloses the work Task 223 could replace: SQL/relation
name and TID preparation, SPI execution, heap fetch, detoast/send, PostgreSQL
array construction, Rust array decode/flatten, and response assembly. It
excludes generation open/schema validation and graph-to-row locator resolution,
which the candidate cannot eliminate. Thus every requested P1 substage is a
subset of a bucket that already fails both gates. Instrumenting those subsets
cannot reverse the decision and would add a benchmark-only surface with no
authorized consumer.

No code, production behavior, or benchmark configuration changes in this
packet. No new tests or benchmark run were needed: the calculation reuses the
accepted, suite-produced Task 222 100k result on the exact retained production
path. Semantic/result-identity gates for P2/P3 remain conditional and are not
claimed because no candidate is authorized.

Please rule specifically on whether the whole-bucket dominance proof satisfies
Task 223's attribution/decision acceptance criterion and permits retiring the
counter requirement as decision-obviated. Coder recommendation: **ACCEPT STOP,
retain the Task 222 production path, do not create packets 003/004**. Task 224
remains blocked until this outside verdict is recorded.

