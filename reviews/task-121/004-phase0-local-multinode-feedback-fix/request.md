# Review Request: Task 121 Phase 0 Local Multinode Feedback Fix

## Scope

This packet requests review for commit `af885a363abd8eff9f99f670c18047a1039eff3e`.

It addresses two concrete issues from `reviews/task-121/002-phase0-local-multinode-suite-lane/feedback/2026-06-22-01-reviewer.md`:

- the `spire-local-multinode` suite lane could not drive per-variant `ec_spire` matrix controls;
- `expected_artifacts` did not match the harness layout and were declared even when `skip_bench_suite=true`.

This packet does not claim full Phase 0 AC1 closeout. The command still routes through the existing local harness, so the broader "no bespoke harness" replacement remains open if the reviewer requires literal first-class orchestration inside `ecaz bench suite`.

## Code Changes

- Added `spire-local-multinode` SuiteConfig fields:
  - `storage_format`
  - `coord_index`
  - `remote_index`
  - `reloptions`
  - `coord_reloptions`
  - `remote_reloptions`
- Threaded those fields into the generated `ecaz dev spire-multicluster local-multinode-pg18` command.
- Added matching CLI flags on `local-multinode-pg18`.
- Passed the values to the existing local harness through:
  - `SPIRE_AWS_STORAGE_FORMAT`
  - `COORD_INDEX`
  - `REMOTE_INDEX`
  - `SPIRE_AWS_COORD_RELOPTIONS`
  - `SPIRE_AWS_REMOTE_RELOPTIONS`
- Fixed `expected_artifacts` for the actual local harness layout:
  - smoke log under the step artifact directory;
  - topology under `run_dir/topology.local.json` or the deterministic run-id target path;
  - nested `bench-suite/suite-manifest.json` and `bench-suite/results.jsonl` only when `skip_bench_suite=false`.
- Renamed the default smoke log from `phase13e-local-multinode.log` to `local-multinode.log`.
- Added focused suite tests for:
  - matrix flag expansion;
  - skipped bench artifact gating;
  - enabled bench artifact tracking;
  - semicolon rejection in reloptions.

## Evidence

See `artifacts/manifest.md`.

- `cargo test -p ecaz-cli commands::bench::suite`: 54 passed.
- `cargo build -p ecaz-cli --bin ecaz`: passed with the pre-existing `LoadedDistributedPlacementConfig::path` dead-code warning.
- Dry-run SuiteConfig demonstrates a local multi-node matrix cell emitting `--storage-format`, coord/remote index flags, shared reloptions, coord-only reloptions, and remote-only reloptions.
- Dry-run manifest confirms `skip_bench_suite=true` declares only `local-multinode.log` and `topology.local.json`, not nested bench artifacts.

## Residual Work

This slice deliberately does not replace the underlying local harness with native suite-runner orchestration. It removes the immediate blocker for driving RaBitQ/TurboQuant local multi-node matrix variants through a single SuiteConfig and fixes evidence tracking drift, but Phase 0 should remain open for the first-class orchestration question.
