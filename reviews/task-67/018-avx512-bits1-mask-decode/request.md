# Task 67 Review Request: AVX-512 Bits=1 Mask Decode

## Summary

This packet replaces the pushed query-byte-sum experiment with an AVX-512
bits=1 decode change: the single-candidate and paired bits=1 kernels now build
16-lane dequant vectors with `_mm512_mask_blend_ps` from two packed bytes
instead of materializing stack arrays from the byte LUT.

The packet also adds two cloud-operator fixes needed to make the Intel
measurement reproducible:

- `ecaz cloud install` repairs `/var/lib/pgsql/build` ownership before running
  the postgres-owned build.
- `ecaz cloud bench --simd-mode {scalar,auto}` sets the PostgreSQL systemd
  environment and restarts PostgreSQL before a remote suite.

## Validation

- Local focused test: `cargo test -p ecaz quant::rabitq -- --nocapture`
  passed with 46 tests.
- Cloud install: `327e083ca` installed on `10k-intel`.
- Suite audits passed for both mask-decode configs.

## AWS Intel Measurement

Host lane: `10k-intel`, Intel AVX-512 cloud host, 10k real fixture,
`rabitq` quant_bits=1, nprobe sweep `16,32,64`, 200 queries/iterations.

Recall was unchanged between scalar and auto:

| nprobe | scalar recall@10 | auto recall@10 |
| --- | ---: | ---: |
| 16 | 0.9985 | 0.9985 |
| 32 | 1.0000 | 1.0000 |
| 64 | 1.0000 | 1.0000 |

Latency did not meet the Task 67 speed gate:

| nprobe | scalar mean | auto mean | speedup |
| --- | ---: | ---: | ---: |
| 16 | 2.59 ms | 1.48 ms | 1.75x |
| 32 | 4.04 ms | 1.95 ms | 2.07x |
| 64 | 7.00 ms | 2.80 ms | 2.50x |

## Evidence

See `artifacts/manifest.md` for exact commands, SHAs, and packet-local logs.

## Review Notes

This packet asks for review of the code and measurement evidence, but it does
not claim Task 67 completion. The primary bits=1 gate remains below the target,
and I did not rerun sidecar variants after the primary gate missed.
