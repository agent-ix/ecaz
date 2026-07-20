# Review request: reject distributed system columns

## Scope

Please review implementation commit `754eb7b91` as the remediation for packet
029 P2-3 and FR-078's `EC_UNSUPPORTED_PROJECTION` contract.

PostgreSQL represents `ctid`, `xmin`, `cmin`, `xmax`, `cmax`, and `tableoid`
with negative attribute numbers. Remote physical rows have no meaningful
coordinator-local identity for those fields, but the distributed CustomScan
previously reconstructed only positive user attributes and allowed projected
or qualified system columns to evaluate against NULL.

This checkpoint:

- walks the original planner query target list and jointree as soon as a
  physical/multinode DistANN candidate index is identified;
- rejects relation-local, level-zero negative-attnum Vars with
  `EC_UNSUPPORTED_PROJECTION` before path selection;
- retains a second check over the chosen CustomScan target list and extracted
  base quals as a plan-construction backstop;
- recursively covers system Vars nested inside expressions while ignoring
  Vars belonging to outer or other relations; and
- adds live `ctid` projection and `xmin` qual regressions to the existing
  three-owner Published-generation fixture.

The earlier hook is necessary because a system-column target can make
PostgreSQL choose another path before `PlanCustomPath`; rejecting only the
selected custom plan would leave that query shape uncovered.

## Validation

See `artifacts/manifest.md`. The full three-owner PG18 handoff/publication/read
fixture passes with both new planner errors, and production-feature PG18 clippy
passes with warnings denied.

This safety rejection does not change supported scan, quantizer, rerank,
posting, or storage behavior, so it does not require a new performance matrix.
Packet 045 remains the isolated 10k/50k/100k evidence for the supported
physical fanout path.

## Requested decision

Please confirm the two planner checkpoints fail closed for distributed
system-column projections and quals, closing packet 029 P2-3 without changing
supported user-column semantics.
