# Manifest: Task 95 Packet 002 DiskANN Binary-Sidecar Bench

- Head SHA: `ed0fe1a23` (code under test from packet 001's `4a67d05b0`)
- Task bucket: `reviews/task-95/`
- Packet path: `reviews/task-95/002-diskann-binary-sidecar-bench/`
- Lane: local PG18 pgrx fixture, Apple M5 Pro (arm64, NEON)
- Host/socket: `/Users/peter/.pgrx`, port `28818`; database `task93_bench`
- Extension install: sha256
  `eee435b6b349732b773a48ce8829ef576d310418a23c0aeb21d995f871ae6d11`
  (`install-ecaz-pg18.log`), verified unchanged on disk immediately after
  the suite run (`shasum` prefix `eee435b6b349732b`).
- Fixtures: dbpedia real10k + real100k, 1536-dim; `ec_diskann`
  `storage_format=pq_fastscan` (persisted binary sidecars;
  `prefilter_kind=auto` selects the binary-sidecar prefilter)
- Isolation: prefixes `task95_p2_diskann_pqfs_real{10k,100k}`
- Suite config: `crates/ecaz-cli/suites/task95-phase2-diskann-binary.json`
- Cells: kernel-on (default GUCs) vs kernel-off
  (`ec_diskann.candidate_batch_scoring=off`)

## Run note: discarded first run (mid-run install swap)

The first suite invocation produced inconsistent cells (10k cells with zero
binary counter rows, 100k cells with rows). Root cause: the installed
`ecaz.dylib` changed mid-run — this machine's pgrx tree is shared with the
reviewer agent, and an install landed between the 10k and 100k stages, so
the 10k cells executed a pre-hamming32 build. The first run's artifacts
were discarded; the cited run was executed start-to-finish against the
sha-verified install above. (Also discovered: ad-hoc psql ORDER BY probes
with subquery operands do not use the ANN index — Sort+SeqScan even with
`enable_seqscan=off` — so such probes are not valid counter evidence.)

## Recall byte-equality — PASS

| corpus | kernel-on recall@10 | kernel-off |
|---|---|---|
| real10k | 0.9938 | identical |
| real100k | 0.9719 | identical |

## `[block-kernel-counters]` — `quant=binary isa=neon`, clean toggles

```text
real10k:  surface=diskann quant=binary isa=neon flushes=4026 candidates=39703 kernel_elapsed_ms=0.658755
real100k: surface=diskann quant=binary isa=neon flushes=4272 candidates=74900 kernel_elapsed_ms=1.069569
```

- 16.6 ns/cand (10k) and 14.3 ns/cand (100k) on the in-AM batch path —
  versus 125–285 ns/cand for the f32 RaBitQ kernels on the same surfaces,
  reflecting pure integer XOR+POPCNT work.
- Kernel-off cells emit zero rows.

## End-to-end latency (32 iterations)

| corpus | kernel-on p50/p95 | kernel-off p50/p95 |
|---|---|---|
| real10k | **2.17 / 3.05 ms** | 2.75 / 3.77 ms |
| real100k | **3.78 / 5.76 ms** | 3.91 / 5.84 ms |

Kernel-on is faster at both corpora (10k p50 −21%). Most of this win is
structural: the batch arm scores `tuple.binary_words` (`&[u64]`) directly,
while the per-candidate path converts words→bytes→words with two `Vec`
allocations per candidate.

## Per-ISA scoring share (off-path Criterion, `local-cargo-bench-hamming32.log`)

`cargo bench --features pg18,bench --bench quant_score hamming32`
(release, 32 candidates per iteration):

| bits (words) | scalar | NEON dispatch | ratio |
|---|---|---|---|
| 1536 (24) | 136.5 ns | 116.8 ns | **1.17×** |
| 12288 (192) | 890.1 ns | 810.5 ns | **1.10×** |

**Stop-condition disclosure (< 1.5×, document and continue):** scalar
`u64::count_ones` compiles to hardware popcount on this host, so the NEON
`vcntq_u8` margin is structurally thin — ~4.3 vs ~3.7 ns per candidate at
1536 bits. The kernel keeps integer-exact parity and the end-to-end win
comes from the batch path itself; the same hardware-popcount consideration
is why the AVX2 backend is a documented scalar placeholder pending the
Intel-lane measurement (packet 001 §Design).

## Artifacts

Suite outputs, per-cell load/recall/latency logs, the install log,
Criterion log, shared truth caches.
