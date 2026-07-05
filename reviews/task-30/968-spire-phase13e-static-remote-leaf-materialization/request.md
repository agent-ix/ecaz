# Review Request: Static Remote Leaf Materialization

## Summary

This checkpoint implements the first real remote data placement primitive for Phase 13e. The AWS registration flow now exports coordinator-owned leaf base assignments, ships that TSV through the local psql client, joins it to each remote heap table by stable `id`, and materializes remote SPIRE leaf V2 objects with remote heap CTIDs.

The new SQL function is `ec_spire_materialize_static_remote_leaf_assignments(...)`. It validates array lengths, PID/object-version consistency, contiguous per-leaf row indexes, vec_id encoding, finite gammas, and publishes a replacement strict epoch on the remote index containing the materialized leaf placements.

## Scope

- Extends `ec_spire_index_leaf_base_assignment_snapshot(...)` with `parent_pid`.
- Adds `scripts/spire-aws/materialize-remote-leaf-base-assignments.sql`.
- Updates `scripts/spire-aws/register.sh` so the leaf equality gate observes the remote after materialization.
- Keeps the remote CTID mapping remote-local by joining exported coordinator row ids to the remote corpus table.

## Remaining Gaps

This is not the full Phase 13e closeout. It covers static base leaf materialization only. Remaining work still includes live PG18 distributed fixture proof, distributed CustomScan correctness with remote rows, delta/update materialization policy, parallel fanout, and connection-pooling evidence.

## Validation

Artifacts are under `artifacts/` and summarized in `artifacts/manifest.md`.

- `cargo check -p ecaz --lib`: pass.
- `cargo fmt --all -- --check`: pass, with existing stable-rustfmt warnings.
- `bash -n scripts/spire-aws/register.sh`: pass.
- `git diff --check`: pass.

