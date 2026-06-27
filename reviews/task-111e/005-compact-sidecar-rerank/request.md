# Task 111e: Compact Sidecar Rerank Evidence

## Summary

This packet measures the compact rerank representation lever that was still
missing from Task 111e. It uses a fresh 50k real-corpus RaBitQ-1 dense
page-local candidate frontier with `rerank=off`, then runs table-side sidecar
reranking for `f32`, `f16`, and `rabitq8` at `candidate_k=50` and
`candidate_k=100`.

The result is a narrow decision:

- Keep `f16` as the viable compact table-side rerank representation. It
  preserves the `f32` recall/NDCG in this slice while halving bytes.
- Reject `rabitq8` for the immediate high-recall Task 111e path. It is much
  smaller and faster, but recall@10 falls to 0.9460.
- Prefer table-side `tid-sorted` placement/read order over random-id reads.
  Random reads dominate p50 IO even at 50 to 100 candidates.

This is a measurement packet only; no code changes are under review in this
checkpoint.

## Evidence

Artifacts are under
`reviews/task-111e/005-compact-sidecar-rerank/artifacts/`.

The suite config is packet-local:

```text
reviews/task-111e/005-compact-sidecar-rerank/artifacts/task111e-compact-sidecar-suite.json
```

The runner completed:

```text
audit passed: 3 steps
suite report: completed 3, failed 0, skipped 0
```

## Key Results

At `candidate_k=50`, `f16` matched `f32` quality:

| Variant | Read mode | Recall@10 | Sidecar size | Bytes touched p50 | Sidecar p50 |
| --- | --- | ---: | ---: | ---: | ---: |
| f32 | tid-sorted | 0.9940 | 292.97 MiB | 300.00 KiB | 4.938 ms |
| f16 | tid-sorted | 0.9940 | 146.48 MiB | 150.00 KiB | 6.563 ms |
| rabitq8 | tid-sorted | 0.9460 | 73.81 MiB | 75.59 KiB | 1.577 ms |

At `candidate_k=100`, the same pattern held:

| Variant | Read mode | Recall@10 | Sidecar size | Bytes touched p50 | Sidecar p50 |
| --- | --- | ---: | ---: | ---: | ---: |
| f32 | tid-sorted | 0.9960 | 292.97 MiB | 600.00 KiB | 9.225 ms |
| f16 | tid-sorted | 0.9960 | 146.48 MiB | 300.00 KiB | 12.036 ms |
| rabitq8 | tid-sorted | 0.9460 | 73.81 MiB | 151.17 KiB | 2.273 ms |

The placement/read-order lever is large. At `candidate_k=100`, `tid-sorted`
table-side reads cut p50 IO from:

```text
f32:     33.676 ms random-id -> 2.975 ms tid-sorted
f16:     53.006 ms random-id -> 1.921 ms tid-sorted
rabitq8: 31.732 ms random-id -> 1.214 ms tid-sorted
```

## Caveat

The candidate SQL p50 in this packet is about 251 to 272 ms, which is much
slower than earlier Task 111e packet 001 frontier runs. This packet should
therefore be reviewed as comparative evidence for compact representation and
table-side placement/read order, not as final end-to-end promotion latency.

## Review Ask

Please review whether this is sufficient to close the compact rerank
representation question for 111e by carrying `f16` forward, rejecting `rabitq8`
for the immediate high-recall path, and explicitly separating true index-side
rerank placement into a follow-up implementation slice.
