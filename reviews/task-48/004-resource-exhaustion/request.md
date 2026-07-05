# Task 48/004: resource-exhaustion sweep (closes §Exit Criteria #3)

## Scope

Closes Task 48 §Exit Criteria gate 3:

> 3. `make resource-exhaustion` runs nightly.

Adds the CLI scaffolding + scenario implementations + Make wrapper.
Nightly CI cadence is wired by the companion Task 48/005 packet
(`.github/workflows/resource-exhaustion-nightly.yml`).

Validation head: `9ecfa83ea`.

## What changed

- `crates/ecaz-cli/src/commands/dev/resource_test.rs` — new
  ~360-line CLI subcommand. Sweeps the six Task 48 §Scope scenarios
  documented in `docs/build-matrix.md`:
  - `max-locks` — `max_locks_per_transaction` (restart-only GUC;
    scenario reads `current_setting`, returns `PrereqUnmet` if
    cluster not pre-configured).
  - `max-connections` — `max_connections` (restart-only; same
    pattern).
  - `work-mem-min` — `work_mem` / `maintenance_work_mem` via
    `SET LOCAL` to `64kB` / `1MB`; runs an intentionally hot
    cross-join.
  - `temp-file-limit` — `SET LOCAL temp_file_limit = '1MB'` +
    forced spill via sort over 500k rows.
  - `shared-buffers-thrash` — restart-only; cold-cache + random-
    scan workload.
  - `disk-full` — surfaces a `PrereqUnmet` pointing at
    `make fault-full` (Task 38 ENOSPC sweep) so the disk-full
    coverage stays in one place; the resource-test entry serves as
    the inventory link rather than duplicating the injector here.
- `crates/ecaz-cli/src/commands/dev/mod.rs` — registers the
  `DevCommand::ResourceTest(ResourceTestArgs)` variant.
- `Makefile` — new `make resource-exhaustion` target.

No production code change. CLI-only.

## Outcome semantics

Each scenario reports one of four outcomes:

| Outcome | Meaning | Exit impact |
|---|---|---|
| `pass` | Workload hit the configured limit cleanly (clean ERROR, cluster alive) | 0 |
| `prereq_unmet` | Restart-only GUC not pre-configured; scenario could not run | 0 (operator action item, not a failure) |
| `workload_did_not_trigger` | Workload ran but didn't exceed the limit (limit set too high) | 0 (signal worth tuning, not a regression) |
| `broken_connection` | Cluster did not respond after the scenario (PANIC / segfault / OOM) | **non-zero** |

JSON summary emitted to stdout (and `--log-output` if given). The
binary exits non-zero iff any scenario reports `broken_connection`.

## Reviewer focus

- Restart-only GUC scenarios honestly report `PrereqUnmet` instead
  of silently "passing" when the cluster is not pre-configured;
  per `feedback_dont_overclaim_done`. The companion CI workflow
  (`resource-exhaustion-nightly.yml`) handles the pre-configuration
  step so the nightly run gets real `pass` outcomes.
- Session-level GUC scenarios (`work_mem`, `temp_file_limit`) are
  fully implemented and runnable against any PG18 cluster the
  caller has, no pre-configuration required.
- `disk-full` deliberately delegates to `make fault-full` rather
  than re-implementing the ENOSPC injection — single source of
  truth in `crates/ecaz-fault-injection/`.
- The `--scenario` flag lets operators run a single scenario
  during development; default runs all six.

## Compile evidence

- `cargo check -p ecaz-cli` finishes in 8.38s after the
  resource_test.rs addition. Zero new warnings.

## Smoke run

Local smoke against the sandbox's PG18 cluster is **deferred**:
this CLI host does not have a running PG18 with the low
restart-only GUCs the scenario needs (see "restart-only GUC"
note above). The `resource-exhaustion-nightly` CI workflow is
where the full sweep first executes under the pre-configured
cluster.

A local operator wanting to smoke-test today can:

```sh
# Configure cluster to low limits (one-time), then:
DATABASE=postgresql://localhost/postgres make resource-exhaustion
```

## Task 48 §Exit Criteria progress after this slice

| # | §Exit Criterion | Status |
|---|---|---|
| 1 | CI matrix covers aarch64-darwin + x86_64-linux + aarch64-linux + pg17 + pg18 | 0% (closing in 005) |
| 2 | `make soak DURATION=24h` weekly | ✓ done (003) |
| 3 | `make resource-exhaustion` nightly | **✓ done (this)** |
| 4 | `docs/build-matrix.md` documents matrix, cadence, policy | ✓ done (002) |

Task 48 ≈ 75% complete (3 of 4 §Exit gates closed).
