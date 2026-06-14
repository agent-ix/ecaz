# Task 106 packet 004 - targeted AWS bench results

Status: completed AWS targeted bench packet plus gap-2 fixed-probe AWS rerun
(2026-06-14). Coder lane.

## Summary

This packet contains the completed narrowed AWS rerun for Task 106. It is not
a full quant/index/option sweep; it covers only the Task 106 affected benchmark
surfaces, across both AWS lanes and all standard scales.

Both AWS main suites completed successfully:

- `aws-intel`: 149/149 steps succeeded, 826 `results.jsonl` rows.
- `aws-graviton`: 149/149 steps succeeded, 826 `results.jsonl` rows.

The fixed-probe gap-2 rerun also completed successfully:

- `aws-intel-gap2-rerun-06020c8c0`: 9/9 steps succeeded, 36
  `results.jsonl` rows.
- `aws-graviton-gap2-rerun-06020c8c0`: 9/9 steps succeeded, 36
  `results.jsonl` rows.

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

Follow-up on 2026-06-14: local diagnosis found the probe placement bug. The
HNSW PqFastScan path can use binary traversal scoring for grouped candidates,
which bypassed the old increment site. The probe has been moved to the grouped
candidate dispatch boundary in `src/am/ec_hnsw/scan.rs`. Local PG18 validation
now shows the same HNSW grouped-PQ query emits a width-only
`block-kernel-counters` row, and the local latency command prints
`width_8_15=256 width_16_31=194`.

AWS fixed-probe rerun result: both lanes now emit the expected width-only
`block-kernel-counters` rows for HNSW grouped-PQ batch-on at 10k, 50k, 100k,
and 1m across ef_search 40/80/120. Per lane totals across the 12 histogram
rows are:

- `width_lt8=2496`
- `width_8_15=45516`
- `width_16_31=181377`
- `width_ge32=0`

Interpretation: the AWS histogram question is now measured. Widths are
overwhelmingly 8-31 and never >=32 in this rerun, so gap 2 should close as a
measured skip for a grouped-PQ traversal block kernel rather than an
implementation target.

See `artifacts/manifest.md` for exact commands and artifact paths.
