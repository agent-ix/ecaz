---
task: 172
packet: 005-throughput-runner-qps
role: coder
status: review-requested
head: 41e499327481f95fc3e631f73934a3eca36b81fa
date: 2026-07-29
---

# Review request: concurrent latency throughput

## Requested decision

Please review commit `41e499327481f95fc3e631f73934a3eca36b81fa`,
which makes the existing concurrent latency runner report the real observation
window and completed-query throughput needed by Task 172.

This is a runner-capability checkpoint. It does not claim Task 172 complete and
does not run or promote the physical benchmark matrix.

## Scope

`ecaz bench latency` now appends three stable fields to each result row:

- `concurrency`, recording the configured worker count;
- `wall_ms`, spanning the earliest timed query start through the latest timed
  query completion across all workers; and
- `qps`, calculated as all completed timed queries divided by that concurrent
  wall interval.

The wall interval deliberately excludes initial connection setup and the
untimed warmup phase. If `worker_batch_size` causes timed workers to reconnect,
the reconnect and its replacement warmup remain inside the interval after the
first timed query starts. This reflects the cost of the configured execution
mode instead of summing per-query latencies, which would overstate elapsed time
under concurrency.

The fields are appended after the pre-existing columns so existing consumers'
column positions remain stable.

## Task 172 coverage

This checkpoint supplies the missing QPS and concurrent observation-window
measurements for the required concurrency sweep `1, 2, 4, 8, 16`. Existing
latency output already reports p50, p95, p99, maximum latency, backend memory,
and optional distributed-stage counters.

The physical multicluster fixture currently invokes the child latency command
with concurrency one and does not yet propagate these appended fields into its
physical result rows. That integration remains open and is intentionally not
included here because
`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs` is reserved to the
Task 204/205 coder under the active handoff.

## Validation

See `artifacts/validation.md` and `artifacts/manifest.md`.

- Targeted Rust formatting check: pass.
- Staged diff whitespace check: pass.
- Focused unit test: blocked before the test binary ran by the in-flight Task
  205 compile error in reserved `remote_endpoint.rs`.
- Repository lint: blocked by the same compile error.
- Repository-wide format check: fails on pre-existing formatting drift across
  unrelated files; the targeted file itself passes `rustfmt --check`.

No benchmark was run for this capability-only checkpoint.

## Reviewer focus

1. Confirm that earliest timed start to latest timed completion is the correct
   denominator for concurrent completed-query throughput.
2. Confirm that excluding only initial connect/warmup, while retaining any
   reconnect/warmup inside the timed run, matches the intended operational
   semantics.
3. Confirm that appending the columns preserves the existing positional output
   contract.
