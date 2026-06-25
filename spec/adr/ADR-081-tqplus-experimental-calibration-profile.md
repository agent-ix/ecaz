---
type: ADR
id: ADR-081
title: "TQ+ as an Experimental TurboQuant Calibration Profile"
status: PROPOSED
impact: Affects Task 89, IVF TurboQuant build/scan metadata, TurboQuant query preparation, FR-013, FR-038, ADR-070, ADR-071, and ADR-072.
date: 2026-06-25
---
# ADR-081: TQ+ as an Experimental TurboQuant Calibration Profile

## Context

Task 86 found a promising TurboQuant+ calibration result on IVF, but the
production rollout was reverted before Task 86 slim closeout. Task 89 owns the
next build-and-measure cycle.

There are two separate questions that should not be collapsed:

- **Algorithm family:** TQ+ is TurboQuant with a learned calibration transform,
  not a separate quantizer family like RaBitQ or grouped PQ.
- **Durable reader shape:** if TQ+ adds metadata or per-vector scalars that old
  TurboQuant readers cannot safely interpret, the index still needs an explicit
  compatibility discriminator.

The old Task 86 prototype used a separate `turboquant_tqplus` storage format
tag. That exact shape is no longer available as-is: IVF now uses storage-format
tag `4` for `coarse_rerank`. More importantly, promoting a new public
`storage_format` before re-evaluating QJL and other TurboQuant modes would make
the operator surface more durable than the evidence.

## Decision

Phase 1 of Task 89 SHALL build TQ+ as an **experimental calibration profile of
TurboQuant on IVF**, not as a new public storage format.

The implementation direction is:

- `storage_format = 'turboquant'` remains the quantizer-family selection.
- TQ+ is selected by an experimental/internal calibration option, tentatively
  `turboquant_calibration = 'tqplus_experimental'`.
- The option is IVF-only until Task 89 evidence justifies broader AM work.
- The public `storage_format = 'turboquant_tqplus'` spelling is not introduced
  in Phase 1.
- The final public spelling is deferred. It may become a TurboQuant option, a
  separate storage format, or a deprecated alias only after the IVF evidence
  covers the relevant TurboQuant scoring modes.

## What Changes

TQ+ changes the TurboQuant build/query/scoring pipeline in calibrated space:

1. During IVF build, fit calibration metadata from the deterministic training
   sample after the existing TurboQuant rotation.
2. Encode database vectors through that calibration before packing TurboQuant
   codes.
3. Persist calibration metadata with the IVF index.
4. Persist a per-vector renormalization scalar only if the Phase 1 measurement
   shows that it is required for recall or score-error quality.
5. Prepare queries through inverse calibration and bias handling before building
   the TurboQuant scorer state.
6. Score with the existing TurboQuant scorer family wherever possible.

This is deliberately not limited to the no-QJL 4-bit lane. No-QJL 4-bit is a
fast and convenient first cell, but Task 89 Phase 1 must also evaluate the
currently reachable QJL/gamma-aware TurboQuant path and any existing IVF
TurboQuant scoring modes that can be measured without inventing a new bit-mode
surface.

## Compatibility Rules

The experimental profile must not silently create an index that an ordinary
TurboQuant reader interprets as uncalibrated TurboQuant.

Until the final durable shape is selected, one of these safeguards is required:

- a metadata version or feature flag that rejects unknown TQ+ calibration
  indexes on unsupported builds; or
- a length-prefixed optional metadata block following ADR-070, with strict
  reject-unknown behavior for the TQ+ calibration section.

Existing `storage_format = 'turboquant'` indexes remain uncalibrated by default
and keep their current reader behavior.

## Measurement Commitments

Task 89 Phase 1 evidence must compare uncalibrated TurboQuant against TQ+ on IVF
using `ecaz bench suite`:

- DBPedia 10k / 50k / 100k.
- Recall, latency, and storage.
- No-QJL 4-bit plus the reachable QJL/gamma-aware TurboQuant fixture.
- Query preparation time where available.
- Calibration metadata bytes and any per-vector scalar bytes separately from
  packed code bytes.
- Insert drift at 10%, 25%, and 50% post-build inserts against a full-rebuild
  baseline before any production promotion.

If the QJL or non-default TurboQuant mode cannot be measured with the current
IVF harness, Phase 1 must record that as an implementation gap instead of
promoting based only on no-QJL 4-bit.

## Promotion Gate

After IVF Phase 1:

- If TQ+ improves recall or score error without unacceptable latency/storage
  cost across the measured TurboQuant modes, Task 89 may propose the public
  durable shape.
- If TQ+ only wins on one narrow mode, keep it experimental and do not port it
  across AMs.
- If TQ+ regresses QJL or insert drift, stop and document the boundary before
  any SPIRE/HNSW/DiskANN work.

## Alternatives Considered

### New Public `turboquant_tqplus` Storage Format Immediately

Rejected for Phase 1. It is clear to operators, but it makes a durable public
format decision before the QJL and drift evidence exists. It also conflicts with
the old tag-4 assumption because IVF tag `4` now means `coarse_rerank`.

### Hidden Fork with No Reloption

Rejected. It would make benchmarking possible, but it would not answer how TQ+
slots into the operator model. The experimental option is intentionally close to
the likely public option shape while still avoiding promotion.

### No-QJL 4-bit Only

Rejected. No-QJL 4-bit was adopted primarily for speed. Task 89 is explicitly a
reevaluation of whether TQ+ changes the quality/speed trade-off across
TurboQuant modes, so QJL/gamma-aware measurement must stay in scope.

## Related Decisions

- ADR-070: On-disk forward-compat encoding convention.
- ADR-071: Unified quantizer interface across access methods.
- ADR-072: Index-local quantized codec adapters.
- ADR-076: Universal block kernel pattern.
- Task 89: TurboQuant TQ+ validation.
