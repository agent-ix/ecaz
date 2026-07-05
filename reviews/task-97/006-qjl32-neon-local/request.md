# Task 97 Packet 006: qjl32 NEON Local Kernel

This packet adds the target-gated NEON implementation for the qjl32 family.
It is a local code slice only: no AWS and no GitHub CI were run.

## Changes

- Implements `src/quant/qjl32/neon.rs`.
- Returns `Isa::Neon` only on AArch64 hosts where runtime feature detection
  reports NEON support.
- Keeps scalar fallback on non-AArch64 hosts or if NEON is not reported.
- Uses four-lane NEON multiply chunks for the MSE and QJL sign terms, then
  accumulates the lane products in scalar order to preserve the existing
  qjl32 tolerance contract.

## Validation

- `cargo test qjl32 --lib -- --color never`
  - 9 passed; 0 failed
- `cargo check --target aarch64-unknown-linux-gnu --lib`
  - blocked before compiling ecaz by missing local cross C compiler:
    `aarch64-linux-gnu-gcc`

Logs are under `artifacts/`.

## Review Request

Please review the NEON qjl32 implementation and fallback behavior. The local
environment cannot complete AArch64 `cargo check` because the cross C compiler
is missing, and Graviton 4 runtime dispatch/vector-length evidence remains
deferred until AWS validation is approved.
