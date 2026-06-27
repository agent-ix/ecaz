# Task 107 Phase 3 Product Decision

## Decision

Task 107 should close with a narrow result:

- Drop local multi-disk / multi-store SPIRE as a product surface for now.
- Keep multinode SPIRE only as a research or internal-validation surface.
- Do not promote SPIRE RaBitQ or SPIRE TurboQuant as product-relevant
  alternatives to the current comparator frontier without a separate follow-up
  that materially reduces latency and adds correct per-store storage
  accounting.

This is not a recommendation to delete all SPIRE code immediately. It is a
recommendation not to productize the multi-disk shape and not to market the
distributed shape as competitive based on the current evidence.

## Evidence Boundary

New Task 107 evidence is limited to the operator-corrected checklist:

- Phase 1: single node with 2 disks, RaBitQ and TurboQuant, 100k and 1m.
- Phase 2: one coordinator plus two remotes, RaBitQ and TurboQuant, 100k and
  1m.

The packet does not count packet-004 drift cells as required checklist cells:
single-node/single-disk control attempts, four-disk attempts, and the
non-tablespace 2-disk attempt are historical/out-of-scope evidence. Where a
same-host single-store reference is mentioned below, it is used only as
supporting context because it exists in packet 004; Task 106 remains the
official already-covered single-node/single-store AWS packet.

## Phase 3 Questions

### 1. Does local multi-disk/store SPIRE beat single-store SPIRE?

No.

At the 1m product scale, RaBitQ 2-disk did not improve recall and was slower
than the available same-host single-store reference:

| Shape | k10 recall nprobe 8/16/24/32/64 | k10 c1 mean nprobe 8/16/24/32 |
| --- | --- | --- |
| RaBitQ single-store reference | 0.8110 / 0.8820 / 0.9060 / 0.9340 / 0.9690 | 187.9 / 336.2 / 487.4 / 620.9 ms |
| RaBitQ 2-disk | 0.8110 / 0.8820 / 0.9060 / 0.9340 / 0.9690 | 203.3 / 361.9 / 521.1 / 658.0 ms |

TurboQuant 2-disk at 1m looked roughly similar to the RaBitQ 2-disk result:
same recall curve and k10 c1 mean `188.1 / 343.6 / 499.1 / 620.1 ms` at
nprobe `8 / 16 / 24 / 32`. That does not show a local multi-disk product win.

Task 106's already-covered single-node SPIRE RaBitQ AWS Intel lane is also
faster than the Task 107 local 2-disk lane at comparable recall bands:
`recall@10=0.9760` at nprobe32 with k10 p50 `71.6 ms`, and
`recall@10=0.9850` at nprobe64 with k10 p50 `124.6 ms`.

### 2. Does multi-node SPIRE improve over matched single-node SPIRE at 1m?

Yes, for latency shape, but not enough to settle product relevance.

RaBitQ distributed 1m improved strongly over the available same-host
single-store reference at the same nprobe range:

| Shape | k10 recall nprobe64 | k10 c1 mean nprobe 8/16/24/32 |
| --- | ---: | --- |
| RaBitQ single-store reference | 0.9690 | 187.9 / 336.2 / 487.4 / 620.9 ms |
| RaBitQ distributed | 0.9510 | 89.3 / 103.6 / 111.8 / 121.3 ms |

TurboQuant distributed 1m had the same general profile: k10 recall
`0.7940 / 0.8640 / 0.8860 / 0.9140 / 0.9490` at nprobe
`8 / 16 / 24 / 32 / 64`; k10 c1 mean `93.1 / 108.1 / 121.5 / 131.9 ms` at
nprobe `8 / 16 / 24 / 32`.

The distributed read path was real, not a local fallback: packet 004 records
`result_source=remote_heap_candidates`, `remote_pid_sum=6400`, and
`dispatch_sum=200` for the 1m production-read checks, with timeout/cancel and
degraded-skip sums at zero.

