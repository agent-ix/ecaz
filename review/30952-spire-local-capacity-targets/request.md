# Review Request: SPIRE Local Capacity Targets

Code checkpoint: `87353dd8` (`Publish SPIRE local capacity targets`)

## Scope

- Adds `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md`.
- Publishes the Phase 12.9 local production-readiness smoke target profile:
  - maximum ready remotes per coordinator query: `8`;
  - maximum remote leaf PIDs per coordinator query: `256`;
  - maximum selected PIDs per remote node: `64`;
  - maximum concurrent distributed-read coordinator sessions: `1`;
  - maximum concurrent remote-search dispatches across coordinator backends:
    `8`;
  - maximum concurrent remote-search dispatches per remote node: `1`;
  - maximum concurrent coordinator-routed writer workloads: `1`;
  - maximum concurrent work per remote node: one read dispatch or one prepared
    write branch.
- Documents required nonzero remote-search budget GUCs for local smoke packets
  and the expected strict/degraded overload behavior.
- Updates `docs/SPIRE_LOCAL_READINESS.md` so local smoke packets cite the active
  capacity profile and cannot raise targets without packet-local benchmark or
  contention logs.
- Marks the Phase 12.9 capacity-target row complete.

## Validation

- `git diff --check 87353dd8^ 87353dd8`

Packet-local log is under `artifacts/`; see `artifacts/manifest.md` for the
command and result line.

## Review Focus

- Confirm the target profile satisfies the Phase 12.9 row without overclaiming
  AWS/RDS or product-scale capacity.
- Confirm the global dispatch target of `8` plus per-node target of `1` is
  coherent with one fully fanned-out eight-remote local query and one in-flight
  read dispatch per remote.
- Confirm the conservative one-at-a-time read/write workload limits are
  acceptable while the benchmark, write cancellation, placement contention, and
  async write-dispatch rows remain open.
