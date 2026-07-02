# Task 123 Review Request: Local Multinode Production-Read Variants

## Scope

This packet asks for review of commit `cb1304515`:

- Adds `bench_production_read_variants` to the FR-038 `spire-local-multinode` suite step.
- Emits repeated `--bench-production-read-variant` arguments from suite expansion.
- Teaches `local-multinode-pg18` to generate multiple nested production-read `spire-pipeline` steps from one fixture build.
- Keeps the existing no-variant behavior unchanged: one default-cap step and one rowcap step with the configured projection/GUCs.

Variant strings use this shape:

```text
name=<slug>;projection=id,source;guc=ec_spire.pre_materialization_prune=on
```

Global `bench_session_gucs` still apply to every generated production-read step. Per-variant `guc=` entries are appended, so the next Task 123 communications packet can set a common raised payload cap once and vary only projection/prune.

## Why

Reviewer feedback on the reopened 121/123 work asked for the core communications measurement across both:

- Raised full-payload shipping (`id,source`) and narrow/optimized projection (`id`).
- Pre-materialization prune on/off.

Doing that with packet 015 support alone would require rebuilding the four-node local fixture for each condition. This patch keeps the measurement inside `ecaz bench suite` while allowing a single local multinode fixture to produce the requested matrix.

## Validation

Packet-local artifacts:

- `artifacts/cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Result: exit 0. The log contains stable-channel rustfmt warnings for existing unstable config keys.
- `artifacts/cargo-test-suite-local-multinode-variants.log`
  - Command: `cargo test -p ecaz-cli spire_local_multinode_step_expands_local_four_instance_lane -- --nocapture`
  - Result: 1 passed.
- `artifacts/cargo-test-production-read-variant-parser.log`
  - Command: `cargo test -p ecaz-cli parses_bench_production_read_variant -- --nocapture`
  - Result: 1 passed.

## Review Notes

This is still not a Task 123 closeout. It is runner support for the follow-on measurement packet, intended to avoid ad hoc shell glue and repeated cluster rebuilds while measuring the open communications/prune findings.
