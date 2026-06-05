# Packet 019: Measurement Closeout Repair

## Summary

This packet repairs packet 001's missing measurement context and records the current synthetic/real100k closeout edge cases. It is deliberately **not** a full Task 65b closeout request: the synth10k L200 point and real100k recall edge are both close, but not clean strict passes.

There is no code change in this packet.

## Packet 001 Carryovers

### Host Topology

`artifacts/host-topology.log` records the local reference host:

- MacBook Pro `Mac17,9`
- Apple M5 Pro
- 18 total cores: 6 Super + 12 Performance
- 64 GB memory

`artifacts/precheck-host.log` records the PostgreSQL side:

- PostgreSQL 18.3
- `shared_buffers=128MB`
- `maintenance_work_mem=64MB`
- baseline `max_parallel_maintenance_workers=2`

The packet 014/019 parallel runs use PGOPTIONS overrides to request 8 maintenance/parallel workers for the tuned measurements.

### Probe vs SQL Timing

The packet 001 `6.72s` SQL build vs `62.045s` probe divergence is a measurement-surface mismatch, not a SQL build regression:

- SQL timing is from the real PostgreSQL `ambuild` path.
- `bench diskann-build-probe` is a standalone Vamana-core diagnostic for algorithm counters and should not be used as the closeout wall-clock oracle.
- Later timing evidence uses backend `ec_diskann_ambuild_timing` rows parsed from the actual loader/SQL build path.

### Synth10k

The historical fixture files were not present in the checkout, so this packet regenerated a deterministic stream fixture:

- generate 10,200 vectors with `ecaz corpus generate --seed 42`
- first 10,000 rows as corpus
- final 200 rows as queries

The generated TSV files are not committed because of size, but the generation commands, loader hashes, suite configs, manifests, and results are packet-local.

## Results

Task 65 accepted synth10k R32/L200 reference:

| list size | Task 65 recall@10 |
|---:|---:|
| 64 | 0.1610 |
| 200 | 0.2625 |
| 800 | 0.3270 |

Current Task 65b stream-fixture results:

| config | build | L64 | L200 | L800 | verdict |
|---|---:|---:|---:|---:|---|
| w8/b1 R32/L200 | 16.96s | 0.1690 | 0.2570 | 0.3285 | L200 misses strict 0.2575 floor by 0.0005 |
| w8/b64 R32/L200 | 4.26s | 0.1645 | 0.2500 | 0.3255 | faster, but L200 misses floor |

The b1 path is the serial-equivalence batch path and preserves L64/L800, but it is still 0.05pp short at L200 under a literal 0.5pp threshold. The tuned b64 path is not acceptable for synth L200 if synth is treated as a strict gate.

## Real100k Recall Edge

Packet 014's late b1024 artifacts show:

| config | build | L200 recall@10 |
|---|---:|---:|
| w8/b512 | 32.53s | not rerun |
| w8/b768 | 29.77s | 0.9700 |
| w8/b1024 | 27.12s | 0.9675 |

If the real100k recall floor is interpreted against Slice A L200 `0.9755`, the strict floor is `0.9705`. The b768 time-gate cell misses that by 0.0005; b1024 is faster but worse on recall.

## Review Ask

Please review this as an evidence packet for the remaining closeout decision:

- Accept b1 synth as statistically equivalent despite a 0.0005 strict miss at L200, or require another synth tuning slice.
- Accept b768 real100k as the time-gate cell with an explicitly documented 0.0005 strict recall miss, or require another real100k tuning slice.

My recommendation is not to mark Task 65b complete from this packet alone; it narrows the remaining work to those two threshold decisions.
