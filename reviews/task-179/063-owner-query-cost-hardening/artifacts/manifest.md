# Artifact manifest

- Head SHA: `4587c0d0980bf7c2d56c3dbb751ec36e4492ff08`
- Task bucket / packet: `reviews/task-179/063-owner-query-cost-hardening/`
- Timestamp: `2026-07-13T21:41:59-07:00`
- Host / PostgreSQL: local Intel x86_64 / PostgreSQL 18.3
- Lane: local focused correctness and static validation
- Fixture / storage format / rerank mode: three-owner loopback physical
  generations / current physical row+graph generation format / not applicable
- Isolation: one logical source with one index per physical owner; no shared
  benchmark-table surface

## Artifacts

| Artifact | Command | Result | SHA-256 |
| --- | --- | --- | --- |
| `three-owner-pg18.log` | `cargo pgrx test pg18 test_distann_three_owner_physical_handoff` | 1 passed, 0 failed, 2511 filtered | `5d3aceed001bec0092eb0f0f710c33ebd0958dcfa0e766e62d490cb1bd83456f` |
| `query-cache-unit.log` | `cargo test --lib --no-default-features --features pg18 physical_query_cache_requires_matching_digest_and_reuses_arc` | 1 passed, 0 failed | `a15995de83a9583c1150b8306acb2c34616584c7722f9d4f8f9a430347390785` |
| `persisted-head-unit.log` | `cargo test --lib --no-default-features --features pg18 persisted_head_graph` | 2 passed, 0 failed | `33090a2107fd80f0ddad6fc7ead82b7497407e9845c6682fb74010f129c3696f` |
| `clippy-pg18.log` | `cargo clippy --lib --no-default-features --features pg18 -- -D warnings` | pass | `d8d1c6dc571c652d95ea94e10953378eec3ca9d56fc44b2d90efe76b0cc22d5b` |

The logs were captured with `script -q -e -c`, retain their command exit code,
and were normalized to LF before hashing. No corpus, raw SSM tree, tunnel or
polling exhaust, PostgreSQL operational log, or regenerable cache is included.
