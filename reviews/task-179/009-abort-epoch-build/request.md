---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 009 coordinator build abort

Implements `ec_distann_abort_epoch_build(index_regclass regclass, build_id uuid)
RETURNS void` (FR-078:354-360), the second of the coordinator trio (after
`build_epoch`, before `epoch_build_status`).

## Commit

- `bf909050b` — `ec_distann_abort_epoch_build`.

Artifacts + provenance in `artifacts/manifest.md`.

## Contract mapping (FR-078:354-360)

- **Idempotently abort every unpublished generation**: each participant binding
  is iterated; the local participant's generation is aborted through the
  idempotent `ec_distann_abort_epoch_handoff` primitive (a no-op when no
  generation exists yet, e.g. abort right after `begin_epoch_build`).
- **Remove the coordinator build gate**: the registration is moved to `Aborted`;
  the gate mask only matches `Registered/Building/Ready/Decided`, so this clears
  it. The test asserts the source gate mask directly: 0 before begin, non-zero
  while Registered, 0 after abort.
- **Release the session-level lock when held by the caller**, and **only after
  the gate-clearing transaction commits**: `schedule_session_lock_release_for_control`
  registers the release on the commit callback; an error/rollback preserves the
  committed gate and lock (no precommit release).
- **State rules**: absent or already-`Aborted` build id → no-op success;
  `Decided`/`Published` → `EC_BUILD_STATE` (cannot abort a decided build);
  `Registered`/`Building`/`Ready` → abortable. The registration row is locked
  `FOR UPDATE` under the control `ShareRowExclusiveLock` before the state read,
  serializing against concurrent begin/build/decide.

## Deliberately scoped

Remote participant abort is **not** yet driven — a multi-node roster
**fails closed** (`EC_BUILD_STATE: remote participant abort is not yet
implemented`) rather than silently skipping a remote generation. It lands with
the remote-transport slice alongside multi-node `build_epoch`. Single-node
(coordinator-in-roster) abort is complete and tested.

## Validation

- `cargo check` + strict `cargo clippy` (`pg18 pg_test`, `-D warnings`) — pass at
  `bf909050b`.
- `cargo pgrx test pg18 test_distann_abort_epoch_build_clears_gate_and_is_idempotent`
  — 1/1 pass.

Leaving the request open for outside review.
