# Task 67 Packet 023: RaBitQ8 Least-Squares Kernel Evidence

## Summary

This packet extends `ecaz bench rabitq-kernel` with a `bits8ls` row that
measures `PreparedEstimator::estimate_ip_least_squares_scalar_only`, matching
the `rabitq8ls` sidecar rerank scoring path.

The AWS AVX-512 run confirms the least-squares bits=8 path clears the Task 67
per-kernel speed target:

| variant | mode | scalar ns/score | auto ns/score | speedup |
| --- | --- | ---: | ---: | ---: |
| `bits8ls` | `single-least-squares` | 807.72 | 120.74 | 6.69x |

This closes the missing direct kernel row for the fourth Task 67 bits=8
variant. Packets 020 and 023 together now cover `rabitq8`, `rabitq8ls`,
`rabitq8c3`, and `rabitq8c4` at the kernel layer.

## Validation

- `cargo build -p ecaz-cli` passed locally.
- Local `rabitq-kernel` auto smoke run passed and emitted the `bits8ls` row.
- Scalar and auto packet-local suite configs passed `ecaz bench suite audit`.
- AWS `10k-intel` install of `c72003b7b` passed.
- AWS scalar and auto raw kernel suites passed and synced artifacts.
- AWS `10k-intel` was paused after measurement; final status is
  `state: paused`, `~$0.00/hr running`.

## AWS AVX-512 Kernel Results

Code under test: `c72003b7b0438965c586a231b34753d1b745c94f`.

Scalar run backend: `scalar`.
Auto run backend: `avx512f+vpopcntdq+bw+bf16`.

| variant | mode | scalar ns/score | auto ns/score | speedup |
| --- | --- | ---: | ---: | ---: |
| bits1 | batch | 461.74 | 81.71 | 5.65x |
| bits1 | single-dispatch | 449.24 | 134.19 | 3.35x |
| bits4 | batch | 3546.69 | 393.00 | 9.03x |
| bits4 | single-dispatch | 3522.71 | 403.56 | 8.73x |
| bits8 | batch | 819.56 | 68.32 | 12.00x |
| bits8 | single-dispatch | 890.64 | 142.45 | 6.25x |
| bits8ls | single-least-squares | 807.72 | 120.74 | 6.69x |
| bits8c3 | batch | 819.55 | 70.25 | 11.67x |
| bits8c3 | single-dispatch | 818.14 | 135.50 | 6.04x |
| bits8c4 | batch | 819.42 | 70.13 | 11.68x |
| bits8c4 | single-dispatch | 830.11 | 131.47 | 6.31x |

Source artifacts:

- `artifacts/scalar/rabitq-kernel-scalar.log`
- `artifacts/auto/rabitq-kernel-auto.log`
- `artifacts/scalar/suite-manifest.json`
- `artifacts/auto/suite-manifest.json`

## Interpretation

This packet addresses a narrow evidence gap: packet 020 had kernel evidence for
`rabitq8`, `rabitq8c3`, and `rabitq8c4`, but not a row named after the
least-squares `rabitq8ls` sidecar path. The added row confirms that the
least-squares path benefits from the same AVX-512 bits=8 dequant kernel and
clears the 5x per-kernel target.

This packet does not change the SQL wall-time interpretation discussed in
packets 017, 021, and 022.
