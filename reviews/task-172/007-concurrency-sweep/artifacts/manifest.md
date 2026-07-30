# Artifact manifest

- Task bucket / packet: `reviews/task-172/007-concurrency-sweep`
- Evidence type: runner capability and focused validation; no measurement
- Code head SHA: `203f285fd626590ff3ea4cb630a5f8d202504366`
- Branch: `task-203-ec-distann-conformance`
- Created: `2026-07-29` (America/Los_Angeles)
- Lane / fixture / storage format / rerank mode: not applicable; no benchmark
  lane was run
- Isolation surface: not applicable

## Files

| Artifact | Purpose |
| --- | --- |
| `validation.md` | Commands, outcomes, and lint context |

## Code under review

- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/suite.rs`

Key contract:

```text
--concurrency-sweep 1,2,4,8,16
concurrency_sweep: [1, 2, 4, 8, 16]
```

## Key result lines

```text
concurrency_sweep_overrides_single_value_and_preserves_order: PASS
expands_latency_with_cache_state_label: PASS
targeted rustfmt: PASS
git diff --check: PASS
targeted Clippy probe: BLOCKED by unrelated existing warnings
```

No corpus, cluster, query, ground-truth, benchmark, or suite output was
generated.
