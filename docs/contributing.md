# Contributing

## Prerequisites

- [Rust](https://rustup.rs/) stable + nightly (for fuzzing and Miri)
- [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) 0.17
- PostgreSQL 17 or 18 development headers
- [valgrind](https://valgrind.org/) (for iai-callgrind benchmarks)
- [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (for fuzzing)

## Development Workflow

```bash
cargo pgrx init          # one-time: build local Postgres for testing
```

### Code Quality

```bash
make fmt                 # format code
make fmt-check           # check formatting (CI)
make lint                # clippy, deny warnings (default: pg18)
make lint-pg17           # clippy against pg17
make audit-unsafe        # verify SAFETY comments on unsafe blocks
```

### Testing

```bash
make test                # unit tests (no Postgres required)
make pg-test             # pgrx integration tests (pg18)
make pg-test-pg17        # pgrx integration tests (pg17)
make proptest            # property-based tests
make layout-check        # struct layout and size assertions
make miri                # Miri on pure-Rust paths (requires nightly)
```

### Benchmarks

```bash
make bench               # all criterion microbenchmarks
make bench-quant_score   # specific benchmark
make bench-iai           # instruction-count benchmarks (requires valgrind)
make dhat-encode         # heap profiling: encode path
make dhat-score          # heap profiling: score path
```

### SQL Benchmarks

Requires a running PostgreSQL instance with the extension installed:

```bash
make bench-sql-latency
make bench-storage
make bench-recall-sql
```

### Recall

```bash
make recall              # pure-Rust recall benchmark (~5 min for 10K vectors)
```

### Fuzzing

Requires cargo-fuzz and nightly Rust. Each target runs for 10 minutes:

```bash
make fuzz-parse-text
make fuzz-unpack
make fuzz-element-decode
make fuzz-neighbor-decode
```

### Build and Install

```bash
make build               # release shared library
make install             # install into local Postgres (requires sudo)
make clean               # remove build artifacts
```

## CI

| Target | Scope | When |
| --- | --- | --- |
| `make ci-quick` | fmt, lint, test, layout, unsafe audit | every PR |
| `make ci-nightly` | ci-quick + bench, iai, proptest, miri | nightly |

## Dependency Licenses

```bash
make deny                # check dependency licenses
```

## Review Workflow

This project uses a review-packet workflow. See [AGENTS.md](../AGENTS.md) for the full structure and conventions.
