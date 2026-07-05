# Task 50 Packet 014: SPIRE Snapshot Live Relation Facade

## Code Under Review

- Commit: `535b620249b0569d8a942c4ba5b2a5d260551912`
- Task: `plan/tasks/50-unsafe-structural-reduction.md`
- Packet: `reviews/task-50/014-spire-snapshot-live-relation-facade/`

## Scope

This packet completes the direct Task 50 pass for
`src/am/ec_spire/coordinator/snapshots.rs`, one of the top-15 residual unsafe
modules from the Task 50 planning map.

The change adds small live-index relation helpers and routes repeated snapshot
relation construction and relid reads through them. The refactor keeps the
existing relation lifetime and PostgreSQL ownership model intact while reducing
the number of local unsafe blocks in snapshot diagnostic paths.

No benchmark lane is claimed for this packet. The touched code is SPIRE
snapshot/debug inspection plumbing rather than a scoring, placement, or
candidate traversal hot path.

## Unsafe Block Count

Command:

`make unsafe-block-count PATHS='src/am/ec_spire/coordinator/snapshots.rs'`

| File | Planning Baseline | Packet Result | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/snapshots.rs` | 62 | 41 | -21 (-33.9%) |

This satisfies the Task 50 per-module target of at least a 30% reduction from
the post-Task-35 starting count.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_spire/coordinator/snapshots.rs'`:
  `62` before and `41` after.
- `rustfmt --edition 2021 --check src/am/ec_spire/coordinator/snapshots.rs`:
  passed with existing stable-rustfmt warnings about unstable config keys.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with existing warnings.
- `cargo test coordinator::snapshots --lib --no-default-features --features pg18`:
  built the test binary, then failed to launch outside PostgreSQL with
  `undefined symbol: CacheRegisterRelcacheCallback`.
- `cargo fmt --all --check`: still reports pre-existing formatting drift in
  files outside this slice; touched-file rustfmt check passed.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`:
  still blocked by the existing repo-wide clippy backlog.
- `git diff --check`: passed.

## Artifacts

See `artifacts/manifest.md`.

