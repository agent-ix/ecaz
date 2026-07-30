# Artifact manifest

- Task bucket / packet: `reviews/task-172/006-metrics-mode-contract`
- Evidence type: suite capability and focused validation; no measurement
- Code head SHA: `854c6be176c0d4cd0dddac14b3ba035867c4c148`
- Primary code commit: `8942413d6`
- Validation follow-up: `854c6be17`
- Branch: `task-203-ec-distann-conformance`
- Created: `2026-07-29` (America/Los_Angeles)
- Lane / fixture / storage format / rerank mode: not applicable; no benchmark
  lane was run
- Isolation surface: not applicable

## Files

| Artifact | Purpose |
| --- | --- |
| `validation.md` | Commands, results, and the repository-wide lint blocker |

## Code under review

`crates/ecaz-cli/src/commands/bench/suite.rs`

Key contract:

```text
metrics_mode=benchmark
metrics_mode=full_metrics
```

The effective mode is recorded in the suite-manifest step tags and inserted
into every normalized result row.

## Key result lines

```text
distann_local_multinode_labels_and_expands_metrics_modes: PASS
distann_benchmark_metrics_mode_rejects_heavy_instrumentation: PASS
throughput_uses_concurrent_wall_time_not_summed_query_durations: PASS
targeted rustfmt: PASS
targeted Clippy unnecessary_lazy_evaluations: PASS
make lint: BLOCKED by unrelated ec_ivf/quantizer.rs manual_checked_ops
```

No corpus, cluster, query, ground-truth, benchmark, or suite output was
generated.
