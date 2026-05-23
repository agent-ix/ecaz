# Task 51 Packet 024 Artifact Manifest

- head SHA: `bf84190ad`
- task bucket: `reviews/task-51/024-rabitq8-sidecar-clip-tuning/`
- timestamp: 2026-05-23
- lane: IVF/RaBitQ only
- fixture: preserved isolated local 50k prefix `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- storage format: `ec_ivf` index with `storage_format=rabitq`, sidecar measurement variants using q8 codes
- rerank mode: sidecar free-I/O and real DB `tid-sorted`
- isolated one-index-per-table surface: yes

## Artifacts

- `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/manifest.md`
  - command family: `ecaz bench suite`
  - purpose: candidate frontier recall sweep for existing `rabitq8`
  - key result: recall stayed `0.9480` for `candidate_k` 50, 100, and 200
- `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/results-report.jsonl`
  - key result lines cited by `request.md`: `candidate_k`, `recall@k`, `sidecar_p50`, `total_bound_p50`
- `benchmarks/task51-local-rabitq8ls-sidecar/manifest.md`
  - command family: `ecaz bench suite`
  - purpose: local scoring variant sweep for q8 sidecars
  - key result: `rabitq8c4` reached `recall@k=0.9950` with `sidecar_bytes_per_vector=1548`
- `benchmarks/task51-local-rabitq8ls-sidecar/artifacts/cargo-test-ecaz-cli-sidecar.log`
  - command: `cargo test -p ecaz-cli --no-default-features sidecar`
  - key result: 7 passed, 0 failed
- `benchmarks/task51-local-rabitq8ls-sidecar/artifacts/cargo-build-ecaz-cli.log`
  - command: `cargo build -p ecaz-cli --no-default-features`
  - key result: build passed
- `benchmarks/task51-local-rabitq8ls-sidecar/artifacts/results-report.jsonl`
  - key result lines cited by `request.md`: `variant`, `read_mode`, `recall@k`, `sidecar_p50`, `sidecar_bytes_per_vector`

## Local Validation Note

`cargo test -p ecaz --lib least_squares_estimator_uses_o_dot_as_shrinkage --no-default-features --features pg18` compiled but failed to run outside the pgrx loader with `undefined symbol: CacheRegisterRelcacheCallback`. This was not used as passing evidence; the packet relies on the focused CLI tests, build, and local suite results.
