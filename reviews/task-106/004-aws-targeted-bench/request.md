# Task 106 packet 004 - targeted AWS bench results

Status: completed AWS targeted bench packet (2026-06-14). Coder lane.

## Summary

This packet contains the completed narrowed AWS rerun for Task 106. It is not
a full quant/index/option sweep; it covers only the Task 106 affected benchmark
surfaces, across both AWS lanes and all standard scales.

Both AWS main suites completed successfully:

- `aws-intel`: 149/149 steps succeeded, 826 `results.jsonl` rows.
- `aws-graviton`: 149/149 steps succeeded, 826 `results.jsonl` rows.

The raw logs, manifests, and result streams are packet-local under
`artifacts/aws-intel/` and `artifacts/aws-graviton/`. The AWS instances were
stopped after artifact sync on 2026-06-14 to conserve spend.

## Included Matrix

- AWS lanes: `aws-intel`, `aws-graviton`.
- Scales: 10k, 50k, 100k, 1m.
- Included cells:
  - `ec_ivf` + `rabitq` + `quant_bits={1,2,4,8}`, scratch SoA on/off,
    recall + latency.
  - `ec_ivf` default `Auto` storage, scratch SoA on/off, recall + latency.
  - `ec_spire` + `rabitq`, candidate batch scoring on/off, recall + latency +
    `spire-pipeline`.
  - `ec_hnsw` + grouped-PQ (`pq_fastscan`), candidate batch scoring on/off,
    recall + latency. These are the gap-2 grouped-PQ cells.

## Explicit Exclusions

The configs intentionally exclude DiskANN, explicit TurboQuant comparator
lanes, broad PQ-FastScan benches, and unrelated quant/index/option
combinations. SPIRE pq_fastscan is excluded because that surface is not
implemented for SPIRE and is a permanent exclusion (operator decision, ADR-077
§9.4), not a benchmark target.

HNSW grouped-PQ is included rather than deferred. Both lanes carry it for
config symmetry and cross-lane confirmation.

## Configs

- `task106-aws-targeted-fixture-prep.json` stages canonical DBpedia fixtures
  under `/var/lib/pgsql/18/datasets/staged-task106-targeted` if the AWS host
  does not already have them.
- `task106-aws-intel-targeted.json` is the main 149-step AWS Intel suite.
- `task106-aws-graviton-targeted.json` is the main 149-step AWS Graviton
  suite.

## Validation

Local dry-runs passed and wrote packet-local dry-run manifests/logs. AWS
fixture staging and AWS audits passed on both hosts before the main suites ran.

The AWS suites initially hit EBS capacity at the 1m IVF/RaBitQ cells. The EBS
volumes were expanded from 400G to 800G, the suites resumed, and the resumed
runs completed. No indexes were dropped while benchmark steps were in flight
after resume.

SPIRE rows are single-node SPIRE, not distributed SPIRE: `profile=ec_spire`,
`local_store_count=1`, one EC2 host per lane. SPIRE pq_fastscan remains
excluded because it is not implemented.

## Gap-2 Grouped-PQ Result

The HNSW grouped-PQ recall and latency cells ran successfully on both AWS
lanes. The specific flush-width histogram expected for the gap-2 decision was
not observed: the HNSW grouped-PQ latency logs contain `task87-counters` rows,
but they report zero flushes/candidates and there are no
`block-kernel-counters` or `width_*` histogram rows.

That is not a benchmark failure. It means this run does not provide positive
histogram evidence for a grouped-PQ traversal block kernel; the next decision
should treat gap 2 as requiring probe/counter wiring inspection before calling
the histogram question closed.

See `artifacts/manifest.md` for exact commands and artifact paths.
