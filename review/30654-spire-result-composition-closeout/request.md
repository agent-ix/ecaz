# Review Request: SPIRE Result Composition Closeout

## Summary

Task 30 SPIRE Phase 7 now feeds libpq executor heap candidates into
`ec_spire_remote_search_coordinator_result_summary(...)` so ready remote plans
produce a final remote heap result instead of stopping at the executor-only
surface.

Code checkpoint: `ab9ad5746889` (`Compose SPIRE remote heap results`)

## Scope

- Composes coordinator-local heap rows and remote origin-node heap rows into
  one ranked result-summary path.
- Dedupes composed heap rows by `vec_id` using the existing remote merge
  comparator before applying `top_k`.
- Keeps mixed local+remote plans local-safe by building selected-local
  manifest/placement views before opening local object stores.
- Extends `ec_spire_remote_catalog_index_cleanup(...)` to remove applied
  manifest header/entry rows keyed by `remote_index_oid`, closing reviewer F4.
- Adds `make spire-multicluster-smoke` and extends the smoke harness to assert
  `coordinator_result=remote_heap_candidates,ready,remote_ready,1`.
- Updates the Phase 7 task note.

## Validation

- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
- `cargo pgrx test pg18 test_ec_spire_remote_catalog_index_cleanup`
  - `test tests::pg_test_ec_spire_remote_catalog_index_cleanup ... ok`
- `bash -n scripts/run_spire_multicluster_pg18_smoke.sh`
- `git diff --check`
- `make spire-multicluster-smoke SPIRE_MULTICLUSTER_SMOKE_FLAGS="--skip-install --artifact-dir review/30654-spire-result-composition-closeout/artifacts --run-id 30654"`

Artifact: `artifacts/multicluster-smoke-success.log`

```text
connection_status=libpq_connection_opened,secret_provider
candidate_count=1
heap_summary=remote_heap_candidates,ready,1
heap_row=2,origin_node_row_locator,true
coordinator_result=remote_heap_candidates,ready,remote_ready,1
manifest_executor=libpq_connection_opened,ready,ready
remote_manifest_applied=1,1
remote_manifest_entries=1,1
SPIRE multicluster PG18 smoke passed
```

`cargo fmt --check` was not used as a gating signal for this slice because it
reports unrelated pre-existing formatting diffs in `src/am/ec_ivf/scan.rs`,
`src/am/ec_spire/options.rs`, `src/am/ec_spire/scan.rs`, and
`src/am/ec_spire/update.rs`.

## Notes

The first sandboxed smoke attempt failed because PostgreSQL could not bind its
Unix socket from inside the sandbox. The passing smoke run used the approved
`make spire-multicluster-smoke` command outside the sandbox and stores its logs
in this packet.

## Post-Review Closeout

Post-review checkpoint: `3cb45efcaa15` (`Fix SPIRE PG18 closeout regressions`)

- Resolved the P1 plan mismatch in `5a5ed267` by marking Phase 7 coordinator
  transport and distributed epoch manifest complete with links back to this
  packet's smoke evidence.
- Resolved the P3 formatter drift in `0e64adba`; `cargo fmt --check` now exits
  0, with only the existing stable-rustfmt warnings about unstable
  `imports_granularity` / `group_imports`.
- The first broad `cargo pgrx test pg18` run exposed five closeout failures:
  allocator advancement on invalid SPIRE assignment rows, recursive primary
  leaf placement ignoring centroid assignment indexes, and two Rust unit tests
  reading pgrx GUCs from parallel test threads. `3cb45efcaa15` fixes those.
- Re-run `cargo pgrx test pg18` passes. Raw log:
  `artifacts/cargo-pgrx-test-pg18-success.log`.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  was run and still fails on repository-wide lint debt unrelated to
  `3cb45efcaa15`; raw log:
  `artifacts/cargo-clippy-pg18-failure.log`.
- P2 (`ec_spire_remote_pipeline_steps(index_oid)`) is intentionally deferred
  as a Phase 8 preflight/follow-up packet rather than part of Phase 7 closeout.

Key PG18 result lines:

```text
test result: ok. 1462 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 21 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Key clippy failure shape:

```text
error: could not compile `ecaz` (lib) due to 120 previous errors
error: could not compile `ecaz` (lib test) due to 133 previous errors
```
