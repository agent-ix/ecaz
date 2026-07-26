# Review Request: Armed SPIRE Remote Socket Fault Drill

## Summary

This Task 38 checkpoint adds an executable SPIRE counterpart to the armed
DistANN remote-socket drill from packet 002.

`ecaz dev spire-multicluster local-multinode-pg18` now accepts
`--remote-socket-fault reset|slow`. The native runner:

1. starts only the coordinator with the Linux fault provider;
2. pins the first participant's real named PostgreSQL Unix socket as the exact
   peer;
3. prepares the physical SPIRE fixture while the provider is disarmed;
4. proves a disarmed production-read-profile baseline;
5. arms one production-read-profile query;
6. requires the configured delay or a reset result, plus an exact-peer
   `fault=1` marker;
7. removes the arm file; and
8. requires the next production-read-profile query to recover successfully.

Reset mode accepts either a clean SQL error or SPIRE's clean degraded result.
In both cases the exact provider marker, rather than the SQL outcome alone,
proves that the real participant transport was faulted.

`make fault-spire-remote-socket-smoke` exposes the native two-participant
diagnostic lane.

## Validation

See `artifacts/manifest.md` and `artifacts/local-validation.log`.

- modified Rust files formatted with stable `rustfmt`;
- `git diff --check` passed;
- `cargo check -p ecaz-cli` passed in 2m18s.

The current macOS arm64 host cannot load the Linux provider, so this packet
does not claim live socket-reset/slow execution. The new runner must be
executed on Linux before Task 38 closeout.

## Reviewer Focus

- Does the named Unix peer match the coordinator-to-participant PostgreSQL
  connection without catching coordinator control traffic?
- Is production-read-profile SQL the correct real SPIRE remote transport
  surface for baseline, fault, and recovery?
- Is accepting a clean degraded reset result sound when the exact provider
  marker independently proves the reset?
- Should slow mode use a stronger delta-over-baseline threshold after the first
  Linux measurement?

## Remaining Task 38 Work

- execute DistANN and SPIRE reset and slow modes on Linux and retain marker,
  runner, and syscall-trace evidence;
- execute provider-backed DistANN local EIO/ENOSPC/slow-disk;
- replace the cgroup plan with a live systemd-scoped OOM lane and execute it on
  a supported Linux host;
- obtain outside review for packets 002 and 003.
