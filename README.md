# turboquant

A PostgreSQL extension written in Rust that adds the `turboquant` type — a lossless fixed-point decimal backed by a 64-bit integer mantissa and an explicit scale.

## Design

| Field   | Type  | Description                              |
|---------|-------|------------------------------------------|
| `value` | `i64` | Raw integer mantissa                     |
| `scale` | `i16` | Decimal digits after the point           |

`1.50` is stored as `value=150, scale=2`. Arithmetic never goes through floating-point.

## Operations

| SQL Operator | Description         |
|-------------|---------------------|
| `+`         | Addition            |
| `-`         | Subtraction         |
| `*`         | Multiplication      |
| `=`, `<`, `>`… | Comparison      |

Aggregate: `turboquant_sum(col)`.

Casts: `float8 → turboquant`, `turboquant → float8`.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [pgrx](https://github.com/pgcentralfoundation/pgrx): `cargo install cargo-pgrx`
- PostgreSQL dev headers

## Getting started

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install pgrx toolchain
cargo install cargo-pgrx
cargo pgrx init   # downloads and builds a local Postgres

# Run tests
cargo pgrx test

# Install into local Postgres
cargo pgrx install
```

## SQL usage

```sql
CREATE EXTENSION turboquant;

SELECT '3.14'::turboquant + '1.00'::turboquant;  -- 4.14
SELECT turboquant_sum(price) FROM orders;
```
