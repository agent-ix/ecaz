# Packet 017 — coordinator recovery locks

Task bucket: `reviews/task-179/`; packet `017-coordinator-recovery-locks/`.
Head SHA: `3c24daf4109281cda32d37112ab2972f2924908e`.
Surface: coordinator build/decide/recover/abort session-lock lifecycle and T4
candidate validation. No benchmark or corpus data is involved.

## Artifacts

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | PG18 compile and strict clippy pass; two focused PG18 tests pass |

All tests use the local PG18 pgrx instance. The lock-guard test uses two real
loopback PostgreSQL backends; the multi-epoch test uses one real autocommit
backend across separate begin/build/decide/recover transactions. This is not a
measurement packet and has no shared-table or one-index-per-table benchmark
surface.
