# Review Request: Pipeline Stage Funnel Diagnostics

## Summary

This checkpoint starts Task 120 Phase 1 by making the existing SPIRE local
pipeline snapshot durable in `ecaz bench spire-pipeline --funnel-output`
JSONL.

Each `spire_candidate_funnel` record now includes `pipeline_stages`, a
per-query array copied from `ec_spire_index_scan_pipeline_snapshot`:

- `routing`
- `placement`
- `prefetch`
- `candidates`
- `heap_rerank`
- `remote_fanout`

For each stage the funnel output preserves status, item/ready/blocked counts,
route/candidate/rerank/fanout counts, `next_blocker`, and `recommendation`.
This gives Task 120 Phase 1 runs a packet-local way to tie containment and
miss-attribution rows back to the stage budget that shaped the query.

No scan behavior, index format, or default policy changes in this slice.

## Validation

- `cargo fmt --package ecaz-cli --check` passed with existing stable-rustfmt
  warnings.
- `cargo test -p ecaz-cli spire_pipeline` passed: `21 passed; 0 failed`.

## Notes

This is intentionally a small measurement-surface change. It does not satisfy
Task 120 closeout evidence; subsequent packets still need actual
`ecaz bench suite` 10k/50k/100k containment and recall/latency/storage runs.
