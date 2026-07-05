# Artifact Manifest

- packet: `733-c1-spire-customscan-remote-restart-rejoin`
- head SHA: `c815c26d`
- date: 2026-05-14
- measurement artifacts: none

This packet has no measurement artifacts. Validation is compile/static only;
the focused `cargo pgrx test pg18` runtime attempt failed at local loader
startup with `undefined symbol: pg_re_throw` before test execution.
