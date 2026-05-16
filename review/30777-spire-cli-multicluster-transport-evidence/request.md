# 30777 - SPIRE CLI multicluster transport evidence

Runtime evidence head: `f2950a3678c7c69cc5e114a22570f79b6167d16f`

Task-note commit: `e943f44542fb483e3b3c2d263306542e983a812f`

## Summary

This packet captures runtime evidence for the `ecaz` operator entrypoint added
in packet `30776`:

```text
target/debug/ecaz dev spire-multicluster transport-overlap-pg18 \
  --artifact-dir review/30777-spire-cli-multicluster-transport-evidence/artifacts \
  --run-id 30777 \
  --skip-install
```

The command ran the one-coordinator/two-remote PG18 transport-overlap fixture
through the CLI path. It reused the already installed pg_test extension because
the preceding code slice only added the CLI wrapper and did not change backend
extension code.

Key result from
`artifacts/multicluster-transport-overlap.log`:

```text
transport_overlap_row=2,ready,none,0,304,304,3
transport_overlap_row=3,ready,none,0,3,3,3
fast_completed_before_slow=true
SPIRE multicluster PG18 transport overlap passed
```

This proves the new CLI entrypoint can start the local multi-instance fixture,
capture packet-local logs, and preserve the existing transport-overlap
acceptance signal: a ready fast remote returns before the deliberately slow
ready remote.

## Scope Boundaries

This packet does not claim full Stage E readiness. The strict/degraded
epoch/lifecycle/fault matrix still needs packet-local logs for each matrix row.
This is evidence for the CLI-owned setup/teardown and transport-overlap lane
only.

## Artifacts

- `artifacts/multicluster-transport-overlap.log`
- `artifacts/coord-postgres.log`
- `artifacts/remote-fast-postgres.log`
- `artifacts/remote-slow-postgres.log`
- `artifacts/manifest.md`

## Review Questions

- Is this sufficient runtime evidence to close the narrow "transport-overlap
  through `ecaz`" operator lane?
- Should the next fixture packet extend this same command with a first
  strict/degraded fault case, or should fault cases be split into one
  subcommand per fault family?
