# Review Request: 50k Cold-Cache Candidate Sweep

Task: 111h

Head SHA: `9b6f91a8dedc124c0f27fe062285ed3c77c0b4a7`

This packet adds a PG18 `ecaz bench suite` cold-start probe for the 50k fixture across:

- `source/f32` width 32
- `index/f16` width 32
- `index/rabitq4` width 128
- `index/rabitq8` width 64 with `rabitq_rerank_least_squares=0`, `rabitq_rerank_clip=4`
- `index/turboquant` width 32

The suite completed successfully:

- `artifacts/suite-audit.log`: 46-step audit passed.
- `artifacts/suite-status.log`: `completed=46 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/summary-cold-start-50k.md`: compact recall/latency/storage table.
- `artifacts/manifest.md`: command provenance and artifact inventory.

Important caveats:

- Latency is intentionally one cold-start query per nprobe (`iterations=1`, `concurrency=1`), not a stable percentile distribution.
- The suite ran `dev evict-relation-cache --prefix ...` immediately before each latency step. Those suite evict steps succeeded in `suite-manifest.json` / `suite-status.log`, but their per-step log files are 0-byte in this run. I added post-run dry-run probes (`artifacts/evict-dry-run-*.log`) to prove relation/file coverage.
- The dry-run probes cover the corpus heap, toast heap/index, pkey, and IVF index for each prefix. They do not evict the query table or simulate remote storage.
- `artifacts/suite/truth-50k-k10.json` is a generated truth cache and is intentionally not committed.

Headline results:

| Candidate | Recall@10 np32 / np128 / np200 | Single-query cold latency np32 / np128 / np200 | IVF index | Total footprint |
| --- | --- | --- | --- | --- |
| `source/f32` w32 | 0.9520 / 0.9875 / 0.9895 | 5.91 ms / 9.99 ms / 12.1 ms | 13.8 MiB | 808.7 MiB |
| `index/f16` w32 | 0.9520 / 0.9875 / 0.9895 | 21.7 ms / 10.1 ms / 12.5 ms | 172.5 MiB | 967.4 MiB |
| `index/rabitq4` w128 | 0.9200 / 0.9450 / 0.9460 | 17.5 ms / 12.8 ms / 13.7 ms | 54.0 MiB | 848.9 MiB |
| `index/rabitq8` clip4 w64 | 0.9550 / 0.9915 / 0.9930 | 7.84 ms / 14.1 ms / 13.3 ms | 93.4 MiB | 888.3 MiB |
| `index/turboquant` w32 | 0.9300 / 0.9550 / 0.9565 | 7.74 ms / 10.9 ms / 12.7 ms | 62.3 MiB | 857.1 MiB |

Preliminary read:

- `source/f32` still looks like the strongest default: smallest IVF index, high recall, and best/tied cold latency samples.
- Current `index/f16` remains hard to justify: same recall as `source/f32`, much larger index, and no cold-start latency win in this probe.
- `index/rabitq8` clip4 is the only index-side quantized candidate that recovered source-quality recall on this 50k fixture, but it is larger and not faster than `source/f32`.
- `index/rabitq4` and `index/turboquant` remain below source-quality recall in this run.

Review focus:

1. Is the cold-start methodology acceptable as a diagnostic packet, given the explicit single-sample limitation?
2. Is the eviction evidence sufficient now that the suite evict logs are empty but the suite status and dry-run probes are packet-local?
3. Should clipped RaBitQ8 be retained as an iteration candidate only, rather than promoted, because the broader 10k/50k/100k/1M matrix does not yet include this exact knob combination?
