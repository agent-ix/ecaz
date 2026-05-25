# Artifact Manifest

- head SHA: `74be9d04d9fefe9c851666ea36260762251c7c66`
- task bucket: `reviews/task-30/971-spire-phase13e-distributed-customscan-read`
- timestamp: `2026-05-25T18:35:26Z`
- lane: Phase 13e.2 local PG18 distributed CustomScan read path
- fixture: one coordinator plus three remotes, static remote leaf placement, strict and degraded consistency
- storage format: `rabitq`
- rerank mode: none
- surface: isolated scratch clusters with one index per coordinator/remote table

## Artifacts

- `phase13e-static-remote-strict-degraded.log`
  - command: `scripts/run_spire_phase13e_static_remote_placement_pg18.sh --skip-install --artifact-dir target/phase13e-static-remote-strict-degraded-validation-8`
  - key lines:
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

- `strict-remote-node2-failure.log`
  - command: emitted by the fixture strict-mode query after stopping remote node 2.
  - key line: `ERROR:  ec_spire remote write shape fingerprint failed to open connection for node_id 2`

- `node-3-materialize-degraded.log`
  - command: fixture re-materialization of node 3 with `consistency_mode=degraded`.
  - key result: `active_epoch=1`, `leaf_count=1`, `assignment_count=3`, `status=materialized`

- `node-4-materialize-degraded.log`
  - command: fixture re-materialization of node 4 with `consistency_mode=degraded`.
  - key result: `active_epoch=1`, `leaf_count=1`, `assignment_count=3`, `status=materialized`

- `cargo-check-ecaz-lib.log`
  - command: `cargo check -p ecaz --lib`
  - result: passed.

- `cargo-fmt-check.log`
  - command: `cargo fmt --all --check`
  - result: passed with existing stable-rustfmt warnings about nightly-only import grouping options.

- `bash-n-phase13e-fixture.log`
  - command: `bash -n scripts/run_spire_phase13e_static_remote_placement_pg18.sh`
  - result: passed.

- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed.
