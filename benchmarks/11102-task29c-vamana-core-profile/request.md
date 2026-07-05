# Task 29c Vamana Core Profile

## Request

Review the Task 29c Vamana core timing profile and updated local build
performance conclusion for the DiskANN landing branch.

Measured head: `b9eba6670c9d5774e87e6e6f5ea42de38a43fefa`

This packet keeps the raw local PG18 logs under `artifacts/`.

## Summary

The first Task 29c phase profile in packet `11101` correctly found that almost
all measured time was inside the build/persist phase, but this packet found an
important measurement caveat: those earlier `~490s` local builds used a
debug/dev-installed extension. Reinstalling the same head with
`cargo pgrx install --release` changes the local real-10k index-only build from
`497.950s` to `79.238s`.

Release-installed `ec_diskann`, isolated local real-10k 1536-d corpus,
`graph_degree=32`, `build_list_size=100`, `alpha=1.2`:

- total index-only build: `79.238s`
- heap scan: `1.261s`
- training: `0.130s`
- sidecar setup: `0.002s`
- payload derivation: `0.293s`
- build/persist: `77.485s`
- Vamana medoid: `1.566s`
- Vamana graph: `75.903s`
- Vamana persist: `0.014s`
- write pages: `0.059s`
- index size: `4,939,776` bytes (`4824 kB`)

The Vamana pass split shows pass 1 dominates:

- pass 0: `21.539s` elapsed, `11.229s` greedy search,
  `9,651,859` greedy distance calls
- pass 1: `54.363s` elapsed, `21.015s` greedy search,
  `6.886s` robust prune, `9.876s` backlink repair,
  `12,864,074` greedy distance calls,
  `17,837,238` robust-prune distance calls,
  `614,031` backlink distance calls

Reference `ec_hnsw` on the same table with the analogous local surface
`m=32`, `ef_construction=100`, `build_source_column=source`:

- index build: `5.23s`
- index size: `15,130,624` bytes (`14 MB`)

## Recommendation

The `~490s` build time cited by packet `11101` and earlier Task 29 landing
summaries should not be treated as a release-performance blocker. It was a
debug-extension artifact. The release local result is still slower than HNSW
for build (`79.238s` vs `5.23s` on real-10k), but the remaining gap is now a
known Vamana graph-construction cost, not tuple persistence or WAL/page-write
cost.

Task 29c should land this timing observability with one follow-up expectation:
future performance packets must state whether the extension was installed in
release or debug/dev profile, and release mode should be the default for local
performance claims.

The first post-landing optimization target should be pass-1 Vamana graph
construction, specifically robust-prune/backlink repair distance work. The
existing heap-frontier experiment should stay reverted because packet `11101`
showed it regressed corrected build time.

## Artifacts

See `artifacts/manifest.md`.
