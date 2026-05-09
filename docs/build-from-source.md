# Build From Source

This guide covers a repeatable local build from a fresh checkout. Ecaz is a
Rust PostgreSQL extension built with pgrx. The default build target is
PostgreSQL 18; PostgreSQL 17 is maintained as a compatibility target.

The recommended local setup uses a pgrx-managed PG18 under `$PGRX_HOME`
(`$HOME/.pgrx` by default). That keeps the extension build, PostgreSQL
installation, socket directory, and test cluster aligned.

## Supported Platforms

| Area | Status |
| --- | --- |
| PostgreSQL | PG18 primary target; PG17 compatibility target |
| pgrx | `cargo-pgrx` 0.17 |
| Rust | Stable toolchain |
| Linux | Active development and test platform on x86_64 |
| macOS | Active development and benchmark platform on Apple Silicon, including Apple M5 IVF and DiskANN tuning lanes |
| CPU target | Local builds use `target-cpu=native`; build release artifacts on the same CPU family that will run them |

The native CPU setting is intentional for local vector-search development. If
you need portable binaries or cross-compilation, adjust `.cargo/config.toml`
before building release artifacts.

## 1. Install Native Prerequisites

Install Rust stable first:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

Install the native build tools and libraries needed to compile PostgreSQL and
the extension.

On Debian or Ubuntu:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential clang pkg-config flex bison \
  libssl-dev libreadline-dev zlib1g-dev \
  libxml2-dev libxslt1-dev
```

If you plan to install into a system PostgreSQL instead of the pgrx-managed
server, also install the matching server development package, for example
`postgresql-server-dev-18` or `postgresql-server-dev-17`. PostgreSQL 18 packages
may require the PostgreSQL Global Development Group apt repository on some
distributions.

On macOS with Homebrew:

```bash
brew install llvm openssl readline zlib bison
```

If you plan to install into a Homebrew PostgreSQL server, also install the
matching server package, for example `postgresql@18` or `postgresql@17`, and use
that package's `pg_config` path during install.

## 2. Install pgrx

Install the pgrx cargo subcommand version used by the repo:

```bash
cargo install cargo-pgrx@0.17
```

Initialize a PG18 development installation managed by pgrx:

```bash
cargo pgrx init --pg18 download
```

This downloads, builds, and installs PostgreSQL 18 under `$PGRX_HOME`. It can
take several minutes on a clean machine.

If you already have PostgreSQL installed and want pgrx to use it, pass an
explicit `pg_config` instead:

```bash
cargo pgrx init --pg18 /path/to/pg_config
```

Do not rely on an implicit `pg_config` when more than one PostgreSQL version is
installed.

## 3. Build And Run The Extension

From the repository root, build, install, start PG18, and open `psql`:

```bash
cargo pgrx run --release pg18
```

Inside `psql`, create the extension and run a small smoke query:

```sql
CREATE EXTENSION IF NOT EXISTS ecaz;

DROP TABLE IF EXISTS ecaz_smoke;
CREATE TABLE ecaz_smoke (
    id bigint generated always as identity primary key,
    embedding ecvector(4)
);

INSERT INTO ecaz_smoke (embedding)
VALUES
    (encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0]::float4[], 4, 42)),
    (encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0]::float4[], 4, 42)),
    (encode_to_ecvector(ARRAY[-1.0, 0.0, 0.0, 0.0]::float4[], 4, 42));

CREATE INDEX ecaz_smoke_hnsw_idx
ON ecaz_smoke USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 8, ef_construction = 64);

SELECT id
FROM ecaz_smoke
ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::float4[]
LIMIT 2;
```

Expected output:

```text
 id
----
  1
  2
(2 rows)
```

The `<#>` operator is negative inner product, so ascending order returns the
highest inner-product matches first.

## 4. Install Into An Existing PostgreSQL

For a system or Homebrew PostgreSQL installation, install with the exact
`pg_config` for the server you will run:

```bash
cargo pgrx install --sudo --release --pg-config /path/to/pg_config
```

Use `--sudo` when the PostgreSQL extension directories are owned by root. If
your PostgreSQL installation is user-writable, omit `--sudo`.

Then connect with your normal `psql` command and run:

```sql
CREATE EXTENSION ecaz;
```

## 5. Upgrade Or Uninstall

To rebuild and reinstall the current checkout into the same PostgreSQL
installation, rerun the same install command with the same `pg_config`:

```bash
cargo pgrx install --sudo --release --pg-config /path/to/pg_config
```

After installing new extension SQL files, update each database that has Ecaz
installed:

```sql
ALTER EXTENSION ecaz UPDATE;
```

For a specific extension version:

```sql
ALTER EXTENSION ecaz UPDATE TO '0.1.1';
```

Reconnect existing sessions after reinstalling the shared library so PostgreSQL
loads the new extension binary.

To remove Ecaz from a database:

```sql
DROP EXTENSION ecaz;
```

If objects depend on Ecaz types, functions, operators, or indexes, PostgreSQL
will reject the drop unless you remove those objects first or use `CASCADE`:

```sql
DROP EXTENSION ecaz CASCADE;
```

Use `CASCADE` carefully; it drops dependent database objects.

## 6. Install The Operator CLI

The `ecaz` CLI is the supported operator surface for repeatable corpus setup,
benchmarks, stress runs, and local pgrx SQL helpers.

```bash
cargo install --path crates/ecaz-cli
```

With the pgrx PG18 cluster running, a quick SQL check looks like:

```bash
ecaz dev sql --pg 18 --raw --sql "SELECT extversion FROM pg_extension WHERE extname = 'ecaz';"
```

All CLI commands accept PostgreSQL connection flags such as `--database`,
`--host`, `--port`, `--user`, and `--password`, with libpq environment variable
fallbacks. See the [Operator CLI README](../crates/ecaz-cli/README.md) for the
full command tree.

## 7. Validation Commands

For normal development, start with static and unit coverage:

```bash
make fmt-check
make lint
make test
```

For PostgreSQL integration coverage, use PG18 unless you are specifically
touching PG17 compatibility:

```bash
make pg-test
```

PG17 compatibility checks are explicit:

```bash
cargo pgrx init --pg17 download
make lint-pg17
make pg-test-pg17
```

## Troubleshooting

If pgrx cannot find PostgreSQL, rerun `cargo pgrx init` with an explicit
`--pg18 /path/to/pg_config` or `--pg17 /path/to/pg_config`.

If `CREATE EXTENSION ecaz` fails because the extension files are missing,
reinstall with the same `pg_config` used by the server you are connected to.

If a shell cannot find `ecaz` after `cargo install --path crates/ecaz-cli`, add
Cargo's bin directory to `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```
