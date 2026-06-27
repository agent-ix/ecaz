# Task 122 Packet 002: SPIRE Bounded Materialization Prune

## Summary

This is the first behavior slice for Task 122 Phase 2.

SPIRE V2 leaf scans already score column payloads in a batch. After that batch
score, bounded non-deduped scans can know the current heap threshold before
materializing a row. This change skips row materialization when all of the
following are true:

- the candidate is visible from flags;
- the score is finite;
- there are no delete-delta vec_ids that require row materialization before
  delete filtering;
- candidate dedupe is disabled;
- the bounded heap already has a minimum inner-product threshold;
- the candidate inner product is strictly below that threshold.

Skipped rows are reported through the existing truncated-candidate diagnostics,
so candidate accounting remains visible without introducing a new SQL-visible
diagnostic column in this slice.

## Scope

Touched file:

- `src/am/ec_spire/scan/candidates.rs`

This keeps the existing block scorer path intact: the column payload slab is
still batch-scored before the pruning check.

## Validation

Passed:

- `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics_reports_candidate_truncation`

See `artifacts/cargo-test-spire-diagnostics.log`.

## Measurement

No benchmark claim is made in this packet. The next required step is an
`ecaz bench suite` A/B run for SPIRE bounded TQ scans at 10k/50k/100k before
claiming latency or materialization wins.
