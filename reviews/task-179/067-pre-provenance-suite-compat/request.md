---
task: 179
packet: 067-pre-provenance-suite-compat
role: coder
status: review-requested
head: f51d512cb
date: 2026-07-14
---

# Review request: pre-provenance suite compatibility

Please review `f51d512cb` as the narrow runner prerequisite for packet 036's
historical latency isolation.

Current `distann-local-multinode` physical benchmarks query
`ec_distann_physical_seed_strategy()` solely to label result provenance. That
SQL helper was introduced after the packet 036 commits, even though both
historical commits already used persisted head-index seeding. The runner now
probes for the helper with `to_regprocedure`; current extensions still emit
their self-attested strategy, while older extensions emit the deliberately
neutral label `pre-provenance` instead of failing or assigning an inferred
strategy.

Requested decision: does this preserve current behavior while making exact
historical extension commits suite-runnable without concealing provenance?

Validation: `cargo check -p ecaz-cli` passes with one pre-existing dead-code
warning in corpus loading. Stable repository-wide rustfmt reports unrelated
pre-existing formatting differences; rustfmt reported no diff in the touched
block.

