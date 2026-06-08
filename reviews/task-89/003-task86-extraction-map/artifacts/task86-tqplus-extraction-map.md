# Task 89 TQ+ Extraction Map From Reverted Task 86

Date: 2026-06-08

Head inspected: `0bdc8a025`

Preserved commits inspected:

- `e0ae9fe7d` — initial IVF TQ+ measurement profile.
- `c7e85e8ac` — validation and production naming cleanup for the Task 86
  measurement branch.
- `55e492899` — IVF page coverage for the TQ+ delta gate.

## Reusable Shared Math

The reusable TQ+ core is in the `src/quant/prod.rs` diff from
`e0ae9fe7d` and hardened by `c7e85e8ac`.

Reusable APIs to reintroduce under production names:

- `TqPlusCalibration { shift, scale }`
- `TqPlusNoQjl4BitEncoded { mse_packed, renorm }`
- `PreparedTqPlusNoQjl4BitQuery { lut, bias }`
- `ProdQuantizer::fit_tqplus_calibration`
- `ProdQuantizer::encode_tqplus_no_qjl_4bit`
- `ProdQuantizer::prepare_ip_query_tqplus_no_qjl_4bit`
- `ProdQuantizer::score_tqplus_no_qjl_4bit_from_parts`
- `ProdQuantizer::score_tqplus_no_qjl_4bit_from_prepared_unrenormalized_parts`

Keep the Task 86 hardening from `c7e85e8ac`:

- calibration methods are production-visible, not `*_for_test`;
- fitted beta quantiles are cached;
- calibration `scale` values must be finite and non-zero;
- APIs assert/validate the no-QJL 4-bit lane.

## Shared Math Behavior

The preserved code implements this pipeline:

1. Normalize the source vector.
2. Rotate through the existing TurboQuant SRHT signs.
3. Fit per-coordinate `shift` and `scale` from the training sample.
4. Encode each source by applying `shift` and `scale` in rotated space before
   standard 4-bit MSE assignment.
5. Store the source norm as `renorm`.
6. Prepare each query by folding inverse calibration into the per-coordinate
   LUT and a scalar bias.
7. Score from `(renorm, mse_packed)` as:
   `renorm * (sum(lut[dim, code[dim]]) + bias)`.

This is AM-agnostic and should live in `src/quant/prod.rs`.

## Code To Avoid Re-Landing Verbatim

The Task 86 AM integration should not be cherry-picked directly:

- It adds `StorageFormat::TurboQuantTqPlus = 4`.
- It accepts `storage_format = 'turboquant_tqplus'` and `'tqplus'`.
- It stores IVF calibration arrays in `metadata.pq_codebook_head`.
- It labels calibration tuple storage through `IvfPqCodebookTuple`.

ADR-076 rejects that as the preferred production API. New code should expose:

```sql
WITH (
  storage_format = 'turboquant',
  turboquant_profile = 'tqplus'
)
```

Tag 4 should only reappear as an IVF legacy-reader compatibility path if the
Task 86 packet-011 fixtures need direct decode coverage.

## IVF Rewrite Map

Old Task 86 locations:

- `src/am/ec_ivf/options.rs`
  - Old: top-level `StorageFormat::TurboQuantTqPlus`.
  - New: add separate `TurboQuantProfile::Standard | TqPlus`.
- `src/am/ec_ivf/quantizer.rs`
  - Old: `IvfQuantizerProfile::TurboQuantTqPlus`.
  - New: keep top-level `IvfQuantizerProfile::TurboQuant` and add nested
    TurboQuant profile, or add a profile field to `IvfQuantizer`.
- `src/am/ec_ivf/build.rs`
  - Old: `train_tqplus_model` only when storage format is tag 4.
  - New: train only when storage format is TurboQuant and profile is TQ+.
- `src/am/ec_ivf/insert.rs`
  - Old: branch on `metadata.storage_format == TurboQuantTqPlus`.
  - New: branch on metadata profile/calibration flag.
- `src/am/ec_ivf/scan.rs`
  - Old: scan opaque caches `IvfTqPlusModel`.
  - New: same cache shape is still useful, but keyed by profile metadata.

The old scan cache is safe to reuse conceptually: calibration is loaded once
into scan opaque, not per posting.

## Validation To Preserve

From `c7e85e8ac`:

- shape mismatch rejects;
- non-finite `shift` rejects;
- non-finite or near-zero `scale` rejects;
- unsupported non-no-QJL-4-bit lanes reject;
- dispatch requires a model and matches direct shared scoring.

From `55e492899`:

- metadata header/version decode coverage;
- quant_bits decode coverage;
- tuple tag/length/value validation coverage;
- posting gamma finite checks.

For ADR-076, these tests need updated names and assertions that refer to
TurboQuant profile metadata rather than tag 4 as the production write path.

## First Code Slice After ADR Approval

The smallest useful implementation commit after Phase 1 approval should be:

1. Reintroduce the shared TQ+ math in `src/quant/prod.rs`.
2. Add unit tests for:
   - calibration shape;
   - non-finite/zero scale rejection through a validation helper;
   - no-QJL 4-bit lane requirement;
   - score equivalence from encoded struct vs. raw parts.
3. Do not touch AM reloptions or page layouts in that first commit.

That keeps the first code review focused on math correctness and avoids mixing
storage-format policy with quantizer behavior.

## Open Questions For The Port

- Should shared validation be a `TqPlusCalibration::validate(dim)` method or
  remain AM-local? Prefer shared validation so AMs cannot drift.
- Should `renorm` continue to live in each AM's existing gamma/scalar slot?
  IVF and HNSW have obvious scalar fields; SPIRE and DiskANN need explicit
  storage binding review.
- Should legacy IVF tag 4 decode be implemented immediately or deferred until
  fixture-read tests exist? ADR-076 permits it only as read-only compatibility.
