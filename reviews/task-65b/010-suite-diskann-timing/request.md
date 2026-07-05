# Task 65b Packet 010: Suite DiskANN Timing Capture

## Summary

This checkpoint updates `ecaz bench suite` so the Task 65b Slice F/H worker and batch sweeps can parse DiskANN build timing evidence from load artifacts.

Before this change, `capture_parallel_workers` was effectively IVF-only because it looked for `workers_launched` in `ec_ivf build timing` rows. DiskANN already emits `ec_diskann_ambuild_timing` fields from the build path, but the suite runner did not parse them into `results.jsonl` or worker-count result rows.

After this change:

- DiskANN load artifacts can satisfy `capture_parallel_workers` using `parallel_effective_workers`;
- DiskANN timing rows are normalized as `ec_diskann_build_timing`;
- numeric timing/counter fields are available to thresholds and reports, including:
  - `parallel_epochs`
  - `parallel_batch_size`
  - `parallel_proposal_ms`
  - `parallel_reducer_ms`
  - `parallel_same_epoch_candidate_reads`
  - `parallel_total_candidate_reads`

IVF parsing remains unchanged.

## Why This Advances Task 65b

Task 65b still needs Slice F/H measurement: real10k, real100k, and a per-worker scaling curve. Those runs need normalized evidence for the exact fields the design packets made load-bearing, especially reducer wall time and stale-read counters. This packet closes that harness gap without doing any PG socket work.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz-cli suite::tests::parses_`
- `cargo check -p ecaz-cli`

The focused parser run passed `13` suite parser tests, including the new DiskANN timing coverage.

## Evidence

- Manifest: `reviews/task-65b/010-suite-diskann-timing/artifacts/manifest.md`
- Format log: `reviews/task-65b/010-suite-diskann-timing/artifacts/cargo-fmt-check.log`
- Parser test log: `reviews/task-65b/010-suite-diskann-timing/artifacts/cargo-test-suite-parsers.log`
- CLI check log: `reviews/task-65b/010-suite-diskann-timing/artifacts/cargo-check-ecaz-cli.log`

## Review Ask

Please review this as a Slice F/H harness checkpoint. It does not claim the final Task 65b performance gates; it makes the required DiskANN timing fields visible to the suite runner before the corpus-scale worker and batch sweep.
