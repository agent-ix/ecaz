# Artifact manifest

- Head SHA: `e6e03dfc21350275c22fa5113222606ab0d37a43`
- Implementation commit: `e6e03dfc2` (`test(distann): exercise physical publish fault windows`)
- Release runner git commit: `e6e03dfc21350275c22fa5113222606ab0d37a43`
- Release runner SHA-256: `db3ad9151a9de3928235f8998229ca46917ba22ac2157ee2cef7b3f97b1a735f`
- Suite config SHA-256: `3e3931565fda83b763a2c7a92b56c9c32bee07b2899507691e8692f6ad876ec3`
- Task bucket / packet: `reviews/task-179/053-physical-publish-fault-windows`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local PG18 physical DistANN lifecycle fault fixture
- Host: x86_64 Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM,
  Linux 6.18.33.2 WSL2, 1 TiB ext4 virtual disk
- PostgreSQL: 18.3, three loopback processes with
  `shared_preload_libraries=ecaz`
- Run: `2026-07-13T04:21:49-07:00` through
  `2026-07-13T04:21:57-07:00`
- Fixture: one coordinator/source that is also owner 1 plus two remote owners;
  90 deterministic rows at dimension 4, graph degree 8, head cap 4096
- Storage format: WAL-logged distributed-control physical graph, row, and
  directory relations
- Rerank mode: exact frozen-row materialization from the physical owner
- Isolation surface: isolated source/control tables and one generation per
  physical owner; no shared-table or corpus benchmark surface

This is correctness/fault evidence, not a performance measurement. It uses no
corpus/query TSV and reports no benchmark latency or storage verdict.

## Commands

Exact-commit release runner build:

```text
cargo build --release -p ecaz-cli
```

Suite config audit:

```text
target/release/ecaz bench suite audit \
  --config reviews/task-179/053-physical-publish-fault-windows/artifacts/fault-suite.json \
  --log-file reviews/task-179/053-physical-publish-fault-windows/artifacts/audit-final.log
```

Canonical fault suite:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/053-physical-publish-fault-windows/artifacts/fault-suite.json \
  --continue-on-error \
  --log-file reviews/task-179/053-physical-publish-fault-windows/artifacts/suite-run.log
```

Final status:

```text
target/release/ecaz bench suite status \
  --manifest reviews/task-179/053-physical-publish-fault-windows/artifacts/fault-run/suite-manifest.json \
  --log-file reviews/task-179/053-physical-publish-fault-windows/artifacts/status.log
```

Focused static/parser validation:

```text
cargo check -p ecaz-cli
cargo test -p ecaz-cli distann_physical_topology_and_gate_are_structured
```

## Artifact index

- `fault-suite.json`: canonical one-step physical fault config with four
  required thresholds.
- `fault-run/suite-manifest.json`: exact expanded command, runner SHA, duration,
  exit status, expected artifacts, and 4/4 passing threshold results.
- `fault-run/results.jsonl`: normalized Ready/Published topology and all three
  fault/recovery `drill_outcome` rows.
- `fault-run/distann-local-multinode.log`: complete decision-grade fixture log.
- `fault-run/distann-multinode-summary.log`: compact topology, serving, fault,
  recovery, and remote-row summary.
- `release-cli-build.log`: exact-SHA release runner build.
- `suite-run.log`: suite driver output.
- `audit-final.log`: final config audit.
- `status.log`: final 1/1 completion with zero missing/stale artifacts.
- `cargo-check.log`: scoped CLI compile validation.
- `parser-test.log`: focused normalized-row parser regression.

PostgreSQL server logs, run directories, precommit attempts/audits, and
operational polling output are not committed.

## Key cited results

```text
status: completed=1 failed=0 missing_artifacts=0 stale=0
thresholds: 4/4 pass

participant_down_partial pass=true decision=Pending registration=Decided
  active_count=0 local_state=Ready remote_acked_state=Published unavailable_node=3
post_ack_pre_pointer pass=true decision=Pending registration=Decided
  active_count=0 owner_states=Ready,Published,Published
idempotent_recovery pass=true decision=Applied registration=Published
  active_count=1 owner_states=Published,Published,Published

Published topology records=33/24/33, non_owned=0/0/0, orphans=0/0/0
physical_topology_gate pass=true owners=3 remote_verified=2 source_rows=90
```
