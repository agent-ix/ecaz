# Review Request: Byte LUT Kernel Probe

## Summary

This checkpoint prototypes a TurboVec-style byte-pair LUT for our 4-bit no-QJL TQ scorer.

The current exact no-QJL 4-bit paths scan packed bytes without vector decompression:

- Direct scorer: two nibbles per byte, two codebook loads, two multiplies.
- Existing dim-LUT scorer: two nibbles per byte, two per-dimension LUT loads.
- Prototype byte-LUT scorer: one query-side 256-entry LUT row per packed byte, one lookup per byte.

The prototype is `cfg(any(test, feature = "bench"))` only and does not affect production scoring.

## Evidence

Artifact manifest: `reviews/task-86/004-byte-lut-kernel/artifacts/manifest.md`

Focused validation:

```text
cargo test -p ecaz --lib --no-default-features --features pg18 quant::prod::tests::byte_lut_no_qjl_4bit_probe_reports_kernel_delta -- --nocapture
```

Result:

```text
task86_byte_lut_probe dim=1536 candidates=512 repeats=32 scores=16384 direct_ns_per_score=9356.24 dim_lut_ns_per_score=4448.95 byte_lut_ns_per_score=5458.79 byte_lut_speedup_vs_direct=1.714 byte_lut_speedup_vs_dim_lut=0.815 checksum=65.023254
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1977 filtered out; finished in 0.70s
```

## Interpretation

The byte-pair LUT is correct and faster than direct codebook multiply, but it is slower than our existing per-dimension LUT scorer in this focused probe.

That makes it a poor first production target for Task 86:

- It increases per-query LUT memory from `dim * 16` floats to `(dim / 2) * 256` floats.
- It does not beat the existing dim-LUT scorer on the local debug-profile probe.
- It does not change vector storage size or remove a decompression step, because our current no-QJL 4-bit scorer already scans packed bytes directly.

## Review Focus

- Whether this is sufficient negative evidence to deprioritize byte-pair LUTs.
- Whether byte-pair LUTs should still be reconsidered only under a release-profile `ecaz bench suite` index lane, or dropped unless a cache-blocked variant changes the memory behavior.
