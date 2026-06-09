# Task 92 Phase 1 Skeleton Fit Audit

Head SHA: `5cdcf38a529ddec50665a4ea44b806f03383897f`

This audit walks the seven in-scope quant/kernel families through the ADR-076
module convention:

```text
src/quant/<kernel>/
  mod.rs
  scalar.rs
  neon.rs
  sve.rs
  avx2.rs
```

All entries dispatch through the Task 91-selected `QuantCodec::score_ip_batch`
method and use width gating at `CandidateBatch::len() >= 32`.

## Universal Requirements

- `mod.rs` validates candidate count, output count, payload length, and
  metadata shape before scoring.
- Whole block32 ranges use the selected ISA function.
- Tail candidates use the scalar reference implementation.
- Scalar output is bit-exact with pre-kernel scorer output.
- SIMD output is within ADR-076 tolerance: <= 4 ULP or `1e-6` relative error,
  with recall@k preservation as the bench-level gate.
- Runtime dispatch records `(am, quant_kind, isa)` counters and off-path scalar
  counters.

## Quant Walkthrough

| Quant family | Kernel module | Prepared query / model | Candidate metadata | Fit result |
|---|---|---|---|---|
| TurboQuant no-QJL 4-bit | `src/quant/lut32/` after Task 92 backfill | `PreparedLutNoQjl4BitQuery` | `Gamma` or `None`; gamma ignored by scorer | Fits today; Task 87 `lut32.rs` is the reference implementation to split. |
| TurboQuant 2-bit | `src/quant/tq2/` or final task-selected name | LUT or packed query state for 2-bit scorer | `Gamma` for generic scoring; `None` where exact mode ignores it | Fits block32; scalar reference must stay bit-exact with current packed 2-bit scorer. |
| TurboQuant QJL | `src/quant/tq_qjl/` | generic `PreparedQuery` or QJL-specific prepared state | `GammaAndResidualSigns` | Fits if residual signs are borrowed per candidate and validated before block scoring. |
| RaBitQ | `src/quant/rabitq_block/` or final task-selected name | `PreparedEstimator` / `RaBitQScorer` | `RaBitQ` or `None`; gamma must be zero where current AMs require it | Fits; scalar path should call current estimator first, then SIMD block variants can batch payload slabs. |
| grouped-PQ / PqFastScan | `src/quant/grouped_pq_block/` or final task-selected name | query LUT plus trained grouped codebooks/model shape | `GroupedPq { group_count }` | Fits after Task 91 model binding; suffix-max/bound pruning remains prepared-query state. |
| Hamming / binary sidecar | `src/quant/hamming_block/` | packed query words | `Binary` | Fits; correctness is exact integer popcount before AM score polarity conversion. |
| HNSW exact modes | `src/quant/turboquant_exact/` or TurboQuant submodules | `PreparedQuery`, `PreparedLutNoQjl4BitQuery`, tiled LUT, or int8 prepared state | `Gamma`, `GammaAndResidualSigns`, or `None` by exact mode | Fits; HNSW traversal mode stays AM-owned while scorer math moves under QuantCodec. |

## ISA Notes

### Scalar

Scalar modules are the reference implementation. They must not use approximate
math that differs from current pre-kernel scoring.

### NEON

NEON modules are compiled only for `aarch64` and selected only when the runtime
feature detector confirms NEON. NEON is expected on supported aarch64 hosts, but
dispatch still falls back to scalar.

### SVE

SVE modules must be vector-length agnostic. The measurement target is AWS
Graviton 4. A packet may report `sve-256` only if it records a runtime vector
length of 256 bits in its artifacts.

### AVX2

AVX2 modules must account for missing AVX-512/VPOPCNTDQ support. Binary/Hamming
AVX2 kernels should use an AVX2-compatible popcount strategy or fall back to
scalar; they must not silently require AVX-512 features.

## Counter Shape

Task 92 Phase 2 should extend the Task 87 counter surface with:

- `surface`: `spire`, `ivf`, `hnsw`, `diskann`, `unknown`
- `quant_kind`: `turboquant`, `rabitq`, `grouped_pq`, `binary`
- `isa`: `scalar`, `neon`, `sve`, `avx2`
- `flushes`
- `candidates`
- `elapsed_nanos`
- `kernel_flushes`
- `kernel_candidates`
- `kernel_elapsed_nanos`
- `scalar_flushes`
- `scalar_candidates`
- `scalar_elapsed_nanos`

The current Task 87 `lut32_*` fields should become compatibility aliases or be
replaced only with a parser-compatible SQL output format documented in the
Task 92 implementation packet.

## Bench Suite Axis

Task 92 Phase 5 should add a `quant=<name>` axis to `ecaz bench suite`.

Required dry-run behavior:

- expand valid `(AM, corpus, quant, isa, kernel-on/off)` cells;
- skip structurally absent AM/quant combinations with an explicit reason;
- emit `missing_kernel` markers for quant/ISA cells reserved for Tasks 93-98
  but not implemented yet;
- keep raw artifacts under the owning packet, not temporary paths.

## Fit Conclusion

All seven in-scope families fit ADR-076 without redesign. The only prerequisite
is Task 91 Phase 2's grouped-PQ model-binding retouch, which Task 92 already
declares as a dependency before implementation.
