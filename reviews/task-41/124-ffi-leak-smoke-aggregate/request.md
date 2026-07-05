# Task 41 FFI leak smoke aggregate

## Code Under Review

- Code commit: `8f2e6acdb466fa495de333d36d9ae7df143ab7c7`
- Branch: `task41-inv1-inv3-followups`
- Task: `plan/tasks/41-ffi-safety-boundary.md`

## Summary

This checkpoint adds the Task 41 `make ffi-leak-smoke` aggregate target. The new
target composes the existing fault-smoke lanes from Task 38:

- `fault-mem-smoke`
- `fault-cancel-smoke`
- `fault-timeout-smoke`
- `fault-lock-smoke`
- `fault-resource-smoke`

The aggregate gives invariant #1 a named leak-smoke validation surface without
duplicating the underlying fault-injection harness.

## Safety Effect

`ffi-leak-smoke` now exercises the existing memory, cancellation, timeout,
lock-timeout, and resource dry-run smoke surfaces through one reviewable make
target. The inherited `FAULT_SMOKE_FLAGS ?= --dry-run` default keeps target
wiring validation cheap and side-effect-light unless a caller explicitly opts
into live fault-smoke execution.

Live fault-smoke execution was not run in this turn.

## Review Focus

- Confirm that the aggregate includes the right Task 41 leak-smoke lanes.
- Confirm that keeping the inherited dry-run default is the right behavior for
  the make target.
- Identify any additional fault-smoke lane that should be part of invariant #1
  closeout.

## Validation

Packet-local validation details are recorded in `artifacts/manifest.md`.
