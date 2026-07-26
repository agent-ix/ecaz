---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
task: 38
packet: 008-apple-m5-closeout
status: review-requested
---

# Review Request: Task 38 Apple-M5 Closeout

Review corrected canonical status checkpoint `af908f44c` and the completion
audit in `artifacts/m5-closeout-audit.md`.

## Scope

This is a status/evidence checkpoint, not Task 38 closeout. It reconciles the
three M5-verifiable gaps identified by packet 005:

- the interrupt inventory and exact follow-up accounting are approved in
  packet 006;
- the cancellation mutation control is approved in packet 007; and
- the unrecovered-palloc negative control is approved in packet 007.

The canonical task now says the three previously identified M5-verifiable gaps
are complete. It keeps the authoritative `fault-full` aggregate as an
implementation gap and keeps its live provider, remote-socket, cgroup-v2 OOM,
no-`PANIC`, and cleanup gates open for designated Intel/Linux execution.

## Requested Review

Please verify:

- every original Task 38 objective and canonical criterion is accounted for;
- packet 005's three M5 gaps are supported by later approved evidence;
- the status does not convert source approval into runtime approval;
- no Intel/Linux, AWS, remote-host, CI, or nightly evidence is claimed; and
- the remaining aggregate-implementation and Intel/Linux gates are complete
  and precise.

No build, test, PostgreSQL workload, AWS, remote host, CI, or Intel execution is
requested for this status review.

## Response To `feedback/2026-07-26-01-reviewer.md`

### Finding 1 — `fault-full` omission

Accepted. The canonical status and audit no longer call the Linux matrix
exclusive or the only remaining boundary. They now identify a separate,
M5-source-reviewable implementation gap: assemble an authoritative
`make fault-full` operator that includes mutation/socket/cgroup surfaces and
sequences provider modes, restart/restore, exact markers/arm files, and
same-run slow baselines. Live designated-host execution remains a subsequent
gate. “Apple-M5 implementation complete” was narrowed to the three approved M5
evidence gaps and reviewed source slices.

### Finding 2 — exact Linux gates

Accepted. Both canonical status and audit now retain:

- all three DistANN codecs and applicable heap/index/WAL/temp paths, exact
  mode/path markers, accepted outcomes, recovery/shared postconditions, and a
  same-run measured slow delta;
- DistANN accepted clean reset, expected source identity,
  baseline-plus-latency threshold, and exact expected-source recovery;
- SPIRE validated healthy baseline, accepted reset error/degraded result,
  stable slow profile equality, timing threshold, and recovered profile
  equality;
- cgroup exact row-count equality and clean post-recovery stop; and
- the global postmaster no-`PANIC` and buffer/lock cleanup gates.

### Finding 3 — disarmed marker wording

Accepted. The audit now says “evidence,” then identifies the seven armed
rejection markers plus the final seven-fixture completion marker and approved
control flow as the disarmed-success evidence. It no longer claims seven
disarmed-success markers.
