# Task 97 Packet 007: qjl32 SVE Dispatch Local Slice

This packet adds the target-gated qjl32 SVE/SVE2 dispatch path and parity test
hooks. It is a local code slice only: no AWS and no GitHub CI were run.

## Changes

- Implements `src/quant/qjl32/sve.rs` using the existing grouped-PQ
  `global_asm!` pattern.
- Prefers runtime `sve2` over `sve` and returns `Isa::Sve2` or `Isa::Sve`
  when the corresponding backend is selected.
- Adds a vector-length helper for future Graviton evidence.
- Adds an optional qjl32 SVE parity test that executes on SVE hosts and returns
  early elsewhere.

## Validation

- `cargo test qjl32 --lib -- --color never`
  - 10 passed; 0 failed
- Local AArch64 check remains unavailable because this host has the Rust target
  installed but lacks both `aarch64-linux-gnu-gcc` and `clang`.

Logs are under `artifacts/`.

## Review Request

Please review the qjl32 SVE/SVE2 dispatch path, the scalar-order accumulation
choice, and the test hook shape. Graviton 4 runtime dispatch, measured vector
length, and `isa=sve2` counter evidence are still deferred until AWS validation
is approved.
