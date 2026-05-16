---
id: NFR-004
title: Safety and Stability
type: non-functional-requirement
artifact_type: NFR
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-002"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-004: Safety and Stability

## Requirement

### No Backend Crashes

The extension SHALL NOT cause a PostgreSQL backend to crash under any input. All errors SHALL be reported via `ereport(ERROR)`, not `panic!` or segfault.

Rust panics in pgrx are caught and converted to PostgreSQL ERRORs — this is acceptable. Uncaught panics that bypass pgrx's catch mechanism are NOT acceptable.

### Memory Safety

- No use of `unsafe` code outside of pgrx FFI wrappers and GenericXLog calls
- All `unsafe` blocks SHALL have a `// SAFETY:` comment explaining the invariant
- No memory leaks in scan state, build state, or vacuum state (all freed in end/cleanup callbacks)

### WAL Correctness

- All index mutations are WAL-logged via GenericXLog
- After crash + WAL replay, the index SHALL be usable without REINDEX

### Licensing

- Extension code: MIT
- All transitive Cargo dependencies: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, or ISC
- `cargo deny check licenses` SHALL pass

### Hardening Verification

Task 34 defines the current local-first hardening baseline. Ecaz SHALL maintain
documented hardening lanes for supply-chain checks, unsafe/static hygiene,
Miri, cargo-careful, fuzzing, Kani, Flux, Loom, Shuttle, sanitizers, SQLsmith,
and one-shot exploratory analyzers such as Rudra and MIRAI where useful.

Hardening lanes SHALL record whether they are PR-gate, nightly, weekly/manual,
or report-only. Missing optional tools SHALL fail with actionable setup text
rather than obscure cargo subcommand errors. Live PostgreSQL, sanitizer, and
SQLsmith lanes SHALL state their PG18 and cluster prerequisites.

## Analysis Requirement Rules

Analysis requirements SHALL be written as evidence-producing requirements, not
as informal "run a tool sometime" notes.

1. Each analysis requirement SHALL name the risk class it covers, such as
   memory safety, supply-chain risk, undefined behavior, concurrency, parser
   robustness, or measurement validity.
2. Each requirement SHALL name at least one tool, command, script, or review
   method that produces evidence for that risk class.
3. Each requirement SHALL state the gate level: PR, nightly, weekly/manual,
   local-only, or report-only.
4. Each requirement SHALL state the artifact that proves the result, such as a
   test log, analyzer log, fuzz corpus/crash artifact, coverage audit, or
   review-packet manifest.
5. Each requirement SHALL state its interpretation rule: pass/fail threshold,
   allowed skip condition, false-positive triage process, or follow-up filing
   rule.
6. Tooling-sensitive requirements SHALL state platform and environment
   prerequisites instead of silently treating missing tools as passing.
7. A requirement SHALL NOT claim production correctness from a tool whose model
   excludes the relevant boundary. For example, Miri and Kani evidence for pure
   Rust helpers does not prove pgrx/SPI/libpq callback safety.

## Ecaz Application

The Task 34 hardening surface applies the analysis rules as follows:

| Risk Class | Required Lane | Gate Level | Evidence Artifact | Interpretation |
| --- | --- | --- | --- | --- |
| Formatting and static Rust hygiene | `make fmt-check`, `make lint`, `make lint-hardening` | PR after burn-in | command log | Fail on formatting drift or Clippy warnings outside the accepted baseline. |
| Supply chain | `make cargo-audit`, `make deny-full`, `make cargo-vet` | PR candidate / report mode | audit, deny, and vet logs | Fail or file follow-up for vulnerable, unlicensed, or unaudited dependency findings according to lane maturity. |
| Unsafe boundary discipline | `make audit-unsafe`, `make cargo-geiger`, Rudra/MIRAI/Flux pilots | PR for baseline audit; report-only for exploratory analyzers | unsafe baseline diff and analyzer logs | New uncommented unsafe fails; exploratory analyzer findings are triaged into follow-up tasks or explicitly closed. |
| Pure Rust memory and UB checks | `make miri-expanded`, `make careful`, sanitizer local lanes | nightly/local | tool logs | Passing evidence covers pure Rust modeled paths only; pgrx/SPI/libpq boundaries require PG18/live evidence. |
| Parser and decoder robustness | `make fuzz-all-short`, individual cargo-fuzz/AFL targets | nightly/manual | fuzzer logs and crash artifacts | Any reproducible crash becomes a P0/P1 follow-up with minimized input attached. |
| Bounded invariants and concurrency | `make kani`, `make flux`, `make loom`, `make shuttle` | nightly/report-only | proof/model-check logs | Proofs apply only to the bounded harnesses named in the log. |
| PostgreSQL/live callback safety | `make pg-test`, PG18 sanitizer lanes, SQLsmith | PG18/manual until stable | pg_test, sanitizer, SQLsmith logs | Required before promoting claims about executor, CustomScan, SPI, libpq, or memory-context behavior. |

## Measurement

- `make hardening-local` for stable local checks that do not need a live cluster.
- `make hardening-nightly-local` for slower Miri, cargo-careful, fuzz, Kani,
  Loom, Shuttle, and sanitizer smoke lanes.
- `make audit-unsafe` and the unsafe baseline file for "no new uncommented
  unsafe" enforcement.
- `make cargo-audit`, `make deny-full`, and `make cargo-vet` for supply-chain
  reporting and gate candidates.
- Fuzz testing: feed random byte sequences to parsers, tuple decoders,
  metadata decoders, item-pointer decoders, and vector normalization paths.
- Code review and packet-local logs for unsafe/static analyzer findings.

## Acceptance Criteria

### NFR-004-AC-1

`docs/hardening.md` documents every Task 34 hardening lane, its command, its
tool prerequisites, and whether the lane is local, nightly, PG18/live-cluster,
or standalone/report-only.

### NFR-004-AC-2

`make hardening-local` runs the stable local hardening subset without requiring
a live PostgreSQL cluster.

### NFR-004-AC-3

`make hardening-nightly-local` runs the toolchain-sensitive local lanes or
skips unsupported platform-specific sanitizer lanes with an explicit message.

### NFR-004-AC-4

The review packet for a hardening-lane change stores raw tool logs and records
which lanes passed, skipped, or remain manually gated.

### NFR-004-AC-5

New unsafe blocks require nearby `SAFETY` comments, and the unsafe audit
baseline detects new uncommented unsafe lines.
