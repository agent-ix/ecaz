# Task 51 Review Packet 016: IVF Sidecar Real-I/O Modes

- head SHA: `0b359e5ddbee42a7cba45042f7da577d1accf7d4`
- timestamp: `2026-05-23T15:22:19Z`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/016-ivf-sidecar-real-io/`
- code commit: `0b359e5ddbee42a7cba45042f7da577d1accf7d4`
- benchmark packet: `benchmarks/task51-local-ivf-sidecar-real-io/`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; this packet is IVF/RaBitQ only
- fixture: `ec_real_50k`, reused preserved isolated prefix `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- storage format: `rabitq`
- candidate frontier: IVF approximate `LIMIT 50`, then sidecar rerank to top 10
- sidecar variants: `f16`, `rabitq8`
- sidecar read modes: `free`, `random-id`, `tid-sorted`
- isolated one-index-per-table surface: yes, inherited from `benchmarks/task51-local-ivf-rabitq-sidecar/`

## Code Scope

Commit `0b359e5ddbee42a7cba45042f7da577d1accf7d4` adds:

- `SidecarReadMode` CLI enum with `free`, `random-id`, and `tid-sorted`;
- `--rebuild-sidecar-table` table materialization for real-I/O modes;
- fixed-width unlogged `bytea` sidecar tables for f32/f16/rabitq8 payloads;
- separate sidecar I/O and sidecar scoring timers in the result table;
- `ecaz bench suite` fields `read_modes` and `rebuild_sidecar_table`.

## Validation Artifacts

### `cargo-test-ecaz-cli-sidecar.log`

Command:

```text
cargo test -p ecaz-cli sidecar
```

Result:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 352 filtered out; finished in 0.00s
```

Existing warnings are present in `src/am/mod.rs` and `src/am/ec_ivf/build.rs`;
they are not introduced by this packet.

### `git-diff-check.log`

Command:

```text
git diff --check
```

Result: passed with no output.

## Benchmark Evidence

Benchmark packet:

- `benchmarks/task51-local-ivf-sidecar-real-io/`

Authoritative suite status:

```text
[suite:task51-local-ivf-sidecar-real-io] completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Key result lines cited by `request.md`:

- f16 nprobe 64: `random-id sidecar_io_p50=17.961 ms`, `tid-sorted sidecar_io_p50=1.403 ms`;
- f16 nprobe 128: `random-id sidecar_io_p50=17.969 ms`, `tid-sorted sidecar_io_p50=1.339 ms`;
- rabitq8 nprobe 64: `random-id sidecar_io_p50=16.655 ms`, `tid-sorted sidecar_io_p50=0.942 ms`;
- rabitq8 nprobe 128: `random-id sidecar_io_p50=17.354 ms`, `tid-sorted sidecar_io_p50=0.902 ms`;
- f16 recall@10 reaches `0.9980` at nprobe 96/128;
- rabitq8 recall@10 remains `0.9470-0.9480` at nprobe 64-128.

## Decision Record

The previous free-I/O sidecar harness remains useful as an upper-bound oracle,
but real-I/O storage shape is load-bearing. This packet rejects naive random-id
sidecar lookup and keeps a batched physical-order sidecar read path as the
plausible product-shape candidate for the final Task 51 Pareto decision.
