# Review Request: SPIRE Multicluster PG18 Smoke

## Summary

This packet adds a PG18 smoke harness that starts two local PostgreSQL clusters, registers one as a SPIRE remote node, and validates the real libpq paths across the cluster boundary.

Code checkpoint: `464e9fbfcd30` (`Make SPIRE multicluster smoke self logging`)

## Scope

- Added `scripts/run_spire_multicluster_pg18_smoke.sh`.
- Added stable harness flags for packet runs (`--skip-install`, `--artifact-dir`, `--run-id`) so future PG18 multicluster smoke checks can use a reusable script approval prefix instead of ad hoc shell pipelines.
- Added a pg-test-only `tests.ec_spire_test_rewrite_placement_node(...)` helper so the external smoke can turn a local placement into a remote placement without exposing the debug rewrite surface in production builds.
- The smoke validates:
  - secret-provider conninfo lookup opens a libpq connection to the remote cluster;
  - remote candidate retrieval returns one candidate;
  - remote heap candidate resolution reports `remote_heap_candidates,ready,1`;
  - remote heap rows use `origin_node_row_locator`;
  - remote epoch manifest execution applies one manifest header and one manifest entry on the remote cluster.

## Validation

Artifact: `artifacts/multicluster-smoke-success.log`

```text
connection_status=libpq_connection_opened,secret_provider
candidate_count=1
heap_summary=remote_heap_candidates,ready,1
heap_row=2,origin_node_row_locator,true
manifest_executor=libpq_connection_opened,ready,ready
remote_manifest_applied=1,1
remote_manifest_entries=1,1
SPIRE multicluster PG18 smoke passed
```

Static checks:

- `bash -n scripts/run_spire_multicluster_pg18_smoke.sh`
- `cargo fmt`
- `git diff --check`

## Notes

The smoke assumes the PG18 pg_test extension build has already been installed. If `ECAZ_SKIP_INSTALL` is unset, the harness will run `cargo pgrx install --test --pg-config "$PGBIN/pg_config" --features "pg18 pg_test" --no-default-features` before creating the clusters.
