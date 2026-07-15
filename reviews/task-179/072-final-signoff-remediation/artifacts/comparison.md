# Exact A/B comparison

Both pairs use the same release runner (`45491d105`), checked-in suite config, staged real corpora, physical three-owner topology, degree 32, head cap 4096, BW4/H100, 20 queries, 200 recall trials, 10 warmups, and 30 measured latency iterations. Every scale machine-attests the installed extension SHA and release profile on all three nodes.

## Packet-066 refactor isolation

Parent `59da26b8e02314f8f10d737b0a101f8e6e1d41e4` versus refactor `0043c3e746bef0baf6977dc8ae426006d7a0a887`:

| Scale | Recall before -> after | Physical p95 ms before -> after (delta) | Physical bytes before -> after (delta) |
| --- | --- | --- | --- |
| 10k | 1.0000 -> 1.0000 | 43.9 -> 43.5 (-0.4) | 242,761,728 -> 242,761,728 (0) |
| 50k | 0.9800 -> 0.9800 | 56.3 -> 55.6 (-0.7) | 1,242,742,784 -> 1,242,726,400 (-16,384) |
| 100k | 0.9500 -> 0.9500 | 53.2 -> 54.3 (+1.1) | 2,496,626,688 -> 2,496,659,456 (+32,768) |

Decision: recall is bit-identical at every scale. The mixed sub-millisecond/1.1 ms p95 movement and 0/-2/+4 PostgreSQL-page storage movement do not support a performance change. This closes the previously unmeasured neutrality assertion conservatively.

## Final remediation isolation

Review parent `34b61fb3c55d0333cec2213c6714858dd5b43e68` versus remediation `45491d1052ef0369a9f418b055b462663cf5612c`:

| Scale | Recall before -> after | Physical p95 ms before -> after (delta) | Physical bytes before -> after (delta) |
| --- | --- | --- | --- |
| 10k | 1.0000 -> 1.0000 | 43.5 -> 43.4 (-0.1) | 242,761,728 -> 242,745,344 (-16,384) |
| 50k | 0.9800 -> 0.9800 | 54.6 -> 55.9 (+1.3) | 1,242,726,400 -> 1,242,734,592 (+8,192) |
| 100k | 0.9500 -> 0.9500 | 54.3 -> 54.6 (+0.3) | 2,496,659,456 -> 2,496,659,456 (0) |

Decision: the remediation is recall-neutral across the required matrix. The p95 deltas are small and mixed, while storage moves by -2/+1/0 PostgreSQL pages; neither supports a gain or regression. Topology and remote engagement passed at every scale.

The first remediation-after attempt was interrupted during 50k due to database maintenance and entirely excluded. The table uses only the later fresh, non-resumed arm documented by `remediation-after/suite-manifest.json`.

## Protocol disposition

These exact comparisons deliberately hold the 10-warmup/30-measurement protocol constant within each pair. Historical packets 065/066/068/070 also used 10+30, which was an undocumented runtime-limited change from earlier 10+50 runs. Cross-protocol numbers must not be compared directly; the exact within-pair conclusions above do not cross protocols.
