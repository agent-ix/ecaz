# Task 63 HNSW RaBitQ Host Config Provenance

## Summary

This metadata-only packet clarifies the faster-host benchmark handoff for M5
path differences. The checked-in `suite.json` remains the Linux/newer-Intel
benchmark-host config. If the m5 laptop needs different PostgreSQL socket or
staged dataset paths, the M5 run must use a checked-in sibling SuiteConfig in
this packet rather than an untracked local edit of `suite.json`.

No benchmark artifacts were added or removed, and no benchmarks were run. Local
AMD output remains baseline/tuning evidence only. Final Task 63 completion
still requires publishable HNSW-only 50k/100k measurements from the faster
benchmark hosts and a recorded recommend/experimental/shelve decision.

## Files

- `benchmarks/task63-hnsw-rabitq-format/manifest.md`

## Validation

Static inspection only:

- The suite runner records the config path/hash in `suite-manifest.json`.
- The Task 63 measured suite steps remain HNSW-only and limited to 50k/100k.
- No local benchmark commands were run.
