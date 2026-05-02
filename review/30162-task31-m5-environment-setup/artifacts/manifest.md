# Artifact Manifest: Task 31 M5 Environment Setup

Head SHA for packet preparation: `2133e685faedf374842161335ac105fdf82aa1b5`

Packet/topic: `review/30162-task31-m5-environment-setup`

Scope: environment setup and smoke evidence only. No recall, latency, build-time, or long benchmark sweeps were run.

## Artifacts

- `machine-metadata.log`
  - lane / fixture / storage format / rerank mode: machine metadata, no corpus, no index, no rerank.
  - command: `date -u; sw_vers; uname -a; system_profiler SPHardwareDataType; sysctl ...; hostinfo`
  - timestamp: `2026-05-02T22:49:08Z`
  - isolated/shared surfaces: not applicable.
  - key lines: macOS `26.4.1` build `25E253`; `MacBook Pro` `Mac17,9`; `Apple M5 Pro`; `18` cores; `64 GB`; `hostinfo` reports `18` physical/logical processors and `64.00` GB memory.

- `tool-versions.log`
  - lane / fixture / storage format / rerank mode: toolchain versions, no corpus, no index, no rerank.
  - command: `rustc --version --verbose; cargo --version --verbose; cargo pgrx --version; pg_config --version; psql --version; postgres -V; clang --version; brew list --versions rust postgresql@18`
  - timestamp: `2026-05-02T22:49:32Z`
  - isolated/shared surfaces: not applicable.
  - key lines: Rust/Cargo `1.95.0` Homebrew; `cargo-pgrx 0.17.0`; PostgreSQL `18.3 (Homebrew)`; Apple clang `21.0.0`.

- `pgrx-pg18-status.log`
  - lane / fixture / storage format / rerank mode: pgrx config after init, no corpus, no index, no rerank.
  - command: `cargo pgrx status; cargo pgrx info pg-config pg18; cargo pgrx info path pg18; cargo pgrx info version pg18; cat ~/.pgrx/config.toml; cat ~/.pgrx/data-18/PG_VERSION`
  - timestamp: `2026-05-02T22:49:38Z`
  - isolated/shared surfaces: not applicable.
  - key lines: pgrx config points `pg18` to `/opt/homebrew/opt/postgresql@18/bin/pg_config`; PG version `18`.

- `ecaz-cli-cargo-check.log`
  - lane / fixture / storage format / rerank mode: CLI compile check, no corpus, no index, no rerank.
  - command: `cargo check -p ecaz-cli`
  - timestamp: `2026-05-02T22:52Z`
  - isolated/shared surfaces: not applicable.
  - key lines: `Finished dev profile`; `ecaz-cli v0.1.0`.

- `ecaz-dev-install-ecaz-pg-test.log`
  - lane / fixture / storage format / rerank mode: CLI-owned PG18 install helper, release extension install, no corpus, no index, no rerank.
  - command: `cargo run -p ecaz-cli -- dev install ecaz-pg-test --pg 18 --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
  - timestamp: `2026-05-02T23:11Z`
  - isolated/shared surfaces: not applicable.
  - key lines: installed to `/opt/homebrew/lib/postgresql@18/ecaz.dylib`; backend assertion passed; sha256 `e31a131209dbca77477b6642697fea391a316ef27099fea23819cb01b391f6c8`.

- `ecaz-dev-sql-pg18-smoke.log`
  - lane / fixture / storage format / rerank mode: CLI-owned PG18 SQL smoke, no corpus, no index, no rerank.
  - command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql "SELECT current_setting('server_version') ..."`
  - timestamp: `2026-05-02T23:04Z`
  - isolated/shared surfaces: not applicable.
  - key lines: server version `18.3 (Homebrew)`; installed extension `ecaz 0.1.1`.

- `ecaz-corpus-list-smoke.log`
  - lane / fixture / storage format / rerank mode: CLI DB smoke against corpus registry, no corpus loaded, no index, no rerank.
  - command: `cargo run -p ecaz-cli -- --host /Users/peter/.pgrx --port 28818 --database postgres corpus list --log-file ...`
  - timestamp: `2026-05-02T22:58Z`
  - isolated/shared surfaces: shared database registry inspection only.
  - key line: `(no corpora loaded in postgres)`.

- `pg18-create-extension-smoke.log`
  - lane / fixture / storage format / rerank mode: direct psql bootstrap smoke before CLI SQL path was fixed, no corpus, no index, no rerank.
  - command: `/opt/homebrew/opt/postgresql@18/bin/psql -h /Users/peter/.pgrx -p 28818 -d postgres -c 'CREATE EXTENSION IF NOT EXISTS ecaz;' ...`
  - timestamp: `2026-05-02T22:57Z`
  - isolated/shared surfaces: not applicable.
  - key lines: `CREATE EXTENSION`; installed extension `ecaz 0.1.1`.

- `pgrx-install-pg18.log`
  - lane / fixture / storage format / rerank mode: failed direct bootstrap install attempt, no corpus, no index, no rerank.
  - command: `cargo pgrx install --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
  - timestamp: `2026-05-02T22:54Z`
  - isolated/shared surfaces: not applicable.
  - key lines: failed with macOS unresolved PostgreSQL backend symbols before `.cargo/config.toml` gained dynamic lookup flags.

- `pgrx-install-pg18-dynamic-lookup.log`
  - lane / fixture / storage format / rerank mode: direct bootstrap install workaround before codifying the setup fix, no corpus, no index, no rerank.
  - command: `cargo pgrx install --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` with macOS dynamic-lookup linker flags.
  - timestamp: `2026-05-02T22:55Z`
  - isolated/shared surfaces: not applicable.
  - key lines: copied `ecaz.control`, copied `ecaz.dylib`, wrote `ecaz--0.1.1.sql`, finished installing.

- `pgrx-start-pg18.log`, `pgrx-status-current.log`, `pgrx-socket-current.log`, `psql-current-connect.log`
  - lane / fixture / storage format / rerank mode: local PG18 cluster status and connectivity, no corpus, no index, no rerank.
  - commands: `cargo pgrx start pg18`; `cargo pgrx status`; `ls ~/.pgrx/.s.PGSQL.28818*`; `psql ... -c 'select 1;'`
  - timestamp: `2026-05-02T22:57Z` through `2026-05-02T23:13Z`
  - isolated/shared surfaces: not applicable.
  - key lines: socket exists at `/Users/peter/.pgrx/.s.PGSQL.28818`; direct `psql` returns `1`; `cargo pgrx status` still reports stopped, so socket/psql is the authoritative current proof.

- `ecaz-cli-help.log`, `ecaz-cli-help-dynamic-lookup.log`, `ecaz-dev-sql-help.log`, `ecaz-dev-install-help.log`, `ecaz-dev-sql-pg18-cargo-run.log`, `ecaz-corpus-list-cargo-run.log`
  - lane / fixture / storage format / rerank mode: supporting CLI command evidence and captured command wrappers.
  - commands: help and cargo-run wrappers for the CLI smokes above.
  - timestamp: `2026-05-02T22:52Z` through `2026-05-02T23:04Z`
  - isolated/shared surfaces: not applicable.
  - key lines: command trees expose `dev sql`, `dev install ecaz-pg-test`, `corpus`, and `bench`; the initial CLI run without dynamic lookup failed before the repo fix, and the later runs succeed.
