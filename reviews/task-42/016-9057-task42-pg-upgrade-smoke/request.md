# Review Request: Task 42 PG Upgrade Smoke

## Summary

This checkpoint closes the Task 42 `pg_upgrade` smoke lane for the current PG18
surface.

Code commit: `a4be6a557bfcae06f4e8e12c9c28d5773e3f13da` (`Add Task 42 pg_upgrade smoke lane`)

Changes:

- Added `make pg-upgrade-smoke`.
- Added `ecaz dev pg-upgrade-smoke` as the operator-owned CLI wrapper.
- Added `scripts/run_pg_upgrade_smoke_pg18.sh`.
- Documented the qemu `--unresolved-symbols=ignore-all` limitation as decode-only.
- Clarified that the current upgrade matrix is a registry-consistency check until a second writable format ships.
- Documented the PG18 same-binary `pg_upgrade` smoke in `docs/on-disk-format.md`.

## Smoke Shape

The lane:

1. Initializes old/new PG18 clusters.
2. Creates `ecaz` in the old cluster.
3. Inserts four `ecvector(4)` rows.
4. Builds one `ec_hnsw` index.
5. Verifies pre-upgrade nearest-neighbor parity (`pre_top2=1,2`).
6. Runs full `pg_upgrade`.
7. Starts the upgraded cluster.
8. Verifies post-upgrade nearest-neighbor parity and index presence.
9. Runs `pg_amcheck --install-missing` against the upgraded database.

## Validation

Packet-local artifacts are under `artifacts/`.

- `make pg-upgrade-smoke PG_UPGRADE_SMOKE_FLAGS="--skip-install --run-dir target/pg-upgrade-smoke-task42-local-3 --smoke-log target/pg-upgrade-smoke-task42-local-3.log"`:
  - `pre_top2=1,2`
  - `post_top2=1,2`
  - `pre_index_count=1`
  - `post_index_count=1`
  - `pg_amcheck=passed`
  - `PG18 pg_upgrade smoke passed`
- `bash -n scripts/run_pg_upgrade_smoke_pg18.sh`: passed.
- `cargo fmt --all -- --check`: passed with existing stable-toolchain warnings about unstable rustfmt options.
- `cargo test -p ecaz-cli cli_parses_pg_upgrade_smoke_command`: passed (`1 passed`), with the existing unused-import warning in `src/am/mod.rs`.

The first non-escalated local smoke attempt failed because sandboxed PostgreSQL
could not bind Unix-domain sockets. The passing run used the normal repo tree
and the local installed PG18 pgrx tree, with elevated socket permission only.

## Reviewer Focus

- Is the same-binary PG18 `pg_upgrade` fixture sufficient as the Task 42 smoke
  until PG19 is available?
- Is `pg_amcheck --install-missing` the right behavior for an isolated throwaway
  upgraded cluster?
- Should this lane remain operator/manual only, or should a future CI job run it
  on a schedule once CI pgrx setup is stable?

## Remaining Task 42 Gaps

- WAL record version tags remain paired with Task 37 because current writes use
  PostgreSQL GenericXLog page records rather than extension-owned WAL payloads.
- Historical live corpus directories under `fixtures/upgrade/{vN}` activate
  when a new incompatible writable format version ships.
