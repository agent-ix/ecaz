# Review Request: Task 68 Packet 001 SPIRE Build Timing Notices

Code commit: `318641d6fa091291fb07f52bfdb30958b8facad8`

## Summary

This packet adds the instrumentation surface needed for Task 68 Phase 1 characterization. It does not claim a performance win and does not change SPIRE on-disk or object-store formats.

The new `ec_spire_ambuild_timing` `NOTICE` reports:

- setup
- heap scan and build tuple collection
- training sample collection
- top-level shared k-means time and call count
- top-level assignment time
- recursive routing k-means time, call count, and max level
- recursive assignment time
- draft assembly
- top-graph construction
- PQ4 training placeholder (`pq4_training_ms=0`, because current SPIRE build path does not call grouped PQ4 training)
- object-store writes
- publish/manifest/root-control work
- total build time

This mirrors the DiskANN build-profile precedent: add structured AM timing first, then run packet-local `CREATE INDEX` / suite measurements that capture the notice.

## Static Call Audit

Artifact: `artifacts/common-training-call-audit.txt`

Build-path `common_training::*` consumers are:

- `src/am/ec_spire/build/training.rs`: auto-list resolution, single-level and relation-build k-means, batch assignment, normalization, deterministic sample selection.
- `src/am/ec_spire/build/recursive.rs`: per-level recursive k-means and batch assignment.

Non-build SPIRE common-training consumers found by the same audit:

- `src/am/ec_spire/update/materialization.rs`: split replacement materialization k-means.
- `src/am/ec_spire/update/routing.rs`: scheduled merge centroid normalization.

## Validation

Artifact manifest: `artifacts/manifest.md`

```text
cargo check -p ecaz --lib --no-default-features --features pg18
```

Result: passed.

## Notes For Reviewer

This packet is a characterization prerequisite, not the Task 68 Phase 1 measurement packet. The next Task 68 packet should run the 10k and 100k one-index-per-table SPIRE builds through `ecaz bench suite` / packet-local `CREATE INDEX` logs and use these `NOTICE` fields for the wall-time split and ranked P0 list.
