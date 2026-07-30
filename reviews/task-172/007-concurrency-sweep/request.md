---
task: 172
packet: 007-concurrency-sweep
role: coder
status: review-requested
head: 203f285fd626590ff3ea4cb630a5f8d202504366
date: 2026-07-29
---

# Review request: first-class latency concurrency sweep

## Requested decision

Please review commit `203f285fd626590ff3ea4cb630a5f8d202504366`,
which adds a first-class concurrency sweep to `ecaz bench latency` and exposes
it through `ecaz bench suite`.

This is a runner-capability checkpoint. It does not run the physical matrix or
claim Task 172 complete.

## Scope

`ecaz bench latency` accepts:

```text
--concurrency-sweep 1,2,4,8,16
```

or repeated `--concurrency-sweep` flags. When supplied, the sweep overrides the
single `--concurrency` value and runs every tuning-value × concurrency point in
the declared order.

Each point reports:

- count, mean, standard deviation, min, p50, p95, p99, and max;
- configured concurrency;
- concurrent observation-window duration;
- completed-query QPS;
- optional backend RSS/HWM samples; and
- optional IVF/DistANN stage and materialization-work attribution.

Zero and duplicate concurrency levels fail before a database connection is
opened.

## Attribution labeling

For a concurrency sweep, counter and memory-series labels append
`concurrency=<n>` to the tuning label. The existing space-key parser therefore
normalizes concurrency as its own field alongside the tuning axis.

The historical single-`--concurrency` path preserves its previous counter label
shape.

## Suite contract

Latency steps accept:

```json
{"concurrency_sweep": [1, 2, 4, 8, 16]}
```

The suite validates nonzero unique levels and expands the field to the CLI
flag. This keeps the required Task 172 curve inside `ecaz bench suite`, without
packet-local sweep scripts.

## Remaining physical integration

The physical DistANN fixture still invokes its latency child with concurrency
one and parses one table row. Wiring this capability into that fixture remains
open. The active handoff reserves
`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs` to the Task 204/205
coder, so this checkpoint does not modify it.

## Validation

See `artifacts/validation.md` and `artifacts/manifest.md`.

- Targeted formatting and diff checks pass.
- The concurrency normalization/validation test passes.
- The suite expansion test passes with the required `1,2,4,8,16` curve.
- A targeted Clippy probe reached unrelated existing warnings in
  `build_probe.rs` and `dev/worktree.rs`; neither file was modified.

No benchmark or cluster was run.

## Reviewer focus

1. Confirm that `iterations` means total completed queries per
   tuning/concurrency point rather than per worker.
2. Confirm the declared-order execution and fail-closed duplicate handling.
3. Confirm attribution and memory labels remain mechanically separable by
   concurrency in normalized result rows.
