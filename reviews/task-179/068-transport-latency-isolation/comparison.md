# Packet 036 bounded-transport isolation

This is an exact historical A/B of packet 036. The before extension is the
checkpoint's parent,
`9a0f21f0824c675d06e9e87747eb36a70859611f`; the after extension is
`ceb15f73ac69fcd98896457c9578fadae2ff0c09`. Both sides use the same release
`ecaz bench suite` runner at `519177225`, staged real corpora, PG18 host,
three-owner topology, degree 32, head cap 4096, BW4/H100, top-k 10, 20
queries, 200 recall trials, 10 warmups, and 30 warm-cache latency iterations.

| Scale | Recall before -> after | Recall-workload mean ms before -> after | Physical p95 ms before -> after | Generation bytes before -> after | Control bytes before -> after |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | `1.0000 -> 1.0000` | `747.71 -> 588.77` (-21.3%) | `92.5 -> 84.8` (-8.3%) | `242,794,496 -> 242,761,728` (-0.01%) | `24,576 -> 24,576` |
| 50k | `0.9800 -> 0.9800` | `558.10 -> 588.97` (+5.5%) | `120.8 -> 115.8` (-4.1%) | `1,242,734,592 -> 1,242,734,592` | `24,576 -> 24,576` |
| 100k | `0.9500 -> 0.9500` | `816.38 -> 936.25` (+14.7%) | `116.8 -> 111.4` (-4.6%) | `2,496,659,456 -> 2,496,659,456` | `24,576 -> 24,576` |

Measured disposition: recall is byte-for-byte unchanged at every scale,
storage is unchanged apart from a 32 KiB (0.01%) 10k page-allocation
difference, and warm p95 is 4.1-8.3% lower after the checkpoint. The broader
200-trial recall-workload mean is mixed: -21.3%, +5.5%, and +14.7%. Because
this packet has one sequential run per arm and the single-index control timing
also moved, it does not attribute a speedup or claim statistical neutrality.
It records the mixed timing directly and shows no consistent adverse warm-p95,
recall, or storage effect from the bounded transport checkpoint.

The checkpoint adds bounded connect/call awaits, remote statement timeouts,
interrupt checks around awaits, and typed error mapping. It does not change
graph construction, quantization, traversal parameters, scoring, posting,
rerank, or storage layout. The matrix nevertheless reports recall, latency,
and storage rather than assuming performance neutrality.
