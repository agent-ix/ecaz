# Review Request: Apple M5 Task 38 Handoff

## Summary

This packet records the Task 38 implementation/source-review boundary reached
on the current Apple M5 host. It does not claim that every canonical validation
or exit criterion runnable on M5 has already been demonstrated.

The canonical task status now distinguishes source/implementation completion
from live Intel/Linux execution. DistANN is a first-class fifth AM with
RaBitQ, TurboQuant, and grouped-PQ fixtures; the exact-peer DistANN and SPIRE
socket drills are executable; the seven-fixture systemd-cgroup OOM operator is
implemented; and all historical and current source-review findings are closed.

Packets 001 through 004 have outside approval at the implementation/source
level. That approval is intentionally not treated as behavioral closeout.

## Apple M5 Completion Boundary

Completed and evidenced here:

- five-AM/seven-fixture fault model;
- live local PG18 DistANN coverage for all three supported codec fixtures;
- exact-peer socket provider and armed DistANN/SPIRE diagnostic runners;
- executable cgroup OOM operator with recovery and evidence-path safeguards;
- historical cleanup findings;
- M5 production and test-configured type checking; and
- outside review of packets 001 through 004, including all response cycles.

Held open for the designated Intel/Linux host:

- provider-backed DistANN EIO, ENOSPC, and measured slow-disk execution;
- live DistANN TCP and SPIRE named-Unix reset/slow execution with provider
  markers and syscall traces; and
- all seven `systemd-run --user --scope` cgroup-v2 OOM cases.

Canonical validation still open on M5:

- a durable cancellation-lane mutation control for the deliberately introduced
  invalid-buffer-style bug criterion; and
- a durable resource/palloc negative control proving the smoke detects an
  unrecovered accumulator allocation failure; and
- a complete long-running-loop `CHECK_FOR_INTERRUPTS` inventory (the current
  documentation is useful but not exhaustive).

No AWS, remote-host, or GitHub Actions execution was used for this handoff.
No CI/nightly eligibility is claimed.

## Validation

See `artifacts/manifest.md` and `artifacts/completion-audit.md`.

- `cargo check -p ecaz-cli --tests` passed on Apple M5 in 12m01s for the final
  SPIRE baseline-health response;
- scoped formatting and `git diff --check` passed;
- the focused monolithic CLI unit-test target did not finish linking within
  the bounded M5 validation window and is explicitly not claimed as a pass;
- packet 003 final re-review approved implementation `aea65a78f`; and
- the canonical task remains open for the architecture-specific runtime
  evidence and the M5-verifiable validation gaps listed above.

## Reviewer Focus

- Does the completion audit preserve every original Task 38 requirement?
- Is the M5-versus-Intel/Linux evidence boundary stated without converting
  source review into a runtime claim?
- Is keeping Task 38 open the correct status?
