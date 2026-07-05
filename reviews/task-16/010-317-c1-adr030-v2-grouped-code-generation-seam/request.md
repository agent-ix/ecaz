# Review Request: C1 ADR-030 V2 Grouped Code Generation Seam

## Context

Packet `316` proved that a full v2-shaped alternate write path can be staged when grouped search
codes are supplied by the caller.

The next narrow slice is to replace that caller-supplied payload with a real build-time grouped-code
generation seam, without changing the live builder path.

## Problem

The alternate write path is coherent, but it still depends on synthetic grouped search-code input.

That means there is still no builder-side seam that:

1. trains a grouped model from build inputs
2. rotates source vectors into the grouped-search domain
3. emits grouped packed search codes for staged v2 tuples

## Planned Slice

Add a source-vector-backed grouped-code generation seam that:

1. trains a grouped PQ model from `build_source_column` vectors
2. derives grouped packed search codes from those source vectors
3. fails explicitly when source vectors are unavailable

This slice still excludes:

- no live build switchover
- no grouped-code training metadata persistence
- no scan/runtime use yet

## Implementation

Added the first build-side grouped-code generation seam, backed by source vectors.

New pieces:

1. `BuildGroupedPqModel`
   - grouped codebooks
   - group count / group size
   - transform dimension
   - SRHT sign vector
2. `train_build_grouped_pq_model(...)`
   - trains grouped PQ codebooks from `build_source_column` vectors
   - uses SRHT-rotated source vectors
   - fails explicitly when source vectors are absent or insufficient
3. `derive_grouped_search_code_from_source(...)`
   - rotates one source vector into the grouped domain
   - emits packed grouped search-code bytes

This intentionally stays source-vector-only. If the build does not have raw source vectors, the seam
returns an explicit error instead of pretending the current scalar code is an acceptable grouped
search code.

Tests added:

- grouped model training and code derivation on source-backed tuples
- explicit failure when source vectors are unavailable

## Measurements

This packet is a build-time code-generation seam, so there are no new recall or latency
measurements.

Known validation results for this attempt:

- `cargo test grouped_build_model_trains_and_derives_codes_from_source_vectors --lib`: passed
- `cargo test grouped_build_model_requires_source_vectors --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

The alternate v2 path no longer has to rely on arbitrary caller-supplied grouped codes.

What this de-risks:

1. grouped search codes can now be derived from the same source vectors used for build-time graph
   decisions
2. grouped-code generation is explicitly tied to the raw-vector-backed build lane
3. the next packet can feed real generated grouped codes into the alternate v2 page staging path

## Next Slice

The next narrow slice should connect this seam to the alternate v2 write path:

1. train the grouped model from source-backed build state
2. derive grouped search codes per tuple
3. feed those codes into the existing alternate v2 staging path
