# Artifact manifest

- Head SHA: `efb0aa8cb4c5d9bb6f61b88b03baa671a4f9d10c`
- Implementation commit: `efb0aa8cb` (`feat(cli): expose DistANN head cap in suites`)
- Task bucket / packet: `reviews/task-179/037-head-cap-suite-surface`
- Lane: local CLI compile, focused suite tests, and SuiteConfig dry-run
- Fixture: dry-run only; no PostgreSQL instance or corpus was loaded
- Storage format / rerank mode: not applicable
- Timestamp: `2026-07-12T13:28:00-07:00`

## Commands and results

### CLI check

```text
cargo check -p ecaz-cli
```

Result: exit 0. One pre-existing dead-code warning for
`LoadedDistributedPlacementConfig.path` remains. See `cargo-check-cli.log`.

### Focused suite tests

```text
cargo test -p ecaz-cli distann_local_multinode -- --nocapture
```

Result: exit 0; 2 passed, 0 failed. The tests prove expansion of cap 256 and
rejection below the frozen minimum. See `focused-suite-tests.log`.

### Actual CLI binary build

```text
cargo build -p ecaz-cli
```

Result: exit 0. See `cargo-build-cli.log`.

### Audited suite dry-run

```text
target/debug/ecaz bench suite run \
  --config reviews/task-179/037-head-cap-suite-surface/artifacts/head-cap-dry-run.json \
  --dry-run \
  --log-file reviews/task-179/037-head-cap-suite-surface/artifacts/suite-dry-run.log
```

Result: exit 0. The emitted command contains:

```text
--physical-benchmark --benchmark-iterations 20 --graph-degree 32 \
--head-index-cap 256 --corpus-prefix ec_real_10k
```

The dry-run used neither shared-table nor isolated one-index-per-table
measurement surfaces. `dry-run/suite-manifest.json` is the structured command
record.

## Lint context

A broad warnings-denied CLI clippy command is not a clean repository gate in
this checkout: it reports existing lints in `ecaz-cloud` and unrelated CLI
modules (25 errors in the no-deps binary attempt). No clippy output is cited as
passing evidence. The exact-SHA compile and focused tests above cover the
touched runner paths.

## Artifact index

- `head-cap-dry-run.json`: checked-in one-step SuiteConfig.
- `dry-run/suite-manifest.json`: structured dry-run command and status.
- `suite-dry-run.log`: rendered dry-run transcript.
- `cargo-check-cli.log`: exact-SHA compile check.
- `cargo-build-cli.log`: actual CLI executable build.
- `focused-suite-tests.log`: range-validation and expansion tests.
