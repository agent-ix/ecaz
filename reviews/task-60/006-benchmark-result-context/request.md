# Review Request: Benchmark Result Context Fields

## Scope

This checkpoint tightens the `ecaz bench suite` reporting path for Task 60's benchmark evidence.

The suite already expanded commands with `--storage-format` and `--cache-state`, but normalized result rows were parsed only from command output tables. That meant downstream `results.jsonl` rows did not reliably carry the required `storage_format`, `cache_state`, or host-parity context.

Changes under review:

- Enrich parsed suite result rows with stable context from the manifest and expanded step command:
  - `storage_format`
  - `cache_state`
  - `prefix`
  - `profile`
  - `suite_database`
  - `suite_host`
  - `suite_port`
  - `socket_dir`
- Preserve command-output values when a parsed table already includes the same field.
- Add a focused unit test covering a Task 60-style DiskANN RaBitQ recall row.

## Validation

Artifact: `reviews/task-60/006-benchmark-result-context/artifacts/cargo-test-ecaz-cli-bench-suite.log`

Command:

```sh
cargo test -p ecaz-cli commands::bench::suite::tests
```

Result: passed, 28 tests.

## Remaining Task 60 Gate

This does not replace the required full benchmark run. Task 60 still needs real 100k and 1M `pq_fastscan` vs `rabitq` recall, latency, and storage evidence on the benchmark host.
