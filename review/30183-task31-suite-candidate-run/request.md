# Task 31 Candidate Suite Run

Reviewer: please review the packet-local Task 31 suite execution results.

## Scope

This packet runs the Task 31 candidate-tagged suite slice through the new
`ecaz bench suite` runner, with a packet-local config copy so the raw artifacts
land under `review/30183-task31-suite-candidate-run/artifacts/` instead of the
older `30178` dry-run directory.

The candidate slice executed:

- `recall100-candidates-w500`
- `recall100-candidates-w1000`
- `latency-candidates-w1000`
- `explain-quality-candidate`

I also ran a storage-only suite step under the same packet so the storage claim
is packet-local instead of inferred from the explain output.

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30183-task31-suite-candidate-run/artifacts/audit.log bench suite audit --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json
/Users/peter/.cargo/bin/ecaz --log-file review/30183-task31-suite-candidate-run/artifacts/audit-packet-config.log bench suite audit --config review/30183-task31-suite-candidate-run/task31-m5-ivf-100k.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30183-task31-suite-candidate-run/task31-m5-ivf-100k.packet.json --only-tag candidate --continue-on-error --manifest-output review/30183-task31-suite-candidate-run/artifacts/suite-manifest.json --results-output review/30183-task31-suite-candidate-run/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30183-task31-suite-candidate-run/artifacts/status.log bench suite status --manifest review/30183-task31-suite-candidate-run/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30183-task31-suite-candidate-run/artifacts/report.log bench suite report --manifest review/30183-task31-suite-candidate-run/artifacts/suite-manifest.json --results-output review/30183-task31-suite-candidate-run/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30183-task31-suite-candidate-run/task31-m5-ivf-100k.packet.json --only storage-real100k-n128 --continue-on-error --manifest-output review/30183-task31-suite-candidate-run/artifacts/storage-manifest.json --results-output review/30183-task31-suite-candidate-run/artifacts/storage-results.jsonl
```

No code changes or tests were run for this packet; this is a measurement-only
checkpoint.

## Results

Quality candidate still looks like:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- quality setting under test: `nprobe=96`, `rerank_width=1000`

Candidate recall and latency:

- `w500`, `nprobe=80`: recall@100 `0.9639`, mean query time `10.38 ms`
- `w500`, `nprobe=96`: recall@100 `0.9676`, mean query time `11.11 ms`
- `w1000`, `nprobe=80`: recall@100 `0.9880`, mean query time `12.10 ms`
- `w1000`, `nprobe=96`: recall@100 `0.9920`, mean query time `13.35 ms`
- `w1000`, `nprobe=80`: latency p50 `11.4 ms`, p95 `12.1 ms`, p99 `12.9 ms`
- `w1000`, `nprobe=96`: latency p50 `12.9 ms`, p95 `13.6 ms`, p99 `14.0 ms`

Threshold status from the candidate manifest:

- `quality-candidate-recall100-floor`: pass (`0.992 >= 0.99`)
- `quality-candidate-p50-budget-ms`: pass (`12.9 <= 15.0`)

Supplemental storage step:

- IVF index size: `19.4 MiB` (`202.9 B` per row)
- total indexes: `23.7 MiB`
- table footprint: `1.6 GiB`
- explain query at `nprobe=96`, `rerank_width=1000`: index bytes `20291584`
  (`19 MB`), execution time `17.087 ms`

The storage-only manifest reports threshold failure because it did not execute
the recall and latency rows that those thresholds depend on. The candidate-run
manifest is the source of truth for pass/fail.

## Artifacts

- `artifacts/audit.log`
- `artifacts/audit-packet-config.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/status.log`
- `artifacts/report.log`
- `artifacts/recall100_real100k_pqg8_n128_w500_p80_96.log`
- `artifacts/recall100_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/latency_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.sql`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/truth_real100k_n128_k100.json`
- `artifacts/storage-manifest.json`
- `artifacts/storage-results.jsonl`
- `artifacts/storage_real100k_pqg8_n128.log`
- `artifacts/manifest.md`
