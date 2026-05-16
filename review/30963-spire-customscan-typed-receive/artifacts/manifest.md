# Artifact Manifest: SPIRE CustomScan Typed Receive

- head SHA: `3c421a32e3d4378d6d23bbda628676a0e97178bb`
- packet/topic: `30963-spire-customscan-typed-receive`
- lane / fixture / storage format / rerank mode: Phase 12.2 typed tuple
  transport negotiation and `EcSpireDistributedScan` receive; PG18
  one-coordinator/one-remote CustomScan read fixture; storage format `rabitq`;
  no rerank mode.
- isolated one-index-per-table or shared-table surfaces: isolated
  one-index-per-table fixture surfaces, with separate remote and coordinator
  tables/indexes.

## Success Artifacts

### `customscan-read-typed-success/multicluster-customscan-read.log`

- command: `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --skip-install --artifact-dir /home/peter/dev/ecaz/review/30963-spire-customscan-typed-receive/artifacts/customscan-read-typed-success --run-dir /home/peter/dev/ecaz/target/se30963-customscan-read-typed-final`
- timestamp: `2026-05-12 21:22:46-07:00`
- key result lines:
  - `run_dir=/home/peter/dev/ecaz/target/se30963-customscan-read-typed-final`
  - `->  Custom Scan (EcSpireDistributedScan) on ec_spire_customscan_coord_sql`
  - `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`
  - `payload_probe=ready,2,{"id": 10, "title": "remote alpha"}`
  - `typed_payload_probe=ready,pg_binary_attr_v1,t,t`
  - `SPIRE multicluster CustomScan read passed`

### `customscan-read-typed-success/coord-postgres.log`

- command: same PG18 CustomScan read fixture as above.
- timestamp: `2026-05-12 21:22:46-07:00`
- key result lines:
  - no `ERROR` or typed receive decode warning in the final success coordinator
    log.

### `customscan-read-typed-success/remote-postgres.log`

- command: same PG18 CustomScan read fixture as above.
- timestamp: `2026-05-12 21:22:46-07:00`
- key result lines:
  - no `ERROR` in the final success remote log.

## Static Validation Artifacts

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- timestamp: `2026-05-12 21:23:27-07:00`
- key result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 0.12s`
  - command exited with status `0`
- note: the run reported a pre-existing unused-import warning in
  `src/am/mod.rs`.

### `bash-n-customscan-read.log`

- command: `bash -n scripts/run_spire_multicluster_customscan_read_pg18.sh`
- timestamp: `2026-05-12 21:23:31-07:00`
- key result lines:
  - command exited with status `0`

### `git-diff-check.log`

- command: `git diff --check -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/tests.rs src/am/ec_spire/root/hierarchy_snapshots.rs src/am/ec_spire/scan/tests/runtime_state.rs scripts/run_spire_multicluster_customscan_read_pg18.sh plan/tasks/task30-phase12-spire-production-hardening.md`
- timestamp: `2026-05-12 21:23:36-07:00`
- key result lines:
  - command exited with status `0`

## Failed Debug Artifacts

These directories are retained for traceability only and are not passing
evidence.

### `customscan-read-typed/`

- command: `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --skip-install --artifact-dir /home/peter/dev/ecaz/review/30963-spire-customscan-typed-receive/artifacts/customscan-read-typed --run-dir /home/peter/dev/ecaz/target/se30963-customscan-read-typed`
- timestamp: `2026-05-12 21:14-07:00`
- key result lines:
  - `ERROR:  EcSpireDistributedScan production executor blocked: status remote_heap_resolution_failed, next_blocker remote_heap_resolution, recommendation inspect production remote heap failure category before final row delivery`

### `customscan-read-typed-debug/`

- command: debug rerun of the same PG18 CustomScan typed receive fixture.
- timestamp: `2026-05-12 21:19-07:00`
- key result lines:
  - `WARNING:  ec_spire remote heap receive decode failed: ec_spire remote heap executor typed payload_collations decode failed`
  - `ERROR:  EcSpireDistributedScan production executor blocked: status remote_heap_resolution_failed, next_blocker remote_heap_resolution, recommendation inspect production remote heap failure category before final row delivery`

### `customscan-read-typed-success2/`

- command: debug rerun of the same PG18 CustomScan typed receive fixture.
- timestamp: `2026-05-12 21:18-07:00`
- key result lines:
  - `ERROR:  EcSpireDistributedScan production executor blocked: status remote_heap_resolution_failed, next_blocker remote_heap_resolution, recommendation inspect production remote heap failure category before final row delivery`

### `customscan-read-typed-success3/`

- command: debug rerun of the same PG18 CustomScan typed receive fixture.
- timestamp: `2026-05-12 21:21-07:00`
- key result lines:
  - `WARNING:  ec_spire remote heap receive decode failed: ec_spire remote heap executor typed payload_collations decode failed`
  - `ERROR:  EcSpireDistributedScan production executor blocked: status remote_heap_resolution_failed, next_blocker remote_heap_resolution, recommendation inspect production remote heap failure category before final row delivery`

### `customscan-read-typed-success4/`

- command: debug rerun of the same PG18 CustomScan typed receive fixture.
- timestamp: `2026-05-12 21:22-07:00`
- key result lines:
  - `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`
  - `typed_payload_probe=ready,pg_binary_attr_v1,t,t`
- note: this run proved the code path but preceded the final script assertion
  correction for composite text output. The final success artifact above is the
  cited passing run.
