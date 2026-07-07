# Task 158: RaBitQ scalar-tail SoA plane (vector-wide correction, drop per-candidate tail parsing)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P3 (format-touching; gate on Task 152 attribution)

## Why

Every RaBitQ code carries 3 interleaved tail f32s — `‖o‖`, `o_dot`, `‖x_dec‖`
(`RABITQ_SCALAR_LEN = 12`, `src/quant/rabitq.rs:67`). Because they are
interleaved with the packed code (AoS), every finish step re-reads them with
unaligned `from_le_bytes` plus validity branches, **per candidate, per lane,
after each SIMD sum**: `finish_scalar_only_estimate` (`rabitq.rs:4520`),
`estimate_ip_impl` (`:4294-4308`), and the block kernels' per-lane epilogues
(`rabitq32/neon.rs:96-97`, `mb_neon.rs:122-124`, `mb_avx2.rs:153-155`). The
correction itself — `norm * sum / (o_dot * x_norm)` — is trivially
vectorizable across a 32-block if the three scalars lived in a separate SoA
plane; today the layout forbids it. This is the only genuinely per-candidate
scalar work left in the scoring path (query prep is fully hoisted).

The natural vehicle is the scratch/dense layer, not the base tuple format:
the IVF SoA scratch already copies payloads into a slab
(`src/am/ec_ivf/scan.rs:454-481`) and could split code/tail planes during the
copy for zero extra traffic; the dense posting block and columnar formats
(`0x29`) are gated formats where a plane split is a contained change.

## Scope

- Phase 1 (no format change): split the tail scalars into a parallel plane at
  IVF SoA scratch-fill time; add a block-wide vectorized correction epilogue
  to the bits=1 block path; A/B at 10k/50k/100k.
- Phase 2 (only if Phase 1 wins and Task 152 shows the epilogue share
  matters): extend the plane split to the dense/columnar gated formats as a
  reviewed format slice with the full A/B matrix per CLAUDE.md.

## Out of Scope (hard)

- No base (row) posting-format change. No change to the estimator arithmetic
  or the `O_DOT_FLOOR` validity semantics — same math, different data layout.

## Gate / Exit Criteria

- Byte-equal recall (same arithmetic, reassociated only where provably
  identical; otherwise within documented ulp bounds with recall-parity
  evidence) and a measured latency delta at 10k/50k/100k for Phase 1.
  Phase 2 proceeds only on a Phase 1 win plus attribution support.
