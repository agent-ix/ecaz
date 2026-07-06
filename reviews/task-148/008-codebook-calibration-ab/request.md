# Task 148 Slice 3 Codebook Calibration A/B

Requesting review for Slice 3 measurement and the coarse_rerank correctness fix needed to complete the TQ+ stage2 run.

Code under review:
- `a18c8c063` keeps TQ+ calibration from re-encoding coarse_rerank primary postings as TurboQuant. The TQ calibration model is still used for the index-side TurboQuant rerank sidecar.
- Regression test: `tqplus_coarse_rerank_dense_postings_keep_coarse_payload_width`.

Measurement result: measured negative for promotion. Pure TQ recall improves but violates the latency-neutral gate; stage2@25 recall is unchanged and has slight 100k latency regression. No 1m run was taken because 100k did not pass the latency-neutral win gate.

Evidence:
- Manifest: `artifacts/manifest.md`
- Summary tables: `artifacts/summary.md`
- Baseline results: `artifacts/baseline/results.jsonl`
- After results: `artifacts/tqplus-fixed2/results.jsonl`
- Suite configs: `task148-codebook-calibration-baseline-suite.json`, `task148-codebook-calibration-tqplus-fixed2-suite.json`
- Install/SHA evidence: `artifacts/install-tqplus-a18c8c063.log`, `artifacts/tqplus-fixed2/precheck-build-sha.log`, `artifacts/tqplus-fixed2/postcheck-build-sha.log`

Validation run:
- `cargo test --release --lib tqplus_coarse_rerank_dense_postings_keep_coarse_payload_width`
- Earlier in this packet sequence: `cargo test --release --lib turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently` and `cargo test --release --lib coarse_rerank_accepts_tqplus_turboquant_sidecar_profile`.

Please leave review feedback under `feedback/`; leave this request open.
