# Validation

Date: 2026-07-29 (America/Los_Angeles)

## Targeted formatting

Command:

```text
rustfmt --check --edition 2021 crates/ecaz-cli/src/commands/bench/latency.rs
```

Result: **PASS**. Rustfmt emitted only warnings about unstable configuration
options on the stable toolchain.

The initial repository-wide `make fmt-check` exposed formatting drift in many
unrelated files. The three local suggestions it printed for `latency.rs` were
applied, after which the targeted command above passed. `make fmt` was not run
because it would rewrite unrelated and handoff-reserved files.

## Staged diff

Command:

```text
git diff --cached --check
```

Result: **PASS** before code commit
`41e499327481f95fc3e631f73934a3eca36b81fa`.

## Focused unit test

Command:

```text
cargo test -p ecaz-cli throughput_uses_concurrent_wall_time_not_summed_query_durations
```

Result: **BLOCKED before the focused test binary ran**.

The branch's in-flight Task 205 code does not currently compile. In the
handoff-reserved file `src/am/ec_distann/remote_endpoint.rs`,
`ec_distann_expand_nodes` accepts `candidate_limit` but does not pass it to
`expand_nodes_impl`; the implementation body then refers to a
`candidate_limit` not present in its signature. Rust reports:

```text
error[E0425]: cannot find value `candidate_limit` in this scope
```

It also reports the endpoint argument as unused.

## Lint

Command:

```text
make lint
```

Result: **BLOCKED by the same external compile error**, exit status 2 from
Make and status 101 from Cargo. The build failed before lint could validate the
Task 172 change.

The Task 205 files are explicitly reserved to another coder, so this packet
does not modify or repair them.

## Benchmark policy

No benchmark was run. The checkpoint adds one measurement primitive; the final
Task 172 matrix must wait for the prerequisite Task 204/205/206/208 work and
must be driven by `ecaz bench suite`.
