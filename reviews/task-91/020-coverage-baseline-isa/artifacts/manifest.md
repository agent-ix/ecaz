# Artifact Manifest

- head SHA: `0e725de616f10af04fa9c70a0826eab1d461bdab`
- task bucket: `reviews/task-91/020-coverage-baseline-isa`
- timestamp: `2026-06-09T06:39:18-07:00`
- lane / fixture / storage format / rerank mode: local quality gate validation; no SQL fixture; no index storage format; no rerank mode
- table surface: not applicable; coverage baseline metadata only

## Artifacts

### `coverage/summary.txt`

- source: existing failed GitHub Actions `test-quality-coverage` artifact for
  the pushed PR head before this local fix.
- key result:

```text
quant/isa.rs ... Lines 84 Missed Lines 8 Cover 90.48%
```

### `coverage-baseline-check.log`

- command:

```bash
make coverage-baseline-check
```

- key result:

```text
coverage baseline complete for 43 critical paths
```

### `changed-files.txt`

- command:

```bash
git diff --name-only origin/main...HEAD
```

- purpose: mirrors the pull-request changed-files input used by
  `.github/workflows/ci.yml` for the coverage delta gate.

### `coverage-delta-changed-files-check.log`

- command:

```bash
scripts/check_coverage_delta.sh reviews/task-91/020-coverage-baseline-isa/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv reviews/task-91/020-coverage-baseline-isa/artifacts/changed-files.txt
```

- key result:

```text
coverage ok: quant/isa.rs actual=90.48 baseline=90.48
```

### `make-coverage.log`

- command:

```bash
make coverage COVERAGE_OUTPUT_DIR=reviews/task-91/020-coverage-baseline-isa/artifacts/coverage
```

- result: local setup failure because `cargo-llvm-cov` is not installed.
  No GitHub CI run was triggered for this packet.

### `git-diff-check.log`

- command:

```bash
git diff --check
```

- key result: command exited 0 with no whitespace findings.
