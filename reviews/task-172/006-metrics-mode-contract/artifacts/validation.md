# Validation

Date: 2026-07-29 (America/Los_Angeles)

## Focused tests

Commands:

```text
cargo test -p ecaz-cli distann_local_multinode_labels_and_expands_metrics_modes
cargo test -p ecaz-cli distann_benchmark_metrics_mode_rejects_heavy_instrumentation
cargo test -p ecaz-cli throughput_uses_concurrent_wall_time_not_summed_query_durations
```

Results:

```text
metrics-mode expansion/labeling: 1 passed; 0 failed
benchmark/heavy conflict: 1 passed; 0 failed
concurrent throughput calculation: 1 passed; 0 failed
```

The throughput test had been blocked in packet 005 by the in-flight Task 205
`candidate_limit` compile mismatch. Commit `615fd72b2` from the Task 204/205
coder repaired that reserved code; after integrating the shared branch, the
focused test ran and passed without a Task 172 change to the reserved files.

## Formatting and diff checks

Commands:

```text
rustfmt --check --edition 2021 crates/ecaz-cli/src/commands/bench/suite.rs
git diff --check
```

Result: **PASS**. Rustfmt emitted only stable-toolchain warnings for the
repository's nightly-only import configuration.

## Targeted Clippy

Command:

```text
cargo clippy -p ecaz-cli --bin ecaz --no-deps -- -A warnings -D clippy::unnecessary-lazy-evaluations
```

Result: **PASS**. This specifically rechecks the lint raised against the new
effective-mode helper while allowing unrelated existing warning classes.

## Repository lint

Command:

```text
make lint
```

Result: **BLOCKED outside Task 172**, Cargo status 101 / Make status 2.

The first fatal diagnostic is:

```text
error: manual checked division
  --> src/am/ec_ivf/quantizer.rs:695:34
  = note: -D clippy::manual-checked-ops implied by -D warnings
```

A broader `ecaz-cli --no-deps` Clippy probe also exposed numerous existing
warnings across unrelated benchmark, corpus, and dev modules. Its one finding
in the new helper (`unnecessary_lazy_evaluations`) was corrected in
`854c6be17`; the targeted command above then passed.

The unrelated lint backlog was not modified.

## Benchmark policy

No benchmark was run. The checkpoint defines and labels the two execution
modes. Task 172 still requires a suite-driven benchmark/full overhead A/B and
the final 10k/50k/100k physical matrix after its remaining prerequisites land.
