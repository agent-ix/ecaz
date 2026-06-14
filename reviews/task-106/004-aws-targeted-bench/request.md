# Task 106 packet 004 - targeted AWS bench plan/configs

Status: review request (2026-06-13). Coder lane.

## Summary

This packet implements the narrowed AWS rerun plan for Task 106. It is not a
full quant/index/option sweep. It packages `ecaz bench suite` configs for only
the Task 106 affected benchmark surfaces, across both AWS lanes and all standard
scales.

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
    recall + latency — the **gap-2 flush-width histogram** cells (added
    2026-06-13). The latency on-arm carries `--task87-candidate-batch-counters`
    so the measure-first probe (code commit `3e5a837af`) records the per-flush
    width distribution that decides whether a grouped-PQ traversal block kernel
    is justified (ADR-077 §9.2). Recall on/off is a parity check; latency
    on/off is a noise pair (the probe leaves scoring unchanged) — the
    deliverable is the histogram, not a batch speedup A/B.

## Explicit Exclusions

The configs intentionally exclude DiskANN, explicit TurboQuant comparator
lanes, broad PQ-FastScan benches, and unrelated quant/index/option
combinations. SPIRE pq_fastscan is excluded because that surface is not
implemented for SPIRE and is a permanent exclusion (operator decision, ADR-077
§9.4), not a benchmark target.

HNSW grouped-PQ is now **included** (gap-2 histogram, above) rather than
deferred. Its flush-width histogram is host-independent (graph + ef determine
the widths), so a single lane is sufficient; both lanes carry it for config
symmetry and a cross-lane confirmation.

## Configs

- `task106-aws-targeted-fixture-prep.json` stages canonical DBpedia fixtures
  under `/var/lib/pgsql/18/datasets/staged-task106-targeted` if the AWS host
  does not already have them.
- `task106-aws-intel-targeted.json` is the main 129-step AWS Intel suite.
- `task106-aws-graviton-targeted.json` is the main 129-step AWS Graviton
  suite.

## Validation

Local dry-runs passed and wrote packet-local dry-run manifests/logs. Local
`audit` passes for the fixture-prep config. Local `audit` for the AWS main
configs fails only because this workstation does not have the AWS fixture path
`/var/lib/pgsql/18/datasets/staged-task106-targeted`; those audits should be
re-run on each EC2 host after fixture staging.

See `artifacts/manifest.md` for exact commands and artifact paths.
