# Artifact Manifest: 30752 SPIRE Multicluster Transport Overlap

Packet: `30752-spire-multicluster-transport-overlap`
Head SHA: `e63f5bb813b37b135ae2a32f52dc884b56da9f6a`
Timestamp: `2026-05-10T17:26:31Z`

## `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30752-spire-multicluster-transport-overlap/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode: static formatting / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; only known stable-rustfmt warnings were emitted.

## `cargo-check-pg18.log`

- Command: `script -q -e -c "cargo check --no-default-features --features pg18" review/30752-spire-multicluster-transport-overlap/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode: PG18 compile check / none / n/a / n/a
- Surface isolation: n/a
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.13s`

## `bash-n-transport-overlap.log`

- Command: `script -q -e -c "bash -n scripts/run_spire_multicluster_transport_overlap_pg18.sh" review/30752-spire-multicluster-transport-overlap/artifacts/bash-n-transport-overlap.log`
- Lane / fixture / storage format / rerank mode: shell syntax check / transport-overlap harness / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0.

## `cargo-pgrx-install-pg18-pg-test.log`

- Command: `script -q -e -c "cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features \"pg18 pg_test\" --no-default-features" review/30752-spire-multicluster-transport-overlap/artifacts/cargo-pgrx-install-pg18-pg-test.log`
- Lane / fixture / storage format / rerank mode: PG18 pg_test extension install / none / n/a / n/a
- Surface isolation: installs the extension with `pg_test` helper schema for the local harness.
- Key result: `Discovered 686 SQL entities: 2 schemas, 683 functions, 0 types, 0 enums, 1 sqls, 0 ords, 0 hashes, 0 aggregates, 0 triggers`
- Key result: `Finished installing ecaz`

## `multicluster-transport-overlap.log`

- Command: `bash scripts/run_spire_multicluster_transport_overlap_pg18.sh --skip-install --artifact-dir review/30752-spire-multicluster-transport-overlap/artifacts --run-id 30752-final`
- Lane / fixture / storage format / rerank mode: PG18 local multicluster transport overlap / one coordinator plus two remote PostgreSQL clusters / n/a / n/a
- Surface isolation: transport-only proof using `pg_test` schema helper; no SPIRE index scoring, no remote heap resolution, and no AWS/product-scale claim.
- Key result: `transport_overlap_row=2,ready,none,0,304,304,3`
- Key result: `transport_overlap_row=3,ready,none,0,3,3,3`
- Key result: `fast_completed_before_slow=true`
- Key result: `SPIRE multicluster PG18 transport overlap passed`

## `remote-fast-postgres.log`

- Command: produced by `scripts/run_spire_multicluster_transport_overlap_pg18.sh`.
- Lane / fixture / storage format / rerank mode: PG18 local multicluster transport overlap / fast remote PostgreSQL cluster / n/a / n/a
- Surface isolation: one remote instance in the two-remote fixture.
- Key result: remote fast cluster started and shut down cleanly.

## `remote-slow-postgres.log`

- Command: produced by `scripts/run_spire_multicluster_transport_overlap_pg18.sh`.
- Lane / fixture / storage format / rerank mode: PG18 local multicluster transport overlap / slow remote PostgreSQL cluster / n/a / n/a
- Surface isolation: one remote instance in the two-remote fixture.
- Key result: remote slow cluster started and shut down cleanly.

## `coord-postgres.log`

- Command: produced by `scripts/run_spire_multicluster_transport_overlap_pg18.sh`.
- Lane / fixture / storage format / rerank mode: PG18 local multicluster transport overlap / coordinator PostgreSQL cluster / n/a / n/a
- Surface isolation: coordinator instance invokes only the `tests.ec_spire_test_production_transport_probe(...)` helper.
- Key result: coordinator cluster started and shut down cleanly.

## `git-diff-check.log`

- Command: `script -q -e -c "git diff --check HEAD -- Makefile src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md scripts/run_spire_multicluster_transport_overlap_pg18.sh" review/30752-spire-multicluster-transport-overlap/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode: static whitespace check / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; no whitespace errors in the committed checkpoint paths.
