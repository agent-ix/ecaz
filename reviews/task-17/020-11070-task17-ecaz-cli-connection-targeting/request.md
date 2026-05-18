# Review Request: Explicit `ecaz` connection flags for pg18 DiskANN runs

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/cli.rs`
- `crates/ecaz-cli/src/psql.rs`
- `crates/ecaz-cli/src/commands/corpus/mod.rs`
- `crates/ecaz-cli/src/commands/corpus/list.rs`
- `crates/ecaz-cli/src/commands/corpus/inspect.rs`
- `crates/ecaz-cli/src/commands/corpus/load.rs`
- `crates/ecaz-cli/src/commands/corpus/prepare.rs`
- `crates/ecaz-cli/src/commands/corpus/generate.rs`
- `crates/ecaz-cli/src/commands/bench/mod.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/storage.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`
- `crates/ecaz-cli/src/commands/compare/mod.rs`
- `crates/ecaz-cli/src/commands/compare/pgvector.rs`
- `crates/ecaz-cli/src/commands/stress/mod.rs`
- `crates/ecaz-cli/src/commands/stress/vacuum.rs`
- `crates/ecaz-cli/README.md`

## What this packet is

Task 17 closeout is still missing the real-10k DiskANN Recall@10 artifact.
The blocker on this machine was not more AM code: it was that `ecaz-cli`,
the canonical operator surface for load / bench / compare, only accepted
`--database` and otherwise depended on libpq env vars for host/socket/port.

That forced exactly the wrong workflow for the pg18 scratch cluster:
approval-sensitive env-prefix commands instead of an explicit, reviewable
CLI invocation. This packet fixes that properly by adding first-class
connection flags and threading them through the command tree, including the
parallel worker paths that open their own sessions.

## What changed

### `crates/ecaz-cli/src/cli.rs`

- Added global explicit connection flags:

```rust
#[arg(long, global = true, env = "PGHOST")]
pub host: Option<String>;

#[arg(long, global = true, env = "PGPORT")]
pub port: Option<u16>;

#[arg(long, global = true, env = "PGUSER")]
pub user: Option<String>;

#[arg(long, global = true, env = "PGPASSWORD", hide_env_values = true)]
pub password: Option<String>;
```

- `Cli::run()` now builds one shared `psql::ConnectionOptions` bundle and
  passes it into every subcommand group instead of only passing a raw
  database name.

- Added a parser test pinning:
  - `--host /home/peter/.pgrx`
  - `--port 28818`
  - `--user peter`
  - `--password secret`

### `crates/ecaz-cli/src/psql.rs`

- Replaced the implicit env-reading connect helper with an explicit
  connection bundle:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionOptions {
    pub database: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
}
```

- Added `ConnectionOptions::config()` so every caller uses the same
  `tokio-postgres::Config` materialization path.

- `connect(...)` now takes `&ConnectionOptions` instead of `&str`.

- Added a unit test that pins the explicit-overrides config shape
  (`dbname`, one host, port, user, password).

### Command tree plumbing

- `corpus`, `bench`, `compare`, and `stress` command modules now take
  `&ConnectionOptions` rather than `&str`.
- Read-only commands (`corpus list`, `inspect`, `bench recall`, `bench storage`,
  `bench overhead`, `compare pgvector`) simply reuse `psql::connect(conn)`.
- No SQL semantics changed in these commands; this is connection-targeting
  plumbing only.

### Worker-session fixes

Two commands were rebuilding fresh `tokio-postgres::Config`s from env
inside worker tasks, which would have ignored the new explicit flags.

#### `crates/ecaz-cli/src/commands/bench/latency.rs`

- `run_sweep_point(...)` now clones `ConnectionOptions` into each worker.
- Each latency worker opens its own session with:

```rust
let client = psql::connect(&conn).await?;
```

- This keeps the session-local sweep GUC behavior intact while making the
  worker sessions honor the same host/socket/port target as the bootstrap
  connection.

#### `crates/ecaz-cli/src/commands/stress/vacuum.rs`

- Same fix for the insert / vacuum / scan worker sessions:

```rust
async fn connect_worker(conn: &ConnectionOptions) -> Result<tokio_postgres::Client> {
    psql::connect(conn).await
}
```

- This keeps the global connection model consistent across the CLI rather
  than leaving one subcommand on a hidden env-only path.

### `crates/ecaz-cli/README.md`

- Updated install / connection docs to say the CLI now accepts explicit
  `--database`, `--host`, `--port`, `--user`, and `--password`.
- Added the Unix-socket example shape (`/home/peter/.pgrx`) because that is
  the actual pg18 scratch-cluster target for this task.

## Why this slice

- This is not generic tool polish. It is the narrow blocker fix required to
  use the canonical DiskANN measurement surface on the local pg18 scratch
  cluster without env-var hacks.
- It keeps the fix "do it right" explicit: one reviewed CLI surface,
  shared by `load`, `bench`, and `compare`, instead of wrapper scripts or
  shell-only conventions.
- It makes the connection target coherent for both single-session commands
  and multi-session worker commands, so later pg18 DiskANN recall / latency
  runs do not accidentally split across different connection rules.

## Operator smoke after the fix

- `cargo run -p ecaz-cli -- --help` now shows the new global flags:
  `--host`, `--port`, `--user`, `--password`.
- `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres corpus list`
  successfully reached the local pg18 scratch cluster through `ecaz` and
  returned:

```text
(no corpora loaded in postgres)
```

That confirms the blocker fix is real: the CLI can now target the scratch
cluster explicitly. The remaining issue is operational state on disk / in
the DB, not another missing connection feature.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 209 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally for this slice on `pg18`:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- The actual real-10k DiskANN Recall@10 capture. After this fix the pg18
  scratch cluster is reachable through `ecaz`, but the canonical real-10k
  fixture is not present on this machine and the scratch DB is currently
  empty.
- Adding a dedicated `ecaz` command to enumerate databases. Task 17 only
  needed explicit connection targeting, not a broader admin surface.
- More connection flags (`sslmode`, connect timeout, passfile path, etc.).
  None are needed for the local Unix-socket pg18 signoff path.
