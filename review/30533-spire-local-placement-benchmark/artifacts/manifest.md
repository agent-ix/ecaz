# Artifact Manifest: SPIRE Local Placement Benchmark

- Measurement head: `54059950d269b6a6af812ad354b1801b7be8ea9f`
- Benchmark-driver commit: `bd311f3f` (`Add SPIRE local placement benchmark driver`)
- Packet/topic: `30533-spire-local-placement-benchmark`
- Timestamp: 2026-05-06T13:11:41-07:00
- Cluster: local PG18 pgrx scratch, database `postgres`, socket `/home/peter/.pgrx`, port `28818`
- Corpus fixture: `target/real-corpus/ec_hnsw_real_10k`
- Rows / queries / dimensions: 10,000 / 200 / 1536
- Storage format / rerank mode: `turboquant`, `rerank_width=25`
- Index shape: `ec_spire`, `nlists=32`, `nprobe` swept over `8,24`
- Isolation: each lane uses its own corpus/query tables and index prefix; the two-store lanes use auxiliary store relations.

## Benchmark Driver

Script:

```bash
bash scripts/bench_spire_local_placement_pg18.sh
```

Useful rerun variants:

```bash
bash scripts/bench_spire_local_placement_pg18.sh --install-extension
bash scripts/bench_spire_local_placement_pg18.sh --skip-load --skip-latency
```

The script refuses `--tablespace-path` values outside `/mnt/e` for this packet's extra-drive lane.

## Artifacts

### `load_real10k_2same_pgdefault.log`

- Lane / fixture / storage / rerank: two local stores, both `pg_default`; real 10k; `turboquant`; `rerank_width=25`
- Command:

```bash
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 corpus load --prefix task30_spire_real10k_tq_2same --profile ec_spire --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --manifest-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_manifest.json --allow-manifest-mismatch --dim 1536 --storage-format turboquant --reloption nlists=32 --reloption nprobe=24 --reloption rerank_width=25 --reloption local_store_count=2 --reloption local_store_tablespaces=pg_default,pg_default --log-file review/30533-spire-local-placement-benchmark/artifacts/load_real10k_2same_pgdefault.log
```

- Key lines:
  - corpus hash: `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
  - query hash: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
  - built `task30_spire_real10k_tq_2same_turboquant_idx` in `71.78s`
  - completed prefix in `101.69s`

### `load_real10k_2store_pgdefault_e.log`

- Lane / fixture / storage / rerank: two local stores, store 0 `pg_default`, store 1 `ecaz_spire_e`; real 10k; `turboquant`; `rerank_width=25`
- Command:

```bash
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 corpus load --prefix task30_spire_real10k_tq_2e --profile ec_spire --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --manifest-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_manifest.json --allow-manifest-mismatch --dim 1536 --storage-format turboquant --reloption nlists=32 --reloption nprobe=24 --reloption rerank_width=25 --reloption local_store_count=2 --reloption local_store_tablespaces=pg_default,ecaz_spire_e --log-file review/30533-spire-local-placement-benchmark/artifacts/load_real10k_2store_pgdefault_e.log
```

- Key lines:
  - corpus hash: `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
  - query hash: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
  - built `task30_spire_real10k_tq_2e_turboquant_idx` in `71.92s`
  - completed prefix in `102.44s`

### Latency Tables

Command shape for each lane:

```bash
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix <prefix> --profile ec_spire --k 10 --iterations 100 --sweep 8,24 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output <artifact>
```

Files and key result lines:

- `latency_real10k_1store_pgdefault_table.log`
  - `nprobe=8`: mean `70.1 ms`, p50 `66.7 ms`, p95 `98.4 ms`, p99 `129.7 ms`, HWM `48576 KB`
  - `nprobe=24`: mean `141.7 ms`, p50 `139.2 ms`, p95 `165.1 ms`, p99 `174.6 ms`, HWM `69240 KB`
- `latency_real10k_2same_pgdefault_table.log`
  - `nprobe=8`: mean `62.8 ms`, p50 `62.4 ms`, p95 `76.5 ms`, p99 `79.2 ms`, HWM `49892 KB`
  - `nprobe=24`: mean `140.6 ms`, p50 `138.2 ms`, p95 `156.7 ms`, p99 `166.8 ms`, HWM `51844 KB`
- `latency_real10k_2store_pgdefault_e_table.log`
  - `nprobe=8`: mean `63.8 ms`, p50 `63.1 ms`, p95 `80.0 ms`, p99 `85.6 ms`, HWM `76408 KB`
  - `nprobe=24`: mean `143.5 ms`, p50 `141.1 ms`, p95 `163.8 ms`, p99 `180.5 ms`, HWM `77588 KB`

### Recall Tables

Command shape:

```bash
bash scripts/bench_spire_local_placement_pg18.sh --skip-load --skip-latency
```

Files and key result lines:

- `recall_real10k_2same_pgdefault_table.log`
  - `nprobe=8`: recall@10 `0.9985`, ndcg@10 `0.9999`, mean q-time `62.51 ms`
  - `nprobe=24`: recall@10 `1.0000`, ndcg@10 `1.0000`, mean q-time `141.12 ms`
- `recall_real10k_2store_pgdefault_e_table.log`
  - `nprobe=8`: recall@10 `0.9985`, ndcg@10 `0.9999`, mean q-time `62.94 ms`
  - `nprobe=24`: recall@10 `1.0000`, ndcg@10 `1.0000`, mean q-time `145.09 ms`
- `real10k_truth_k10.json`: packet-local truth cache used by the recall runs.

### `store_relation_tablespaces.tsv`

- Command: emitted by `scripts/bench_spire_local_placement_pg18.sh` after the benchmark runs.
- Key rows:
  - one-store baseline store 0: `pg_default`, filepath `base/5/7900318`
  - same-device two-store store 0: `pg_default`, filepath `base/5/7920538`
  - same-device two-store store 1: `pg_default`, filepath `base/5/7920542`
  - `/mnt/e` two-store store 0: `pg_default`, filepath `base/5/7940763`
  - `/mnt/e` two-store store 1: `ecaz_spire_e`, location `/mnt/e/ecaz_pg_tblspc/spire_e`, filepath `pg_tblspc/7900319/PG_18_202506291/5/7940767`

## Notes

The `/mnt/e` lane is an approved local extra-drive/tablespace lane, not a cloud
or production-hardware claim. On this WSL host the mounted drive is exposed
through the local mount stack; treat these numbers as local placement evidence
and regression baselines, not final multi-NVMe product performance.
