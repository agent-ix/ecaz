# Artifact manifest — aws-round-rabitq-ivf

Authoritative vs failed/incomplete logs in this packet's
`artifacts/` directory. Annotated 2026-05-22 in response to
reviewer feedback `2026-05-22-04-reviewer.md`.

## Authoritative

| File | Covers | Cited in |
| --- | --- | --- |
| `latency-truly-prewarmed.log` | 10k+50k bits=4 latency after the prewarm-quoting fix | `results.md` 50k cells (pre-bits=1) |
| `explain-counters.log` | EXPLAIN counter dump for 50k attribution | `results.md` "EXPLAIN counter attribution" |
| `latency-bits1-v3.log` | 50k bits=1 first-cut scalar-select kernel | `results.md` bits=1 + rerank scalar-select column |
| `latency-bits1-bytelut-fixed.log` | 50k bits=1 byte-LUT (post per-query hoist) | `results.md` byte-LUT column |
| `latency-bits1-width-sweep.log` | 50k width sweep that motivated the new default of 50 | `results.md` width-50 plateau finding |
| `latency-bits1-width50-nprobe-sweep.log` | 50k bits=1 width=50 full nprobe curve | `results.md` final 50k operating curve |
| `latency-100k-bits1.log` | 100k bits=1 + width=50 latency + recall | `results.md` 100k closure |
| `latency-1m-bits1-v3.log` | 1m bits=1 latency (prewarmed, post-warm cells valid) | `results.md` 1m latency table |
| `recall-1m-bits1-q500.log` | 1m bits=1 recall at q=500, ground-truth via exhaustive f32 scan, on m8g.2xlarge | `results.md` 1m recall table |
| `closure-prep.log` | 100k+1m corpus prepare + 100k load | reference for sizes/timings |
| `closure-1m-load.log` | 1m load + index build (1126s build, 1905s total) | reference |
| `cloud-snapshot-closeout.log` | snap-01838d965fa09c433 (post-bits=1 round, pre-100k/1m) | snapshot inventory |

## Failed / incomplete (kept for traceability)

| File | Why it failed | Recover via |
| --- | --- | --- |
| `latency-bits1.log` | First bits=1 build path was missing quant_bits threading (commit 466d436e3); index wrote 4-bit codes but metadata claimed bits=1 → "posting tuple length mismatch: got 857, expected 281" | superseded by `latency-bits1-v3.log` |
| `latency-bits1-bytelut.log` | Bash ate `(byteLUT)` parens in the SSM echo step before any bench rows ran | superseded by `latency-bits1-bytelut-fixed.log` |
| `latency-bits1-v2.log` | SSM dollar-quoting bug ate the inline prewarm SQL; sweep didn't reach the latency rows | superseded by `latency-bits1-v3.log` |
| `latency-1m-bits1.log` | First 1m bench used inline `pg_prewarm($$..$$)` which SSM ate; latency rows are post-warm so valid for nprobe ≥ 64, but the recall step at q=500 was truncated by the 24 KB SSM stdout limit | latency superseded by `latency-1m-bits1-v3.log`; recall by `recall-1m-bits1-q500.log` |
| `latency-1m-bits1-v2.log` | Same SSM dollar-quoting bug (heredoc with `$$..$$`); only the prewarm header captured | same as above |
| `recall-1m-bits1.log` | First 1m recall pass OOM-killed on m8g.xlarge (16 GB) loading 5.8 GB corpus into CLI memory for ground-truth | superseded by `recall-1m-bits1-q500.log` after resizing host to m8g.2xlarge |
| `cloud-snapshot-1m.log` | Zero-byte log; the snapshot CLI succeeded and the snap ID (`snap-0975811a1da6ea302`) is in `docs/aws-bench-workflow.md` and the round-closeout commit message, but the log file itself never captured stdout | snapshot ID is authoritative; this file is a stub |

## Why kept

The failed files document the **iteration history** — they show the
specific bugs (bits=1 build-path threading, SSM dollar-quoting,
OOM on small host) and the commits that fixed each one. Removing
them would erase the trail that led to the final results. The new
labels here let a reader skip them on first read.
