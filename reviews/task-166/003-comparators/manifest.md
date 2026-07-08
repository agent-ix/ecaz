# Manifest — Task 166 M4 packet 003: comparators (hnsw/ivf/spire)

- head SHA: this commit (task-165-ec-distann-m3)
- packet: reviews/task-166/003-comparators
- lane / fixture: Intel local, real DBpedia 10k/50k/100k (dim 1536)
- AMs: ec_hnsw, ec_ivf, ec_spire (the M4 comparators); one-index-per-table
- build profile: release
- runner: `ecaz bench suite run` (FR-038); config m4-comparators.json
- command: `ecaz bench suite run --config .../m4-comparators.json --artifact-dir <pkt>/artifacts --host /home/peter/.pgrx --port 28818 --database ec_distann_bench`
- timestamp: 2026-07-08
- sweeps: registered default_sweep per profile — hnsw [40,64,100,128,160,200],
  ivf [8,16,24,32,48,64], spire [8,16,24,32]
- purpose: the hnsw/ivf/spire columns of the M4 4-way gate, measured on the same
  harness/corpus/host/commit as ec_distann (packet 002) for a fair A/B.
- key results: see reviews/task-166/004-gate-verdict/verdict.md (full 4-way
  recall/latency/storage table). results.jsonl + per-step logs in artifacts/.

## Note

manifest-verification warnings in the load logs are expected
(`allow_manifest_mismatch: true`; per-AM prefixes vs the shared corpus manifest),
not errors.
