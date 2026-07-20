---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 016 P1 remediation (decide/abort hole + privilege gap)

Remediates the two P1s from the packet-012 outside review (`2026-07-11-01-reviewer.md`),
which also carried the cross-packet privilege P1 flagged on 007/009/011/013.

## Commit
- `1311d10c` — decide/abort state fix + 7-endpoint privilege fix.

Artifacts + provenance in `artifacts/manifest.md`.

## P1-1 — decide↔abort lifecycle wedge: FIXED
- `ec_distann_decide_epoch_publish` now locks the registration row `FOR UPDATE`,
  requires state `Ready`, and CASes `Ready → 'Decided'` atomically with the
  decision insert (rows-affected checked). `'Decided'` — declared in the
  bootstrap CHECK / gate mask / slot check but previously set nowhere — is now
  set, so the gate stays active through the decision and single-flight is honest.
- `ec_distann_recover_epoch_publish` transitions `'Decided' → 'Published'`.
- `ec_distann_abort_epoch_build` rejects when a Pending/Activated decision exists
  (and `'Decided'` is already outside abort's abortable set).
- Both poisoned sequences are now rejected and tested; the decided build's
  generation is not destroyed, and it remains recoverable to Published.

## P1-2 — privilege gap on 7 endpoints: FIXED
`distann_internal_privileges` now covers `build_epoch`, `decide_epoch_publish`,
`recover_epoch_publish`, `abort_epoch_build`, `epoch_build_status`,
`generation_topology`, and `epoch_topology` with SECURITY DEFINER + pinned
`search_path` + `REVOKE ALL FROM PUBLIC` (FR-082:230-235).

## Acknowledged, not in this packet (follow-ups)
- 012 P2-1/P2-2: the spec forbids build→decide→recover in one transaction
  (FR-082:281-283) — the multi-epoch/decide-abort tests already use a real
  backend across commits; the single-node pipeline test still runs in one
  transaction and should be split. Crash-window fault GUCs, and the recovery
  locking protocol (source ShareLock → control SRE → revalidate), are open.
- 011 P2-1 (session-lock/recapture contract in build_epoch), 011 P2-2 / 009 P2
  (non-blocking `EC_BUILD_BUSY` via conditional acquire), and the 013/007/etc
  P2s remain open and are triaged for the next remediation slice.
- P3s (error-code overloading, T3 calling `generation_topology`, advisory-lock
  single-flight) noted.

## Validation
- `cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) — pass at `1311d10c`.
- `cargo pgrx test pg18` — `test_distann_decide_abort_guards`,
  `test_distann_build_epoch_single_node`, `test_distann_multi_epoch_publish` all pass.

Leaving the request open for outside review.
