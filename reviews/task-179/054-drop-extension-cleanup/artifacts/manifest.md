# Artifact manifest

- Head SHA: `0e8e8f6e823cae06b53a3dc0157c631a84dabb6d`
- Implementation commits: `a37e3f56f7ae7a3cecf9e1d7cef4c9d5868fd4df`,
  `0e8e8f6e823cae06b53a3dc0157c631a84dabb6d`
- Measured extension source: `9387f72b3209c751ba561f5f976f57954bd30b90`
  (`src/` and `sql/` have no changes between that SHA and packet head)
- Installed release extension SHA-256:
  `13e790e4a14993d49da6aa4d4d18c96d82bacd79f59a025bbaafddff8fcea360`
- Release runner git commit: `0e8e8f6e823cae06b53a3dc0157c631a84dabb6d`
- Release runner SHA-256:
  `76e49ac95c7ffcbb4044e010fb96a7aafdf2b1e20e298580ea0adbe1469758f2`
- Suite config SHA-256:
  `cbf8572cce3bb0d88126968cb7b66520294862d5dc49c256b733c8f6436db676`
- Task bucket / packet: `reviews/task-179/054-drop-extension-cleanup`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local PG18 physical DistANN destructive extension lifecycle
- Host: x86_64 Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM,
  Linux 6.18.33.2 WSL2, 1 TiB ext4 virtual disk
- PostgreSQL: 18.3, three loopback processes with
  `shared_preload_libraries=ecaz`
- Run: `2026-07-13T04:48:34-07:00` through
  `2026-07-13T04:48:40-07:00`
- Fixture: one coordinator/source that is owner 1 plus two remote owners; 90
  deterministic rows at dimension 4, graph degree 8, head cap 4096
- Storage format: WAL-logged distributed-control physical graph, frozen row,
  and unique-directory relations; one Published plus one Ready generation
- Rerank mode: exact frozen-row materialization from the physical owner
- Isolation surface: isolated source/control tables and two generations per
  owner; no shared-table or corpus benchmark surface

This is correctness/lifecycle evidence, not a performance measurement. It
uses no corpus/query TSV and makes no latency, recall, or storage-promotion
claim.

## Commands

Exact-commit release runner build:

```text
cargo build --release -p ecaz-cli
```

Suite config audit:

```text
target/release/ecaz bench suite audit \
  --config reviews/task-179/054-drop-extension-cleanup/artifacts/cleanup-suite.json \
  --log-file reviews/task-179/054-drop-extension-cleanup/artifacts/audit-final.log
```

Canonical cleanup suite:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/054-drop-extension-cleanup/artifacts/cleanup-suite.json \
  --continue-on-error \
  --log-file reviews/task-179/054-drop-extension-cleanup/artifacts/suite-run.log
```

Final status:

```text
target/release/ecaz bench suite status \
  --manifest reviews/task-179/054-drop-extension-cleanup/artifacts/cleanup-run/suite-manifest.json \
  --log-file reviews/task-179/054-drop-extension-cleanup/artifacts/status.log
```

Focused static/parser validation:

```text
cargo check -p ecaz-cli
cargo test -p ecaz-cli distann_drop_extension_cleanup_is_structured -- --nocapture
```

## Artifact index

- `cleanup-suite.json`: canonical one-step, three-owner cleanup config with
  topology and drop-cleanup thresholds.
- `cleanup-run/suite-manifest.json`: exact expanded command, runner commit,
  duration, exit status, expected artifacts, and 2/2 passing thresholds.
- `cleanup-run/results.jsonl`: normalized Ready/Published topology, serving,
  remote-owner, cleanup, and topology-gate rows.
- `cleanup-run/distann-local-multinode.log`: complete decision-grade fixture
  log, including per-owner pre/post cleanup counts.
- `cleanup-run/distann-multinode-summary.log`: compact immutable topology and
  cleanup summary.
- `release-cli-build.log`: exact-SHA release runner build.
- `suite-run.log`: suite driver output.
- `audit-final.log`: final config audit.
- `status.log`: final 1/1 completion with zero missing/stale artifacts.
- `cargo-check.log`: scoped CLI compile validation.
- `parser-test.log`: focused normalized-row parser regression.

PostgreSQL server logs and `target/task179-drop-extension-cleanup/` are not
committed.

## Key cited results

```text
status: completed=1 failed=0 missing_artifacts=0 stale=0
thresholds: 2/2 pass

node=1 ready_before=1 published_before=1 hidden_before=6
  hidden_after=0 extension_after=0 post_drop_dml_rows=1
node=2 ready_before=1 published_before=1 hidden_before=6
  hidden_after=0 extension_after=0 post_drop_dml_rows=1
node=3 ready_before=1 published_before=1 hidden_before=6
  hidden_after=0 extension_after=0 post_drop_dml_rows=1

Ready and Published topology records=33/24/33,
  non_owned=0/0/0, orphans=0/0/0
physical_topology_gate pass=true owners=3 remote_verified=2 source_rows=90
```
