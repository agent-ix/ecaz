# Review Request: SPIRE Pipeline Local Store Counters

## Summary

Further Phase 12.9 harness work for packet-local readiness capture.

This slice extends `ecaz bench spire-pipeline` with
`--include-local-store-overlap`, which aggregates
`ec_spire_index_scan_local_store_read_overlap_harness(...)` per sampled query
and `nprobe` sweep value. The report now has a separate `Local store overlap
counters` section with:

- route, leaf-route, and delta-route sums;
- candidate row sums;
- prefetched object-byte sums;
- read-batch sums; and
- delta-decode sums.

This complements packet `30965` query metrics and keeps object-byte/read-batch
capture on the same CLI benchmark path as routing/local/remote counters. It
does not claim a live benchmark result or close the final readiness artifact
row.

## Files

- `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`
- `crates/ecaz-cli/src/cli.rs`
- `plan/tasks/task30-phase12-spire-production-hardening.md`
- `review/30966-spire-pipeline-local-store-counters/artifacts/manifest.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `cargo test -p ecaz-cli spire_pipeline`
- `cargo check --no-default-features --features pg18`
- `git diff --check aa8e997a^ aa8e997a`

No live PostgreSQL fixture was run for this slice; the covered surface is CLI
parsing, SQL string wiring, aggregate rendering, and PG18 build/check.

## Reviewer Focus

- Confirm local-store overlap counters remain a distinct report section and do
  not blur with the existing local pipeline step counters.
- Confirm object-byte, read-batch, and delta-decode sums are aggregated by
  `(nprobe, node_id, local_store_id)`.
- Confirm the tracker update is scoped to CLI capture capability, not actual
  packet-local benchmark evidence.
