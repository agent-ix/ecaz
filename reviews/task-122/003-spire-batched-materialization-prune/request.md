# Task 122 Packet 003: SPIRE Batched Materialization Prune

## Summary

This continues Task 122 Phase 2 by extending the bounded pre-materialization
prune from Packet 002 into the SPIRE V2 leaf fully batched TQ path.

Packet 002 added the threshold check to the per-column V2 path. The
multi-segment batched path still routed each scored row through
`append_quantized_v2_scored_column_candidate`, which materialized the row before
the bounded heap could reject it. This change applies the same
`pre_materialization_min_ip_to_keep()` gate in that shared scored-column helper
before row decoding.

## Scope

Touched files:

- `src/am/ec_spire/scan/candidates.rs`
- `src/am/ec_spire/scan/tests/diagnostics.rs`

The change remains limited to bounded, non-deduped scans where there are no
delete-delta vec_ids requiring row materialization before delete filtering.

## Diagnostic Accounting

Pre-materialization skips are still reported as truncated candidates. The
diagnostic test now distinguishes rows actually materialized into visible
candidate rows from rows skipped before materialization:

- `candidate_row_count` is the materialized visible candidate count.
- `truncated_candidate_row_count` includes the pre-materialization skipped row.

## Validation

Passed:

- `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics_reports_candidate_truncation`
- `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics`

See `artifacts/cargo-test-spire-truncation.log`,
`artifacts/cargo-test-spire-placement-diagnostics.log`, and
`artifacts/manifest.md`.

## Measurement

No benchmark claim is made in this packet. The next required step remains an
`ecaz bench suite` A/B run for SPIRE bounded TQ scans at 10k/50k/100k before
claiming latency or materialization wins.

