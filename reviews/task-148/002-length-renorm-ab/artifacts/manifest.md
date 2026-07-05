# Task 148 / Packet 002 Length Renorm A/B Artifact Manifest

- task bucket: `reviews/task-148/002-length-renorm-ab`
- head SHA: `9ded201453cb851076f54c1b787d69f6519b0578`
- product branch: `task-124-stage2-pareto`
- lane: local PG18 staged-corpus A/B using `ecaz bench suite`
- fixture source: `data/staged-current/`
- scales: 10k, 50k, 100k
- variants:
  - `tqdefault`: `storage_format=turboquant`, no rerank
  - `stage2`: `storage_format=coarse_rerank`, `rerank_placement=index`, `rerank_format=turboquant`, `rerank_width=50`, `stage2_final_rerank_width=25`
- recall grid: nprobe `[8,16,24,32,48,64]`
- latency grid: nprobe `[32,40]`
- runner environment: `PGHOST=/Users/peter/.pgrx`, `PGPORT=28818`
- timestamp: 2026-07-05 local session; artifact mtimes recorded by filesystem

## Suite Config

`task148-length-renorm-suite.json`

Audit command:

```sh
./target/release/ecaz bench suite audit --config reviews/task-148/002-length-renorm-ab/task148-length-renorm-suite.json
```

The audit passed. A dry-run manifest was written to
`artifacts/dry-run/suite-manifest.json`.

## Baseline Run

- baseline commit: `9bc66bcabe22697b4edc91300914b1e692938c44`
- installed dylib shasum: `51d5192a0909f397363b74da07e2f37aae9b317d8f8d02915226fa7b1918310f`
- matching local release dylib shasum: `51d5192a0909f397363b74da07e2f37aae9b317d8f8d02915226fa7b1918310f`
- install log: `artifacts/install-baseline-escalated.log`
- suite console log: `artifacts/baseline-suite.console.escalated.log`
- results: `artifacts/baseline/results.jsonl`
- suite manifest: `artifacts/baseline/suite-manifest.json`

Command:

```sh
PGHOST=/Users/peter/.pgrx PGPORT=28818 ./target/release/ecaz bench suite --config reviews/task-148/002-length-renorm-ab/task148-length-renorm-suite.json --artifact-dir reviews/task-148/002-length-renorm-ab/artifacts/baseline
```

Installed SHA checks:

- `artifacts/baseline/precheck-build-sha.log`: `9bc66bcabe22697b4edc91300914b1e692938c44`
- `artifacts/baseline/postcheck-build-sha.log`: `9bc66bcabe22697b4edc91300914b1e692938c44`

The baseline run used scorer `int8_approx`.

## Renorm Run

- corrected commit: `9ded201453cb851076f54c1b787d69f6519b0578`
- installed dylib shasum: `bbd27b0f9af389608cc3bb0f9cb4a7a5ccdfe2193453ad92ca93cfe4efcb0e63`
- matching local release dylib shasum: `bbd27b0f9af389608cc3bb0f9cb4a7a5ccdfe2193453ad92ca93cfe4efcb0e63`
- install log: `artifacts/install-renorm-fixed-escalated.log`
- suite console log: `artifacts/renorm-fixed-suite.console.escalated.log`
- results: `artifacts/renorm-fixed/results.jsonl`
- suite manifest: `artifacts/renorm-fixed/suite-manifest.json`

Command:

```sh
PGHOST=/Users/peter/.pgrx PGPORT=28818 ./target/release/ecaz bench suite --config reviews/task-148/002-length-renorm-ab/task148-length-renorm-suite.json --artifact-dir reviews/task-148/002-length-renorm-ab/artifacts/renorm-fixed
```

Installed SHA checks:

- `artifacts/renorm-fixed/precheck-build-sha.log`: `9ded201453cb851076f54c1b787d69f6519b0578`
- `artifacts/renorm-fixed/postcheck-build-sha.log`: `9ded201453cb851076f54c1b787d69f6519b0578`

The corrected run used scorer `int8_approx`.

## Failed Intermediate Run

Commit `a3bcb13d0f8f58950e765ab0642cb168fcc8807d` applied gamma length renormalization to no-QJL scoring, but the first suite failed when it reached the `stage2` sidecar cell:

```text
ERROR: ec_ivf TurboQuant borrowed batch payload/gamma count 50/0 does not match score count 50
```

The failed console log is `artifacts/renorm-suite.console.escalated.log`; partial outputs are under `artifacts/renorm/`. The follow-up commit `9ded201453cb851076f54c1b787d69f6519b0578` keeps in-memory no-QJL batches renormalized when gamma is available, while allowing persisted sidecar batches without gamma to score unrenormalized. This avoids an unreviewed on-disk format change.

## Key Results

Full normalized tables are in `artifacts/summary.md`.

- Pure TQ default 100k recall improved from `92.50%` to `93.13%` at nprobe 64, a `+0.63 pp` delta.
- Pure TQ default 100k latency regressed from `1.66 ms` to `9.92 ms` at nprobe 32 and from `1.85 ms` to `11.80 ms` at nprobe 40.
- Pure TQ default 10k and 50k recall grids were unchanged.
- Stage2 recall was unchanged at every measured nprobe and scale.
- Stage2 latency stayed neutral: 100k nprobe 32 moved from `1.55 ms` to `1.47 ms`; 100k nprobe 40 moved from `1.75 ms` to `1.68 ms`.
- Storage remained effectively unchanged; the total/row differences are parser/reporting noise of 1-2 B/row while index/row stayed identical.

No 1m run was launched. The 100k result failed the latency-neutral gate for pure TQ, and the stage2 cell cannot receive the correction without persisting gamma or a derived scalar in the sidecar format.

## Artifacts

- `task148-length-renorm-suite.json`: checked-in bespoke SuiteConfig.
- `artifacts/dry-run/suite-manifest.json`: suite dry-run manifest.
- `artifacts/baseline/results.jsonl`: baseline structured results.
- `artifacts/baseline/suite-manifest.json`: baseline suite manifest.
- `artifacts/baseline/*.log`: baseline step logs, including SHA pre/post checks.
- `artifacts/renorm-fixed/results.jsonl`: corrected structured results.
- `artifacts/renorm-fixed/suite-manifest.json`: corrected suite manifest.
- `artifacts/renorm-fixed/*.log`: corrected step logs, including SHA pre/post checks.
- `artifacts/summary.md`: normalized A/B tables.
- `artifacts/install-baseline*.log`: baseline install evidence.
- `artifacts/install-renorm*.log`: renorm install evidence.
- `artifacts/*suite.console*.log`: suite console logs, including sandbox failure and failed intermediate attempt.

Regenerable `truth-cache/` data is intentionally not part of the committed evidence set.

## Validation

- `cargo check -p ecaz-cli` passed after the length-renorm implementation.
- `cargo test --release --lib no_qjl_4bit_length_renorm_scale_uses_gamma_and_decoded_norm` passed.
- `cargo test --release --lib turboquant_lut_batch_applies_gamma_length_renorm_epilogue` passed.
- `cargo test --release --lib turboquant_dispatch_uses_lut_for_no_qjl_4bit_lane` passed.
- `cargo test --release --lib turboquant_int8_approx_scorer_prepares_factored_variant` passed.
- After the sidecar gamma guard:
  - `cargo check -p ecaz-cli` passed.
  - `cargo test --release --lib turboquant_no_qjl_4bit_payload_refs_allow_missing_gamma` passed.
  - `cargo test --release --lib turboquant_no_qjl_4bit_batch_requires_gamma_side_input` passed.
