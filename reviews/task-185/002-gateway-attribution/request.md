---
task: 185
packet: 002-gateway-attribution
role: coder
status: open
date: 2026-08-07
seq: 01
head: 436cae964
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

No benchmark run is claimed by this request yet. After review, run the checked-
in suite config and capture the JSON trace, suite manifest, results, and
summary lines under this packet before interpreting gateway candidates.

## Review focus

1. Verify that the origin propagation does not change normal traversal result
   ordering or production builds.
2. Verify that the endpoint's first-arrival provenance is sufficient for the
   planned truth-coverage/set-cover post-processing.
3. Verify that the suite config preserves the disjoint input and physical
   topology contract and keeps the diagnostic separate from production claims.
