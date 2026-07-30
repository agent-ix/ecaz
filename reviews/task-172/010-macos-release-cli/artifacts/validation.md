# Validation

## Reproduction before the change

```sh
cargo build --release -p ecaz-cli
target/release/ecaz bench suite --help
```

Result:

```text
dyld: symbol not found in flat namespace '_MyDatabaseId'
exit 134
```

The dev-profile binary started successfully, proving the defect was specific to
the larger graph retained by release fat LTO.

## PG18 type provenance

Declarations were read from PostgreSQL 18.3 headers installed under
`/opt/homebrew/Cellar/postgresql@18/18.3/include/postgresql/server/`:

| Symbol | Header declaration | Rust storage |
| --- | --- | --- |
| `MyDatabaseId` | `Oid`, `miscadmin.h` | `u32` |
| `MyProcPid` | `int`, `miscadmin.h` | `i32` |
| `MyProcNumber` | `ProcNumber`; typedef `int`, `storage/procnumber.h` | `i32` |
| `ProcGlobal` | `PROC_HDR *`, `storage/proc.h` | `*mut c_void` |
| interrupt flags | `volatile sig_atomic_t`, `miscadmin.h` | `i32` |
| `XactIsoLevel` | `int`, `access/xact.h` | `i32` |

## Release build and launch

```sh
cargo build --release -p ecaz-cli
target/release/ecaz bench suite --help >/dev/null
```

Result: pass; release start exit 0. Build duration was 4m18s. The build emitted
one pre-existing dead-code warning for
`LoadedDistributedPlacementConfig::path` in `commands/corpus/load.rs`; no new
warning is attributable to this change.

## Mach-O data-symbol audit

```sh
nm -m target/release/ecaz |
  rg 'undefined.*(MyDatabaseId|MyProcNumber|MyProcPid|ProcGlobal|InterruptPending|ProcDiePending|QueryCancelPending|XactIsoLevel)'
```

Result: zero matches.

```sh
nm -gU target/release/ecaz |
  rg '_(MyDatabaseId|MyProcNumber|MyProcPid|ProcGlobal|InterruptPending|ProcDiePending|QueryCancelPending|XactIsoLevel)$'
```

Result: all eight symbols are defined by the executable.

## Static checks

```sh
rustfmt --edition 2021 --check crates/ecaz-cli/src/pg_macos_stubs.rs
git diff --check
```

Result: pass. Stable rustfmt printed the repository's existing notices about
nightly-only `imports_granularity` and `group_imports`; formatting itself
passed.

No PostgreSQL test or benchmark was run under the repository policy: this
checkpoint changes only macOS CLI startup linkage, and the release-profile
launch is the direct behavior under review.
