# Review Request: Local Static Remote Placement Fixture

## Summary

This checkpoint proves Phase 13e.1 end-to-end in a local PG18 fixture with one coordinator and three remote PostgreSQL instances. The fixture exports coordinator leaf base assignments before coordinator placements are republished remote, builds one shard table/index per remote, materializes the remote leaf V2 objects, registers post-materialization remote descriptors, publishes coordinator remote placements, and executes a distributed CustomScan read.

The AWS registration flow now follows the same production ordering. Coordinator leaf assignment export runs before remote placement publish, remote materialization publishes the coordinator-exported active epoch, remote identity is captured after materialization, and coordinator registration happens with that post-materialization identity. This fixes the observed retention-gap shape where the coordinator asked for epoch 1 while a remote had published epoch 2.

## Scope

- Updates `scripts/spire-aws/register.sh` to materialize each remote shard before coordinator remote placement publish, then verify required coordinator leaf PIDs against the materialized remote.
- Updates `scripts/spire-aws/export-coordinator-leaf-base-assignments.sql` to compute deterministic pre-publish remote assignment from the configured remote node list.
- Treats `scripts/spire-aws/materialize-remote-leaf-base-assignments.sql` as a rendered template with a concrete assignment-file path, and validates that one exported coordinator active epoch is present.
- Changes `ec_spire_materialize_static_remote_leaf_assignments(...)` to publish the exported coordinator epoch on the remote index.
- Adds `scripts/run_spire_phase13e_static_remote_placement_pg18.sh`, a local PG18 one-coordinator/three-remote production fixture.
- Marks the Phase 13e.1 static remote placement checklist complete in `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`.

## Validation

Artifacts are under `artifacts/` and summarized in `artifacts/manifest.md`.

- `cargo check -p ecaz --lib`: pass.
- `cargo fmt --all --check`: pass, with existing stable-rustfmt warnings.
- `bash -n scripts/spire-aws/register.sh`: pass.
- `bash -n scripts/run_spire_phase13e_static_remote_placement_pg18.sh`: pass.
- `git diff --check`: pass.
- Local PG18 fixture: pass.
  - `placement_summary=2:1,3:1,4:1`
  - `profile_summary=ready|3|3|3|3|6`
  - plan contains `Custom Scan (EcSpireDistributedScan)` and `remote_fanout: 3`
  - read returns remote heap rows: `1,doc 1`, `5,doc 5`, `9,doc 9`, `2,doc 2`, `6,doc 6`, `10,doc 10`

## Remaining Gaps

This closes the static remote placement proof, not the whole Phase 13e plan. Remaining work still includes broader distributed read correctness thresholds, AWS-scale performance evidence through `ecaz bench suite`, parallel fanout, and the connection-pooling decision backed by measurement.
