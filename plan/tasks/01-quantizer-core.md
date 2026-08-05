# Task 01: Quantizer Core

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: complete

## Scope

Implement the scalar quantization math and the `ProdQuantizer` core used by all encode and scoring paths.

## Owns

- `FR-013`
- `FR-015` core scalar behavior

## Dependencies

- None beyond crate scaffolding

## Unblocks

- `FR-004`
- `FR-005`
- `FR-017`
- `FR-018`
- `FR-014`

## Deliverables

- SRHT/FWHT
- Lloyd-Max codebook generation
- MSE/QJL packing and unpacking
- `ProdQuantizer::new`
- `encode`
- `decode_approximate`
- `prepare_ip_query`
- `score_ip_encoded`
- `score_ip_encoded_lite`

## Primary Tests

- `TC-008` to `TC-015`
- `TC-019` to `TC-033`

## Notes

- Keep this scalar-first.
- Freeze public scoring and packing interfaces before SIMD work starts.
