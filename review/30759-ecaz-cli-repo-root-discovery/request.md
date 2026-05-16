# 30759 - ecaz CLI Repo Root Discovery

## Summary

This packet reviews commit `c923e3bbbd0864c9c14e37ce1e97a14ce6dffda6`
(`Fix ecaz CLI repo root discovery`).

The slice closes the packet `30756` reviewer P2 for the operator install path.
`ecaz dev install ecaz-pg-test --pg 18` no longer depends on a brittle
compile-time `CARGO_MANIFEST_DIR` relative path. It now resolves the repository
root by walking upward from:

- the current working directory;
- the compile-time manifest directory; and
- the current executable path.

The resolver accepts only directories that contain both the workspace
`Cargo.toml` and `crates/ecaz-cli/Cargo.toml`, so a bare crate directory is not
mistaken for the repo root.

After rebuilding `target/release/ecaz`, the exact operator install command that
failed in packet `30756` completed successfully and asserted the installed PG18
backend artifact.

## Key Files

- `crates/ecaz-cli/src/commands/dev/support.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli commands::dev::support`
- `cargo build -p ecaz-cli --release`
- `git diff --check -- <changed code/docs>`
- Before-fix repro:
  `target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file tmp/ecaz-dev-install-repro.log`
- After-fix proof:
  `target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file tmp/ecaz-dev-install-after-repo-root-fix.log`

The after-fix install reports:

- `repo=/home/peter/dev/ecaz`
- `pg_config=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `backend artifact assertion passed`
- installed backend SHA256
  `fb5731fdb722fe158e1ee1cf2a1c8a7e897c3074fa62896a81f5909e098d9891`

## Review Focus

- Is marker-based repo-root discovery the right operator behavior for an
  installed binary run from a checkout?
- Are the repo markers specific enough to avoid false positives?
- Does this sufficiently close the 30756 reviewer P2 before Stage G harness work
  depends on `ecaz dev install ecaz-pg-test`?
