# Manifest — Task 166 M4 packet 001: distann suite smoke

- head SHA: this commit (task-165-ec-distann-m3; M4 work continues here until
  its own branch is cut)
- packet: reviews/task-166/001-distann-suite-smoke
- lane / fixture: Intel local, ec_distann, real DBpedia 10k (dim 1536)
- storage: isolated one-index-per-table (m4_smoke_real10k_distann_corpus)
- build profile: release
- runner: `ecaz bench suite run` (FR-038) — NOT a bespoke sweeper
- config: distann-10k-smoke.json (this packet)
- command: `ecaz bench suite run --config reviews/task-166/001-distann-suite-smoke/distann-10k-smoke.json --artifact-dir <pkt>/artifacts --host /home/peter/.pgrx --port 28818 --database ec_distann_bench`
- timestamp: 2026-07-08
- purpose: validate ec_distann runs end-to-end through the standard suite
  (load/recall/latency/storage) before the full M4 4-way matrix.
- key result (artifacts/results.jsonl, recall-10k-distann.log): ec_distann 10k
  recall@10 across sweep [16,32,64,100,200] = 0.9935 / 0.9990 / 0.9995 / 1.0000
  / 1.0000; ndcg@k ~1.0; warm mean q-time 2.8–11 ms (the 57 ms first point is
  cold-cache first-query inflation, not steady state).

## Note

M4 needs ec_distann in the comparison, which the canonical intel-local.json
does not carry (it benches hnsw/ivf/diskann/spire). This bespoke config is
therefore justified per the "Standard ecaz Sweep" rule; the sweep grid is
ec_distann's registered default_sweep from profiles.rs verbatim.
