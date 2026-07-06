# Manifest

- Task bucket: `reviews/task-148/008-codebook-calibration-ab/`
- Head SHA: `a18c8c063d333e16ca3be3ab7ff15dfddb51b231`
- Baseline SHA: `c24402b6ff7df7e8a6f79c5c0938d05315c51a6f`
- Baseline dylib shasum: see `artifacts/baseline-dylib-shasums.txt`
- After dylib shasum: `4a3b36737d8901378fe7c9bbb4dd438013fc293a3bc29eeb8dd5c45e3fd92c63  /opt/homebrew/lib/postgresql@18/ecaz.dylib`
- Install logs: `artifacts/install-baseline-escalated.log`, `artifacts/install-tqplus-a18c8c063.log`
- Suite configs: `task148-codebook-calibration-baseline-suite.json`, `task148-codebook-calibration-tqplus-fixed2-suite.json`
- Baseline artifact dir: `artifacts/baseline/`
- After artifact dir: `artifacts/tqplus-fixed2/`

## Commands

- Baseline suite: `./target/release/ecaz bench suite run --config reviews/task-148/008-codebook-calibration-ab/task148-codebook-calibration-baseline-suite.json --artifact-dir reviews/task-148/008-codebook-calibration-ab/artifacts/baseline --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-148/008-codebook-calibration-ab/artifacts/baseline-suite.console.direct.log`
- After suite: `./target/release/ecaz bench suite run --config reviews/task-148/008-codebook-calibration-ab/task148-codebook-calibration-tqplus-fixed2-suite.json --artifact-dir reviews/task-148/008-codebook-calibration-ab/artifacts/tqplus-fixed2 --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-148/008-codebook-calibration-ab/artifacts/tqplus-fixed2-suite.console.direct.log`
- After install: `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- Tests: `cargo test --release --lib tqplus_coarse_rerank_dense_postings_keep_coarse_payload_width`

## Result Files

- Summary: `artifacts/summary.md`
- Baseline results: `artifacts/baseline/results.jsonl`
- After results: `artifacts/tqplus-fixed2/results.jsonl`
- Baseline suite manifest: `artifacts/baseline/suite-manifest.json`
- After suite manifest: `artifacts/tqplus-fixed2/suite-manifest.json`
- Baseline SHA logs: `artifacts/baseline/precheck-build-sha.log`, `artifacts/baseline/postcheck-build-sha.log`
- After SHA logs: `artifacts/tqplus-fixed2/precheck-build-sha.log`, `artifacts/tqplus-fixed2/postcheck-build-sha.log`

## Key Result Lines

- Pure TQ 100k nprobe64 recall: baseline `0.9250`, TQ+ `0.9344`; nprobe32 latency: baseline `1.71 ms`, TQ+ `2.47 ms`.
- stage2@25 100k nprobe64 recall: baseline `0.9719`, TQ+ `0.9719`; nprobe32 latency: baseline `1.60 ms`, TQ+ `1.64 ms`.
- stage2@25 100k storage: baseline `1.7 GiB`, TQ+ `1.7 GiB`; exact bytes are unchanged in `artifacts/summary.md`.
- 1m skipped: 100k was not latency-neutral.
