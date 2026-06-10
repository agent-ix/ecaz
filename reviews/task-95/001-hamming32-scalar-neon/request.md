# Task 95 Packet 001: hamming32 Scalar + NEON Kernels and DiskANN Routing

First Task 95 slice: Phases A and B together (the algebra is integer-exact,
so the scalar/SIMD parity split that drove separate Task 93 phases does not
apply). Code lives on `task-93-rabitq-block-kernel` because it builds
directly on that branch's partial-width dispatch convention and counter
infrastructure (Task 93 packets 004/005), which are not yet on main; the
two lanes are sequenced for merge together.

## Commit under review

- `4a67d05b0` — hamming32 kernels + DiskANN binary-sidecar batch routing.

## Design

- `src/quant/hamming32/{mod,scalar,neon,sve,avx2}.rs` (ADR-076 layout):
  `popcount(query_words XOR candidate_words)` over `u64` sidecar words,
  with block32 + partial (1..=31) entry points following the Task 93
  packet-004 partial-width convention (graph-AM batches rarely reach 32).
- **Integer-exact parity contract**: every backend must produce identical
  `u32` distances — parity tests assert strict equality. No forced-scalar
  anchor vs tolerance split, no ADR-076 ULP framing; this family is exact
  by construction.
- NEON backend: `veorq_u8` + `vcntq_u8` per-byte counts into a `u8` lane
  accumulator, reduced every 31 chunks (31 × 8 = 248 < 256 keeps lanes
  exact for any word count), `vaddlvq_u8` widening reduction, odd trailing
  word scalar. `# Safety` doc on the `target_feature` impl; runtime NEON
  detection; `Isa::Neon` only when used.
- SVE routes through the NEON backend until the Graviton-lane kernel lands
  (same policy packet 93/005 set); `Isa::Sve/Sve2` never reported until a
  real SVE kernel runs.
- AVX2 is a **documented scalar placeholder**: `u64::count_ones` compiles
  to hardware `POPCNT` on x86_64 already, so whether the task plan's
  `vpshufb` nibble-LUT strategy beats it is a Phase D Intel-lane
  measurement question, not an assumption. Counter rows on x86 report
  `isa=scalar` truthfully until that lands.
- `score_hamming_words_batch_for` (candidate_batch.rs): words-based batch
  wrapper — DiskANN sidecar words stay `&[u64]` end-to-end, eliminating
  the per-candidate `Vec` allocations the byte-shaped per-candidate codec
  path performs today. Shape validation precedes counters; attribution is
  `(surface, Binary, isa)` under the packet-004 partial-width semantics.
- DiskANN: `score_diskann_prepared_prefilter_batch` gains a `BinarySidecar`
  arm behind `ec_diskann.candidate_batch_scoring` (same GUC as the RaBitQ
  arm; off restores per-candidate scoring for kernel-off cells).

## Validation

Logs in `artifacts/` at HEAD `4a67d05b0`: clippy `-D warnings` clean;
hamming32 3/3 (block + partial integer-exact across word counts including
odd-word tails, on real NEON), ec_diskann 14/14 (including
`diskann_binary_sidecar_prefilter_batch_is_exactly_per_candidate` — the
batch path must equal per-candidate scoring exactly on any host),
candidate_batch 10/10.

## Deferred

- **Bench cells** (DiskANN PqFastScan fixture with persisted binary
  sidecars, prefilter latency + recall byte-equality kernel-on/off):
  queued behind the Task 93 50k/100k matrix suite currently occupying the
  local bench database; will land as packet 002 with the standard
  SuiteConfig + `[block-kernel-counters]` `quant=binary` rows.
- SVE kernel + Graviton measurement (Phase C) and the AVX2-vs-POPCNT
  question (Phase D) follow the same lane plan as Task 93 packet 005.

## Review request

Please review the integer-exact parity contract, the NEON accumulator
exactness bound, the words-based wrapper shape, and the DiskANN arm gating.
