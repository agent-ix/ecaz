# Review Request: Benchmark Operator Command Handoff

## Scope

This checkpoint updates the Task 60 benchmark manifest command blocks to use
the installed `ecaz` operator CLI for dry-run, full-run, and report extraction.

The suite matrix is unchanged. This does not run benchmarks and does not add
any comparison gate.

## Validation

Artifacts are under
`reviews/task-60/013-benchmark-operator-command-handoff/artifacts/`.

- `suite-audit.log`: `ecaz bench suite audit` passed with 24 steps. The local
  checkout invoked the CLI through `cargo run -p ecaz-cli` only because this
  sandbox does not have an installed `ecaz` binary.

## Remaining Task 60 Gate

The external benchmark host still needs to run the full 100k/1M Task 60 suite
and record recall, latency, storage, and the 1M shipping decision.
