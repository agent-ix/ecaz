# Request

Validate the repeatable benchmark-profile surface added in `ba02b9c6` and review the attached packet-local benchmark artifacts.

Scope:

- `ecaz` suite runner support for `corpus-fetch`, `corpus-prepare`, and chunked-manifest `load`
- `ecaz compare pgvector --pgvector-am ivfflat` support for repeatable IVF vs pgvector IVFFlat head-to-head runs
- committed reusable suites for:
  - `profile-cross-engine-real10k`
  - `profile-hnsw-100k`
  - `profile-ivf-100k`
  - `profile-ivf-1m`
- runner tier updates in `scripts/run_benchmark_profile.sh`

What this checkpoint claims:

1. The broad benchmark surface is now repeatable from committed suite configs instead of ad hoc commands.
2. The standard comparison surface is green on this PG18/M5 machine:
   - cross-engine `real10k` (`ec_diskann` vs `pgvectorscale`, `ec_hnsw` vs `pgvector`)
   - `ec_hnsw` vs `pgvector` on `real100k`
   - `ec_ivf` quality/latency/storage on `real100k`
3. The scale lane is now covered by a committed `profile-ivf-1m.json` suite with suite-driven fetch, prepare, and chunked load.
4. The measured `1M` checkpoint completed from the corrected load step onward using the committed suite config and previously prepared staged inputs.
5. The `profile-ivf-100k` suite now includes repeatable pgvector IVFFlat comparison steps with matched `nlists/lists=128` and matched `nprobe/probes` sweeps.

Key measured takeaways from the packet-local artifacts:

- `ec_diskann` trails `pgvectorscale` at `64/128/200/400`, but overtakes it at `800`:
  - `800`: `ec_diskann` `p50=2.80 ms`, `recall@10=0.9975`
  - `800`: `pgvectorscale` `p50=3.58 ms`, `recall@10=1.0000`
- `ec_hnsw` is behind `pgvector` on both `real10k` and `real100k` recall:
  - `real10k`, `ef_search=400`: `ec_hnsw recall@10=0.9715` vs `pgvector recall@10=1.0000`
  - `real100k`, `ef_search=400`: `ec_hnsw recall@10=0.9460` vs `pgvector recall@10=0.9970`
- `ec_ivf` on `real100k` reaches the intended candidate-quality lane:
  - `nprobe=96`, `rerank_width=1000`: `recall@100=0.9920`, `p50=11.9 ms`
- `ec_ivf` vs pgvector IVFFlat on `real100k`:
  - `k=10`, `nprobe/probes=96`: `ec_ivf recall@10=0.9980`, `p50=10.6 ms`; `pgvector_ivfflat recall@10=1.0000`, `p50=95.4 ms`
  - `k=100`, `nprobe/probes=96`: `ec_ivf recall@100=0.9920`, `p50=13.1 ms`; `pgvector_ivfflat recall@100=0.9976`, `p50=94.5 ms`
  - pgvector IVFFlat sidecar build `4.42 s`, index size `820,256,768 B`
- `ec_ivf` on the scale lane completed at `real1m` / `990000` rows:
  - build time `724.44 s`
  - `nprobe=128`, `rerank_width=1000`: `recall@10=0.9820`, `p50=22.7 ms`
  - `nprobe=128`, `rerank_width=1000`, `k=100`: `recall@100=0.9640`
  - IVF index size `187.8 MiB`, total relation footprint `15.6 GiB`

Packet-local evidence:

- [artifacts/manifest.md](/Users/peter/dev/tqvector/review/30545-benchmark-profile-surface/artifacts/manifest.md)
- [profile-cross-engine-real10k/suite-manifest.json](/Users/peter/dev/tqvector/review/30545-benchmark-profile-surface/artifacts/profile-cross-engine-real10k/suite-manifest.json)
- [profile-hnsw-100k/suite-manifest.json](/Users/peter/dev/tqvector/review/30545-benchmark-profile-surface/artifacts/profile-hnsw-100k/suite-manifest.json)
- [profile-ivf-100k/suite-manifest.json](/Users/peter/dev/tqvector/review/30545-benchmark-profile-surface/artifacts/profile-ivf-100k/suite-manifest.json)
- [profile-ivf-100k/suite-manifest-ivfflat-compare.json](/Users/peter/dev/tqvector/review/30545-benchmark-profile-surface/artifacts/profile-ivf-100k/suite-manifest-ivfflat-compare.json)
- [profile-ivf-1m/suite-manifest.json](/Users/peter/dev/tqvector/review/30545-benchmark-profile-surface/artifacts/profile-ivf-1m/suite-manifest.json)
