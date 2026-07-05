# Task 67 Packet 028: Cloud Install Extension Features

## Summary

This packet lands a narrow `ecaz cloud install` extension point needed for the
remaining Task 67 Slice I bf16 validation. The cloud install command can now
pass repeatable extra Cargo features to the extension build:

```sh
target/debug/ecaz cloud install --profile 10k-intel --extension-feature rabitq-bf16 ...
```

The default install path is unchanged: without `--extension-feature`, the
generated remote script still runs `cargo pgrx install --sudo --release
--pg-config /usr/bin/pg_config` with no `--features` argument.

Code commit: `567c2a8fb6c6d56fe664ef59b9e833f541b24dd8`

## Why

Packet 025 feedback keeps Task 67 open partly because Slice I needs an Intel
host bf16 decision packet. The existing cloud install path always built the
extension with default features only, so there was no normal way to install the
feature-gated `rabitq-bf16` extension on the AWS Intel lane. This change avoids
one-off SSM shell work and keeps the bf16 measurement path inside the operator
CLI.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz-cloud install_script_ --lib`

Logs are in `artifacts/local/`.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/local/cargo-fmt-check.log`
- `artifacts/local/ecaz-cloud-install-script-tests.log`
