# Task 224 locality gate calculation

Date: 2026-08-25 (America/Los_Angeles)

This calculation uses only the four `physical_benchmark_latency`,
`physical_benchmark_stage`, and
`physical_benchmark_materialization_work` row families in `run/results.jsonl`.
All four shapes ran at the same release extension SHA and reused the exact same
100k physical generation.

## Timer interpretation

- `materialize_owner_binary_send_work` is the summed time in the exact
  PostgreSQL `SendFunctionCall` for every projected non-NULL datum. It includes
  any detoast work performed by the type's binary sender and excludes wrapper,
  sender-cache, array, SPI, and Rust response work.
- `materialize_owner_payload_spi_work` encloses the SPI query, heap lookup,
  payload array construction, profiled-send wrapper, and Rust decoding of the
  returned arrays.
- The conservative MAT-25 heap/SPI ceiling is therefore
  `payload_spi_work - binary_send_work`. It deliberately overstates a pure
  heap/TID-locality opportunity because it also contains non-send SPI,
  executor, wrapper, array, and decode work.
- `materialize_owner_response_construct_work` is timed after SPI decoding and
  is excluded from both candidate ceilings.
- Stage `mean_ms` is summed owner work divided by custom-scan executions, which
  is the same convention used by the preregistered `ms/scan` gate. Owner work
  can overlap, so each percentage is a ceiling rather than a promised
  end-to-end saving.

## Registered 100k gate

A candidate passes if its independently addressable bucket is at least
1.000000 ms/scan or at least 5% of that shape's matched warm end-to-end mean.
If MAT-25 and MAT-26 both pass, only the candidate with the larger percentage
ceiling advances.

| Projection shape | Warm mean (ms) | Payload SPI (ms/scan) | Binary send (ms/scan) | MAT-25 conservative residual (ms/scan) | MAT-26 share | MAT-25 share | Gate result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| id-only | 11.800000 | 0.491981 | 0.005083 | 0.486898 | 0.043076% | 4.126254% | neither |
| narrow scalar | 12.100000 | 0.563287 | 0.010019 | 0.553268 | 0.082802% | 4.572463% | neither |
| vector-bearing | 28.200000 | 7.799571 | 6.967996 | 0.831575 | 24.709206% | 2.948848% | MAT-26 |
| toasted | 46.100000 | 2.049820 | 0.431869 | 1.617951 | 0.936809% | 3.509655% | MAT-25 by the 1 ms rule |

The toasted predicate needed 479 custom-scan executions for 200 end-user
queries because qualified deepening continued until it found enough matching
rows. Even if the toasted MAT-25 residual is instead normalized by end-user
query, its aggregate ceiling is 3.874993 ms/query, or about 8.41% of the
46.10 ms query mean. It remains substantially below MAT-26's 24.709206%
vector-bearing ceiling, so the tie-break is unchanged under either
normalization.

## Locality and TOAST observations

All figures below are means per custom-scan execution.

| Shape | Requested TIDs | Distinct heap blocks | Displaced rows under TID sort | External TOAST values | Total hit/read blocks | Send hit/read blocks | Logical/send bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| id-only | 6.785000 | 6.770000 | 4.890000 | 0.000000 | 6.785000 / 0.000000 | 0.000000 / 0.000000 | 54.280000 / 54.280000 |
| narrow scalar | 6.785000 | 6.770000 | 4.890000 | 0.000000 | 6.785000 / 0.000000 | 0.000000 / 0.000000 | 162.840000 / 162.840000 |
| vector-bearing | 6.785000 | 6.770000 | 4.890000 | 6.785000 | 35.005000 / 3.205000 | 28.220000 / 3.205000 | 41,904.160000 / 83,564.060000 |
| toasted | 16.580376 | 16.524008 | 10.903967 | 8.075157 | 55.052192 / 7.356994 | 38.471816 / 7.356994 | 103,526.947808 / 103,494.647182 |

The scalar shapes are warm-buffer heap fetches with essentially one requested
row per heap block. The wide shapes attribute all reported buffer reads to the
binary-send region, and every requested vector value is externally toasted.
PostgreSQL reported zero timed read nanoseconds, so the result supports a CPU /
detoast / binary-serialization ceiling, not a claim of physical-storage I/O
dominance.

## Disposition requested from the reviewer

**Advance MAT-26 only. Do not advance MAT-25.**

MAT-26 passes both registered thresholds in the vector-bearing arm:
6.967996 ms/scan and 24.709206% of the matched warm mean. MAT-25 technically
passes the absolute threshold in the toasted arm, but loses the preregistered
single-candidate tie-break. Packet 003 should therefore implement one isolated
block-batched detoast/binary-send candidate and prove or reject it with a
same-generation 100k A/B before any 10k/50k/100k closeout matrix.
