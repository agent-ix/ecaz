# Task 153: IVF RaBitQ dense-posting lane under the Task 143 defaults (bit-width sweep)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P2

## Why

Task 143 promoted the new IVF defaults (`dense_posting_blocks` auto→dense,
`turboquant_scorer=int8_approx`) on **TurboQuant evidence only** — the task
file does not mention RaBitQ. But Task 111a recorded that for RaBitQ **every
mode except rb1 spans pages, making the multi-page assembly copy the common
case**, and Task 135/142's posting-visit and drain-policy findings were
likewise measured on TQ. The RaBitQ bit-width family has never been benched
under the promoted default stack, so we do not know whether the dense lane is
a win, a wash, or a regression for rabitq2/4/8 — nor whether the Task 142
drain-policy fix behaves the same with RaBitQ payload strides.

## Scope

- A/B row vs dense posting layout on the IVF RaBitQ lane at 10k/50k/100k,
  bit-width sweep {1, 2, 4, 8}, under current production defaults
  (coalescing + typed views + drain policy as shipped).
- Report posting_visit / scratch_flush / scorer_batch stage counters and the
  width-bucket histogram alongside recall+latency+storage, so multi-page
  assembly cost is attributed, not guessed.
- If any bit-width regresses under dense, recommend a per-format gate (the
  `use_scratch_soa_batch_decode_for_format` seam at
  `src/am/ec_ivf/scan.rs:1827` is the natural place) rather than a default
  revert.

## Out of Scope (hard)

- No new posting format. No re-litigating the Task 111a page-spanning B
  decision (abandoned as dominated) or 111c zero-copy scatter (not promoted).

## Gate / Exit Criteria

- The 2×4 (layout × bit-width) matrix at 10k/50k/100k with stage counters,
  plus a keep/gate recommendation per bit-width. Closes when the matrix and
  recommendation land.
