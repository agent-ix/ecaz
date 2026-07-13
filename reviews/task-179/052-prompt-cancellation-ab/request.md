# Review request: prompt cancellation A/B

## Scope

Please review this isolated 10k/50k/100k measurement packet for implementation
commit `a94e5e9be`, the packet 036 P3 remediation requested in packet 051.

The immutable baseline is packet 050's direct-reader candidate at source head
`8d0c7d6bb`. The prompt-cancel candidate installs source head `9387f72b3`,
whose only production code change from that baseline is `a94e5e9be`;
intervening commits contain review evidence only. Both arms use the same
unchanged release CLI runner and normal `pg18` feature set.

Every scale uses the same staged corpus and queries, three physical owners,
graph degree 32, head index cap 4096, 20 recall queries (200 trials), 10
untimed same-connection warmups, and 50 measured latency queries at
concurrency 1. The candidate config differs from packet 050 only in names,
paths, tags, run directory, and ports.

## Result

The prompt-cancellation poll is recall-neutral and shows no material or
consistent warmed-latency overhead:

| Scale | Recall baseline -> prompt poll | Mean ms baseline -> prompt poll | Mean delta | p95 ms baseline -> prompt poll | p95 delta | Single-index mean delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 -> 1.0000 | 43.20 -> 43.50 | +0.69% | 55.50 -> 55.70 | +0.36% | -0.70% |
| 50k | 0.9800 -> 0.9800 | 54.10 -> 54.50 | +0.74% | 68.30 -> 67.90 | -0.59% | +2.74% |
| 100k | 0.9500 -> 0.9500 | 51.90 -> 49.50 | -4.62% | 70.00 -> 67.40 | -3.71% | +2.89% |

The physical means move slightly up at 10k/50k and down at 100k; p95 changes
direction at 50k, and p99 changes direction again at 100k. Same-data control
movement also varies. This one-run A/B therefore supports performance
neutrality within observed host/tail variation, not a poll-induced regression
or a claimed 100k speedup.

Physical storage is identical at 10k, differs by minus two 8 KiB pages at 50k,
and plus one page at 100k. Those 0%, -0.001318%, and +0.000328% changes are
ordinary relation page-allocation noise; the implementation does not change
the storage format. See `artifacts/comparison.md` and
`artifacts/manifest.md` for p99, build-time, topology, and provenance details.

## Validation state

The candidate suite manifest reports three completed steps, zero failures,
zero missing artifacts, and zero stale steps. All 12 configured topology,
recall, latency, remote-engagement, and strategy-specific thresholds pass.
Every scale has the exact source row count across three Published owners, zero
non-owned rows, zero orphans, and both remote owners verified through the
custom scan.

Packet 051 separately records strict clippy and exact-commit live PG18 proofs
that both mid-await and mid-connect cancellation complete in under one second
under 10-second remote budgets, followed by same-backend reuse.

## Requested decision

Please confirm that packet 051 plus this required matrix close packet 036 P3:
foreground cancellation is prompt and guard-safe, while recall, storage, and
measured warmed latency remain acceptable. This packet does not close Task 179
or unrelated fault-window and Task 172 gates.
