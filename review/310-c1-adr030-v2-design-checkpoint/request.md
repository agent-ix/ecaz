# Review Request: C1 ADR-030 V2 Design Checkpoint

## Context

Packet `280` already answered the first ADR-030 question:

- grouped reinterpretation of the current scalar `4-bit` code stream is **not** the plan

Packets `287`, `307`, and `309` narrowed the next question:

- `ADR-031` is very promising on the current format
- `ADR-032` can make the current format very fast, but low-`ef` recall recovery still looks
  structurally limited
- we do not currently have a proven current-format path to about `1ms` with `>0.90` recall

So this packet is the handoff checkpoint for the long-horizon ADR-030 lane: define the actual v2
architecture before writing implementation code.

## Problem

ADR-030 needs a concrete design, not another abstract FastScan reference and not another attempt to
 salvage the current scalar code layout.

The design checkpoint needs to answer explicitly:

1. which transform fronts the grouped encoding
2. what grouped code structure is persisted
3. whether search and rerank should share one code or use separate payloads
4. what the hot page layout is
5. what the intended query pipeline is
6. how the index versions and migrates
7. what the cheapest feasibility spike is

## Proposed V2 Direction

This checkpoint proposes that ADR-030 become a **versioned index-v2 format** built around three
separate concerns instead of one overloaded code stream.

### 1. Transform

Support both `SRHT` and `OPQ` in the v2 metadata model, but start the actual implementation path
with `SRHT`.

Reasoning:

- `SRHT` already exists in tqvector and is the cheapest route to a first true grouped-code study
- `OPQ` is the strongest candidate if grouped `PQ4` needs a better front-end transform
- format-level support for both avoids painting v2 into a corner while keeping the first spike
  narrow

### 2. Search code

Use true grouped `PQ4`, not scalar-code reinterpretation.

Default target shape for the existing `1536`-dim lane:

- `96` subvectors
- `16` dims per subvector
- `4` bits per subvector code
- one learned 16-centroid codebook per subvector

That produces a `48B` grouped search code per element and matches the classic FastScan/QuickerADC
shape well enough for a real SIMD path later.

### 3. Separate search and rerank payloads

Do **not** force one payload to satisfy both scan throughput and final ranking quality.

Proposed persisted payloads:

- hot grouped `PQ4` search code
- hot binary sidecar (`192B` sign code at `1536` dims)
- cold higher-fidelity rerank payload

The pragmatic first rerank payload is the existing scalar `4-bit` tqvector code kept as a cold
payload, because it already has a scorer and substantially higher fidelity than the grouped
FastScan code. A later v2 follow-up can replace that cold payload with a better residual / `PQ8`
contract if measurements justify it.

### 4. Hot page layout

The hot scan tuple should keep only what layer-0 search touches frequently:

- graph linkage / visibility state
- hot binary sidecar
- hot grouped search code

The cold rerank payload should live separately so scans do not drag a `768B` rerank blob through
cache on every candidate.

That means ADR-030 is not only "new scorer code." It is also:

- new tuple contract
- new page-locality plan
- likely new builder emission order

### 5. Query pipeline

The intended steady-state pipeline is:

1. optional `ADR-031`-style binary prefilter on the hot binary sidecar
2. grouped FastScan scorer on the hot grouped `PQ4` payload
3. tiny rerank on the cold higher-fidelity payload

If later measurements show the grouped scorer is strong enough to stand alone for traversal, the
binary stage can become optional. But the initial design should assume the composed pipeline,
because that has the best current odds of reaching the target frontier.

### 6. Versioning / migration

Treat v2 as rebuild-only.

Needed metadata additions:

- explicit `format_version`
- transform kind and transform parameters
- grouped-code configuration (`subvector_dim`, `subvector_count`, `bits`)
- payload-presence flags (`binary`, `grouped_search`, `cold_rerank`)

Do not try to reinterpret or auto-migrate existing v1 indexes in place.

## Smallest Feasibility Spike

The first bounded spike should stay offline and answer the highest-risk question:

> does true grouped `PQ4` on transformed tqvector data have materially better ranking quality than
> packet `280`'s current-format reinterpretation, at a speed/size point worth pursuing?

Concrete spike:

1. extend `src/bin/approx_score_study.rs`
2. add a true grouped-code study mode that trains grouped codebooks on transformed vectors
3. start with `SRHT`
4. score with both `f32` and quantized LUTs to separate code-quality loss from LUT loss
5. compare against fp32 truth using the same overlap/capture metrics already used in packet `280`
6. add an `OPQ` comparison only if `SRHT` grouped `PQ4` looks borderline rather than clearly good
   or clearly bad

This keeps the first experiment cheap while directly testing the core v2 premise.

## Known Measurements Informing This Design

No new runtime code or measurements landed in this packet. The design is anchored on the existing
recorded surfaces below.

### Packet `280`: reject current-format grouped reinterpretation

- `group_size=8`: `spearman_rho mean=0.7980`, `top10_overlap mean=0.7350`,
  `exact_top10_captured_by_approx_top100 mean=0.9300`, `grouped_f32_ns_per_score=1440.5`
- `group_size=16`: `spearman_rho mean=0.7024`, `top10_overlap mean=0.6500`,
  `exact_top10_captured_by_approx_top100 mean=0.9000`, `grouped_f32_ns_per_score=758.3`,
  `grouped_u8_ns_per_score=1020.6`
- `group_size=32`: `spearman_rho mean=0.6249`, `top10_overlap mean=0.4700`,
  `exact_top10_captured_by_approx_top100 mean=0.8350`, `grouped_f32_ns_per_score=349.2`

### Packet `287`: kept ADR-031 current-format warm baseline

- canonical warm real `50k`, `m=8`, `ef_search=40`
- run 1: `p50=1.480ms`, `p95=2.084ms`, `p99=2.390ms`, `mean=1.507ms`
- run 2: `p50=1.485ms`, `p95=2.047ms`, `p99=2.422ms`, `mean=1.510ms`
- recall summary: `graph_recall_at_10 = 0.8428`

### Packet `307`: kept ADR-032 practical current-format frontier

- `ef=56`: `mean=0.990ms`, `graph_recall_at_10 = 0.8417`
- `ef=64`: `mean=1.043ms`, `graph_recall_at_10 = 0.8519`

### Packet `309`: last rejected ADR-032 structural experiment

- warm canonical `ef=40`: `mean=2.359ms`
- full real `50k` recall: `graph_recall_at_10 = 0.3111`

## Validation

Design-only checkpoint.

- read `AGENTS.md`
- read `ADR-030`, `ADR-031`, `ADR-032`
- read packets `280`, `287`, `307`, and `309`
- no runtime code changed in this packet
- no test / benchmark gate was run because this checkpoint only updates design/task documents

## Exit Criteria

- ADR-030 explicitly states that current-format grouped reinterpretation is retired
- ADR-030 records the v2 payload/layout/query-pipeline direction
- the task file records the approximate build path for ADR-030
- the packet names one smallest, high-signal feasibility spike instead of jumping straight to a
  broad implementation
