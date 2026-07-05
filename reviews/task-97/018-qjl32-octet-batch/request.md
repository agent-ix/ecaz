# Task 97 Packet 018: QJL32 Octet Batch Tail

This packet handles the packet 016 reviewer follow-up: lower the qjl32 AVX2
batch entry from 32 candidates to 8-candidate octets, then keep only true
remainders on scalar scoring.

Code checkpoints:

- `0f4252ef9`: add qjl32 AVX2 octet batch path.
- `fa501976c`: bypass HNSW QJL batching below octet width after local HNSW
  parity remained below `1.0x`.

## Change

- Added `OCTET_WIDTH = 8` to `src/quant/qjl32/mod.rs`.
- Exposed `score_turboquant_qjl_octet8_avx2(...) -> Option<Isa>` for the
  AVX2-only octet path.
- Refactored the existing AVX2 block32 implementation so block32 still runs as
  four octets, while batch tails can call one octet directly.
- Updated `score_turboquant_qjl_batch_inner` to route `floor(n / 32)` blocks,
  then `floor(remainder / 8)` AVX2 octets, then `n % 8` scalar candidates.
- Added an HNSW-only under-octet bypass: QJL exact payload groups smaller than
  8 candidates use the existing scalar scorer/cache path instead of entering
  CandidateBatch.

No block-width contract changed: `score_turboquant_qjl_block32` remains the
32-candidate block kernel. No NEON/SVE path, AWS path, CI configuration, or AM
batching policy beyond the HNSW under-8 bypass changed.

## Counter Contract

Octet-scored candidates are direct block-kernel rows under the dispatched ISA:
`surface=<am> quant=turboquant_qjl isa=avx2 kernel_candidates=<octet count>`.
True remainders are scalar rows under `isa=scalar`.

For HNSW, batches smaller than 8 are intentionally not CandidateBatch rows.
They use `score_and_cache_scan_element`, so Task 99 should not expect
`[block-kernel-counters]` rows for those under-octet groups.

## Local Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test qjl32_ --lib -- --nocapture --color never`
- `cargo test turboquant_exact_payload_flush_bypasses_qjl_batch_under_octet --lib -- --nocapture --color never`
- `cargo test turboquant_qjl_batch_matches_pre_slice_scalar_reference_and_records_counters --lib -- --nocapture --color never`
- `target/debug/ecaz ... dev install ecaz-pg-test --pg 18`
- `target/debug/ecaz ... bench suite run ... --only-tag kernel_on`
- `target/debug/ecaz ... bench suite run ... --only-tag kernel_off`

No GitHub CI or AWS runs were used.

## Local Suite Result

Same fixture as packet 016 (`dim=1024,bits=4,seed=42`) on local x86_64 AVX2,
installed backend SHA
`c47b05a8654ad59ca8db0f0e0bb01af9cf09acf678ddb0a2928d44c36a4b04eb`.

| Surface | Parameter | Kernel off mean | Kernel on mean | Speedup |
| --- | ---: | ---: | ---: | ---: |
| IVF | `nprobe=8` | `1.19 ms` | `1.03 ms` | `1.16x` |
| IVF | `nprobe=16` | `1.51 ms` | `1.20 ms` | `1.26x` |
| SPIRE | `nprobe=8` | `8.91 ms` | `8.65 ms` | `1.03x` |
| SPIRE | `nprobe=16` | `17.4 ms` | `16.8 ms` | `1.04x` |
| HNSW | `ef_search=32` | `1.73 ms` | `1.70 ms` | `1.02x` |

Direct SPIRE scoring counters:

| Parameter | Kernel-off scalar elapsed | Kernel-on AVX2 elapsed | Kernel-on scalar-tail elapsed | Kernel-on total scoring elapsed | Direct scoring speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `nprobe=8` | `22.422690 ms` | `5.691617 ms` | `3.220812 ms` | `8.912429 ms` | `2.52x` |
| `nprobe=16` | `45.124169 ms` | `11.419457 ms` | `6.426059 ms` | `17.845516 ms` | `2.53x` |

HNSW now clears the packet 016 floor: `1.70 ms` kernel-on vs `1.73 ms`
kernel-off. Its direct rows show `3824` AVX2 kernel candidates and `623`
scalar CandidateBatch candidates; under-octet groups are handled by the scalar
cache scorer outside CandidateBatch.

## Request

Please review the qjl32 octet tail path, the HNSW under-octet bypass, and the
packet-local evidence. I am requesting acceptance that the packet 016 follow-up
is closed: SPIRE direct scoring exceeds `1.5x`, and HNSW no longer closes worse
than `1.0x` on the local fixture.
