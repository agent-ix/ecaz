# Review Request: Record Latency Cache State

## Summary

This packet addresses the reviewer ask from Task 59 packet 005 to make focused-vs-final latency discrepancies attributable. The code change adds an explicit `--cache-state` label to `ecaz bench latency`, threads `cache_state` through `ecaz bench suite` latency steps, and renders the label as a latency table column. Because suite result extraction already parses table columns generically, the label is carried into `results.jsonl` without another parser path.

The Task 59 final Graviton suite configs now label latency steps as `post_recall_warm`, matching the step order in the checked-in suite. The main suite 1M load also now uses the retained single-TSV staging paths instead of the rejected chunked-manifest mode.

## Commit

- `d3a99b2ab6ee43075a1a49ee25478cc86e90aee0` Record DiskANN latency cache state

## Validation

- `cargo test -p ecaz-cli commands::bench::latency`
- `cargo test -p ecaz-cli commands::bench::suite::tests::expands_latency_with_cache_state_label`
- `jq empty benchmarks/task59-aws-diskann-final-graviton-suite/suite.json`
- `jq empty benchmarks/task59-aws-diskann-final-graviton-suite/suite-1m-resume.json`

Artifact details are in `artifacts/manifest.md`.

## Notes

- This does not claim a Task 59 latency win. It records cache-state metadata so future latency claims can separate code effects from warm/cold run variance.
- The active 1M AWS resume run was already dispatched before this commit, so its latency artifact will not include the new `cache_state` column unless it is rerun with this head.
