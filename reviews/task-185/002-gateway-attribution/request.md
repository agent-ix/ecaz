---
task: 185
packet: 002-gateway-attribution
role: coder
status: open
date: 2026-08-07
seq: 01
head: 801bf5d78
---

# Review request: physical gateway/basin attribution surface

This checkpoint implements the first Task 185 measurement slice. It adds a
feature-only provenance trace at the real bounded traversal seam and exposes
it through the existing `distann-local-multinode` suite driver.

## Scope

- `ec_distann_physical_seed_gateway_trace_benchmark(...)` reports the selected
  seed ids, per-seed expanded-node and hit counts, hit origin masks, unique
  expanded-node count, and shared-origin overlap.
- The trace is collected from the ordinary physical multi-owner traversal and
  is absent unless `distann-head-attribution-benchmark` is enabled.
- `--gateway-trace` writes one compact JSON trace per physical variant under
  the suite packet artifact directory and refuses to run without the trace
  endpoint installed.
- The new focused unit test proves two seeds sharing a reachable node produce
  the expected per-seed counts and overlap mask.

This is Phase 1 attribution only. It does not select or persist a new policy,
change the production head, alter graph/search parameters, or use evaluation
outcomes to construct a head. The first preregistered run is the retained
Task 182 `training_landmarks` control at 100k, with rows 201--400 reserved as
the disjoint training slice and rows 1--200 evaluated for attribution.

## Frozen run contract

- cap 4096, exact head scoring, 32 returned seeds;
- graph degree 32, BW4/H100, RaBitQ traversal, exact final ranking;
- three-owner physical topology with one immutable generation;
- 200 evaluation queries from rows 1--200;
- feature-build diagnostic only; no production-latency promotion claim; and
- run directory under `/home/peter/.ecaz/clusters`, never under `target/`.

The trace's origin masks support post-processing of per-seed truth coverage and
set-cover marginals by joining `hit_ids` to the packet's held-out truth cache.
The endpoint reports first-arrival/merged-before-expansion provenance; any
candidate policy requires a separately reviewed selector and isolated A/B
screen in packet 003.

## Validation

- PG18 feature build: pass.
- Focused PG18 unit test: pass, 1 test.
- ecaz-cli build: pass with the repository's existing unused `path` warning;
  the suite runner forwards `gateway_trace` into the emitted command.
- Suite audit and dry-run are recorded in packet-local artifacts.

## Captured control run

The checked-in suite completed successfully on 2026-08-07 after correcting the
driver to trace the disjoint training relation. The run used the release
profile with `distann-head-attribution-benchmark` enabled; it is a feature-build
diagnostic and is not comparable to a featureless production latency baseline.

- topology: 100,000 rows across three physical owners, two remote owners,
  topology and engagement gates passed;
- trace: 200 disjoint training queries (rows 201--400), represented in the
  trace as query IDs 1--200, 32 seeds/query, 7,056 unique expansions, 95
  shared-expansion events, no zero-hit queries, and no malformed records;
- truth join: exact inner-product top-10 against the 100k corpus found 1,523
  of 2,000 truth ids (76.15%), with three zero-success queries; the persisted
  4,096-member head's logical-id-to-vec-id reconstruction matched all 4,096
  members;
- seed attribution: aggregate expansion counts by returned-seed position were
  `[1188, 304, 892, 4767, 0, ...]`; ordered marginal truth coverage was
  `[207, 17, 147, 1152, 0, ...]`, so the fourth returned position dominates
  the observed bounded-traversal truth coverage;
- context metrics: recall@32 `0.9205` (95% CI `[0.9078, 0.9316]`), one warm
  latency sample `41.20 ms`, and physical generation bytes `2,496,626,688`.

The trace is attribution evidence, not a candidate-policy result. It does not
justify changing the head or selecting a gateway policy: the next slice must
use the recorded truth join to compute selector features, then preregister one
gateway set-cover selector for the Phase 2 A/B screen. The current analysis is
still diagnostic; it does not use evaluation outcomes for policy selection.

The durable evidence is under `artifacts/run/`; the suite manifest reports one
succeeded step, zero missing artifacts, and a 2,126,724 ms duration. The
stopped 6.4G cluster run directory was removed after capture. An earlier
evaluation-slice attempt and one interrupted retry were superseded and are
not retained as packet evidence.

## Driver correction

The initial implementation traced `physical_queries`, which are the evaluation
rows. Before retaining the result, the driver was corrected to require
`training_query_path`, stage only rows 201--400 into a temporary relation, and
emit `query_prefix=rows_201_400`. The corrected run is the only benchmark result
claimed here.

## Review focus

1. Verify that the origin propagation does not change normal traversal result
   ordering or production builds.
2. Verify that the endpoint's first-arrival provenance is sufficient for the
   planned truth-coverage/set-cover post-processing.
3. Verify that the suite config preserves the disjoint input and physical
   topology contract and keeps the diagnostic separate from production claims.
