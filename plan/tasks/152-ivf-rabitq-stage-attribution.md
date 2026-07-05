# Task 152: IVF RaBitQ per-stage latency attribution (Task 133 equivalent for the RaBitQ lane)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P2

## Why

Task 133's per-stage attribution (scorer 46% / posting visit 42% / topk_collect
8% / dedup+heap 3% at 100k) was measured on the TurboQuant no-QJL 4-bit lane
only, and the entire Task 133-145 optimization wave that followed it was
TQ-only. **No equivalent attribution exists for the IVF RaBitQ lane.** RaBitQ's
balance is structurally different — popcount/arithmetic kernels instead of a
LUT, a bit-width axis (1/2/4/8) with different payload strides and different
kernel routings (bits=1 block32, bits=2 multi-bit block, bits=4/8 pair
estimator per `src/am/ec_ivf/quantizer.rs:629-651`), and a heavier rerank
share at high recall. Ranking RaBitQ follow-up work (Tasks 154/155/156/158)
by instinct instead of a profile violates the repo's own
"assumptions must be confirmed by facts" rule.

## Scope

- Reuse the Task 133 stage-timer/profiler harness (extend `ecaz bench` stage
  counters if a RaBitQ-specific stage is missing — no ad hoc glue).
- Attribute IVF RaBitQ query latency at 10k/50k/100k for at least bits=1 and
  bits=4 (add bits=8 if cheap): scorer vs posting visit vs topk collect vs
  dedup/heap vs rerank (source/f32 default), under current production defaults.
- Deliver a ranked shortlist of follow-up targets with rough Amdahl ceilings,
  explicitly mapping each to the already-filed kernel tasks (154/155/156/157/158)
  or marking them not-worth-it.

## Out of Scope (hard)

- No optimization changes in this task; measurement + direction only.

## Gate / Exit Criteria

- A committed per-stage breakdown at 10k/50k/100k for ≥2 bit-widths with the
  method recorded packet-local, plus the ranked shortlist. Closes when the
  breakdown + shortlist land.
