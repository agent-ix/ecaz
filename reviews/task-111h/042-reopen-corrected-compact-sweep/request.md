# Task 111h / Packet 042 Review Request: Reopen Corrected Compact Sweep

This packet requests review for commit
`9fca7ea8a96f339f5fb6ba5f96b484d0168bcbee`
(`task111h: reopen corrected compact sweep`).

## Scope

Packet 041 rejected the prior final closeout as not evidence-backed for the
compact-format decision. This checkpoint updates the task tracker and task
index so 111h is no longer advertised as complete while the corrected compact
sweep remains open.

The status update preserves the earlier engineering evidence, but marks the
previous final table as stale and makes the missing decision gates explicit:

- RaBitQ-4 clip `{2,3,4}` coverage.
- RaBitQ-8 matched-recall comparison vs index f16 at best clip.
- Exact-dequant scoring, or equivalent compact-format fidelity lever.
- TurboQuant fidelity lever coverage.
- Matched-recall vs index f16 at recall `0.97` and `0.99`.
- Corrected 10k/50k/100k sweep before a final 1M closeout run.

## Validation

No tests or benchmarks were run for this checkpoint. The change is limited to
task/review workflow state under `plan/tasks/`.

## Review Focus

- Whether the reopened status accurately reflects packet 041.
- Whether the new follow-up checklist captures the missing work without
  discarding valid engineering evidence from packets 024-040.
