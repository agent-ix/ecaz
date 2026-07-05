# Task 148 Packet 002: Length Renormalization A/B

## Scope

Slice 2 only. This packet implements TurboQuant 4-bit no-QJL length renormalization for scoring paths that have per-vector gamma available, then A/Bs it against the pre-renorm baseline on the staged 10k/50k/100k corpora.

Code checkpoints:

- `a3bcb13d0f8f58950e765ab0642cb168fcc8807d` applies length renormalization in the no-QJL per-candidate epilogue and single-candidate scoring paths.
- `9ded201453cb851076f54c1b787d69f6519b0578` allows persisted sidecar scoring without gamma to remain unrenormalized instead of forcing an on-disk format change.

No default flip, QJL re-enable, or sub-4-bit format work is included.

## Measurement

Artifacts:

- `artifacts/manifest.md`
- `artifacts/summary.md`
- `task148-length-renorm-suite.json`
- `artifacts/baseline/results.jsonl`
- `artifacts/renorm-fixed/results.jsonl`
- `artifacts/baseline/suite-manifest.json`
- `artifacts/renorm-fixed/suite-manifest.json`
- packet-local install logs and suite console logs under `artifacts/`

The A/B used before/after dylib swaps on the same fixtures. The suite checked `ecaz_build_git_sha()` before and after each run:

- baseline pre/post: `9bc66bcabe22697b4edc91300914b1e692938c44`
- renorm pre/post: `9ded201453cb851076f54c1b787d69f6519b0578`

## Results

Pure TQ no-rerank default:

- 10k and 50k recall: unchanged across the required nprobe grid.
- 100k recall: `+0.62` to `+0.63 pp` across the grid.
- 100k latency: nprobe 32 regressed from `1.66 ms` to `9.92 ms`; nprobe 40 regressed from `1.85 ms` to `11.80 ms`.
- Storage: index bytes per row unchanged.

Stage2@25:

- Recall: unchanged at every measured scale/nprobe.
- Latency: neutral within run noise; 100k nprobe 32 moved from `1.55 ms` to `1.47 ms`, and nprobe 40 from `1.75 ms` to `1.68 ms`.
- Storage: index bytes per row unchanged.

## Verdict

Do not promote length renormalization for the pure TQ default cell. It is not latency-neutral on the int8/SDOT path, and the only recall gain appears at 100k.

For stage2@25, the current persisted dense-block sidecar has no gamma, so the correction is a no-op there unless we make an on-disk format decision. This packet deliberately does not make that format change.

No 1m run was performed because the 100k result failed the latency-neutral gate and stage2 cannot receive the correction without persistence work.

## Validation

- Passed: `cargo check -p ecaz-cli`
- Passed: `cargo test --release --lib no_qjl_4bit_length_renorm_scale_uses_gamma_and_decoded_norm`
- Passed: `cargo test --release --lib turboquant_lut_batch_applies_gamma_length_renorm_epilogue`
- Passed: `cargo test --release --lib turboquant_dispatch_uses_lut_for_no_qjl_4bit_lane`
- Passed: `cargo test --release --lib turboquant_int8_approx_scorer_prepares_factored_variant`
- Passed after sidecar guard: `cargo check -p ecaz-cli`
- Passed after sidecar guard: `cargo test --release --lib turboquant_no_qjl_4bit_payload_refs_allow_missing_gamma`
- Passed after sidecar guard: `cargo test --release --lib turboquant_no_qjl_4bit_batch_requires_gamma_side_input`

No push was performed per handoff.
