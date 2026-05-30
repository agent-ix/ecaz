# Task 67 Packet 021: Scratch-SoA Bits=1 Measurement

## Summary

This packet reruns the real-10k primary bits=1 Slice J lane with
`ec_ivf.scratch_soa_batch_decode` enabled through `ecaz bench suite`.

The result does not close Task 67:

- Recall is unchanged versus packet 017.
- Scratch-SoA improves total primary bits=1 latency versus packet 017's
  no-scratch scalar baseline.
- The measured auto-SIMD scratch lane still does not meet the 3x headline
  wall-time gate for bits=1.

## Results

Packet 017 no-scratch scalar baseline:

| nprobe | recall@10 | mean latency |
| --- | ---: | ---: |
| 16 | 0.9985 | 2.28 ms |
| 32 | 1.0000 | 3.70 ms |
| 64 | 1.0000 | 6.57 ms |

Packet 021 scratch-SoA auto lane:

| nprobe | recall@10 | mean latency | speedup vs packet 017 scalar |
| --- | ---: | ---: | ---: |
| 16 | 0.9985 | 1.15 ms | 1.98x |
| 32 | 1.0000 | 1.45 ms | 2.55x |
| 64 | 1.0000 | 2.30 ms | 2.86x |

Packet 021 scratch-SoA scalar and auto are nearly identical:

| nprobe | scalar mean | auto mean | auto/scalar |
| --- | ---: | ---: | ---: |
| 16 | 1.13 ms | 1.15 ms | 0.98x |
| 32 | 1.53 ms | 1.45 ms | 1.06x |
| 64 | 2.33 ms | 2.30 ms | 1.01x |

Source artifacts:

- `artifacts/scalar/recall-10k-rabitq1-scratch-soa-scalar.log`
- `artifacts/scalar/latency-10k-rabitq1-scratch-soa-scalar.log`
- `artifacts/auto/recall-10k-rabitq1-scratch-soa-auto.log`
- `artifacts/auto/latency-10k-rabitq1-scratch-soa-auto.log`
- `artifacts/scalar/results.jsonl`
- `artifacts/auto/results.jsonl`

## Interpretation

Scratch-SoA is a real SQL-level improvement for primary bits=1, but it is not
enough to satisfy the Task 67 headline gate. Packet 020 already proves the
AVX-512 bits=1 batch kernel itself is 5.59x faster than scalar; this packet
shows the remaining gap is above or around the scan/query path rather than the
raw kernel loop.

The next Task 67 step should be an implementation slice in `ec_ivf` scan
behavior, not another raw kernel experiment. The most direct candidates are:

- reduce scratch-SoA bookkeeping and candidate recording overhead;
- add a batched path that retains the current min-bound pruning behavior; or
- decide explicitly that the Task 67 headline gate is a kernel-throughput gate,
  not total SQL wall time, and document that with reviewer agreement.

## Validation

- Both packet-local suite configs passed `ecaz bench suite audit`.
- AWS `10k-intel` scalar scratch-SoA suite passed and synced artifacts.
- AWS `10k-intel` auto scratch-SoA suite passed and synced artifacts.
- AWS `10k-intel` was paused after measurement; final status is
  `state: paused`, `~$0.00/hr running`.

Please review whether this establishes the remaining Task 67 work as
`ec_ivf` scan-path overhead above the RaBitQ SIMD kernels.