### 3. Is the win large enough to justify implementation, operations, and support?

No for productization.

Multinode SPIRE has a real technical win over single-node SPIRE, but it
requires a coordinator, two remotes, remote tuple transport, endpoint
registration/readiness, per-node materialization, topology-specific cleanup,
AWS/VPC/IAM/SSM operational machinery, and added failure modes. The measured
1m distributed result is still only around the vchord 1m p50 latency band, and
at lower recall than vchord's 1m ceiling.

The local multi-disk shape has the opposite problem: it adds store placement
and disk operational complexity without a measured latency or recall win.

### 4. Against existing non-SPIRE baselines, is SPIRE product-relevant?

Not on the current evidence.

Existing comparator evidence from `benchmarks/comparators-50k-100k-1m/` puts
the 1m frontier at:

| System / setting | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: |
| pgvectorscale DiskANN sl40 | 6.5 ms | 13.3 ms | 0.980 |
| pgvectorscale DiskANN sl400 | 19.5 ms | 32.0 ms | 0.984 |
| pgvectorscale DiskANN sl1000 | 46.4 ms | 75.5 ms | 0.985 |
| vchord RaBitQ default | 90.3 ms | 100.0 ms | 0.9995 |
| pgvector HNSW ef40 | 2.9 ms | 4.7 ms | 0.932 |
| pgvector IVFFlat p100 | 265 ms | 1132 ms | 0.987 |

Task 107's distributed 1m production-read evidence at nprobe64 shows k10
p50/p95 of `117/135 ms` for RaBitQ at recall `0.9510`, and `140/164 ms` for
TurboQuant at recall `0.9490`. That is useful engineering evidence, but not a
product-relevant Pareto point against pgvectorscale DiskANN or vchord. It is
also not close to the lower-recall pgvector HNSW latency corner.

Existing Task 106 ecaz baselines point the same way:

- `ec_ivf` RaBitQ bits=1 at 1m: nprobe64 recall `0.9790`, k10 p50 `50.1 ms`.
- `ec_ivf` RaBitQ bits=4 at 1m: nprobe64 recall `0.9870`, k10 p50 `188.5 ms`.
- `ec_hnsw` grouped-PQ at 1m: ef120 recall `0.9150`, k10 p50 `13.4 ms` in
  the batch-on lane.

SPIRE distributed is competitive only against weak or mismatched comparator
points, not against the strongest existing frontier.

### 5. What is the narrowest SPIRE surface worth retaining?

Retain only the narrow surface needed for distributed-read research and
regression coverage:

- Keep one coordinator plus two remotes as the canonical distributed smoke and
  research topology.
- Keep RaBitQ and TurboQuant remote-read coverage at small scale as regression
  tests for endpoint readiness, tuple transport, routing counters, and cleanup.
- Do not invest further in local multi-disk SPIRE until a new design explains
  why the 2-disk shape should beat single-store.
- Do not expand bit-size or store-count sweeps until the storage accounting and
  comparator gap are addressed.

## Known Measurement Gaps

The multi-store storage metric is not decision-grade. For the 1m 2-disk cells,
`ecaz bench storage` reports the catalog `ec_spire` relation as `168.0 KiB`
and `0.2 B/row` while the same benchmark family's single-store reference
reports `784.8 MiB` and `831.3 B/row`. The 2-disk value omits per-store
tablespace payload and must not be interpreted as a storage reduction.

Packet 004 also did not surface disk IOPS, CPU peak, or memory peak for every
cell in a product-summary table. The latency/recall and routing evidence are
sufficient for the keep/drop/narrow decision, but the missing resource
observability should block any storage-efficiency or cost-efficiency claim.

## Final Answer

The product answer is: drop local multi-disk SPIRE, keep multinode SPIRE only
as a narrow research/regression surface, and do not claim SPIRE is product
competitive until a follow-up closes both the latency gap to existing
comparators and the per-store storage accounting gap.
