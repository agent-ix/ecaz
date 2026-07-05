# Task 97 Packet 016: QJL32 AVX2 Candidate-Parallel Transpose

This packet implements the packet 011 required optimization slice: transpose
the qjl32 AVX2 block kernel from one-candidate-at-a-time SIMD over dimensions
to eight-candidates-at-a-time SIMD with one vector lane per candidate.

Code checkpoint: `a6b0bfd8b0d37efc3559960e18ec01e40f9bb13b`

## Change

- Replaced the qjl32 AVX2 inner loop in `src/quant/qjl32/avx2.rs`.
- The new loop processes each block32 as four octets of eight candidates.
- For each octet, it decodes each candidate's 8-dim 3-bit packed word once,
  transposes those decoded sublanes across candidates, and accumulates MSE/QJL
  in AVX2 registers.
- Final gamma scaling remains per-candidate and per-lane.

No AM batching policy, block width, scalar reference, NEON/SVE path, AWS path,
or CI configuration changed.

## Local Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test qjl32_ --lib -- --nocapture --color never`
- `cargo bench --features bench --bench quant_score 'quant/qjl32_block32' -- --sample-size 10 --warm-up-time 1 --measurement-time 2`
- `target/debug/ecaz ... dev install ecaz-pg-test --pg 18`
- `target/debug/ecaz ... bench suite run ... --only-tag kernel_on`
- `target/debug/ecaz ... bench suite run ... --only-tag kernel_off`

No GitHub CI or AWS runs were used.

## Micro-Bench Result

Criterion qjl32 block32 rows:

- scalar: `[34.613 us 35.383 us 36.611 us]`
- dispatch: `[7.9057 us 8.0926 us 8.3355 us]`

Median dispatch-vs-scalar speedup: `4.37x`.

For comparison, packet 011 measured the old dispatch at about `28.6 us`; this
slice moves the local dispatch row from roughly `1.22x` over scalar to `4.37x`.

## Local Suite Result

Same-head local PG18 suite slices used the existing Task 97 QJL fixture
(`dim=1024,bits=4,seed=42`) and installed backend SHA
`e8dd061ef58acd1700781bf8d769589cc645f74fc42b2ff813d4be2ca27e5818`.

End-to-end latency ratios:

| Surface | Parameter | Kernel off mean | Kernel on mean | Speedup |
| --- | ---: | ---: | ---: | ---: |
| IVF | `nprobe=8` | `1.16 ms` | `1.04 ms` | `1.12x` |
| IVF | `nprobe=16` | `1.50 ms` | `1.21 ms` | `1.24x` |
| SPIRE | `nprobe=8` | `8.70 ms` | `8.79 ms` | `0.99x` |
| SPIRE | `nprobe=16` | `16.9 ms` | `17.1 ms` | `0.99x` |
| HNSW | `ef_search=32` | `1.71 ms` | `1.86 ms` | `0.92x` |

Direct SPIRE scoring counters:

| Parameter | Kernel-off scalar elapsed | Kernel-on AVX2 elapsed | Kernel-on scalar-tail elapsed | Kernel-on total scoring elapsed | Direct scoring speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `nprobe=8` | `21.905986 ms` | `3.438826 ms` | `12.719015 ms` | `16.157841 ms` | `1.36x` |
| `nprobe=16` | `44.298653 ms` | `7.532742 ms` | `24.671362 ms` | `32.204104 ms` | `1.38x` |

The AVX2 kernel rows themselves are much faster, but scalar tails still keep
the direct scoring-share ratio below the Task 97 `1.5x` AVX2 stop-condition
floor. Per packet 011's decision rule, this is now credible AVX2 stop-condition
evidence because the candidate-parallel transpose has been tried.

IVF kernel-on emitted direct `isa=avx2` rows, but kernel-off emitted no direct
block-counter rows, matching packet 011's caveat. HNSW remains scalar-tail-only
under the local `m=8` fixture.

## Request

Please review the transposed qjl32 AVX2 code and the local evidence. I am
requesting acceptance that the required optimization slice was attempted and
that the remaining local AVX2 scoring-share miss is a scalar-tail/AM batching
limit, not the previous AVX2 inner-loop shape.
