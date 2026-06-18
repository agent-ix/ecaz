# Task 111c: IVF Page-Aware Scatter Scorer (score-in-place)

Status: **proposed**.
Priority: P0 latency (the score-in-place win).
Parent: `111-ivf-scan-dense-posting-block-layout.md`.
Depends on: **`111b-ivf-columnar-frozen-list-format.md`** (the columnar format).
Evidence anchor: `reviews/task-111a/{004,007,008}`.

Current risk note (2026-06-17): packets `reviews/task-111c/002-*` and
`reviews/task-111c/003-*` prove the reference TQ page-scatter path is correct
and zero-copy, but currently slower than the copy fallback. Packet 003 measured
31,649 approximate scan us / 35.775 ms execution for page scatter versus 16,589
approximate scan us / 20.720 ms execution for the copy fallback on the 50k TQ
columnar fixture. Promotion depends on fixing the per-page-contiguity/scoring
geometry so scattered zero-copy beats contiguous-copy-then-sequential-read.

## Goal

Score the Task 111b columnar frozen-list **in place**: feed the SIMD block
scorer wide batches (≥32, e.g. 64/128 to amortize per-flush overhead) read
**directly from the page cache, with no assembly copy**, by making each block
kernel's transpose source its per-candidate rows across page boundaries.

This is the slice that makes the columnar format actually beat Approach A. A
pays a scratch-gather copy; this pays none. B paid fragmentation; the columnar
format already removed that. Net target: the theoretical floor — every payload
byte read exactly once from cache, batch width decoupled from the page.

## Why

111b lands the columnar format but still copies page-aligned posting runs into
scratch to reuse the existing contiguous kernels (so 111b ≈ A on latency). The
kernels already transpose row-major payloads into per-dim columns
(`transpose_8x16` and siblings) every query — that transpose *is* a gather from
per-candidate row pointers. If those row pointers can cross page-resident
columns, the transpose reads payloads directly from the buffer and the separate
assembly copy disappears. The SIMD reduction is unchanged; only address
generation + page-boundary handling change.

From 111a evidence the copy is material (≈4 ms / ~13% of an rb8 100k scan at
65 MB), and it scales with payload size — so removing it should lift every
spanning quant mode (TQ + rb2/4/8), with rb1 already fine per-block.

## Scope

- A **page-aware scatter variant** of each active block kernel: consumes a
  column reader (an ordered row-pointer / segment iterator over the 111b
  payload column) and produces width-W scores, handling the page-header
  discontinuity at each 8 KB boundary, with no contiguous reassembly.
  - Per **codec**: TurboQuant (no-QJL / QJL), RaBitQ {1,2,4,8}, grouped-PQ.
  - Per **ISA**: AVX2 (Intel), SVE2-128 (Graviton 4), NEON (M5).
  - Reuse the existing transpose/reduction; generalize only the row source.
- Configurable batch width W (≥32; tune 64/128) decoupled from page capacity.
- Pinned-buffer-set lifetime: hold all pages a batch spans pinned for the scoring
  call (PG18 read-stream batch), since payloads are borrowed not copied.
- Score-index mapping: scores returned in logical-list order; preserve
  deleted-bitmap filtering, live-tid budget, dedup, heap-tid expansion (all keyed
  by logical position).
- Per-ISA coverage gates per ADR-077 (each kernel/regime is an independent
  target); equivalence tests vs the 111b copy-based scan (identical scores).
- Keep the 111b copy-based scan as a correctness fallback / diagnostic GUC.

## Non-Goals

- The columnar format itself (Task 111b).
- Pre-transposed canonical block geometry (Task 111d) — this task keeps the
  scan-time transpose, just feeds it from scattered pages.
- Host-pinned compaction (future 111e).
- Changing scoring math / recall / quantization.

## Phases

1. **One codec × one ISA reference** (e.g. TurboQuant on AVX2): the scatter
   transpose-row-source + page-boundary handling + pinned-buffer set; prove
   identical scores vs the 111b copy scan; measure copy elimination.
2. **Fan out across codecs** (TQ no-QJL/QJL, RaBitQ {1,2,4,8}, grouped-PQ) on
   the reference ISA, each with equivalence tests + width counters.
3. **Fan out across ISAs** (SVE2-128 Graviton, NEON M5) with per-ISA coverage
   gates.
4. **Benchmark gate.** Full matrix vs Approach A and vs 111b copy-scan: latency
   p50/p95/p99, recall parity, flush-width, per-group copy-bytes (must drop to
   ~0), pages read, build time, index size — TQ + RaBitQ {1,2,4,8}, 50k/100k.
   Escalate to 1M (AWS) only if 50k/100k show the score-in-place win and default
   promotion is on the table.

## Acceptance Criteria

1. Page-aware scatter scorer implemented for the active codecs across AVX2,
   SVE2-128, and NEON, behind the gate, with per-ISA coverage gates met.
2. Scores are bit-identical to the 111b copy-based scan (equivalence tests).
3. The per-group/per-scan assembly copy-bytes counter drops to ~0 on the
   columnar path.
4. SIMD flush widths reach the configured W (≥32) for every spanning quant mode.
5. Recall and NDCG unchanged across the matrix.
6. A benchmark packet shows score-in-place **beats Approach A** on latency at
   the high-recall cells (TQ + rb2/4/8) with no recall regression, and reports
   the storage/page-read picture; it makes an explicit promote/iterate decision
   for the columnar score-in-place layout as the dense default.

## Dependencies and Coordination

- Hard dependency on Task 111b (format + page-aware reader).
- ADR-077 block-kernel coverage convention governs the per-ISA gates.
- Enables Task 111d (pre-transpose) and the future 111e host-pinned hatch.
- 1M cells are an AWS lane (Graviton 4 + Intel); both little-endian, so the
  columnar format is volume-portable across them.
