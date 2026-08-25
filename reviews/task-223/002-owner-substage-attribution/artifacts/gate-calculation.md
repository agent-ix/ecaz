# Task 223 100k direct-tuple entry-gate calculation

Date: 2026-08-25 (America/Los_Angeles)

## Accepted source result

The Task 222 reviewer accepted the exact payload mask as the production
default in
`reviews/task-222/005-main-integration/feedback/2026-08-25-01-reviewer.md`.
The matched 100k projected-candidate rows in
`reviews/task-222/004-full-scale-decision/artifacts/run/100k/distann-multinode-summary.log`
state:

```text
physical_benchmark_latency scale=100k variant=projected-candidate ... count=50 mean_ms=11.60 ... cache=warm ... extension_git_sha=c9f79be4a756031b3f8301960fc0f57b77ae60d1 extension_build_profile=release

physical_benchmark_stage scale=100k variant=projected-candidate ... stage=materialize_owner_endpoint_work scans=50 samples=100 elapsed_ns=40896428 elapsed_ms=40.896428 mean_ms=0.817929 ... extension_git_sha=c9f79be4a756031b3f8301960fc0f57b77ae60d1 extension_build_profile=release

physical_benchmark_stage scale=100k variant=projected-candidate ... stage=materialize_owner_payload_sql_work scans=50 samples=100 elapsed_ns=25749936 elapsed_ms=25.749936 mean_ms=0.514999 ... extension_git_sha=c9f79be4a756031b3f8301960fc0f57b77ae60d1 extension_build_profile=release
```

The result is one fresh three-owner physical PG18 release generation, 200
held-out queries, top-k 10, 50 timed warm iterations after 10 warmups, persisted
4,096-row head, BW4/H100, and production lazy-10. The suite step succeeded with
zero missing/stale artifacts. The control and projected candidate share the
same generation; Task 222's reviewer accepted byte-identical predictions,
recall, and storage.

## Registered Task 223 gate

Task 223 permits a direct row-tier tuple candidate only if its addressable
100k residual is at least one of:

- `1.000000 ms/scan`; or
- `5%` of warm end-to-end mean.

Calculation:

```text
addressable whole-bucket upper bound = 0.514999 ms/scan
warm end-to-end mean                 = 11.600000 ms/scan
5% threshold                        =  0.580000 ms/scan
whole-bucket ceiling                 =  4.439647%
shortfall versus 5% threshold        =  0.065001 ms/scan
shortfall versus 1 ms threshold      =  0.485001 ms/scan
```

The `materialize_owner_payload_sql_work` timer encloses every mechanism the
Task 223 candidate could replace: relation-name and SQL construction, TID
argument construction, SPI execution, heap access, binary send functions,
PostgreSQL array construction, Rust array decoding/flattening, and ordered
payload response assembly. It does not include retained-generation open/schema
validation or graph-to-row TID resolution, which a direct row-tier path must
still perform.

Therefore 0.514999 ms is a strict and deliberately optimistic upper bound: a
real direct path has positive tuple fetch, deformation/detoast, binary send,
and response assembly cost. Any requested substage is a subset of this whole
bucket and cannot exceed it. Both registered gates fail before a candidate or
new benchmark-only counter surface is built.

## Disposition

Coder recommendation: **STOP MAT-41 before instrumentation and P2**. Retain the
current Task 222 path. This is not a claim that SPI is free; it is a claim that
even removing the entire addressable bucket cannot meet Task 223's own minimum
materiality rule. Outside review must accept the dominance argument before the
task is marked complete.

