---
type: ADR
id: ADR-083
title: "IVF TurboQuant Calibration Metadata"
status: PROPOSED
impact: Defines the Task 148 IVF-only persistence shape for TurboQuant per-coordinate calibration. Affects ADR-048, ADR-070, ADR-071, ADR-072, ADR-081, ADR-082, and ec_ivf page metadata.
date: 2026-07-05
---
# ADR-083: IVF TurboQuant Calibration Metadata

## Context

Task 148 evaluates a per-coordinate TurboQuant calibration profile for the
4-bit no-QJL lane. The correction changes how row codes are interpreted:

```text
encoded_coord = nearest_codebook((rotated_coord + shift[i]) * scale[i])
decoded_coord = codebook_value / scale[i] - shift[i]
```

Scan therefore needs the same `shift[]` and `scale[]` arrays that build used.
Keeping the arrays only in backend memory would make the index non-durable and
would let another backend silently score calibrated bytes as standard
TurboQuant. ADR-082 also rejects the old Task 86 measurement shortcut that
stored TQ+ arrays in `IvfPqCodebookTuple` and labeled them as PQ codebooks.

## Decision

The IVF Task 148 port SHALL persist TurboQuant calibration as an IVF-owned
metadata extension:

- add `turboquant_profile = 'standard' | 'tqplus'` as the explicit reloption;
- keep `standard` as the default and preserve existing `storage_format =
  'turboquant'` behavior;
- support `tqplus` only for the 4-bit no-QJL TurboQuant lane;
- bump the IVF metadata format version and metadata byte width;
- store a dedicated calibration head ItemPointer in metadata;
- store calibration arrays in dedicated `IvfTqCalibrationTuple` records, not
  `IvfPqCodebookTuple`;
- reject `tqplus` metadata if either array is missing, wrong-dimension,
  non-finite, or has a zero/non-finite scale.

The physical tuple chain is:

```text
metadata.turboquant_calibration_head -> shift tuple -> scale tuple -> INVALID

IvfTqCalibrationTuple:
  tag: 0x2D
  array_kind: 0 = shift, 1 = scale
  value_count: u16
  next_tid: ItemPointer
  values: value_count little-endian f32
```

For Task 148, a calibration tuple may reject dimensions whose array does not fit
on one page. The staged target lane is 1536 dimensions, where one f32 array is
approximately 6 KiB and fits in a normal PostgreSQL page. Multi-segment
calibration arrays can be added by a later task if a wider production dimension
requires them.

## Applicability

The profile applies to two IVF surfaces:

- posting storage when `storage_format = 'turboquant'`;
- index-side compact rerank payloads when `storage_format = 'coarse_rerank'`
  and `rerank_format = 'turboquant'`.

`storage_format = 'coarse_rerank'` postings remain the existing coarse RaBitQ
payload; only the TurboQuant rerank sidecar uses the calibration model.

## Compatibility

Existing indexes are not read as calibrated indexes. `turboquant_profile =
'standard'` writes no calibration head and scan uses the existing standard
TurboQuant scorer.

Because this project keeps only the current IVF metadata version readable, the
format bump is the compatibility boundary. A binary that does not know the new
metadata version rejects the index instead of interpreting calibrated payloads
as standard TurboQuant.

## Consequences

- Task 148 A/B suites can build and scan durable calibrated IVF indexes.
- The hot posting/rerank payload bytes remain 4-bit TurboQuant code bytes.
- Query prep owns the extra calibrated LUT/bias state; per-candidate scoring
  uses the calibrated epilogue.
- The metadata labels distinguish TurboQuant calibration from grouped-PQ
  codebooks, satisfying ADR-082.
- Insert support must load the persisted calibration model before encoding a
  new calibrated posting or sidecar payload.

