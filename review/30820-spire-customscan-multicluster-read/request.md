# Review Request: SPIRE CustomScan Multicluster Read Fixture

## Scope

Code slice for the ADR-067 CustomScan pivot read-path evidence gap. This adds
the final local multi-instance read fixture for `EcSpireDistributedScan`.

- Add `scripts/run_spire_multicluster_customscan_read_pg18.sh`.
- The script starts separate local PG18 coordinator and remote clusters.
- It installs the test extension, creates separate coordinator and remote
  heap/index pairs, rewrites the coordinator leaf placements to remote
  `node_id = 2`, and registers a remote descriptor with the real endpoint
  profile fingerprint.
- It asserts `EXPLAIN` contains `Custom Scan (EcSpireDistributedScan)`.
- It runs
  `SELECT id, title FROM ec_spire_customscan_coord_sql ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1`
  and verifies the returned projected tuple is the remote shard row
  `10,remote alpha`.
- It probes `ec_spire_remote_search_tuple_payload(...)` on the remote and
  verifies the tuple-payload side channel returns the requested `id,title`
  payload.
- Update the Phase 11 tracker to mark the final multi-instance CustomScan read
  fixture complete while leaving ADR-069 writes, production costing, local-only
  AM preservation proof, Stage E migration, and cleanup open.
- Update `ec_spire_custom_scan_status()` next step so it no longer asks for the
  multi-instance read fixture.

This does not implement coordinator-routed INSERT/UPDATE/DELETE/PK-read,
production CustomScan costing, Stage E matrix migration, or materialization
catalog cleanup.

## Validation

- `scripts/run_spire_multicluster_customscan_read_pg18.sh --artifact-dir review/30820-spire-customscan-multicluster-read/artifacts --run-id cscan30820c`
  - Passed.
  - Key lines: `plan=Limit -> Custom Scan (EcSpireDistributedScan)`;
    `read_row=10,remote alpha`;
    `payload_probe=ready,2,{"id": 10, "title": "remote alpha"}`;
    `SPIRE multicluster CustomScan read passed`.
- `cargo test custom_scan_status --lib`
  - Passed: 2 tests.
- `bash -n scripts/run_spire_multicluster_customscan_read_pg18.sh`
  - Passed.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --cached --check`
  - Passed.

## Review Focus

- Check the script's multicluster setup and teardown pattern against the
  existing Stage E smoke scripts.
- Check that the descriptor registration uses the real remote endpoint
  fingerprint rather than the old placeholder identity bytes.
- Check that the fixture evidence covers the final read-path gap without
  claiming ADR-069 write-path completion.

## Artifacts

- `review/30820-spire-customscan-multicluster-read/artifacts/manifest.md`
- `review/30820-spire-customscan-multicluster-read/artifacts/multicluster-customscan-read.log`
- `review/30820-spire-customscan-multicluster-read/artifacts/remote-postgres.log`
- `review/30820-spire-customscan-multicluster-read/artifacts/coord-postgres.log`
- `review/30820-spire-customscan-multicluster-read/artifacts/cargo-test-custom-scan-status-lib.log`
- `review/30820-spire-customscan-multicluster-read/artifacts/bash-n-customscan-read.log`
- `review/30820-spire-customscan-multicluster-read/artifacts/cargo-fmt-check.log`
- `review/30820-spire-customscan-multicluster-read/artifacts/git-diff-cached-check.log`
