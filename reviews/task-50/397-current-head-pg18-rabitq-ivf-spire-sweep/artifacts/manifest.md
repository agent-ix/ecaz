# Manifest: Current-Head PG18 RaBitQ / IVF / SPIRE Sweep Prep

- Task bucket: `reviews/task-50`
- Packet: `reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep`
- Code cleanup commit: `e21d0dd42`
- Branch: `task-50-unsafe-closeout`
- Timestamp: `2026-05-21T21:48:25-07:00`
- Primary target: PG18
- Local scratch connection for follow-on benches: `localhost:28818`, database
  `tqvector_bench`

## Commands And Evidence

- `cargo-fmt-check.log`
  - Command: `cargo fmt --all -- --check`
  - Result: failed before cleanup due repo-wide formatting drift.
- `cargo-fmt-apply.log`
  - Command: `cargo fmt --all`
  - Result: applied mechanical formatting.
- `cargo-fmt-check-clean-final.log`
  - Command: `cargo fmt --all -- --check`
  - Result: passed with stable-rustfmt warnings about nightly-only import
    grouping options.
- `cargo-check-all-targets-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed before final warning cleanup, with unused SPIRE DML re-export
    warnings.
- `cargo-check-all-targets-pg18-bench-clean-final.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed cleanly.
- `cargo-test-no-run-all-targets-pg18-bench.log`
  - Command: `cargo test --no-run --all-targets --no-default-features --features pg18,bench`
  - Result: passed before final cleanup.
- `cargo-test-no-run-all-targets-pg18-bench-clean-final.log`
  - Command: `cargo test --no-run --all-targets --no-default-features --features pg18,bench`
  - Result: passed after cleanup.
- `cargo-build-ecaz-cli.log`
  - Command: `cargo build -p ecaz-cli --bin ecaz`
  - Result: passed.
- `cargo-clippy-all-targets-pg18-bench-d-warnings.log`
  - Command:
    `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`
  - Result: failed with broad existing lint backlog; this was not resolved in
    the merge prep cleanup.
- `rabitq-ivf-spire-local-suite.json`
  - Follow-on local suite config for IVF/RaBitQ and SPIRE/RaBitQ benches.

## Bench Status

Local PG18 RaBitQ smoke run complete on `2026-05-21` at head
`e81dcf8fd16cc02ddf4e88b7861af94c5f80ff48`. Recall holds vs the
2026-05-19 local baseline (`benchmarks/task-50-local-baseline/`); latency
shows a 5–10× regression that is almost certainly attributable to the
installed PG18 extension being a **debug** build rather than release. Full
analysis, evidence, and next-step plan are in
`artifacts/bench-comparison-report.md`.

### Bench artifacts

- `rabitq-ivf-spire-local-suite.json` — suite config (input).
- `suite-audit.log` — `ecaz bench suite audit` output.
- `ecaz-bench-suite-dry-run.log`, `suite-dry-run-manifest.json` — dry-run.
- `ecaz-bench-suite-run.log`, `suite-manifest.json`, `results.jsonl` —
  authoritative result rows for this packet (4 steps, 2 sweep values each).
- `ivf-rabitq-10k-recall-k10.log`,
  `ivf-rabitq-10k-latency-k10-c1.log`,
  `spire-rabitq-10k-recall-k10.log`,
  `spire-rabitq-10k-latency-k10-c1.log` — per-step formatted tables.
- `bench-comparison-report.md` — baseline comparison + issues report
  (this packet's primary review evidence).

### Re-run command

```
target/debug/ecaz bench suite run \
  --config reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep/artifacts/rabitq-ivf-spire-local-suite.json \
  --host localhost --port 28818 --database tqvector_bench \
  --manifest-output reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep/artifacts/suite-manifest.json \
  --results-output reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep/artifacts/results.jsonl \
  --log-file    reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep/artifacts/ecaz-bench-suite-run.log
```

Same hardware as baseline (`DESKTOP-BMB4AFO`, WSL2, i9-10900K, no AVX-512).
Connection mode here is TCP `localhost:28818`; baseline used UDS
`/home/peter/.pgrx`. Recommend reverting local re-runs to UDS for tighter
parity.
