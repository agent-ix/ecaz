# Artifact Manifest

- Implementation HEAD: `dfcbffd4e`
- Review-response HEAD: `631ec2940`
- Task bucket: `reviews/task-38/`
- Packet: `reviews/task-38/003-armed-spire-remote-socket-drill/`
- Capture date: `2026-07-25 America/Los_Angeles`
- Host: macOS arm64
- Fixture shape: planned isolated one-coordinator/three-participant native SPIRE
  multicluster; coordinator-only provider, exact first-participant named Unix
  socket
- Benchmark matrix: not applicable; this checkpoint changes fault-control and
  diagnostic behavior, not production index behavior

## `local-validation.log`

- Modified Rust files were formatted with stable `rustfmt`.
- `git diff --check` passed.
- `cargo check -p ecaz-cli` passed in 2m18s. It emitted the existing unused
  `LoadedDistributedPlacementConfig.path` warning and PostgreSQL-header C
  warnings.
- The focused CLI parser assertion was authored but not separately run; the
  production command surface compiled.

## Evidence Ceiling

This host cannot load `LD_PRELOAD` or run the Linux-built provider. There is no
live `fault=1` marker, exact-peer syscall trace, socket reset/delay measurement,
or cgroup OOM result in this packet. Those remain required Linux evidence.

## `2026-07-26-review-response-validation.log`

- Records scoped formatting and diff checks, production and test-configured CLI
  checks, the stable-profile unit-test type check, and the focused-test
  execution ceiling for the review-response commit.
