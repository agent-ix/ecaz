# Review Request: Armed DistANN Remote Socket Fault Drill

## Summary

This Task 38 checkpoint turns the exact-peer socket provider from startup-only
scaffolding into an executable DistANN multicluster drill.

The provider now accepts an optional arm file. When configured, the provider
remains disarmed until that file exists, so a long-lived PostgreSQL coordinator
can complete physical topology setup without consuming the intended remote
fault. Removing the file disarms subsequent operations without restarting the
postmaster. Provider restart/restore clears the new environment variable, and
the Linux-only provider test covers disarmed success followed by armed
injection.

`ecaz dev distann-multicluster local-multinode-pg18` now accepts
`--remote-socket-fault reset|slow`. The runner:

1. starts only the coordinator with the Linux provider;
2. pins the first real remote owner as `tcp:127.0.0.1:PORT`;
3. completes the physical generation while disarmed;
4. proves a disarmed remote-owner baseline;
5. arms one real distributed owner query;
6. requires reset failure or the configured delay and an exact-peer `fault=1`
   marker;
7. removes the arm file; and
8. requires the existing subsequent remote-owner materialization query to
   recover successfully.

`make fault-distann-remote-socket-smoke` exposes the two-owner diagnostic lane.

## Validation

See `artifacts/manifest.md` and `artifacts/local-validation.log`.

- modified Rust files formatted with `rustfmt`;
- `git diff --check` passed;
- `cargo check -p ecaz-cli` passed;
- `cargo test -p ecaz-fault-injection` passed, 9 tests on macOS (the new
  arm-file runtime test is Linux-gated);
- strict Clippy remains blocked by pre-existing repository findings recorded in
  the artifact.

The current macOS arm64 host cannot load the Linux provider, so this packet
does not claim live socket-reset/slow execution. The new runner must be
executed on Linux before Task 38 closeout.

## Reviewer Focus

- Is the arm-file gate safe and narrow enough for a postmaster-inherited
  provider environment?
- Does starting only coordinator node 1 with an exact node-2 TCP peer prevent
  participant/control traffic from being faulted?
- Does the baseline → arm → marker → disarm → recovery sequence prove real
  DistANN owner/payload behavior rather than a synthetic socket?
- Should socket-slow use a stronger delta-over-baseline threshold after the
  first Linux measurement?

## Remaining Task 38 Work

- execute reset and slow modes on Linux and retain their marker, runner, and
  syscall-trace evidence;
- add the corresponding real SPIRE remote SQL drill;
- execute provider-backed DistANN local EIO/ENOSPC/slow-disk;
- replace the cgroup plan with a live systemd-scoped OOM lane and execute it on
  a supported Linux host;
- obtain outside review for this checkpoint.
