# Artifact Manifest

Head SHA: `ba02b9c6` for the original profile-surface checkpoint; `743647ddb76b4a27880c2ba4a128d361fa68ba49` for the IVFFlat compare update.
Packet: `30545-benchmark-profile-surface`
Machine: Apple M5 / 64 GiB RAM / PG18 on `/Users/peter/.pgrx:28818`
Surface shape: isolated one-index-per-table benchmark prefixes

## profile-cross-engine-real10k

- Timestamp: `May 9 2026 21:55:48`
- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-cross-engine-real10k.json`
- Lane: cross-engine `real10k`
- Fixture: staged DBPedia real10k corpus and queries
- Profiles: `ec_diskann`, `ec_hnsw`, `pgvectorscale`, `pgvector`
- Files:
  - `profile-cross-engine-real10k/suite-manifest.json`
  - `profile-cross-engine-real10k/results.jsonl`
  - `profile-cross-engine-real10k/compare-vectorscale-real10k.log`
  - `profile-cross-engine-real10k/compare-pgvector-real10k.log`
  - `profile-cross-engine-real10k/load-*.log`
  - `profile-cross-engine-real10k/recall-*.log`
  - `profile-cross-engine-real10k/latency-*.log`
  - `profile-cross-engine-real10k/storage-*.log`
- Cited results:
  - `ec_diskann` vs `pgvectorscale`:
    - `400`: `ec_diskann p50=2.42 ms recall@10=0.9970`; `pgvectorscale p50=1.91 ms recall@10=1.0000`
    - `800`: `ec_diskann p50=2.80 ms recall@10=0.9975`; `pgvectorscale p50=3.58 ms recall@10=1.0000`
  - `ec_hnsw` vs `pgvector`:
    - `400`: `ec_hnsw p50=5.19 ms recall@10=0.9715`; `pgvector p50=1.25 ms recall@10=1.0000`
    - `800`: `ec_hnsw p50=9.92 ms recall@10=0.9715`; `pgvector p50=1.85 ms recall@10=1.0000`
  - Sidecar build/index:
    - `pgvectorscale` build `1.53 s`, size `5,136,384 B`
    - `pgvector` build `5.65 s`, size `81,928,192 B`

## profile-hnsw-100k

- Timestamp: `May 9 2026 21:55:53`
- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-hnsw-100k.json`
- Lane: `ec_hnsw` vs `pgvector` on `real100k`
- Fixture: staged DBPedia real100k corpus and queries
- Profiles: `ec_hnsw`, `pgvector`
- Files:
  - `profile-hnsw-100k/suite-manifest.json`
  - `profile-hnsw-100k/results.jsonl`
  - `profile-hnsw-100k/compare-pgvector-real100k-hnsw.log`
  - `profile-hnsw-100k/load-hnsw-real100k-m16.log`
  - `profile-hnsw-100k/recall-hnsw-real100k-ef-sweep.log`
  - `profile-hnsw-100k/latency-hnsw-real100k-ef-sweep.log`
  - `profile-hnsw-100k/storage-hnsw-real100k-m16.log`
- Cited results:
  - `ef_search=64`: `ec_hnsw p50=1.70 ms recall@10=0.8410`; `pgvector p50=1.28 ms recall@10=0.9670`
  - `ef_search=400`: `ec_hnsw p50=4.33 ms recall@10=0.9460`; `pgvector p50=5.80 ms recall@10=0.9970`
  - `pgvector` sidecar build `199.71 s`, size `819,208,192 B`

## profile-ivf-100k

