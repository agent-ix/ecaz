---
agent: coder-1
role: coder
model: gpt-5
date: 2026-06-09
seq: 04
---

# Task 94 Phase 4 Review Request — SVE/SVE2 Grouped-PQ Block Backend

## Scope

This packet implements the Phase 4 SVE/SVE2 backend for the grouped-PQ /
PqFastScan 32-candidate block kernel.

Code checkpoint:

- `f67107166 Add grouped-PQ SVE block backend`

## What Changed

- `src/quant/grouped_pq_block/sve.rs` now has an aarch64 SVE/SVE2 backend
  instead of only a scalar fallback stub.
- The backend uses scalar LUT gathers and a vector-length-agnostic SVE
  accumulation loop over the 32 output scores.
- Runtime detection returns `Isa::Sve2` on SVE2 hosts, `Isa::Sve` on base-SVE
  hosts, and `Isa::Scalar` on unsupported hosts.
- Added a `cntw`-based runtime vector-lane helper for future Graviton-4
  evidence packets.
- Added a conditional parity test for real SVE execution when the local host
  supports SVE.

## Validation

Local only:

```text
cargo test grouped_pq_block --lib
```

Result: 7 passed, 0 failed. See `artifacts/test-grouped-pq-block.log`.

Additional local compile probe:

```text
rustc --target aarch64-unknown-linux-gnu --emit=obj /tmp/grouped_pq_sve_backend_probe.rs -o /tmp/grouped_pq_sve_backend_probe.o
```

Result: passed. See `artifacts/sve-asm-probe.log`.

Full aarch64 cargo check was attempted but blocked by missing
`aarch64-linux-gnu-gcc` while building `ring`; see
`artifacts/aarch64-cargo-check-blocked.log`.

No CI, AWS, or benchmark runs were performed.

## Evidence Limits

Runtime vector length was not measured in this packet because this was local
validation only and not the approved Graviton-4 runtime lane. The future
Graviton-4 evidence packet must report the helper's measured value verbatim.
