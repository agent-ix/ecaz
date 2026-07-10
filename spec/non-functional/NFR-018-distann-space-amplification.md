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
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-018: Distann Space Amplification Budget

## Statement

The total graph-side on-disk size of an ec_distann index (graph heap and TOAST,
unique local directories, codec artifacts, generation/manifest metadata,
logical control index, and head sample, summed across nodes) SHALL NOT exceed
4× the raw vector bytes of the indexed corpus.

## Scope

- Applies to: published-epoch storage at 10k/50k/100k on the standard
  corpora (1536-dim f32).
- Drivers: per-record neighbor-code block (R × code bytes) and closure-build
  duplication (transient, reclaimed after stitch); the budget covers the
  published epoch, not transient build state.
- The co-placed epoch row tier (ADR-085 D11, FR-078) is physically disjoint
  across owners. Its vector column is the single once-stored copy of corpus
  vectors and defines the 1.0× raw baseline; it is excluded from the index
  numerator. Non-vector payload bytes in the row tier are reported separately.
- A full index replica, including a replica whose non-owned records are filtered
  or tombstoned, is not a valid NFR-018 distributed measurement lane.
- Retained old epochs and unpublished Building/Ready generations are reported as
  lifecycle overhead rows rather than silently folded into the active-epoch
  index ratio.
- The report SHALL also emit total row-tier heap/TOAST bytes and end-to-end
  cluster bytes. The raw vector payload is excluded only from the graph-side
  ratio numerator; row-tier tuple/page/TOAST overhead and non-vector payload are
  not silently discarded and remain separately visible.

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
| active epoch row-tier vector bytes across all owners | ≈ 1.0× raw vectors | exactly one row-tier vector per vec_id | topology/storage audit |
| row-tier heap/TOAST bytes and end-to-end cluster bytes | recorded per owner and summed | recorded; no omitted owner/TOAST relation | topology/storage audit |
| non-owner graph records in the measured lane | 0 | 0 | topology audit |

## Verification

Every gate benchmark run includes the storage step; the ratio row appears in
the packet manifest per scale. A threshold breach fails the milestone
closeout.

Multinode measurement mechanism: the suite's storage step runs once per node
and the suite report sums the per-node graph, TOAST, directory, and metadata
bytes for the ratio. With the D11 lean record and the implemented 1-bit RaBitQ
default, ADR-085 D1's corrected dim-1536/R=32 formula is 7,008 record bytes, or
about 1.14× raw vector bytes before PostgreSQL relation overhead. The threshold
still depends on measured 10k/50k/100k physical-owner storage: tuple/page/TOAST/
directory overhead and alternative codecs can materially change the ratio, so
the suite result—not this arithmetic—decides whether to lower `graph_degree`,
select a smaller codec, or use the D1 fallback layout.

## Dependencies

- **Upstream**: [FR-076](../functional/index/distann/FR-076-distann-graph-node-record-format.md),
  [FR-078](../functional/index/distann/FR-078-distann-hash-placement.md) (co-placed
  heap tier — the 1.0× ratio denominator); ADR-085 decisions D1 (duplication),
  D7 (codec), and D11 (co-placed heap rerank / lean record)
