# Task 124 Packet 028: TQ scorer profiler and rejected LUT32 kernel attempts

## Summary

This packet is the first post-reopen slice aimed at the actual TurboQuant
scorer/compute path, not IVF scan plumbing.

Landed code:

- added an ignored, bench-feature lib profiler:
  `quant::lut32::tests::task124_profile_lut32_block32_and_query_prep`;
- the profiler reports TQ-internal no-QJL 4-bit LUT block32
  `ns_per_candidate`;
- it also reports explicit no-QJL LUT query-prep `ns_per_iter`;
- it writes packet-local logs through `ECAZ_TQ_PROFILE_LOG`, avoiding shell
  redirection and the pgrx-linked standalone binary startup issue.

Rejected code:

- a NEON even-dimension/full-chunk fast path for the production 1536D shape;
- a NEON byte-index replication rewrite that replaced the inner-loop vector
  multiply with shifts/ORs.

No kernel optimization is proposed for landing in this packet. The attempted
kernel changes did not produce a stable speed win.

## Why this is Task 124 work

The reopened task requires TQ-scorer before/after evidence. Previous packets
used end-to-end query latency and scan-path counters; those are not sufficient
for the remaining scorer-kernel work.

This packet creates a direct measurement surface for the required next phase:

- scoring kernel itself;
- per-query LUT / query-prep cost.

It does not use f32/source comparisons, storage, promotion, nprobe, or scan-path
materialization changes as the answer.

## Validation

- `cargo fmt --check`: passed
- `cargo test --release --lib --features bench quant::lut32::tests::lut32_neon_backend_matches_scalar_reference_bits_when_available -- --nocapture`: passed
- `ECAZ_TQ_PROFILE_ITERS=200000 ECAZ_TQ_PROFILE_LOG=... cargo test --release --lib --features bench task124_profile_lut32_block32_and_query_prep -- --ignored --nocapture`: passed

Artifact source of truth:

- `artifacts/manifest.md`
- `artifacts/lut32-profile-baseline.log`
- `artifacts/lut32-profile-even-full-chunks.log`
- `artifacts/lut32-profile-even-full-chunks-shift-or.log`
- `artifacts/lut32-profile-restored-baseline.log`
- `artifacts/simd-bench-baseline-existing.log`

The `simd_bench` log is retained only as context for the pre-existing
per-candidate scalar-style scorer surface. The reliable Task 124 scorer
measurement surface in this packet is the lib-only ignored profiler test.

## Results

Primary profiler lines:

| Variant | Block32 ns/candidate | Query prep ns/iter | Decision |
| --- | ---: | ---: | --- |
| Restored baseline | 321.9 | 5003.2 | Current code state |
| Initial baseline run | 233.4 | 3443.9 | Useful lower-noise reference, but not final code-state run |
| Even/full-chunk fast path | 232.2 | 3490.7 | Rejected: tiny apparent delta, not stable on rerun |
| Even/full-chunk + shift/OR byte indexes | 402.8 | 3464.5 | Rejected: clear scorer regression |

The first fast-path run was `232.2 ns/candidate` vs the initial baseline
`233.4 ns/candidate`, but later isolated profiler runs on the restored code
were much slower (`321.9 ns/candidate`). That variance is larger than the
apparent 1.2 ns/candidate fast-path delta, so the fast path is not defensible as
a speed win.

The shift/OR byte-index rewrite was a clear regression (`402.8 ns/candidate`) and
was reverted.

## Decision

Land only the profiler. Do not land either attempted NEON kernel change.

This packet intentionally does not close Task 124 and does not defer the
remaining required levers. The next useful slice should use the new profiler to
run a more controlled no-QJL LUT32 kernel pass, then separately tackle query-prep
cost or batch/flush width.
