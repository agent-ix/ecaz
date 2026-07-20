---
task: 182
packet: 005-query-and-lifecycle
role: coder
status: open
date: 2026-07-16
head: d9411c692
---

# Review request: production trained-head suite path

Commit `d9411c692` makes the Task 182 production policy directly addressable by
the standard `ecaz bench suite` DistANN step. It deliberately uses a new
`production_head_policy` field rather than overloading Task 181's
benchmark-only `head_policy` field.

## Operator path

`production_head_policy` accepts `current_sample_graph` or
`training_landmarks_exact`. The trained policy requires cap 4,096 and a query
TSV. The local multinode driver loads file rows 201–400, in file order, into a
temporary relation with exactly `training_ordinal bigint` and `vector real[]`,
calls `ec_distann_build_epoch_with_training`, then drops the input relation.
The production generation remains bound only to its canonical input digest and
persisted selected-head digest.

The benchmark-only and production policy fields are mutually exclusive.
Training input is required exactly for the two training policies, so a typo or
missing field cannot silently run the current builder.

## Query and attestation

Production A/B arms use the normal `persisted_head` scan mode with no benchmark
seed GUC. The active manifest selects exact landmark scoring. Before measuring,
the driver calls `ec_distann_active_head_policy` and requires the returned
policy to equal the requested production policy. The suite result now includes
a `physical_benchmark_head_policy` row with scoring mode, training count and
digest, cap, returned-seed bound, sample count, and sample digest.

Task 181 coverage/oracle diagnostics remain available only through the
benchmark field/feature and are not a fallback for a production arm.

## Validation

- `cargo check -p ecaz-cli`: pass (one pre-existing dead-field warning in
  corpus load).
- focused suite expansion test: pass; the new JSON field expands to
  `--production-head-policy training_landmarks_exact` and the training path,
  without `--head-policy`.
- focused results parser test: pass; manifest-backed head-policy attestation is
  retained as a structured result row.

The packet-004 PG18 lifecycle test remains the database-level proof for
build/replay/publish/determinism. No benchmark result is claimed here; packet
006 owns the production 10k/50k/100k measurements.
