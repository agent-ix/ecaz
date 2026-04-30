# Task 29 DiskANN Build Probe

## Summary

This packet adds and runs `ecaz bench diskann-build-probe` against the local
PG18 real-10k DiskANN baseline prefix:

- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- reloption surface: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- command path: `ecaz-cli` with explicit `--host`, `--port`, and `--database`

The release CLI replay completed on head `70aa867de5c7f788ab48dc626f390f67d6aa07ae`.

## Result

The in-memory Vamana build replay reached all `10000/10000` rows, so the
current baseline graph is connected from the sampled medoid in this replay.
The more useful signal is graph shape:

- build-core replay time: `73.211s` after `12.448s` fetch and `2.004s` medoid selection
- pass 1 (`alpha=1.0`): candidate pool mean/p95 `101.93/106`, selected mean/p95 `8.22/12`
- pass 2 (`alpha=1.2`): existing-neighbor mean/p95 `18.91/31`, candidate pool mean/p95 `105.42/113`, selected mean/p95 `21.59/32`
- final out degree: min `1`, mean `24.50`, p50 `25`, p95 `32`, p99 `32`, max `32`
- final in degree: min `1`, mean `24.50`, p50 `22`, p95 `43`, p99 `61`, max `3250`

This agrees with the persisted graph diagnostics in `review/11087...`: the
landing blocker is not broken persistence or disconnected graph construction.
The graph is connected, but the first pass heavily underfills forward links,
and the final graph still has a large in-degree hub. That is consistent with
the current recall ceiling near `0.93` from the baseline/probe packets.

## Recommendation

First optimization target: change Vamana build candidate generation/pruning so
the first pass does not start from only medoid-reachable visited candidates on
an almost-empty graph. A narrow next probe should test a local candidate
augmentation strategy before making it production behavior:

- include a bounded exact-nearest sample for each pivot during pass 1, or
- use a larger pass-1 candidate pool independent of the current sparse graph.

Do not tune scan `list_size` further yet: prior sweeps already show recall is
flat across `64..800`, and this packet points at build graph quality rather
than scan breadth.

## Artifacts

See `artifacts/manifest.md`.
