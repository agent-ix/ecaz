# Artifact manifest

- Head SHA: `f11ffcafcdf1e80464540bad05d8364559f2598b`
- Implementation commit: `f11ffcafc` (`feat(cli): add latency warmup iterations`)
- Task bucket / packet: `reviews/task-179/044-latency-warmup-runner`
- Lane: local CLI focused tests and release runner build
- Fixture: pure latency statistics tests plus SuiteConfig command expansion
- Storage format / rerank mode: not applicable
- Timestamp: `2026-07-12T19:32:17-07:00`

All commands run from the clean implementation head above. This is runner
validation, not benchmark evidence.

## Validation commands and results

### Focused latency tests

```text
cargo test -p ecaz-cli 'commands::bench::latency::tests::' -- --nocapture
```

Result: exit 0; 17 passed, 0 failed. See `latency-tests.log`.

### DistANN suite expansion

```text
cargo test -p ecaz-cli distann_local_multinode_step_expands_head_index_cap -- --nocapture
```

Result: exit 0; 1 passed, 0 failed. The test requires the SuiteConfig field to
expand to `--benchmark-warmup-iterations 7`. See `suite-expansion-test.log`.

### Release runner build

```text
cargo build --release -p ecaz-cli
```

Result: exit 0. The optimized `target/release/ecaz` runner was produced from
the exact head above. One pre-existing unused-field warning remains in
`corpus/load.rs`. See `release-cli-build.log`.

### Broader warnings-denied lint

```text
cargo clippy -p ecaz-cli --all-targets --no-deps -- -D warnings
```

Result: expected existing-lane failure, recorded in `cli-clippy-existing.log`.
Exit 101 with 25 pre-existing errors in files outside the three touched files;
no new warning from the warmup code was reported before compilation stopped.

## Artifact index

- `latency-tests.log`: focused latency statistics/session-GUC tests.
- `suite-expansion-test.log`: DistANN SuiteConfig warmup argument expansion.
- `release-cli-build.log`: exact-SHA optimized runner build.
- `cli-clippy-existing.log`: existing unrelated warnings-denied blockers.

No benchmark result is claimed by this packet.
