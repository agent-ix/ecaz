# Artifact Manifest: IVF Adaptive Nprobe

- head SHA: `80f3476c2`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/012-ivf-adaptive-nprobe/`
- lane: local code validation, IVF/RaBitQ Task 51 Exp 5
- fixture: no benchmark fixture in this packet
- storage format: unchanged; applies to `ec_ivf` scan-time selected-list routing
- rerank mode: unchanged
- isolated one-index-per-table surface: not applicable
- vchord / pgvectorscale: not run

## Artifacts

- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`.
- `cargo-test-ec-ivf-scan-pg18.log`: focused scan unit-test attempt; built test binary, failed at runtime on missing PostgreSQL backend symbol in this shell.
- `git-diff-check.log`: whitespace check.

## Key Result Lines

```text
cargo check --no-default-features --features pg18
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.53s
```

```text
git diff --check
```

Focused test caveat:

```text
undefined symbol: CacheRegisterRelcacheCallback
```

