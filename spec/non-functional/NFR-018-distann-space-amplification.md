---
id: NFR-018
title: Distann Space Amplification Budget
type: NFR
status: APPROVED
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

The summed budget above is necessary but not sufficient. **No single node's
graph-side bytes SHALL exceed the summed budget divided by the roster size,
plus the bounded structures NFR-021 permits.** A cluster in which one node holds
the whole index satisfies the summed budget and is nonetheless a breach.

**A full index replica — including a replica whose non-owned records are
filtered or tombstoned, and including a derived, optional, or rebuildable
performance object that holds graph records or full-precision vectors for
vec_ids the node does not own — SHALL NOT be built, and is not a valid
distributed measurement lane.** The FR-084 coordinator traversal replica as
specified is an instance of this excluded class. See
[NFR-021](./NFR-021-distann-distribution-invariant.md) for the governing
per-node invariant.

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
  or tombstoned, is not a valid NFR-018 distributed measurement lane. This
  exclusion is normative in the Statement above; it is restated here because it
  also governs which lanes may produce a reportable ratio.
- Derived, optional, and rebuildable relations count toward both the summed
  budget and the per-node bound. A relation is not excluded because it is a
  cache, a replica, a sample, or a performance object, nor because it is
  disabled by default.
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
| index bytes ÷ raw vector bytes, 100k (summed across nodes) | ≤ 3.0 | ≤ 4.0 | `ecaz bench suite` storage step |
| **max single-node graph-side bytes ÷ raw vector bytes, 100k** | ≈ summed ratio ÷ roster size | ≤ (summed threshold ÷ roster size) + bounded structures | per-node storage audit, measured per arm |
| **max owner graph-side bytes per owned record, normalized growth 100k ÷ 10k** | stable or sublinear | ≤ 2.0 per owner role — the [NFR-021](./NFR-021-distann-distribution-invariant.md) gate; not restated here | suite-computed bytes-per-owned-record ratio (see NFR-021) |
| **max single-node graph-side raw byte growth 100k ÷ 10k** | reported | reported, **not a threshold**: on a fixed roster a valid O(N) shard necessarily grows with `N` | suite `physical_benchmark_storage_growth` row, judgement `reported_not_threshold_fixed_roster` |
| derived/optional relation bytes attributed to their node | reported per relation | no unreported index-derived relation | per-node storage audit |
| published-epoch bytes vs transient build peak | recorded | recorded | build instrumentation in epoch manifest |
| active epoch row-tier vector bytes across all owners | ≈ 1.0× raw vectors | exactly one row-tier vector per vec_id | topology/storage audit |
| row-tier heap/TOAST bytes and end-to-end cluster bytes | recorded per owner and summed | recorded; no omitted owner/TOAST relation | topology/storage audit |
| logical control-index bytes | recorded per participant and summed into graph-side numerator | no omitted control relation | FR-078 `control_index_bytes` topology column |
| non-owner graph records in the measured lane | 0 | 0 | topology audit |

## Verification

Every gate benchmark run includes the storage step; the summed ratio row and the
per-node maximum row appear in the packet manifest per scale. A threshold breach
fails the milestone closeout.

A prior revision carried a raw single-node byte-growth threshold
(100k ÷ 10k ≤ 2.0). That row was unmeetable on a fixed roster (Task 205) and
contradicted both NFR-021's rebased text and the suite machinery; the
normalized bytes-per-owned-record ratio in NFR-021 is the governing growth
gate, and the raw ratio is reported for context only.

**Enforcement mode (audited 2026-08-01).** The 4.0× summed budget and the
per-node bound are normative thresholds, but they are not evaluated by a
built-in suite gate today. The suite mechanically asserts only that every
storage row has a matching ratio row (`assert_distann_storage_ratio_rows`);
the fixture emits the ratio without comparing it to 4.0, and the optional
suite `ThresholdConfig` that could carry the comparison is not required by any
run. Conformance is therefore established by manual review of the emitted
ratio rows in `results.jsonl`, and a closeout citing this NFR SHALL state the
reviewed ratio values. A mechanized budget gate remains open engineering
scope.

The storage step SHALL be measured **per arm**, inside the arm loop, and its
values SHALL be emitted into `results.jsonl`. A storage row computed once and
replayed across arms is not a measurement: it cannot express a difference
between arms and SHALL NOT be cited as evidence that arms have equal storage.
Every index-derived relation, including optional and disabled-by-default
relations, SHALL be attributed to the node holding it and included in that
node's bytes. A relation reported only in a log sidecar and absent from
`results.jsonl` does not satisfy this requirement.

Multinode measurement mechanism: the suite's storage step runs once per node
and the suite report sums the per-node graph, TOAST, directory, generation
metadata, and separately reported logical `control_index_bytes` for the ratio.
With the D11 lean record and the implemented 1-bit RaBitQ
default, ADR-085 D1's corrected dim-1536/R=32 formula is 7,008 record bytes, or
about 1.14× raw vector bytes before PostgreSQL relation overhead. The threshold
still depends on measured 10k/50k/100k physical-owner storage: tuple/page/TOAST/
directory overhead and alternative codecs can materially change the ratio, so
the suite result—not this arithmetic—decides whether to lower `graph_degree`,
select a smaller codec, or use the D1 fallback layout.

## Dependencies

- **Upstream**: [FR-076](../functional/distann/storage/FR-076-distann-graph-node-record-format.md),
  [FR-078](../functional/distann/build/FR-078-distann-hash-placement.md) (co-placed
  heap tier — the 1.0× ratio denominator); ADR-085 decisions D1 (duplication),
  D7 (codec), and D11 (co-placed heap rerank / lean record)
