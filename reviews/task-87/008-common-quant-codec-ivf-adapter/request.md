# Task 87 Packet 008: Common Quant Codec IVF Adapter

## Summary

This packet asks for review of the first code slice for packet 007's
common quant codec shape. It introduces `src/am/common/quant_codec.rs`
and implements the trait for IVF's existing `IvfQuantizer`.

Code checkpoint under review:

- `9ae54988f75440197079f685b473a05cfdcff46f` - `Add common quant codec surface for IVF`

## Changes

- Added a shared `QuantCodec` trait with:
  - codec kind;
  - search-code tag;
  - payload length;
  - build-side source encoding;
  - prepared-query construction;
  - single-candidate scoring;
  - batch scoring over `CandidateBatch`.
- Added shared data enums/structs:
  - `QuantCodecKind`;
  - `QuantSearchCodecTag`;
  - `EncodedQuantPayload`.
- Implemented `QuantCodec` for `IvfQuantizer`.
  - TurboQuant uses the existing IVF prepared query path and preserves
    the specialized no-QJL 4-bit batch scorer.
  - RaBitQ uses the same trait batch surface with scalar fallback today.
  - Grouped-PQ/PqFastScan uses the common scoring surface once the AM
    has prepared model state.
- Added IVF tests proving TurboQuant, RaBitQ, and grouped-PQ can all
  score through the same trait and `CandidateBatch` surface.

## Limits

This is the first adapter, not the whole common-codec migration. It does
not yet:

- replace SPIRE, HNSW, or DiskANN codec enums;
- solve grouped-PQ model ownership in the generic `encode_source` method;
- add DiskANN TurboQuant registration;
- route every broad-scope quant mode through AM scan loops.

Those remain follow-on Task 87 slices.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `artifacts/cargo-test-candidate-batch.log`
  - `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
  - result: `2 passed; 0 failed`
- `artifacts/cargo-test-ivf-quantizer.log`
  - `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - result: `17 passed; 0 failed`

## Review Focus

- Confirm the trait shape is a suitable first code version of packet
  007's common codec surface.
- Confirm the IVF adapter maps all current IVF codec profiles without
  changing existing scoring behavior.
- Confirm the grouped-PQ limitation is acceptable for this first slice:
  model-backed encode/prepare still stays in IVF-specific calls until
  the common surface grows model/context ownership.
