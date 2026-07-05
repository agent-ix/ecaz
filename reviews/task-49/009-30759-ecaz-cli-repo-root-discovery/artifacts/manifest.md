# 30759 Artifact Manifest

Head SHA: `c923e3bbbd0864c9c14e37ce1e97a14ce6dffda6`

Packet: `30759-ecaz-cli-repo-root-discovery`

Scope: ecaz operator CLI repo-root discovery for `dev install ecaz-pg-test`.

Lane: local CLI and PG18 operator install validation. Fixture: installed
`target/release/ecaz` run from `/home/peter/dev/ecaz`. Storage format: not
applicable. Rerank mode: not applicable. Surface shape: operator CLI install
path for the local PG18 pgrx tree, independent of shared-table versus isolated
one-index-per-table index surfaces.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 12:47 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-ecaz-cli.log`
  - Command: `script -q -e -c 'cargo check -p ecaz-cli' ...`
  - Timestamp: 2026-05-10 12:47 PDT
  - Result line: `Finished dev profile ... target(s) in 0.39s`.

- `cargo-test-ecaz-cli-dev-support.log`
  - Command: `script -q -e -c 'cargo test -p ecaz-cli commands::dev::support' ...`
  - Timestamp: 2026-05-10 12:47 PDT
  - Result lines: `running 4 tests`; `4 passed; 0 failed`.

- `cargo-build-ecaz-cli-release.log`
  - Command: `script -q -e -c 'cargo build -p ecaz-cli --release' ...`
  - Timestamp: 2026-05-10 12:47 PDT
  - Result line: `Finished release profile ... target(s) in 5m 49s`.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 12:47 PDT
  - Result: pass, no whitespace errors.

- `ecaz-dev-install-before-fix.log`
  - Command:
    `target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file tmp/ecaz-dev-install-repro.log`
  - Timestamp: 2026-05-10 12:24 PDT
  - Result: reproduced the original blocker,
    `resolving repo root from crates/ecaz-cli` followed by
    `No such file or directory (os error 2)`.

- `ecaz-dev-install-after-fix.log`
  - Command:
    `target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file tmp/ecaz-dev-install-after-repo-root-fix.log`
  - Timestamp: 2026-05-10 12:38 PDT
  - Result lines: `repo=/home/peter/dev/ecaz`;
    `pg_config=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`;
    `backend artifact assertion passed`;
    `sha256=fb5731fdb722fe158e1ee1cf2a1c8a7e897c3074fa62896a81f5909e098d9891`.
