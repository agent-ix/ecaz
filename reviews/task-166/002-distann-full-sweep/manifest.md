# Manifest — Task 166 M4 packet 002: distann full sweep (10k/50k/100k)

- head SHA: this commit (task-165-ec-distann-m3)
- packet: reviews/task-166/002-distann-full-sweep
- lane / fixture: Intel local, ec_distann, real DBpedia 10k/50k/100k (dim 1536)
- storage: isolated one-index-per-table (m4_real{10,50,100}k_distann_corpus)
- build profile: release
- runner: `ecaz bench suite run` (FR-038); config distann-full-sweep.json
- command: `ecaz bench suite run --config .../distann-full-sweep.json --artifact-dir <pkt>/artifacts --host /home/peter/.pgrx --port 28818 --database ec_distann_bench`
- timestamp: 2026-07-08
- sweep: ec_distann.top_k = [16,32,64,100,200] (registered default_sweep)

## Key results (artifacts/results.jsonl)

recall@10 (warm) and index size, top of sweep (top_k=200):

| scale | recall@10 (sweep 16→200)                    | q-time warm | index size |
|-------|---------------------------------------------|-------------|------------|
| 10k   | 0.9935 / 0.9990 / 0.9995 / 1.0000 / 1.0000  | 2.5–10.7 ms | 110.3 MiB  |
| 50k   | 0.9150 / 0.9545 / 0.9840 / 0.9880 / 0.9950  | 3.3–13.8 ms | 423.6 MiB  |
| 100k  | 0.8685 / 0.9260 / 0.9650 / 0.9770 / 0.9925  | 3.5–14.6 ms | 815.2 MiB  |

First point of each recall row (44/53/89 ms) is cold-cache first-query
inflation, not steady state (warm points follow).

## Note

Bespoke config justified: the canonical intel-local.json does not carry
ec_distann. Sweep grid = ec_distann's registered default_sweep verbatim. The
hnsw/ivf/spire comparator columns for the 4-way M4 gate run through the same
harness/corpus in packet 003.
