# Task 31 Suite Filtered Thresholds and Strict Resume

Reviewer: please review the follow-up suite runner automation slice.

## Scope

This tightens the auto-mode behavior added in the previous packets:

- Thresholds now support exact-match `filters`, so a multi-row sweep can target
  a specific row such as `nprobe=96`.
- `--resume-from` is now strict: it rejects prior manifests when the config hash
  differs or a succeeded step's expanded command differs from the current run.
- Threshold results preserve the configured filters in the manifest.
- The Task 31 sample suite thresholds now filter the quality candidate rows at
  `nprobe=96`.

## Validation

Ran:

```text
cargo fmt --package ecaz-cli
cargo test -p ecaz-cli suite
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only-tag candidate --manifest-output review/30182-task31-suite-filtered-thresholds-resume/artifacts/candidate_filter_dry_run_manifest.json
```

`cargo test -p ecaz-cli suite` passed 13 tests. The build emitted the existing
PG18 server-header warnings from `csrc/pg18_pgstat_shim.c`; no suite-specific
warnings were introduced.

## Artifacts

- `artifacts/candidate_filter_dry_run_manifest.json`
- `artifacts/manifest.md`

No benchmark steps were executed for this packet.

## Deferred

- Dedicated typed numeric result columns for downstream plotting.
- Boolean tag expressions beyond repeated `--only-tag`.
