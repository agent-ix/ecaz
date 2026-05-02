# Task 31 M5 Environment Setup

Reviewer: please review this setup checkpoint before the first Task 31 IVF benchmark pass.

## Scope

This packet verifies the new M5 laptop can build the repo, install the PG18 extension, run PG18 smoke SQL, and run the `ecaz-cli` operator surface. It intentionally does not run recall, latency, storage, build-time, memory-HWM, or long IVF sweeps.

Setup-enablement code was needed and was committed separately as `ca24195548ac410d474b7a6eadc8acba2e50c14b`, with a message-only follow-up at `2133e685faedf374842161335ac105fdf82aa1b5`:

- `.cargo/config.toml` now gives macOS pgrx extension dylibs `-undefined dynamic_lookup`, avoiding manual `RUSTFLAGS=...` commands that trip approval flow.
- `ecaz dev sql` now discovers PG installs recorded by `~/.pgrx/config.toml`, which is what `cargo pgrx init --pg18 /opt/homebrew/.../pg_config` creates on this Homebrew-backed machine.
- `ecaz dev install ecaz-pg-test` now verifies macOS/Homebrew installs using `pg_config --pkglibdir` and the installed `ecaz.dylib` name.

## Machine

Raw metadata: `artifacts/machine-metadata.log`

- Model: MacBook Pro `Mac17,9`
- Chip: Apple M5 Pro
- CPU: 18 cores, reported as 18 physical and 18 logical processors by `hostinfo`
- Memory: 64 GB
- OS: macOS `26.4.1`, build `25E253`
- Kernel: Darwin `25.4.0`, `RELEASE_ARM64_T6050`
- Note: direct `sysctl hw.*` calls are sandbox-denied in this session, but `system_profiler` and `hostinfo` provide the machine/CPU/memory facts.

## Tools

Raw versions: `artifacts/tool-versions.log`

- Rust: `rustc 1.95.0` Homebrew, host `aarch64-apple-darwin`
- Cargo: `cargo 1.95.0` Homebrew
- pgrx CLI: `cargo-pgrx 0.17.0`
- PostgreSQL: `18.3 (Homebrew)`
- `pg_config`: `/opt/homebrew/opt/postgresql@18/bin/pg_config`
- `pkglibdir`: `/opt/homebrew/lib/postgresql@18`
- `sharedir`: `/opt/homebrew/share/postgresql@18`
- Compiler: Apple clang `21.0.0`

Installed during this setup:

- `brew install rust postgresql@18`
- `cargo install cargo-pgrx --version 0.17.0 --locked`
- `cargo pgrx init --pg18 /opt/homebrew/opt/postgresql@18/bin/pg_config`
- repository-local Git identity set to `Codex <codex@openai.com>` to allow required checkpoint commits.

## PG18 / pgrx Status

Raw pgrx and connection evidence:

- `artifacts/pgrx-pg18-status.log`
- `artifacts/pgrx-start-pg18.log`
- `artifacts/pgrx-status-current.log`
- `artifacts/pgrx-socket-current.log`
- `artifacts/psql-current-connect.log`

Status:

- `~/.pgrx/config.toml` maps `pg18` to `/opt/homebrew/opt/postgresql@18/bin/pg_config`.
- `~/.pgrx/data-18` exists and has `PG_VERSION = 18`.
- Socket exists at `/Users/peter/.pgrx/.s.PGSQL.28818`.
- Direct `psql` to that socket returns `select 1`.
- `cargo pgrx status` reports `Postgres v18 is stopped` even while socket and `psql` connectivity work, so current setup should trust socket/SQL checks over that status output.

## Validation

Raw logs:

- `artifacts/ecaz-cli-cargo-check.log`
- `artifacts/ecaz-dev-install-ecaz-pg-test.log`
- `artifacts/ecaz-dev-sql-pg18-smoke.log`
- `artifacts/ecaz-corpus-list-smoke.log`
- `artifacts/pg18-create-extension-smoke.log`

Commands and results:

- `cargo check -p ecaz-cli`: passed.
- `cargo test -p ecaz-cli commands::dev`: passed after the CLI clean-setup fix, 5 tests.
- `cargo fmt --all -- --check`: passed; rustfmt emits warnings about unstable `imports_granularity` / `group_imports` config under stable Rust.
- `cargo run -p ecaz-cli -- dev install ecaz-pg-test --pg 18 --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`: passed; installed `/opt/homebrew/lib/postgresql@18/ecaz.dylib`; backend assertion passed.
- `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql ...`: passed; server version `18.3 (Homebrew)`, extension `ecaz 0.1.1`.
- `cargo run -p ecaz-cli -- --host /Users/peter/.pgrx --port 28818 --database postgres corpus list --log-file ...`: passed; no corpora are loaded in `postgres`.
- Direct `psql CREATE EXTENSION IF NOT EXISTS ecaz`: passed before the CLI SQL path was fixed; retained as bootstrap evidence.

## Blockers / Gaps

- No benchmark corpus exists locally yet: no `data/` directory and no `*_corpus.tsv` / `*_queries.tsv` files were found under the repo.
- The current `postgres` database has no loaded corpora.
- `cargo pgrx status` appears unreliable for this Homebrew-backed pgrx config because it reports stopped despite a live socket and successful SQL.
- The first benchmark pass should use a fresh pgrx backend after the release `ecaz-cli` install, to ensure the loaded backend is the release-installed extension.

## Next IVF Baseline Smoke

Because no corpus is currently staged or loaded, begin with a tiny synthetic IVF smoke to prove the operator loop end to end, then move to the real Task 31 10k/25k/100k surfaces.

Suggested first smoke commands:

```sh
mkdir -p data/task31_m5_smoke
cargo run -p ecaz-cli -- corpus generate --output data/task31_m5_smoke/task31_m5_smoke_corpus.tsv --n 10000 --dim 1536 --seed 31 --kind corpus
cargo run -p ecaz-cli -- corpus generate --output data/task31_m5_smoke/task31_m5_smoke_queries.tsv --n 20 --dim 1536 --seed 3100 --kind queries
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task31_m5_smoke_pqg8 --profile ec_ivf --corpus-file data/task31_m5_smoke/task31_m5_smoke_corpus.tsv --queries-file data/task31_m5_smoke/task31_m5_smoke_queries.tsv --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=128 --reloption nprobe=8 --reloption rerank=heap_f32 --reloption rerank_width=500 --log-file review/30163-task31-m5-ivf-smoke/artifacts/load.log
cargo run -p ecaz-cli -- --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_smoke_pqg8 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --log-output review/30163-task31-m5-ivf-smoke/artifacts/recall_q3.log
```

That smoke is intentionally small. The first real Task 31 baseline packet should then fetch/prepare or otherwise stage the DBpedia real corpus and run the landed PQ-FastScan group-size-8 surfaces using release-installed extension builds.
