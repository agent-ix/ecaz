# Review Request: AWS Remote Shard Load Flow

Task: Task 30 Phase 13e SPIRE AWS Production Gap Closure

Code commit: `300ca60fef6bac22754aa227154eb71a14cda721`

## Summary

This slice moves the AWS operator path away from coordinator-only loading and
fake descriptor identity registration.

Changes:

- `scripts/spire-aws/load.sh` now renders a static placement config from the
  topology, runs `ecaz corpus load --distributed-placement-config`, and loads
  each plan shard onto its owning remote via the plan's `remote_load_args`.
- Each remote load is followed by `ecaz corpus inspect` on that remote prefix.
- `scripts/spire-aws/register.sh` now requires the distributed placement plan,
  queries each live remote using `remote_identity_query_sql`, renders
  coordinator descriptor SQL with `ecaz corpus render-spire-registrations`, and
  applies the rendered SQL on the coordinator.
- The legacy `register-remotes.sql` no longer derives identity bytes from
  `node_id`; callers must pass `remote_index_identity_hex`.
- `infra/spire-aws/Makefile` now orders load before register for correctness and
  representative passes, and passes the tier-specific distributed plan into the
  registration step.

This still does not publish coordinator placement-directory rows for remote
leaf PIDs. The next required slice is the production coordinator placement
publication path that replaces test-only placement rewrites.

## Evidence

See `artifacts/manifest.md`.

- `artifacts/bash-syntax-spire-aws-load-register.log`: `bash -n` passed for the
  changed AWS load/register scripts.
- `artifacts/shellcheck-spire-aws-load-register.log`: `shellcheck` was not
  installed in this environment, so shellcheck lint was not run.

## Reviewer Notes

Please review the operator flow and quoting around plan-derived SQL and
`remote_load_args`. The remaining correctness risk is not in these scripts: it
is that coordinator placement publication for remote-owned leaves is still not
implemented.
