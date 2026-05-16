---
id: StR-006
title: Benchmark Evidence Discipline
type: stakeholder-requirement
artifact_type: StR
status: APPROVED
relationships:
  - target: "ix://agent-ix/tqvector/NFR-007"
    type: "derives"
    cardinality: "1:1"
  - target: "ix://agent-ix/tqvector/NFR-008"
    type: "derives"
    cardinality: "1:1"
  - target: "ix://agent-ix/tqvector/NFR-015"
    type: "derives"
    cardinality: "1:1"
---
# StR-006: Benchmark Evidence Discipline

## Need

Ecaz performance work now spans local desktop sweeps, review packets, and future AWS/RDS-class runs. Users and reviewers need benchmark claims to be reproducible, consistently shaped, and clearly scoped.

## Expectation

Any recall, latency, storage, memory, build-time, ingest, vacuum, or distributed transport claim SHALL state whether it is local development evidence, review-packet evidence, or a product benchmark claim. Measurement claims SHALL cite packet-local raw artifacts when they are used to justify a landed task or README/spec claim. Candidate comparisons SHALL use the shared benchmark reporting schema so quantizers, storage formats, AMs, and option sets can be compared without changing table semantics.

## Success Criteria

1. `docs/benchmarks.md` separates local results from product benchmark claims.
2. Review packets that cite measurements store raw logs under the packet `artifacts/` directory.
3. `spec/tests.md` traces benchmark requirements to concrete evidence or explicitly marks the gap.
4. `docs/benchmark-reporting-standard.md` defines the shared fields for AM, quantizer, storage-format, and option-set comparisons.
