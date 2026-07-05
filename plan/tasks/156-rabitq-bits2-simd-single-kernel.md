# Task 156: RaBitQ bits=2 single-candidate SIMD kernels (currently true scalar everywhere)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P3

## Why

bits=2 has **zero per-candidate SIMD on any architecture**: kernel selection
only branches on bits 1/4/8 (`select_x86_query_dequant_slot`,
`src/quant/rabitq.rs:2037`; `select_neon_query_dequant_kernel`, `:2105`), so
bits=2 falls to the generic scalar kernel (`:2076`, `:2131`). Its only SIMD
coverage is the multi-bit block transpose (`mb_neon`/`mb_avx2`), reachable
only through the 32-block driver. Everything else runs true scalar: single-code
scoring (e.g. rerank_format='rabitq2' if ever implemented, per-candidate
fallback paths), the `<4`/`<8` transpose tails (`mb_neon.rs:88`,
`mb_avx2.rs:106`), and `estimate_ip_batch`, which **errors** on bits=2
(`rabitq.rs:1119`) — which is also why SPIRE's slab scorer cannot batch
bits=2 (`src/am/ec_spire/quantizer/mod.rs:748` routes bits!=1 to
`estimate_ip_batch`).

## Scope

- Add NEON + AVX2 single-candidate bits=2 kernels (2-bit unpack → dequant LUT
  or arithmetic form, whichever microbenches faster) and register them in both
  selectors; lift the bits=2 rejection in `estimate_ip_batch` once a batch
  path exists (pair kernel optional — measure whether single + hoisted
  dispatch suffices).
- Differential-test byte-equality vs the scalar kernel.
- A/B on a bits=2 consumer lane at 10k/50k/100k (IVF rabitq2 storage lane;
  include SPIRE bits=2 if that surface is active).

## Out of Scope (hard)

- No change to the multi-bit block transpose routing (Task 93/99 decisions
  stand). No new bits=2 consumers — this makes existing paths faster only.

## Gate / Exit Criteria

- Byte-equal recall, and either a measured latency win on a real bits=2 lane
  or an honest negative + a recorded decision that bits=2 traffic is too
  marginal to justify routing (in which case the kernels stay unrouted).
  If no production lane exercises bits=2 meaningfully, say so with counters
  and close as low-value — do not manufacture a consumer.
