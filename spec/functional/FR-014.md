---
id: FR-014
title: SIMD Acceleration
type: functional-requirement
status: APPROVED
object_type: process
traces:
  - NFR-001
  - FR-013
  - FR-005
  - FR-017
---
# FR-014: SIMD Acceleration

## Requirement

The extension SHALL provide SIMD-accelerated implementations of performance-critical functions on both x86_64 (AVX2+FMA) and aarch64 (NEON) architectures, with scalar fallback on both.

### Accelerated Functions

| Function | Description | Scalar | AVX2+FMA | NEON |
|---|---|---|---|---|
| `fwht` | Fast Walsh-Hadamard Transform | Yes | Yes | Yes |
| `score_ip_encoded` | LUT-based inner product scoring | Yes | Yes | Yes |
| `score_ip_encoded_lite` | Code-to-code inner product scoring | Yes | Yes | Yes |
| `qjl_bit_expand` | QJL bit unpacking for correction term | Yes | Yes | Yes |

### Runtime Feature Detection

The extension SHALL use runtime CPU feature detection (`std::is_x86_feature_detected!` / `std::arch::is_aarch64_feature_detected!`) to select the fastest available implementation at first call. The detection result SHALL be cached for subsequent calls.

The extension SHALL NOT require AVX2 or NEON at compile time. A pure scalar build SHALL produce correct results on any architecture, with degraded throughput.

### SIMD Correctness Guarantee

For every SIMD-accelerated function, the output SHALL be bit-identical to the scalar implementation (within floating-point associativity tolerance: relative error < 1e-6).

### Build Configuration

- Development builds: `-C target-cpu=native` (enables compile-time SIMD for the host CPU)
- Release/distribution builds: runtime detection only — no hard requirement on any SIMD ISA
- SIMD functions SHALL use `#[target_feature(enable = "...")]` attributes with `unsafe` calling convention, guarded by feature detection at the call site

## Acceptance Criteria

### FR-014-AC-1: Scalar fallback correctness
On a CPU without AVX2 or NEON, all quantizer functions SHALL produce correct results using the scalar path.

### FR-014-AC-2: SIMD-scalar equivalence
For each accelerated function, the SIMD output SHALL match the scalar output within 1e-6 relative error on 1000 random inputs.

### FR-014-AC-3: No SIGILL on unsupported CPU
Running the extension on a CPU without AVX2 (x86_64) or NEON (aarch64) SHALL NOT produce an illegal instruction fault.

### FR-014-AC-4: Throughput improvement
The AVX2+FMA implementation of `fwht` at dim=2048 SHALL achieve at least 3x throughput versus the scalar implementation (measured by `cargo bench`).
