# Artifact Manifest

- head SHA: `2dc3fd22b68b3796c438b1eed393124cc11dcd7a`
- task bucket: `reviews/task-65b`
- packet: `reviews/task-65b/019-measurement-closeout`
- timestamp UTC: `2026-06-05T23:11:00Z`
- lane: local PG18, `ec_diskann`, synthetic 10k, `pq_fastscan`
- storage format: `pq_fastscan`
- isolation: one index per table prefix

## Packet 001 Carryover: Host Topology

Artifact:

- `host-topology.log`

Key host result:

- MacBook Pro `Mac17,9`
- Apple M5 Pro
- 18 total cores: 6 Super + 12 Performance
- 64 GB memory

PostgreSQL precheck:

- `precheck-host.log`
- PostgreSQL 18.3 on `aarch64-apple-darwin25.2.0`
- `shared_buffers=128MB`
- `maintenance_work_mem=64MB`
- baseline `max_parallel_maintenance_workers=2`
- benchmark suite load steps override PGOPTIONS to `max_parallel_maintenance_workers=8` and `max_parallel_workers=8` where noted.

## Packet 001 Carryover: Probe / SQL Reconciliation

The packet 001 probe-vs-SQL divergence remains explained as a measurement-surface mismatch:

- SQL build timing comes from the real PostgreSQL `ambuild` path and includes heap scan, quantizer training/encoding, Vamana build, persistence, and metadata/page writes.
- `bench diskann-build-probe` is a standalone in-memory diagnostic over the Vamana graph core. It reconstructs exact vector distances and emits algorithm counters; it is useful for shape diagnostics such as visited counts, backlinks, reprunes, and in-degree distribution, but it is not a wall-clock proxy for SQL `CREATE INDEX`.
- Later packet 014/019 timing evidence uses the backend `ec_diskann_ambuild_timing` NOTICE parsed by the suite, so closeout timing is anchored to the SQL build path, not to the probe.

## Synthetic Fixture Generation

Generated data is intentionally not committed because the TSVs are large. The deterministic commands are recorded in:

- `generate-synth10200-stream.log`
- `generate-synth10k-corpus.log`
- `generate-synth10k-queries.log`

The accepted closeout comparison uses the stream fixture:

- command: `./target/debug/ecaz --log-file reviews/task-65b/019-measurement-closeout/artifacts/generate-synth10200-stream.log corpus generate --output data/task65b_synth10k_stream_closeout/synth10200.tsv --n 10200 --dim 1536 --seed 42 --kind corpus`
- split: first 10,000 rows are corpus; final 200 rows are queries.
- corpus SHA256 from loader: `ccd9a13cdf99eded145fe92ba65d135a57495b55513444caf35c54d5bdcc6f2f`
- query SHA256 from loader: `155086601cd7b0487dab8cd6d4418faf0d2bfd4e0a8b7410d3adbdd31bd81b71`

The earlier separate-query-seed run is retained as exploratory evidence only:

- corpus seed `42`
- query seed `4242`
- best L200 recall was lower than the stream fixture and is not used for the gate comparison.

## Synthetic Results

Task 65 accepted synth10k R32/L200 reference:

- L64/L200/L800 Recall@10: `0.1610 / 0.2625 / 0.3270`
- strict 0.5pp floor at L200: `0.2575`

### `workers=8`, `batch=1`, R32/L200

Artifacts:

- `synth10k-b1-suite-manifest.json`
- `synth10k-b1-results.jsonl`
- `load-synth10k-w8-b1-r32-l200.log`
- `recall-synth10k-w8-b1-r32-l200.log`
- `storage-synth10k-w8-b1-r32-l200.log`

Key result:

- build_index: `16.96s`
- backend total_ms: `16961`
- `parallel_effective_workers=8`
- `parallel_batch_size=1`
- Recall@10 L64/L200/L800: `0.1690 / 0.2570 / 0.3285`

Gate interpretation:

- L64 improves vs Task 65.
- L800 improves vs Task 65.
- L200 is `0.2570`, which is `0.0005` below the strict `0.2575` floor.

### `workers=8`, `batch=64`, R32/L200

Artifacts:

- `synth10k-recall-storage-manifest.json`
- `synth10k-recall-storage-results.jsonl`
- `load-synth10k-w8-b64-r32-l200.log`
- `recall-synth10k-w8-b64-r32-l200.log`
- `storage-synth10k-w8-b64-r32-l200.log`

Key result:

- build_index: `4.26s`
- backend total_ms: `4256`
- `parallel_effective_workers=8`
- `parallel_batch_size=64`
- Recall@10 L64/L200/L800: `0.1645 / 0.2500 / 0.3255`

Gate interpretation:

- The tuned batch-64 setting is materially faster but does not preserve the synth L200 floor.

## Packet 014 Real100k Recall Edge

The b1024 artifacts from packet 014 show:

- build_index: `27.12s`
- Recall@10 L200: `0.9675`

This is faster than b768 but lower recall. The current real100k choice remains a trade-off:

- b512: `32.53s`, over the 30s time gate.
- b768: `29.77s`, Recall@10 L200 `0.9700`.
- b1024: `27.12s`, Recall@10 L200 `0.9675`.

No real100k cell in the currently available evidence cleanly satisfies both strict time and strict recall-vs-Slice-A if the real100k recall floor is interpreted as `0.9755 - 0.005 = 0.9705`.
