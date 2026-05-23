# Artifact Manifest: Local IVF/RaBitQ Geometry Suite

- task bucket: `reviews/task-51/006-local-ivf-rabitq-geometry`
- timestamp: `2026-05-23T05:25:55Z`
- benchmark packet: `benchmarks/task51-local-ivf-rabitq-geometry`
- benchmark packet commit: `cb140d6be` (`Add Task 51 local IVF RaBitQ geometry suite`)
- benchmark head SHA at run start: `d4be1037f50dfa4f8357c849404df37ca084620c`
- lane: local IVF/RaBitQ geometry sweep
- fixture: DBpedia real 10k
- storage format: `rabitq`
- rerank mode: `heap_f32`
- rerank width: 50
- isolated one-index-per-table surfaces: yes
- AWS: not used
- vchord: not run
- pgvectorscale: not run

## Artifacts

- `benchmarks/task51-local-ivf-rabitq-geometry/suite.json`
  - checked-in `SuiteConfig`
- `benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-manifest.json`
  - structured suite manifest
- `benchmarks/task51-local-ivf-rabitq-geometry/artifacts/results.jsonl`
  - structured results
- `benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-run-final.log`
  - final successful suite run
- `benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-status.log`
  - command: `target/release/ecaz bench suite status --manifest benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-manifest.json --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-status.log`
  - result: `completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-report.log`
  - parsed load, recall, latency, and storage report
- `benchmarks/task51-local-ivf-rabitq-geometry/manifest.md`
  - benchmark packet provenance and headline results
- `diff-check.log`
  - command: `git diff --check -- reviews/task-51/006-local-ivf-rabitq-geometry`
  - result: passed

## Notes

This packet does not close Task 51 and does not request AWS. It records the
first local IVF/RaBitQ geometry screen and supports promoting `nlists=64` and
`nlists=128` to the next larger local suite.
