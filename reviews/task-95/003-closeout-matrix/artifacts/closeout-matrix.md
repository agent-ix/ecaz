# Task 95 Closeout Matrix (per acceptance criteria)

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | hamming32 module: scalar + NEON + SVE + AVX2 | scalar+NEON live; SVE routes through NEON (never reports Isa::Sve until a real kernel runs); AVX2 documented scalar placeholder | packet 001 |
| 2 | DiskANN binary-sidecar routes through batch method | done (words-based, behind ec_diskann.candidate_batch_scoring) | packets 001/002 |
| 3 | Recall byte-equal at every cell | PASS (10k 0.9938, 100k 0.9719; identical on/off) | packet 002 |
| 4 | ≥2× scoring-share per ISA | NEON measured **1.17×/1.10×** — invokes the task's per-ISA stop condition (document and continue): scalar u64 count_ones is hardware POPCNT on every target host, so the SIMD margin is structurally thin for Hamming | packet 002 Criterion log |
| 5 | End-to-end no regression | PASS — kernel-on p50 −21% (10k) / −3% (100k), driven by eliminating per-candidate word↔byte allocations | packet 002 |
| 6 | pg_test DiskANN binary-sidecar surfaces | deferred to a Linux runner per the standing macOS `_BufferBlocks` dyld policy; compile gates + live bench cells (real PG18 backend) cover the touched behavior | packet 002 |
| 7 | Safety docs on intrinsic modules | done (# Safety on the NEON impl; exactness bound documented) | packet 001 |
| 8 | Per-AM closeout matrix | this artifact |

## SVE (Phase C) scope-down — data-driven

SVE `cnt` performs the same per-byte popcount as NEON `vcntq_u8`. NEON
already measures only 1.10–1.17× over the scalar reference because the
scalar path is hardware `POPCNT`/`CNT` on every deployment target; an SVE
variant of identical algebra cannot plausibly clear the 1.5× per-ISA bar.
The only consuming surface (DiskANN prefilter) batches at ~10 candidates
average — far below block width — so wider vectors add no block leverage
either. Mirroring the accepted Task 96 stop-condition and Task 98 Phase A
scope-down precedents: **the Graviton 4 SVE measurement for Task 95 is
scoped out**; if a future binary surface produces ≥32-wide batches, the
question reopens with that surface's width data.

## AVX2 (Phase D) disposition

The open question is empirical and Intel-hosted: whether a `vpshufb`
nibble-LUT kernel beats scalar hardware `POPCNT` for ~24-word sidecars.
The placeholder keeps `isa=scalar` truthful on x86. This is the single
remaining measurement, owned by the Intel desktop lane; the same
1.10–1.17× NEON result bounds the expected return.
