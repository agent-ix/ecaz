---
id: NFR-018
title: Distann Space Amplification Budget
type: NFR
status: PROPOSED
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/ecaz/FR-076"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-018: Distann Space Amplification Budget

## Statement

The total on-disk size of an ec_distann index (all node records, metadata,
and head sample, summed across nodes) SHALL NOT exceed 4× the raw vector
bytes of the indexed corpus.

## Scope

- Applies to: published-epoch storage at 10k/50k/100k on the standard
  corpora (1536-dim f32).
- Drivers: per-record neighbor-code block (R × code bytes) and closure-build
  duplication (transient, reclaimed after stitch); the budget covers the
  published epoch, not transient build state.

## Rationale

Neighbor-code duplication is the deliberate space-for-network-locality trade
(the reference system reports ~10× with full-dimension OPQ at high degree);
ecaz's smaller codes and moderate degree should stay well under that, and an
explicit budget forces the ADR-085 D1/D7 arithmetic to be validated by
measurement rather than assumed.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| index bytes ÷ raw vector bytes, 100k | ≤ 3.0 | ≤ 4.0 | `ecaz bench suite` storage step |
| published-epoch bytes vs transient build peak | recorded | recorded | build instrumentation in epoch manifest |

## Verification

Every gate benchmark run includes the storage step; the ratio row appears in
the packet manifest per scale. A threshold breach fails the milestone
closeout.

## Dependencies

- **Upstream**: [FR-076](../functional/index/distann/FR-076-distann-graph-node-record-format.md);
  ADR-085 decisions D1 (duplication) and D7 (codec)
