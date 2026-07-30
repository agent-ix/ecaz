# Validation

Date: 2026-07-29 (America/Los_Angeles)

## Focused tests

Commands:

```text
cargo test -p ecaz-cli concurrency_sweep_overrides_single_value_and_preserves_order
cargo test -p ecaz-cli expands_latency_with_cache_state_label
```

Results:

```text
concurrency normalization/validation: 1 passed; 0 failed
suite concurrency-sweep expansion: 1 passed; 0 failed
```

The first test covers:

- the required ordered curve `1,2,4,8,16`;
- fallback to one `--concurrency` value;
- rejection of zero; and
- rejection of duplicates.

The suite test confirms JSON config expansion to:

```text
--concurrency-sweep 1,2,4,8,16
```

## Formatting and diff checks

Commands:

```text
rustfmt --check --edition 2021 crates/ecaz-cli/src/commands/bench/latency.rs crates/ecaz-cli/src/commands/bench/suite.rs
git diff --check
```

Result: **PASS**. Rustfmt emitted only stable-toolchain warnings for the
repository's nightly-only import configuration.

## Targeted Clippy probe

Command:

```text
cargo clippy -p ecaz-cli --bin ecaz --no-deps -- -A warnings -D clippy::manual-contains -D clippy::needless-range-loop -D clippy::unnecessary-sort-by -D clippy::unnecessary-lazy-evaluations
```

Result: **BLOCKED outside Task 172**.

The two fatal diagnostics are existing findings in:

```text
crates/ecaz-cli/src/commands/bench/build_probe.rs:319
crates/ecaz-cli/src/commands/dev/worktree.rs:120
```

No diagnostic points to either Task 172 file changed by this checkpoint. Packet
006 separately records the repository-wide `make lint` blocker in
`src/am/ec_ivf/quantizer.rs`.

## Benchmark policy

No benchmark was run. The physical fixture integration is intentionally absent
because the active handoff reserves that file to the Task 204/205 coder. The
final Task 172 matrix must use this suite surface after that integration lands.
