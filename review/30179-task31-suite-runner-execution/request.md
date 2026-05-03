# Task 31 Suite Runner Execution Slice

Reviewer: please review the first executable implementation slice for ADR-050.

## Scope

This builds on `review/30178-task31-suite-runner-dry-run/` and turns the suite
prototype into a first usable runner:

- `ecaz bench suite run --config <path>` executes selected steps sequentially.
- `run --dry-run` preserves the dry-run manifest flow.
- `ecaz bench suite audit --config <path>` validates suite shape and required
  local load inputs.
- `ecaz bench suite status --manifest <path>` summarizes manifest state.
- `ecaz bench suite report --manifest <path>` emits a minimal markdown report.
- The legacy `ecaz bench suite --config <path> --dry-run` form still works as a
  compatibility alias.

The implementation records per-step selection, command, expected artifacts,
status, timestamps, duration, and exit code in `suite-manifest.json`. Child
commands inherit connection flags; `PGPASSWORD` is passed via the child process
environment rather than being written into the manifest command.

## Spec and Docs

- Added `US-017: Run Configured Benchmark Suites`.
- Added `FR-038: Configured Benchmark Suite Runner`.
- Updated `FR-037`, `NFR-007`, `NFR-009`, `spec/spec.md`, and `spec/tests.md`.
- Updated `crates/ecaz-cli/README.md` with suite schema, commands, tuning usage,
  and RDS/Graviton guidance.

## Smoke Artifacts

Artifacts are under `artifacts/`:

- `audit.log`
- `dry_run.log`
- `status.log`
- `report.log`
- `suite-manifest.json`
- `legacy-suite-manifest.json`
- `manifest.md`

The smoke commands used `cargo run -p ecaz-cli -- ...` so they validated the
new code before install. They did not execute the actual benchmark suite steps.

## Validation

Ran:

```text
cargo fmt --package ecaz-cli
cargo test -p ecaz-cli suite
cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/audit.log bench suite audit --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json
cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/dry_run.log --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --manifest-output review/30179-task31-suite-runner-execution/artifacts/suite-manifest.json
cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/status.log bench suite status --manifest review/30179-task31-suite-runner-execution/artifacts/suite-manifest.json
cargo run -p ecaz-cli -- --log-file review/30179-task31-suite-runner-execution/artifacts/report.log bench suite report --manifest review/30179-task31-suite-runner-execution/artifacts/suite-manifest.json
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only storage-real100k-n128 --manifest-output review/30179-task31-suite-runner-execution/artifacts/legacy-suite-manifest.json
```

`cargo test -p ecaz-cli suite` passed 8 tests. The build emitted the existing
PG18 server-header warnings from `csrc/pg18_pgstat_shim.c`; no suite-specific
warnings were introduced.

## Deferred

- Normalized `results.jsonl` metric extraction.
- Rich markdown result tables from recall/latency/storage logs.
- Tags and `--only-tag`.
- Resume-from-manifest execution.