- Timestamp: `May 9 2026 21:56:00`
- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-ivf-100k.json`
- Lane: `ec_ivf` quality, latency, storage on `real100k`
- Fixture: staged DBPedia real100k corpus and queries
- Profile: `ec_ivf`
- Storage format / rerank mode: `pq_fastscan`, `pq_group_size=8`, `nlists=128`, `rerank=heap_f32`
- Files:
  - `profile-ivf-100k/suite-manifest.json`
  - `profile-ivf-100k/results.jsonl`
  - `profile-ivf-100k/load-real100k-n128-w500.log`
  - `profile-ivf-100k/recall10-nprobe-sweep-w500.log`
  - `profile-ivf-100k/latency-nprobe-sweep-w500.log`
  - `profile-ivf-100k/recall100-candidates-w500.log`
  - `profile-ivf-100k/recall100-candidates-w1000.log`
  - `profile-ivf-100k/latency-candidates-w1000.log`
  - `profile-ivf-100k/storage-real100k-n128.log`
  - `profile-ivf-100k/explain-quality-candidate.log`
- Cited results:
  - `nprobe=96`, `rerank_width=500`: `recall@10=0.9980`, `p50=10.1 ms`
  - `nprobe=96`, `rerank_width=1000`: `recall@100=0.9920`, `p50=11.9 ms`
  - IVF index size `19.4 MiB`, total footprint `1.6 GiB`

### profile-ivf-100k IVFFlat comparison addendum

- Head SHA: `743647ddb76b4a27880c2ba4a128d361fa68ba49`
- Timestamp: `May 9 2026 22:56:56 PDT`
- Command:
  - measured run: `./target/debug/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-ivf-100k.json --only compare-pgvector-ivfflat-real100k-k10 --only compare-pgvector-ivfflat-real100k-k100`
  - threshold-only resume after selected-step threshold fix: `./target/debug/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-ivf-100k.json --resume-from artifacts/benchmark-profiles/profile-ivf-100k/suite-manifest.json --only compare-pgvector-ivfflat-real100k-k10 --only compare-pgvector-ivfflat-real100k-k100`
- Lane: `ec_ivf` vs pgvector IVFFlat on `real100k`
- Fixture: staged DBPedia real100k corpus and queries
- Profiles: `ec_ivf`, pgvector `ivfflat`
- Storage format / rerank mode: `ec_ivf` uses `pq_fastscan`, `pq_group_size=8`, `nlists=128`, `rerank=heap_f32`; pgvector sidecar uses exact `vector(1536)` with `USING ivfflat (embedding vector_ip_ops) WITH (lists=128)`
- Rerank / rescore:
  - `k=10`: `ec_ivf.rerank_width=500`
  - `k=100`: `ec_ivf.rerank_width=1000`
  - pgvector IVFFlat exact-scores vectors from probed lists
- Surface shape: isolated one-index-per-table ecaz prefix plus pgvector exact-vector sidecar table
- Files:
  - `profile-ivf-100k/suite-manifest-ivfflat-compare.json`
  - `profile-ivf-100k/results-ivfflat-compare.jsonl`
  - `profile-ivf-100k/compare-pgvector-ivfflat-real100k-k10.log`
  - `profile-ivf-100k/compare-pgvector-ivfflat-real100k-k100.log`
- Cited results:
  - pgvector IVFFlat sidecar build `4.42 s`, size `820,256,768 B`
  - `k=10`, `nprobe/probes=48`: `ec_ivf p50=6.49 ms recall@10=0.9820`; `pgvector_ivfflat p50=46.6 ms recall@10=0.9790`
  - `k=10`, `nprobe/probes=96`: `ec_ivf p50=10.6 ms recall@10=0.9980`; `pgvector_ivfflat p50=95.4 ms recall@10=1.0000`
  - `k=100`, `nprobe/probes=80`: `ec_ivf p50=16.5 ms recall@100=0.9880`; `pgvector_ivfflat p50=77.2 ms recall@100=0.9923`
  - `k=100`, `nprobe/probes=96`: `ec_ivf p50=13.1 ms recall@100=0.9920`; `pgvector_ivfflat p50=94.5 ms recall@100=0.9976`

## profile-ivf-1m

- Timestamp: `May 9 2026 21:56:07`
- Command:
  - suite definition: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-ivf-1m.json`
  - measured completion checkpoint: resumed from the corrected downstream steps after setup outputs already existed:
    `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-ivf-1m.json --only load-real1m-n1024-w1000 --only recall10-real1m-nprobe-sweep --only latency-real1m-nprobe-sweep --only recall100-real1m-candidates --only storage-real1m-n1024 --only explain-real1m-quality-candidate`
- Lane: `ec_ivf` scale lane on `real1m`
- Fixture: DBPedia OpenAI3 parquet fetch + staged chunked prepare, with `990000` corpus rows in the prepared manifest
- Profile: `ec_ivf`
- Storage format / rerank mode: `pq_fastscan`, `pq_group_size=8`, `nlists=1024`, `rerank=heap_f32`, `rerank_width=1000`
- Files:
  - `profile-ivf-1m/suite-manifest.json`
  - `profile-ivf-1m/results.jsonl`
  - `profile-ivf-1m/load-real1m-n1024-w1000.log`
  - `profile-ivf-1m/recall10-real1m-nprobe-sweep.log`
  - `profile-ivf-1m/latency-real1m-nprobe-sweep.log`
  - `profile-ivf-1m/recall100-real1m-candidates.log`
  - `profile-ivf-1m/storage-real1m-n1024.log`
  - `profile-ivf-1m/explain-real1m-quality-candidate.log`
- Cited results:
  - load total `2438.74 s`; index build `724.44 s`
  - `nprobe=64`, `rerank_width=1000`: `recall@10=0.9640`, `p50=15.6 ms`
  - `nprobe=128`, `rerank_width=1000`: `recall@10=0.9820`, `p50=22.7 ms`
  - `nprobe=128`, `rerank_width=1000`, `k=100`: `recall@100=0.9640`
  - IVF index size `187.8 MiB`, total footprint `15.6 GiB`
