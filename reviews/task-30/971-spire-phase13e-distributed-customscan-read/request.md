# Review Request: SPIRE Phase 13e Distributed CustomScan Read

## Summary

This slice completes the local Phase 13e.2 proof for the distributed CustomScan read path.

Changes:

- Added production operator variants for static remote placement and remote leaf materialization with explicit `strict` or `degraded` epoch consistency mode.
- Updated the production scan path so degraded mode skips pre-dispatch blocked remotes before merge, matching the existing degraded executor diagnostics.
- Updated final read profile status so executor-level degraded skips surface as `degraded_ready`.
- Extended the Phase 13e static remote placement PG18 fixture to prove:
  - remote placements on nodes 2, 3, and 4,
  - `EXPLAIN` uses `Custom Scan (EcSpireDistributedScan)` with `remote_fanout: 3`,
  - distributed top-k rows match the exact deterministic baseline,
  - strict remote failure returns an error and no partial rows,
  - degraded remote failure returns partial rows with `degraded_skipped_dispatch_count=1`.

Code commit: `74be9d04d9fefe9c851666ea36260762251c7c66`

## Evidence

Primary fixture log: `artifacts/phase13e-static-remote-strict-degraded.log`

Key lines:

- `placement_summary=2:1,3:1,4:1`
- `profile_summary=ready|3|3|3|3|6`
- `Custom Scan (EcSpireDistributedScan)`
- `remote_fanout: 3`
- `read_rows` equals `exact_rows`: `1,5,9,2,6,10`
- `strict_remote_failure_exit_code=3`
- `strict_remote_failure_text=ERROR:  ec_spire remote write shape fingerprint failed to open connection for node_id 2`
- `degraded_profile_summary=degraded_ready|3|2|2|2|1|0|0|6|none`
- `degraded_rows`: `4,8,12,3,7,11`
- `SPIRE Phase 13e static remote placement PG18 fixture passed`

Validation logs:

- `artifacts/cargo-check-ecaz-lib.log`
- `artifacts/cargo-fmt-check.log`
- `artifacts/bash-n-phase13e-fixture.log`
- `artifacts/git-diff-check.log`

## Notes

The fixture intentionally stops node 2 for the strict/degraded branch. Strict mode fails closed. Degraded mode republishes the surviving remote indexes and coordinator epoch as degraded through production operator functions, then returns only rows from nodes 3 and 4 while reporting one skipped dispatch.
