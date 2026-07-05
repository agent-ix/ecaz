# Task 67 Review Request: AVX-512 Bits=1 Sign-Flip Experiment

## Summary

This packet records an abandoned AVX-512 bits=1 experiment. Commit
`47fed5ba2` changed the AVX-512 bits=1 kernels to sign-flip query lanes and
add them instead of multiplying by +/-1 dequant lanes. The math was exact for
bits=1, and the focused RaBitQ tests passed, but the AWS Intel benchmark was
slower than scalar in the primary lane.

Because the measurement was negative:

- `47fed5ba2` was reverted by `5c51abfc8`.
- The prior byte-LUT AVX-512 bits=1 implementation was restored and pushed as
  `12ed902df`.
- This packet does not ask to merge the sign-flip optimization as a forward
  path.

## Validation

- Local focused test for `47fed5ba2`: `cargo test -p ecaz quant::rabitq -- --nocapture`
  passed with 46 tests.
- AWS install for `47fed5ba2` completed on `10k-intel`.
- Scalar and auto primary suites completed and synced artifacts.

## AWS Intel Measurement

Host lane: `10k-intel`, 10k real fixture, `rabitq` quant_bits=1, nprobe
sweep `16,32,64`, 200 queries/iterations.

Recall was unchanged:

| nprobe | scalar recall@10 | auto recall@10 |
| --- | ---: | ---: |
| 16 | 0.9985 | 0.9985 |
| 32 | 1.0000 | 1.0000 |
| 64 | 1.0000 | 1.0000 |

Latency regressed relative to scalar:

| nprobe | scalar mean | auto mean | scalar/auto |
| --- | ---: | ---: | ---: |
| 16 | 1.12 ms | 1.19 ms | 0.94x |
| 32 | 1.49 ms | 1.56 ms | 0.96x |
| 64 | 2.16 ms | 2.35 ms | 0.92x |

## Evidence

See `artifacts/manifest.md` for commands, SHAs, S3 run IDs, and packet-local
logs.

## Result

Negative experiment. The code was reverted and the byte-LUT implementation was
restored. Task 67 remains incomplete.
