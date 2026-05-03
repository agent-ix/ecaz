# Task 31 Quality Suite Rerun After Score-Ranked Probe Order

Reviewer: please review this packet-local quality-candidate rerun after the
probe-order change from `30186`.

## Scope

This packet reruns the Task 31 quality candidate on the committed
`422e5ddd` checkpoint after changing `ec_ivf` to preserve centroid-ranked probe
order through block emission.

The packet-local suite config covers:

- `recall100-candidates-w1000`
- `latency-candidates-w1000`
- `explain-quality-candidate`

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30187-task31-suite-quality-score-ranked/artifacts/install-pg18.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30187-task31-suite-quality-score-ranked/task31-m5-ivf-100k-quality.packet.json --manifest-output review/30187-task31-suite-quality-score-ranked/artifacts/suite-manifest.json --results-output review/30187-task31-suite-quality-score-ranked/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30187-task31-suite-quality-score-ranked/artifacts/status.log bench suite status --manifest review/30187-task31-suite-quality-score-ranked/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30187-task31-suite-quality-score-ranked/artifacts/report.log bench suite report --manifest review/30187-task31-suite-quality-score-ranked/artifacts/suite-manifest.json --results-output review/30187-task31-suite-quality-score-ranked/artifacts/results.jsonl
```

No cargo or pgrx tests were run in this packet; the code checkpoint validation
is recorded in `30186`.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- quality setting under test: `nprobe=96`, `rerank_width=1000`

Recall@100:

- `nprobe=80`: `0.9880`, mean query time `11.98 ms`
- `nprobe=96`: `0.9920`, mean query time `13.08 ms`

Latency:

- `nprobe=80`: p50 `11.3 ms`, p95 `12.4 ms`, p99 `12.8 ms`
- `nprobe=96`: p50 `13.1 ms`, p95 `14.2 ms`, p99 `15.1 ms`

Threshold status:

- `quality-candidate-recall100-floor`: pass (`0.992 >= 0.99`)
- `quality-candidate-p50-budget-ms`: pass (`13.1 <= 15.0`)

Representative EXPLAIN/counters for `nprobe=96,rerank_width=1000`:

- index bytes: `20291584` (`19 MB`)
- execution time: `16.070 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `2420`
- postings pruned by bound: `75340`
- candidates inserted: `2420`
- rerank rows: `1000`

## Comparison To `30183`

At the same `nprobe=96,rerank_width=1000` quality point:

| metric | `30183` | `30187` |
|---|---:|---:|
| recall@100 | `0.9920` | `0.9920` |
| recall mean q-time | `13.35 ms` | `13.08 ms` |
| latency p50 | `12.9 ms` | `13.1 ms` |
| latency p95 | `13.6 ms` | `14.2 ms` |
| latency p99 | `14.0 ms` | `15.1 ms` |
| explain execution | `17.087 ms` | `16.070 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `6509` | `2420` |
| postings pruned by bound | `71251` | `75340` |
| rerank rows | `1000` | `1000` |

## Interpretation

The score-ranked probe walk did what it was designed to do:

- it preserved recall and suite-threshold pass/fail
- it kept the same selected-list count, posting-page count, and posting-visit
  count
- it materially improved pruning, cutting `postings scored` from `6509` to
  `2420` on the representative quality query

The latency signal is not a clear win from this single rerun. The explain
execution time improved, but the 100-iteration latency table stayed roughly
flat-to-worse at the `p96,w1000` point. That means this checkpoint is a real
counter-shape improvement, but not yet a clean benchmark win.

## Next Checkpoint

Use this packet to choose one of two follow-ons:

- rerun balanced and quality latency repeatability on `422e5ddd` to determine
  whether the latency table is just noisy while the counter win is real
- take the next rerank-focused implementation slice, likely around reducing
  heap-f32 rerank overhead now that bound pruning is stronger

## Artifacts

- `artifacts/install-pg18.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/status.log`
- `artifacts/report.log`
- `artifacts/recall100_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/latency_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.sql`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/truth_real100k_n128_k100.json`
- `artifacts/manifest.md`
