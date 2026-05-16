# Review Request: SPIRE Multicluster Transport Overlap

## Summary

Code checkpoint: `e63f5bb813b37b135ae2a32f52dc884b56da9f6a`

This slice closes the Phase 11.4 / Stage C timing-evidence gap for the
production async transport adapter:

- Added a `pg_test`-only helper,
  `tests.ec_spire_test_production_transport_probe(...)`, that resolves
  conninfo through secret names and calls the production async transport
  adapter. It does not expose raw conninfo and is not a production SQL surface.
- Added `scripts/run_spire_multicluster_transport_overlap_pg18.sh`, which
  starts one coordinator plus two separate remote PG18 clusters and runs a
  fixed slow/fast probe through the production adapter.
- Added `make spire-multicluster-transport-overlap`.
- Updated Phase 11 and the production coordinator/executor design with the
  packet evidence and with reviewer P3 guidance from 30751: C5 should consume
  the strict/degraded fault matrix as its AM-boundary source of truth, and the
  Stage D heap rows in that matrix are reserved category names until the heap
  executor emits them.

The successful harness run recorded:

```text
transport_overlap_row=2,ready,none,0,304,304,3
transport_overlap_row=3,ready,none,0,3,3,3
fast_completed_before_slow=true
SPIRE multicluster PG18 transport overlap passed
```

This is a transport-overlap proof only. It does not claim C5 AM integration,
remote endpoint scoring, remote heap resolution, or AWS/product-scale
performance.

## Key Files

- `src/lib.rs`
- `scripts/run_spire_multicluster_transport_overlap_pg18.sh`
- `Makefile`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `bash -n scripts/run_spire_multicluster_transport_overlap_pg18.sh`
- `cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features "pg18 pg_test" --no-default-features`
- `bash scripts/run_spire_multicluster_transport_overlap_pg18.sh --skip-install --artifact-dir review/30752-spire-multicluster-transport-overlap/artifacts --run-id 30752-final`
- `git diff --check HEAD -- Makefile src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md scripts/run_spire_multicluster_transport_overlap_pg18.sh`

## Review Questions

- Is a `pg_test`-only helper under the existing `tests` schema the right
  boundary for local multi-instance timing evidence without widening the
  production SQL surface?
- Does the one-coordinator/two-remote harness satisfy the Stage C
  "ready remotes are not serialized behind a slow remote" evidence gap, while
  correctly leaving Stage D/E open?
- Is the C5 matrix source-of-truth note sufficient to process the 30751 P3
  feedback for now?
