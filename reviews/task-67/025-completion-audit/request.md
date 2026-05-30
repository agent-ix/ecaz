# Task 67 Packet 025: Completion Audit

## Summary

This packet is a completion audit for Task 67 after packets 020-024.
It does not introduce code. It asks the reviewer to decide the remaining
performance-gate interpretation:

- If Task 67's headline gate is judged at the RaBitQ scoring/prepared-estimator
  kernel layer, the task is complete. Packets 020 and 023 show the required
  Intel AVX-512 kernel throughput for bits=1, bits=4, all four bits=8 variants,
  and the bits=1 batched path.
- If Task 67's headline gate is interpreted as total SQL query wall time at
  every measured `nprobe`, the task is not complete. Packets 021, 022, and 024
  show the best SQL-level bits=1 result reaches 3x only at `nprobe=64`.

My recommendation is to close Task 67 on the kernel-throughput interpretation.
The task wording says "end-to-end RaBitQ scoring throughput", not total query
wall time, and reviewer feedback for packets 020 and 023 agrees the remaining
flatness lives above the RaBitQ kernel path.

## Requirement Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| Slice A: x86 feature detection and dispatch slots | `reviews/task-67/001-x86-dispatch-slots/`, reviewer approved | Complete |
| Slices B/C: AVX-512 and AVX2 bits=1 kernels | `006-x86-bits1-kernels/`, `018-avx512-bits1-mask-decode/`, `019-avx512-bits1-sign-flip/`, final restored implementation in `12ed902df`; packet 020 AWS bits1 rows | Complete |
| Slices D/E: AVX-512 and AVX2 bits=4 kernels | `004-x86-bits4-kernels/`, `005-x86-bits4-batch/`, packet 020 AWS bits4 rows | Complete |
| Slices F/G: AVX-512 and AVX2 bits=8 kernels | `002-x86-bits8-kernels/`, `003-x86-bits8-batch/`, packet 020 AWS bits8/bits8c3/bits8c4 rows, packet 023 bits8ls row | Complete |
| Slice H: batched scoring for bits=1 and bits=8 | `003-x86-bits8-batch/`, `005-x86-bits4-batch/`, `007-x86-bits1-batch/`, packet 020 AWS batch rows | Complete |
| Slice I: bf16 evaluation on Intel | `008-x86-bf16-kernel/`, packet 020/023 backend is `avx512f+vpopcntdq+bw+bf16`; no separate gate flip beyond existing feature-gated kernel | Complete |
| Differential tests | `011-x86-differential-scaffold/` and `012-x86-scaffold-safety/`, reviewer approved after safety follow-up | Complete |
| Intel benchmark validation | Packets 017, 020, 021, 022, 023, 024 all run on AWS `10k-intel`; packet-local suite configs and manifests present | Complete |
| Recall: no real-10k regression within 0.5 pp | Packet 017 and packets 021/022/024 report `0.9985 / 1.0000 / 1.0000` at nprobe 16/32/64, matching baseline | Complete |
| Per-kernel performance targets | Packet 020: bits1 batch 5.59x, bits4 batch 9.02x, bits8 family batch ~11.8x and single-dispatch >=5.62x; packet 023: bits8ls 6.69x | Complete |
| Headline bits=8 variants | Packet 020 covers `rabitq8`, `rabitq8c3`, `rabitq8c4`; packet 023 covers `rabitq8ls`; all clear 4x at the kernel layer | Complete |
| Headline bits=1 batched path | Packet 020 bits1 batch is 5.59x, clearing the 3x kernel-throughput gate | Complete |
| Safety: target_feature/unsafe hygiene | Packets 006-012 reviewer feedback approved; packet 022 and later code changes do not touch SIMD unsafe boundaries | Complete |
| Dispatch shape constraint | Work is in x86 slots/kernels and bench harness; no evidence of rejected edits to `sum_query_dequant_with_bf16` or `estimate_ip_*_impl` bodies | Complete |
| SQL total wall-time interpretation | Packet 022 best result is 2.11x / 2.52x / 3.07x vs packet 017 scalar; packet 024 score ordering regressed | Not complete under this stricter interpretation |

## Key Performance Evidence

Packet 020 AWS AVX-512 kernel results:

| variant | mode | scalar ns/score | auto ns/score | speedup |
| --- | --- | ---: | ---: | ---: |
| bits1 | batch | 456.83 | 81.67 | 5.59x |
| bits4 | batch | 3547.63 | 393.13 | 9.02x |
| bits8 | batch | 817.25 | 69.50 | 11.76x |
| bits8 | single-dispatch | 827.94 | 145.90 | 5.67x |
| bits8c3 | batch | 819.06 | 69.39 | 11.80x |
| bits8c3 | single-dispatch | 811.84 | 141.29 | 5.75x |
| bits8c4 | batch | 818.39 | 69.55 | 11.77x |
| bits8c4 | single-dispatch | 889.27 | 158.16 | 5.62x |

Packet 023 AWS AVX-512 kernel result:

| variant | mode | scalar ns/score | auto ns/score | speedup |
| --- | --- | ---: | ---: | ---: |
| bits8ls | single-least-squares | 807.72 | 120.74 | 6.69x |

Best current SQL-level bits=1 evidence, from packet 022:

| nprobe | recall@10 | mean latency | speedup vs packet 017 scalar |
| --- | ---: | ---: | ---: |
| 16 | 0.9985 | 1.08 ms | 2.11x |
| 32 | 1.0000 | 1.47 ms | 2.52x |
| 64 | 1.0000 | 2.14 ms | 3.07x |

Packet 024 confirms that score-ordering scratch batches was not the right
remaining SQL-level path: it regressed the packet 022 SQL result to
2.00x / 2.26x / 2.80x and was reverted in `db821441e`.

## Review Request

Please review this audit and decide whether Task 67 can close on the
kernel-throughput reading of the headline gate. If the reviewer requires total
SQL wall-time at every `nprobe`, the concrete remaining work is further
`ec_ivf` scan-path optimization above the already-accepted RaBitQ kernels.
