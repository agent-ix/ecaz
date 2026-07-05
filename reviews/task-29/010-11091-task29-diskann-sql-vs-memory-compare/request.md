# Task 29 DiskANN SQL vs Memory Compare

## Summary

This packet runs the build probe comparison mode added in
`5a7893236165e99891efb218675b42abc7088b55`. The command compares, for the same
query rows:

- exact top-10 over source vectors,
- in-memory Vamana graph search top-10,
- persisted SQL `ec_diskann` top-10.

The run uses local PG18 only, `ecaz-cli`, and explicit connection flags.

## Result

The in-memory graph again has `0.9995` recall@10 over all 200 queries at
`scan_list_size=100`. For the first five query rows:

- query `10000`: exact/memory/sql all match `10/10`
- query `10001`: exact and in-memory match `10/10`, but SQL matches exact only `8/10`
- query `10002`: exact/memory/sql all match `10/10`
- query `10003`: exact/memory/sql all match `10/10`
- query `10004`: exact/memory/sql all match `10/10`

For query `10001`, exact and in-memory both returned:

```text
8885,9785,9957,9826,9717,9926,9944,9855,9915,7782
```

Persisted SQL returned:

```text
8885,9785,9957,9826,9926,9944,9855,9915,9976,9999
```

So the persisted path missed exact IDs `9717` and `7782` and substituted
`9976` and `9999`.

## Recommendation

Do not spend the next slice on build-graph augmentation. The build graph is
good enough in memory.

Next landing blocker: persisted scan parity. Add a narrower diagnostic that
captures the persisted DiskANN scan candidate frontier for query `10001` and
answers whether IDs `9717` and `7782` are:

- never reached by persisted traversal,
- reached but discarded before output,
- reached but scored/reranked behind `9976` and `9999`.

The likely production fix should be in persisted scan traversal/scoring/rerank,
not in Vamana build candidate generation.

## Artifacts

See `artifacts/manifest.md`.
