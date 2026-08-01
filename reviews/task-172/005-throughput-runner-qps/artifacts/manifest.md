# Artifact manifest

- Task bucket / packet: `reviews/task-172/005-throughput-runner-qps`
- Evidence type: runner capability and local static validation; no measurement
- Code head SHA: `41e499327481f95fc3e631f73934a3eca36b81fa`
- Branch: `task-203-ec-distann-conformance`
- Created: `2026-07-29` (America/Los_Angeles)
- Lane / fixture / storage format / rerank mode: not applicable; no benchmark
  lane was run
- Isolation surface: not applicable

## Files

| Artifact | Purpose |
| --- | --- |
| `validation.md` | Commands, outcomes, and the exact external build blocker |

## Code under review

`crates/ecaz-cli/src/commands/bench/latency.rs`

The change records the earliest timed query start and latest timed query finish
for every worker, reduces those bounds across workers, and reports:

```text
concurrency, wall_ms, qps
```

`qps = completed timed queries / concurrent wall seconds`.

## Key result lines

```text
targeted rustfmt: PASS
git diff --cached --check: PASS
focused cargo test: BLOCKED before test execution (Task 205 compile error)
make lint: BLOCKED by the same Task 205 compile error
make fmt-check: repository-wide pre-existing drift; targeted file PASS
```

No corpus, cluster, query, ground-truth, benchmark, or suite output was
generated.
