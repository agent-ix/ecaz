# Task 92 Acceptance Matrix

Head SHA: `84ab8c433b987bc135a40fdc8ec7934e9ae66a76`

## Acceptance Criteria

| Criterion | Evidence | Status |
|---|---|---|
| ADR-076 accepted | `spec/adr/ADR-076-universal-block-kernel-pattern.md`, `spec/adr/index.md`, Packet 016 | Met |
| `(AM, quant, isa)` counter surface live/queryable | `src/am/common/candidate_batch.rs`, `src/lib.rs`, `crates/ecaz-cli/src/commands/bench/mod.rs`, Packets 003/004/007 | Met |
| Off-path counter validated against Task 87 LUT32 | Packet 014 local SPIRE TurboQuant LUT32 calibration: same workload had `flushes=1024`, `candidates=65453`, kernel-on `lut32_flushes=1024`, kernel-off `lut32_flushes=0`; reviewer approved as local calibration smoke | Met for Task 92 infrastructure; Graviton 4 runtime evidence deferred |
| ISA helper unit-tested | `src/quant/isa.rs` tests and Packet 005 | Met |
| LUT32 module-layout backfill | `src/quant/lut32/{mod,scalar,neon,sve,avx2}.rs`, Packets 006/008 | Met |
| Skeleton/template documented and checked across in-scope quants | `docs/block-kernel-development.md`, Packet 001 skeleton fit audit, Packet 009 docs | Met |
| Bench suite quant axis with missing-kernel marker | `crates/ecaz-cli/suites/task92-quant-axis-smoke.json`, `crates/ecaz-cli/src/commands/bench/suite.rs`, Packets 010/011 | Met |

## Deferred Scope

Task 92 ships infrastructure and safe fallback stubs; it does not ship a new
real SVE2 block kernel. Per the task file and `docs/block-kernel-development.md`,
AWS Graviton 4 evidence is deferred to Tasks 93-98 when the first real SVE2
backend lands.

The first such packet must include:

- a Graviton 4 smoke result showing runtime dispatch reports `Isa::Sve2`;
- measured runtime vector length before any width-specific claim;
- direct `(AM, quant, isa)` counter rows, including `isa=sve2` for real SVE2
  kernel rows and `isa=scalar` for scalar tails;
- full Graviton 4 benchmark evidence only when that kernel task makes a
  performance claim.

The Packet 014 reviewer noted that the suite latency artifacts captured
`[task87-counters]` compatibility rows rather than direct
`[block-kernel-counters]` rows. The closeout explicitly carries that as Task
93+ evidence scope: direct rows must be present in the first real SVE2 kernel
packet, where the row can prove something material.
