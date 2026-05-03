# Task 31 Suite Runner Auto-Mode Slice

Reviewer: please review the next ADR-050 implementation slice after
`review/30179-task31-suite-runner-execution/`.

## Scope

This adds the first auto-mode conveniences on top of executable suites:

- Step-level `tags` in suite configs.
- `ecaz bench suite run --only-tag <tag>` selection.
- `ecaz bench suite run --resume-from <manifest>` to skip selected steps that
  already succeeded in an earlier manifest.
- `ecaz bench suite run --results-output <path>` plus default
  `<artifact_dir>/results.jsonl` after executed runs.
- `ecaz bench suite report --results-output <path>` to write normalized JSONL
  rows while emitting the markdown report.
- Parsed result rows for completed `load`, `recall`, `latency`, and `storage`
  artifacts.

The Task 31 sample suite now tags each step by kind, corpus size, index shape,
and candidate lane so optimization runs can target groups without editing the
suite JSON.

## Spec and Docs

- Updated `FR-038` with tags, resume, results extraction, and richer reports.
- Updated `spec/tests.md` to move TC-020 to the first auto-runner surface.
- Updated `crates/ecaz-cli/README.md` with `--only-tag`, `--resume-from`, and
  `results.jsonl` usage.

## Smoke Artifacts

Artifacts are under `artifacts/`:

- `only_tag_manifest.json`: dry-run manifest selecting only recall-tagged steps.
- `empty_results.jsonl`: report output for the dry-run manifest; expected empty.
- `result_fixture_manifest.json`: synthetic succeeded manifest pointing at
  existing Task 31 packet logs.
- `result_report.log`: report output showing parsed load/recall/latency/storage
  rows.
- `results.jsonl`: normalized result rows from the fixture manifest.
- `manifest.md`: packet-local artifact source of truth.

The fixture manifest intentionally reuses existing raw logs instead of executing
benchmarks again.

## Validation

Ran:

```text
cargo fmt --package ecaz-cli
cargo test -p ecaz-cli suite
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json --dry-run --only-tag recall --manifest-output review/30180-task31-suite-runner-auto-mode/artifacts/only_tag_manifest.json
cargo run -p ecaz-cli -- bench suite report --manifest review/30180-task31-suite-runner-auto-mode/artifacts/only_tag_manifest.json --results-output review/30180-task31-suite-runner-auto-mode/artifacts/empty_results.jsonl
cargo run -p ecaz-cli -- --log-file review/30180-task31-suite-runner-auto-mode/artifacts/result_report.log bench suite report --manifest review/30180-task31-suite-runner-auto-mode/artifacts/result_fixture_manifest.json --results-output review/30180-task31-suite-runner-auto-mode/artifacts/results.jsonl
```

`cargo test -p ecaz-cli suite` passed 11 tests. The build emitted the existing
PG18 server-header warnings from `csrc/pg18_pgstat_shim.c`; no suite-specific
warnings were introduced.

## Deferred

- Suite-level success thresholds.
- Tag expressions beyond repeated `--only-tag`.
- Resume conflict handling when commands or config hashes differ.
- Dedicated normalized numeric columns for each metric family.
