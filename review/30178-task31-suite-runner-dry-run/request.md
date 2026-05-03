# Task 31 Suite Runner Dry-Run Slice

Reviewer: please review the first implementation slice for ADR-050.

## Scope

This adds a narrow `ecaz bench suite` surface that parses a JSON suite config,
expands configured steps to ordinary `ecaz` commands, and writes a dry-run
manifest. It does not execute suite steps yet.

## Code Changes

- Added `crates/ecaz-cli/src/commands/bench/suite.rs`
- Registered `bench suite` in `crates/ecaz-cli/src/commands/bench/mod.rs`
- Added sample config `crates/ecaz-cli/suites/task31-m5-ivf-100k.json`

## Supported Step Kinds

- `load`
- `recall`
- `latency`
- `storage`
- `explain`
- `raw`

The implementation currently requires `--dry-run`; running without it fails
with an explicit "suite execution is not implemented yet" error.

## Dry-Run Artifact

The sample Task 31 suite was expanded with:

```text
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --manifest-output review/30178-task31-suite-runner-dry-run/artifacts/suite-manifest.json
```

Artifact:

- `artifacts/suite-manifest.json`

The manifest records the suite name, schema version, config SHA256, dry-run
status, redacted connection target, and every expanded step command.

## Validation

Ran:

```text
cargo fmt --package ecaz-cli
cargo test -p ecaz-cli suite
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --manifest-output review/30178-task31-suite-runner-dry-run/artifacts/suite-manifest.json
```

`cargo test -p ecaz-cli suite` passed 4 suite unit tests. The build emitted
existing PG18 server-header warnings from `csrc/pg18_pgstat_shim.c`; no suite
warnings remained.

## Deferred

- Executing suite steps
- Step tags and `--only-tag`
- Resume support
- SQL-file generation for `explain` steps
- Normalized `results.jsonl`
- Markdown summary report
