# Task 106 Full Affected Sweep

## Summary

This packet contains the local Intel PG18 full affected sweep for Task 106 across 10k, 50k, 100k, and 1m fixtures. The sweep was driven by `ecaz bench suite` configs checked into this packet, with raw logs, manifests, reports, and parsed result rows under `artifacts/`.

Coverage includes IVF RaBitQ b1/b2/b4/b8 scratch on/off, IVF auto and explicit TurboQuant scratch on/off, SPIRE RaBitQ candidate-batch on/off plus pipeline, HNSW PQ-FastScan grouped-PQ candidate-batch on/off, and the SPIRE PQ-FastScan negative gap.

## What Changed

- Added packet-local suite configs for the full sweep, 1m continuation, SPIRE recall supplemental, and SPIRE pipeline supplemental.
- Added a small CLI recall fallback so SPIRE recall can recover when predicted-source lookup rejects bulk `ANY($1::bigint[])`: it falls back to per-id equality reads.
- Added packet-local status/report artifacts and `artifacts/manifest.md`.

## Results

- Main suite: `completed=125 failed=9 stale=40`
- 1m continuation: `completed=37 failed=3 stale=0`
- SPIRE recall supplemental: `completed=2 failed=0`
- SPIRE pipeline supplemental: `completed=8 failed=0`

The main suite's 40 stale 1m cells were completed by the 1m continuation. The two failed 1m SPIRE recall cells were rerun successfully by the recall supplemental after the CLI fix. The original SPIRE pipeline failures were bad config/output-path issues and were replaced by the clean SPIRE pipeline supplemental, which succeeded for all 10k/50k/100k/1m batch-on/off cells.

The remaining load failures are expected negative coverage for SPIRE PQ-FastScan without a persisted grouped-PQ model:

- `load-10k-spire-pqfastscan-gap`
- `load-50k-spire-pqfastscan-gap`
- `load-100k-spire-pqfastscan-gap`
- `load-1m-spire-pqfastscan-gap`

Each reports the expected diagnostic: `ec_spire PQ-FastScan encoding requires a persisted grouped-PQ model`.

## Evidence

- Artifact manifest: `artifacts/manifest.md`
- Main suite manifest/report: `artifacts/suite/suite-manifest.json`, `artifacts/main-suite-report.md`
- 1m continuation manifest/report: `artifacts/suite-1m-continuation/suite-manifest.json`, `artifacts/continuation-suite-report.md`
- SPIRE recall supplemental: `artifacts/suite-spire-recall-supplemental/suite-manifest.json`, `artifacts/spire-recall-supplemental-report.md`
- SPIRE pipeline supplemental: `artifacts/suite-spire-pipeline-supplemental/suite-manifest.json`, `artifacts/spire-pipeline-supplemental-report.md`

Key 1m evidence:

- SPIRE recall supplemental batch-on recall: `0.9540/0.9700/0.9760/0.9800` at nprobe `16/24/32/48`
- SPIRE recall supplemental batch-off recall: `0.9540/0.9700/0.9760/0.9800` at nprobe `16/24/32/48`
- SPIRE pipeline supplemental batch-on p50: `52.838/64.479/73.647/97.896 ms`, recall `0.9540/0.9700/0.9760/0.9800`
- SPIRE pipeline supplemental batch-off p50: `52.243/66.979/87.149/104.560 ms`, recall `0.9540/0.9700/0.9760/0.9800`
- HNSW PQ-FastScan grouped-PQ 1m load total: `4581.31s`
- HNSW PQ-FastScan grouped-PQ 1m batch-on recall: `0.8260/0.8370/0.8580/0.8640` at ef_search `80/120/200/400`
- HNSW PQ-FastScan grouped-PQ 1m batch-off recall: `0.8260/0.8370/0.8580/0.8640` at ef_search `80/120/200/400`

## Review Notes

The load logs include manifest mismatch warnings because the suite reuses existing real corpora with task-specific table prefixes and runs with `--allow-manifest-mismatch`. The warnings are provenance-relevant and are called out in `artifacts/manifest.md`; they did not fail the load steps.

The authoritative SPIRE pipeline evidence is the supplemental suite because the earlier continuation wrote two funnel outputs to a literal `${artifact_dir}` path. Those unintended files were removed after the clean packet-local supplemental JSONL files were produced.
