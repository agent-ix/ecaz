---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 015 multi-epoch publish (recover T4a predecessor)

Extends the publish path to a second epoch on the same index (FR-082:271-277,
604-614), building on packets 012 (decide) and 013 (recover first-epoch).

## Commit
- `d52b979f` — recover T4a predecessor swap + build_epoch parent binding.

Artifacts + provenance in `artifacts/manifest.md`.

## What changed
- **build_epoch** now binds the current `ec_distann_active_epoch` fingerprint as
  the build-spec/manifest `parent_fingerprint` (empty for a first epoch), so a
  successor's decision `parent == active` check holds.
- **recover_epoch_publish** handles a decision with a predecessor: CAS-swap the
  active pointer predecessor→successor, record the decision **Activated** (not
  Applied — predecessor retirement pending T4b), and open one Pending
  `ec_distann_predecessor_disposition` per predecessor participant binding
  (via an `INSERT ... SELECT` joining the decision's predecessor identity with
  the predecessor's bindings). First-epoch path still records Applied. The CAS
  guard and idempotent replay are preserved.

## Deliberately scoped / follow-ups
- **T4b** (mark each predecessor disposition Retired via `ec_distann_mark_epoch_retired`
  + `ec_distann_apply_epoch_retire`, then Applied) and the coordinator
  retirement endpoints (`retire`/`force_retire`/`abandon`) are the next slice —
  T4b needs constructing a `DistannRetireDecisionV1`.
- The test uses a real backend because multi-epoch spans commits (each epoch's
  session locks release on commit); it is rerun-safe and cleans up.

## Validation
- `cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) — pass at `d52b979f`.
- `cargo pgrx test pg18 test_distann_multi_epoch_publish` — 1/1 pass.

Leaving the request open for outside review.
