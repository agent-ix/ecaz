# Review Request: Task 144 Packet 007 - 10k Release Matrix Pilot

## Scope

This packet provides the first release-mode `ecaz bench suite` evidence for the
Task 144 A/B matrix at 10k:

- variants: `single`, `fixed_b2`, `closure_e010_b8`
- query modes: fixed nprobe, `ec_spire.probe_distance_ratio=1.25`, adaptive nprobe
- metrics: distinct recall, latency p50/p95, scanned route/list counts, storage
- per-query artifacts: stage containment and result identity JSONL for all nine
  pipeline cells

No code changed in this packet. It is evidence only, following packets 004/005
for the default-off pruning and closure code and packet 006 for the suite config.

## Key Results

The release backend is installed and verified:

- `artifacts/install-release-pg18.log` records the installed `ecaz.so` SHA256:
  `a821e3ee67501cc7489dcc9380e2bfab867b33388f600ef1f8109d19751a5bf8`
- `artifacts/precheck-release-profile.log` records `ecaz_build_profile() = release`

Load/storage succeeded through `ecaz bench suite`:

| variant | storage total | index total | index per row |
|---|---:|---:|---:|
| single | 177.2 MiB | 17.9 MiB | 1880.1 B |
| fixed_b2 | 194.1 MiB | 34.9 MiB | 3655.3 B |
| closure_e010_b8 | 177.7 MiB | 18.5 MiB | 1938.2 B |

The nine pipeline cells succeeded after generating the truth cache. Best 10k
recall rows:

| cell | best distinct recall | nprobe | p50 | p95 | scanned |
|---|---:|---:|---:|---:|---:|
| single fixed | 0.9935 | 96 | 7.610 ms | 8.666 ms | 19200 |
| fixed_b2 fixed | 0.9955 | 96 | 9.761 ms | 10.776 ms | 19200 |
| closure_e010_b8 fixed | 0.9940 | 96 | 8.606 ms | 10.335 ms | 19200 |
| fixed_b2 adaptive | 0.9965 | 96 | 9.774 ms | 10.654 ms | 19200 |
| closure_e010_b8 adaptive | 0.9955 | 96 | 8.222 ms | 9.779 ms | 19200 |
| fixed_b2 ratio125 | 0.9150 | 96 | 8.395 ms | 9.391 ms | 411 |

At ratio `1.25`, probe distance pruning is clearly active but too aggressive in
this pilot: it reduces scanned routes/lists from 19,200 to 411 at nprobe 96, but
the best distinct recall only reaches 0.9150 for `fixed_b2` and 0.7680 for
`closure_e010_b8`.

## Evidence

Packet source of truth:

- `artifacts/manifest.md`

Suite outputs:

- `artifacts/suite-manifest-10k-r3.json`
- `artifacts/results-10k-r3.jsonl`
- `artifacts/suite-manifest-10k-r4-pipelines.json`
- `artifacts/results-10k-r4-pipelines.jsonl`

Per-cell logs and per-query evidence:

- `artifacts/load-10k-*.log`
- `artifacts/storage-10k-*.log`
- `artifacts/pipeline-10k-*.log`
- `artifacts/stage-containment-10k-*.jsonl`
- `artifacts/result-identity-10k-*.jsonl`

## Reviewer Notes

The suite config from packet 006 audits and dry-runs, but this packet found a
real execution gap: `bench spire-pipeline --include-recall` expects
`--truth-cache-file` to already exist. I generated `truth-10k-k10.json` via
`ecaz bench recall` to complete the 10k pilot, but I did not commit that file
because AGENTS.md bans truth-cache commits.

Before running the 50k/100k release matrix, I plan to add a first-class
truth-cache prerequisite step or runner support so clean suite runs do not need
manual cache generation.
