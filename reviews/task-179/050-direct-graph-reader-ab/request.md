# Review request: direct physical graph reader A/B

## Scope

Please review this isolated 10k/50k/100k measurement packet for implementation
commit `afcc2d6af`, the packet 020 P2-3b remediation requested in packet 049.

The immutable baseline is packet 048's normal persisted-head candidate at
source head and runner `24ec63788`. The direct-reader candidate installs source
head `8d0c7d6bb`, whose only production code change from that baseline is
`afcc2d6af`; intervening commits contain review evidence only. Both arms use
the same unchanged release CLI runner and normal `pg18` feature set.

Every scale uses the same staged corpus and queries, three physical owners,
graph degree 32, head index cap 4096, 20 recall queries (200 trials), 10
untimed same-connection warmups, and 50 measured latency queries at
concurrency 1. The candidate suite is a path/port-renamed copy of the packet
048 config and requires `seed_strategy=persisted_head`.

## Result

The direct reader is recall-neutral across the complete matrix. Its warmed
latency result is mixed and tracks the same-data host control rather than
showing an attributable improvement or regression:

| Scale | Recall baseline -> direct | Mean ms baseline -> direct | Mean delta | p95 ms baseline -> direct | p95 delta | Single-index mean delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 -> 1.0000 | 42.40 -> 43.20 | +1.89% | 55.00 -> 55.50 | +0.91% | +10.89% |
| 50k | 0.9800 -> 0.9800 | 57.10 -> 54.10 | -5.25% | 74.40 -> 68.30 | -8.20% | -6.80% |
| 100k | 0.9500 -> 0.9500 | 50.90 -> 51.90 | +1.96% | 69.10 -> 70.00 | +1.30% | +1.47% |

The physical and single-index arms move in the same direction at every scale.
Accordingly, this single-run A/B does not support claiming the 50k reduction
as a direct-reader win, nor the small 10k/100k increases as regressions. It
supports a narrower finding: removing per-hop dynamic SPI planning and tuple
copying is performance-neutral within the observed run-to-run host movement.

Physical storage differs by two 8 KiB pages at 10k, one page at 50k, and zero
at 100k. Those +0.006749%, +0.000659%, and 0% movements are relation
page-allocation noise; the implementation does not change the storage format.
See `artifacts/comparison.md` and `artifacts/manifest.md` for p99, build time,
topology, and provenance details.

## Validation state

The candidate suite manifest reports three completed steps, zero failures,
zero missing artifacts, and zero stale steps. All 12 configured topology,
recall, latency, remote-engagement, and strategy-specific thresholds pass.
Every scale has the exact source row count across three Published owners, zero
non-owned rows, zero orphans, and both remote owners verified through the
custom scan.

Packet 049 separately records strict production and benchmark-feature clippy
plus the exact-commit live PG18 three-owner fixture. This packet supplies the
required scan-behavior A/B rather than repeating that validation.

## Requested decision

Please confirm that the combined packet 049 implementation review and this
required matrix close packet 020 P2-3b. In particular, please confirm that the
native relation/index lifecycle and corruption checks are sound and that the
measured recall/storage neutrality with no attributable latency movement is
acceptable. This packet does not close Task 179 or unrelated open findings.
