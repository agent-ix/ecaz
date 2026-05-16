# Hardening Governance

Task 49 owns the rule that a hardening lane must either exercise real ECAZ code
or stay out of the lane inventory. Synthetic-only harnesses are not allowed to
ship as green checks.

## Tiers

| Tier | Meaning | Promotion bar |
| --- | --- | --- |
| PR | Blocks pull requests. | Two weeks of nightly/local stability, wall clock under five minutes, and at least one real finding or injected-bug validation. |
| Nightly | Runs on a slower cadence. | Reproducible setup, useful signal, and a documented failure triage owner. |
| Local | Developer/local packet evidence only. | Tool runs and produces interpretable output. |
| Manual | Used for campaigns or reports. | Packet records command, version, and findings. |

Promotion packets must include the lane name, source tier, target tier, wall
clock budget, signal evidence, and rollback plan. Lanes that repeatedly fail
for toolchain or environment reasons must be demoted with a repair packet
instead of blocking unrelated work.

The `test` PR lane is the single budget carveout: it is governed by the CI wall
clock budget for the full unit suite rather than the five-minute promotion rule
used for smaller hardening lanes.

## Current Inventory

Use `make hardening-tiers-report` for the current SHA-local report.

| Lane | Tier | Time budget | Last-passing SHA |
| --- | --- | --- | --- |
| `fmt-check` | PR | <1 min | current CI run |
| `lint` | PR | <5 min | current CI run |
| `test` | PR | variable | current CI run |
| `layout-check` | PR | <1 min | current CI run |
| `audit-unsafe` | PR | <1 min | current CI run |
| `proptest` | PR | <5 min | current CI run |
| `test-hardening-local` | Local | <5 min | packet evidence |
| `deny-full` | Local | <5 min | packet evidence |
| `cargo-audit` | Local | <1 min | packet evidence |
| `miri-expanded` | Nightly | variable | packet evidence |
| `careful` | Nightly | variable | packet evidence |
| `fuzz-all-short` | Nightly | variable | packet evidence |
| `kani` | Nightly | variable | packet evidence |
| `sanitizer-asan` | Nightly | variable | packet evidence |
| `sanitizer-lsan` | Nightly | variable | packet evidence |
| `cargo-geiger` | Manual | variable | packet evidence |
| `pg-test` | Manual | variable | packet evidence |
| `sqlsmith-pg18` | Manual | variable | packet evidence |

## Retired Synthetic Lanes

The Task 34 `hardening/rudra`, `hardening/flux`, `hardening/loom`, and
`hardening/shuttle` crates were removed because they proved synthetic examples
instead of ECAZ behavior. They can return only through Tasks 40, 44, or 45 with
real imports from `src/` and reviewer-visible signal evidence.

`make hardening-validate` enforces that retained hardening crates import real
repository code and that the retired synthetic lanes do not reappear as Makefile
targets or aggregate dependencies.
