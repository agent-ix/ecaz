---
head_sha: 183f76a7d44cf68df0df42db5cdd2448812b9deb
task: task-123
packet: reviews/task-123/021-post-ab-closeout
date: 2026-06-30
---

# Task 123 Packet 021 Artifact Manifest

- Packet type: status sync / closeout (no new measurement).
- Purpose: record the no-promote / re-scope closeout of the reopened Task 123
  multi-instance core-algorithm scope, implementing the packet 020 reviewer
  acceptance, and align the shipped prune default with that conclusion.
- Code change in the closeout commit: `ec_spire.pre_materialization_prune` GUC
  default flipped `true` → `false` in `src/am/ec_spire/options/mod.rs`
  (opt-in plumbing; unit tests unaffected via `#[cfg(test)]` override).

## Evidence Chain

- Communications datapoint (accepted):
  `reviews/task-123/017-multinode-communications-prune-ab/`
  and feedback `.../feedback/2026-06-29-02-reviewer.md`.
- Dedupe prune threshold fix (code LGTM):
  `reviews/task-123/018-dedupe-prune-threshold/`
  (commit `d2ffbdaa9`) and feedback `.../feedback/2026-06-29-01-reviewer.md`.
- Engaged-guard b2/b4 multi-instance A/B:
  `reviews/task-123/019-dedupe-prune-multinode-ab/`.
- Closeout request + acceptance:
  `reviews/task-123/020-post-ab-closeout-request/`
  and feedback `.../feedback/2026-06-30-01-reviewer.md`.
- Task 121 companion closeout:
  `reviews/task-121/030-multi-instance-closeout/`.
- Single-instance record retained:
  `reviews/task-123/008-completion-record/` (reviewer sign-offs
  `.../feedback/2026-06-27-01..03-reviewer.md`).

## Requirement Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| Recall stays at prune-off value under engaged b2/b4 prune | Packet 019: recall@10 = 1.0000 prune on and off, both surfaces | Satisfied |
| Communications attribution (bytes not the driver) | Packet 017 accepted; ~1540× payload delta with flat latency | Satisfied |
| Prune shown to be a latency lever | Packet 019: prune-on/off latency flat | Not demonstrated (no promotion) |
| Prune leaf-side engagement (rows dropped) captured | `truncated_candidate_row_count` absent; on/off structural counters byte-identical | Not measured — deferred to Task 131 |
| Shipped default matches no-promote conclusion | GUC default flipped to off; main default behavior unchanged | Satisfied |
| Closeout benchmark gate (behavior-changing merge) | Default-off ⇒ main default read path == pre-merge; nothing promoted; recall scales established in earlier packets | Satisfied (nothing promoted) |

## Non-Claims

No cross-network performance, no realistic payload transport claim, no prune
latency win, no measured prune engagement, no default SPIRE promotion.
