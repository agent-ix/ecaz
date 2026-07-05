# Review Request: IVF Adaptive Nprobe

Please review Task 51 Exp 5 code commit:

- code commit: `80f3476c2` (`Add opt-in IVF adaptive nprobe`)
- files: `src/am/ec_ivf/mod.rs`, `src/am/ec_ivf/options.rs`, `src/am/ec_ivf/scan.rs`
- artifact manifest: `reviews/task-51/012-ivf-adaptive-nprobe/artifacts/manifest.md`

## Scope

This adds an opt-in adaptive nprobe policy for `ec_ivf`:

- `ec_ivf.adaptive_nprobe`, default `false`
- `ec_ivf.adaptive_nprobe_score_gap_micros`, default `1000`

When enabled, scan-time centroid selection may reduce the requested nprobe by half if the score gap between the retained half-frontier and the next centroid exceeds the configured threshold. Default behavior is unchanged because the policy is disabled unless explicitly set.

The policy is intentionally conservative and scan-time only. It does not change the on-disk format, build path, index reloptions, rerank mode, or RaBitQ encoding.

## Validation

Passed:

```text
cargo check --no-default-features --features pg18
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.53s
```

Passed:

```text
git diff --check
```

Focused unit-test attempt:

```text
cargo test --no-default-features --features pg18 --lib am::ec_ivf::scan
```

The test binary compiled, but running it in this shell failed on a PostgreSQL backend symbol:

```text
undefined symbol: CacheRegisterRelcacheCallback
```

I am recording that as a local harness limitation, not a passing test.

## Next Required Work

This is code only. It still needs local benchmark evidence with adaptive nprobe explicitly enabled, reporting recall tail and p50/p95/p99 before any AWS promotion.

