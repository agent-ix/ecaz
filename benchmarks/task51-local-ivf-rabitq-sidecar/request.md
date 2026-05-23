# Task 51 Local IVF/RaBitQ Sidecar Upper-Bound

This packet adds local Exp 7 preflight evidence for IVF/RaBitQ only. It uses an isolated `ec_ivf` RaBitQ `rerank=off` index to fetch a 50-row approximate frontier, then locally reranks the frontier through f32, f16, and bits=8 RaBitQ sidecar representations.

Status: suite completed successfully.

```text
completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Key result at the useful 50k local recall corner:

- `nprobe=64`, `f32`: recall@10 `0.9940`, candidate SQL p50 `107.509 ms`, sidecar p50 `1.552 ms`, total bound p50 `109.450 ms`, sidecar `292.97 MiB`.
- `nprobe=64`, `f16`: recall@10 `0.9940`, candidate SQL p50 `107.509 ms`, sidecar p50 `2.561 ms`, total bound p50 `110.184 ms`, sidecar `146.48 MiB`.
- `nprobe=64`, `rabitq8`: recall@10 `0.9505`, candidate SQL p50 `107.509 ms`, sidecar p50 `1.120 ms`, total bound p50 `108.628 ms`, sidecar `73.81 MiB`.

The important signal is that local sidecar rerank CPU is small relative to current IVF approximate scan time, while f32/f16 preserve recall on the same candidate frontier and `rabitq8` trades too much recall at `candidate_k=50`.

Caveats:

- Local-only, not AWS/Graviton evidence.
- Measurement harness only; no product sidecar storage path was added.
- q-count is 200 as a local preflight cost waiver.
- No vchord or pgvectorscale was run.

Authoritative artifacts are listed in `manifest.md`.
