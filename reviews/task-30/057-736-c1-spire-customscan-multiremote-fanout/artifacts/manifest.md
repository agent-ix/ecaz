# Artifact Manifest

- packet: `736-c1-spire-customscan-multiremote-fanout`
- head SHA: `aaf76df5`
- date: 2026-05-14
- measurement artifacts: none

This packet has no measurement artifacts. Validation is static/compile-only;
the focused `cargo pgrx test pg18` runtime attempt failed at local loader
startup with `undefined symbol: pg_re_throw` before test execution.
