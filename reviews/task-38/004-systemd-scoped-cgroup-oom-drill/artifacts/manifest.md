# Artifact Manifest

- Implementation HEAD: `79af1107b`
- Task bucket: `reviews/task-38/`
- Packet: `reviews/task-38/004-systemd-scoped-cgroup-oom-drill/`
- Capture date: `2026-07-25 America/Los_Angeles`
- Host: macOS arm64
- Fixture shape: planned seven isolated one-index-per-table PG18 clusters,
  each with its postmaster, repeated AM-build workload, and resident pressure
  in one user systemd scope
- Storage: durable logs below the packet artifact directory; transient PG data
  below a separate target-local runtime directory
- Benchmark matrix: not applicable; this checkpoint changes fault-control and
  recovery diagnostics, not production index behavior

## `local-validation.log`

- Modified Rust files pass stable `rustfmt --check`.
- `git diff --check` passed.
- Final `cargo check -p ecaz-cli` passed in 11.77s after an earlier full
  incremental compile passed in 28.41s. Both emitted only the existing unused
  `LoadedDistributedPlacementConfig.path` warning and PostgreSQL-header C
  warnings.
- A full CLI rebuild completed in 8m20s. The new `cgroup-smoke` route parsed
  and dispatched, then returned the expected Linux-only error on macOS before
  artifact or runtime directory creation.
- `cargo fmt --all -- --check` was blocked by unrelated existing formatting
  drift across corpus, hardening mirror, AM, quantizer, storage, and fixture
  files. It made no changes.
- The focused CLI parser assertion was authored but not separately run after
  the successful production route dispatch.

## Evidence Ceiling

This host cannot execute cgroup v2 or a systemd user scope. There is no live
`Result=oom-kill`, AM workload marker, memory event, postmaster crash-recovery,
or seven-fixture result in this packet. Those remain required Linux evidence.
