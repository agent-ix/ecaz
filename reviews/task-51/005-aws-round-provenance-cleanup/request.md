# Review Request: AWS Round Provenance Cleanup

- task: 51
- packet: `reviews/task-51/005-aws-round-provenance-cleanup`
- benchmark commit: `e76021b85` (`Add AWS RaBitQ IVF packet manifest`)
- scope: benchmark provenance only; no benchmark reruns

## Summary

This cleanup adds the missing packet-root manifest for
`benchmarks/aws-round-rabitq-ivf/` and links the existing artifact-local
manifest back to it.

No AWS, vchord, pgvectorscale, or new benchmark run was performed. The update
only makes the historical AWS IVF/RaBitQ packet easier to audit by naming:

- authoritative artifacts,
- failed/incomplete artifacts,
- benchmark surfaces and snapshots,
- the corrected `nlists` rebuild-rule status,
- and the remaining Task 51 gaps that are not closed by the historical packet.

## Files Changed

- `benchmarks/aws-round-rabitq-ivf/manifest.md`
- `benchmarks/aws-round-rabitq-ivf/artifacts/MANIFEST.md`

## Local Validation

- `git diff --check -- benchmarks/aws-round-rabitq-ivf/manifest.md benchmarks/aws-round-rabitq-ivf/artifacts/MANIFEST.md`: passed

See `artifacts/manifest.md` for command metadata.
