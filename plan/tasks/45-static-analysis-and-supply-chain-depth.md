# Task 45: Static Analysis and Supply-Chain Depth

Status: **proposed** — extends Task 34's `cargo-audit` / `cargo-deny` /
`cargo-vet` baseline with custom static analysis, API/ABI stability
tracking, SBOM generation, and supply-chain audit delegation.

## Scope

Two related areas:

### Static analysis depth

- **`dylint` custom lints** specific to ECAZ patterns:
  - `ecaz_panic_across_ffi` (paired with Task 41).
  - `ecaz_no_unwrap_in_pg_callback` — forbids `.unwrap()` /
    `.expect()` inside any function reachable from a PG callback.
  - `ecaz_no_raw_lwlock` — forbids direct `pg_sys::LWLockAcquire` /
    `LWLockRelease` outside the RAII wrapper module (paired with
    Task 41).
  - `ecaz_unsafe_needs_safety_comment` — already enforced via
    `scripts/check_unsafe_comments.sh`; promote to a real lint so it
    runs in `cargo clippy`.
  - `ecaz_no_direct_pg_alloc` — forbids `malloc` / `Box::leak` for
    PG-side memory; mandates palloc-equivalent paths.
- **`cargo-public-api`** to track the workspace's public API surface
  per crate; the diff lands in CI per PR.
- **`cargo-semver-checks`** to assert the public API does not break
  SemVer on a release commit.
- **Clippy strictness ratcheting** — burn down the
  `CLIPPY_HARDENING_BASELINE_ALLOW` list in `Makefile`, one lint at a
  time, with a packet per cluster.

### Supply-chain depth

- **`cargo-vet` criteria**: import audit results from
  `mozilla`, `google`, `bytecodealliance`, or other established
  delegations; require local audits for any dependency not covered.
- **SBOM**: generate a CycloneDX or SPDX SBOM via `cargo-sbom` /
  `cargo-cyclonedx` for every release; ship it as a release artifact.
- **`cargo-vendor` and reproducible builds**: verify a vendored build
  produces a bit-identical binary across two runs on the same host;
  ratchet toward bit-identity across hosts.
- **Crate signing / provenance**: validate that release builds use
  crates with attested provenance (where available via crates.io's
  ongoing work).
- **License compliance**: `cargo-deny licenses` with an explicit
  allow-list; any new dependency with an unlisted license requires
  packet justification.
- **Yank policy**: a script that fails CI if any dependency in
  `Cargo.lock` has been yanked since the last green build.

## Why

Task 34 set up `cargo-audit`, `cargo-deny`, `cargo-vet`, and
`cargo-geiger` as local lanes. That's the baseline. The next level is
catching bugs and policy violations *specific to ECAZ* that no
general-purpose lint catches, and giving operators and downstream users
a credible supply-chain story:

- An ECAZ-specific lint suite encodes invariants that exist in the
  reviewer's head today and are enforced by convention. Reviewers move
  on; conventions decay; lints survive.
- API stability: ECAZ is consumed via SQL and via `ecaz-cli`. A
  silent break in the Rust API surface of `ecaz-cloud` or the CLI
  produces churn for downstream users. `cargo-public-api` and
  `cargo-semver-checks` make that visible.
- SBOM is increasingly required for enterprise consumption (executive
  orders, NIS2, etc.). Generating one is free; not having one blocks
  some procurement processes.
- Audit delegation: `cargo-vet` works only if criteria and imports are
  actually configured. The Task 34 init produced an empty vet config
  in report mode; this task makes it real.

## Approach

### Static analysis

1. **`dylint` crate.** Add `crates/ecaz-lints/` exporting the lints
   listed above. Each lint has a self-test that proves it fires on a
   bad fixture and stays quiet on a good one.
2. **Wiring.** Add a `make dylint` lane and include it in
   `hardening-local`. CI runs it per-PR.
3. **`cargo-public-api`.** Add `make public-api-diff` that compares
   against `origin/main`'s recorded surface. The recorded surface is
   committed as `docs/public-api/{crate}.txt`. New PRs that change the
   surface must update the file.
4. **`cargo-semver-checks`.** Add `make semver-check` and run it on
   release branches. Surface-breaking changes require a major version
   bump or explicit packet justification.
5. **Clippy ratcheting.** For each lint in
   `CLIPPY_HARDENING_BASELINE_ALLOW`, file a short packet that either
   removes the allow and fixes the call sites, or documents the lint
   as permanently disabled with a rationale. The list shrinks each
   release.

### Supply chain

6. **`cargo-vet` criteria.** Import established audit sets:

   ```sh
   cargo vet import google
   cargo vet import mozilla
   ```

   Establish ECAZ-specific criteria (e.g., `safe-to-deploy`,
   `crypto-reviewed`) and apply them to dependencies that handle
   security-sensitive paths (any TLS / auth / signature crate).

7. **SBOM.** `make sbom` produces
   `target/sbom/ecaz-{version}.cdx.json` and uploads it as a release
   artifact. The lane runs per-release, not per-PR.

8. **Reproducible builds.** `make repro-check` builds twice and
   compares binary hashes; first iteration only enforces same-host
   determinism, future iteration tracks `mtime`-free archives across
   hosts.

9. **Yank watch.** `make yank-watch` runs `cargo audit --yanked` (and
   the dedicated `cargo-yank-check` if available) and fails CI on
   yanked deps.

10. **License allowlist.** Extend `deny.toml` with a strict
    `licenses` block; new dependency licenses require explicit
    addition.

### Lanes

11. **Make lanes:**
    - `make dylint`, `make public-api-diff`, `make semver-check`
    - `make sbom`, `make repro-check`, `make yank-watch`
    - `make supply-chain-full` — aggregates the supply-chain lanes for
      release-time review.

## Validation

- Each `dylint` lint fires on its self-test fixture and stays quiet on
  current `src/`.
- `make public-api-diff` reports zero diff on a no-op PR.
- `make semver-check` rejects a deliberately added breaking change.
- `make sbom` produces a CycloneDX file that validates against the
  schema and lists every workspace dependency.
- `make repro-check` produces identical hashes across two consecutive
  builds.
- `make yank-watch` flags a deliberately-yanked test dependency.

## Exit Criteria

- `crates/ecaz-lints/` houses the lint suite and is wired into
  `hardening-local`.
- `docs/public-api/` committed and CI-enforced.
- `cargo-vet` reports zero unaudited dependencies (or each unaudited
  dependency is explicitly listed with rationale).
- SBOM generation runs on release tags.
- License allowlist enforced; deny config rejects any unlisted
  license.
- `CLIPPY_HARDENING_BASELINE_ALLOW` is empty (or each remaining entry
  documented as permanently allowed).

## Dependencies

- Task 41 (FFI safety) defines the lints `dylint` enforces.
- Task 34's `cargo-deny` / `cargo-audit` / `cargo-vet` lanes are
  prerequisites.
- Independent of Tasks 36–40, 42–44.
