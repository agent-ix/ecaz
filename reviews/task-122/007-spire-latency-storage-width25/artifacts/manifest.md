# Task 122 Packet 007 Artifact Manifest

- head SHA: `6c057c462b067e152ef5c86ca6da53f925fd865d`
- task bucket: `reviews/task-122/007-spire-latency-storage-width25`
- timestamp: `2026-06-27T14:29:12Z`
- runner: `ecaz bench suite`
- backend: PG18, `/Users/peter/.pgrx`, port `28818`, database `tqvector_bench`
- build profile: `release`
- fixture: `data/staged-current/ec_real_{10k,50k,100k}_corpus.tsv`
- profile: `ec_spire`
- storage formats: `turboquant` and `rabitq`
- bits: `4`
- query count: `100`
- latency iterations: `100`
- k: `10`
- matrix: scale `10k/50k/100k`, format `turboquant/rabitq`, rerank width `25`, nprobe `24/96/192`
- table isolation: isolated one-prefix-per-scale-per-format tables
- runner status: `audit passed: 18 steps`; final `suite-manifest.json` records `18` succeeded steps

## Commands

Audit:

```sh
/Users/peter/.cargo/bin/ecaz bench suite audit \
  --config reviews/task-122/007-spire-latency-storage-width25/artifacts/task122-spire-latency-storage-width25.json \
  --log-file reviews/task-122/007-spire-latency-storage-width25/artifacts/suite-audit.log
```

Dry run:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/007-spire-latency-storage-width25/artifacts/task122-spire-latency-storage-width25.json \
  --dry-run \
  --log-file reviews/task-122/007-spire-latency-storage-width25/artifacts/suite-dry-run.log
```

Backend/GUC check:

```sh
/Users/peter/.cargo/bin/ecaz dev sql \
  --pg 18 \
  --db tqvector_bench \
  --socket-dir /Users/peter/.pgrx \
  --raw \
  --sql "SELECT ecaz_build_profile(); SELECT current_setting('ec_spire.pre_materialization_prune') AS pre_materialization_prune;" \
  --log-output reviews/task-122/007-spire-latency-storage-width25/artifacts/guc-check.log
```

Run:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/007-spire-latency-storage-width25/artifacts/task122-spire-latency-storage-width25.json \
  --host /Users/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-122/007-spire-latency-storage-width25/artifacts/suite-run.log
```

## Artifacts

- `task122-spire-latency-storage-width25.json`: checked-in suite config.
- `suite-audit.log`: suite audit output.
- `suite-dry-run.log`: dry-run command trace.
- `guc-check.log`: release backend and prune-GUC confirmation.
- `suite-run.log`: full suite command trace.
- `suite/suite-manifest.json`: structured suite manifest and step statuses.
- `suite/results.jsonl`: structured load, latency, and storage results.
- `suite/load-*.log`: six fresh load logs for TQ/RaBitQ 10k/50k/100k prefixes.
- `suite/latency-*.log`: six latency logs for width `25`.
- `suite/storage-*.log`: six storage logs.

No corpus TSVs, truth caches, or generated ground-truth files are committed in
this packet.

## Key Latency Results

Mean latency at width `25`:

```text
scale  format      nprobe 24  nprobe 96  nprobe 192
10k    turboquant  2.21 ms    4.70 ms    4.90 ms
10k    rabitq      2.19 ms    4.62 ms    4.78 ms
50k    turboquant  4.49 ms    10.2 ms    17.3 ms
50k    rabitq      4.51 ms    9.99 ms    17.2 ms
100k   turboquant  6.49 ms    14.8 ms    25.7 ms
100k   rabitq      6.39 ms    14.8 ms    25.6 ms
```

Packet 006 measured the corresponding recall pattern:

```text
10k:  recall@10 1.0000 for all tested nprobe values.
50k:  nprobe 24 => 0.9450, 96 => 0.9940, 192 => 1.0000.
100k: nprobe 24 => 0.8940, 96 => 0.9860, 192 => 0.9980.
```

## Key Storage Results

Total table+index storage and ec_spire index size:

```text
scale  format      total     ec_spire index
10k    turboquant  167.9 MiB 8.9 MiB
10k    rabitq      168.0 MiB 9.0 MiB
50k    turboquant  836.3 MiB 41.4 MiB
50k    rabitq      836.5 MiB 41.6 MiB
100k   turboquant  1.6 GiB   81.4 MiB
100k   rabitq      1.6 GiB   81.7 MiB
```

The release run shows no meaningful TQ latency or storage advantage over RaBitQ
in this SPIRE width-25 matrix. Quality remains governed by nprobe, per packet
006, and the high-recall 50k/100k points cost about `17.3 ms` and `25.7 ms`.
