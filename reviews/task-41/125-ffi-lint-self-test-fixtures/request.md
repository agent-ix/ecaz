# Task 41 FFI lint self-test fixtures

## Code Under Review

- Code commit: `d2a0c82e3fb2eda012388be51bbc24990746e56a`
- Branch: `task41-inv1-inv3-followups`
- Task: `plan/tasks/41-ffi-safety-boundary.md`

## Summary

This checkpoint adds built-in verifier fixtures to `scripts/ffi_lint.py` and
runs them from `make ffi-lint`.

The self-test proves the Task 41 raw-resource lint rejects deliberately bad
fixtures for:

- raw buffer pin/lock API usage outside `src/storage/buffer_guard.rs`,
- raw `LWLockAcquire` usage outside `src/storage/lock_guard.rs`,
- an unadopted `read_stream_next_buffer` result.

It also proves allowed fixtures remain clean when raw APIs appear in the owning
wrapper modules and when a read-stream buffer is locally adopted by
`PinnedBufferGuard`.

## Safety Effect

This strengthens invariant #3 enforcement by making the lint lane prove its
own negative cases before scanning production code. A future regression that
weakens the raw-resource boundary checks should fail `make ffi-lint` even if
production code happens not to exercise that exact pattern at the time.

Invariant #2 is intentionally out of scope for this packet.

## Review Focus

- Confirm that the negative fixtures cover the resource-boundary checks added
  in packet 123.
- Confirm that the allowed wrapper/adoption fixtures are narrow enough and do
  not mask production-code violations.
- Confirm that `make ffi-lint` is still a cheap PR-tier lane with the added
  self-test.

## Validation

Packet-local validation details are recorded in `artifacts/manifest.md`.

