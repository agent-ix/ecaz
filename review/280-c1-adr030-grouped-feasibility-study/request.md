# Review Request: C1 ADR-030 Grouped Scoring Feasibility Study

## Context

Packet `279` covers the ADR-031 sign-derived binary-prefilter study. The user
asked to study both `ADR-031` and `ADR-030` so they can be compared directly on
the same real-corpus surface.

`ADR-030` is higher-risk than `ADR-031` because its grouped-LUT/FastScan story
assumes a quantized layout closer to grouped PQ, while tqvector currently stores
per-dimension scalar `4-bit` codes with a shared global codebook.

## Problem

Before runtime integration, we need to answer the feasibility question:

1. does the grouped-scoring reinterpretation map cleanly onto tqvector's
   current scalar-coded format
2. if not exactly, is there still a grouped approximate scorer on the existing
   encoding that is strong enough to compare meaningfully against ADR-031

Without that check, ADR-030 risks being treated as "obviously applicable" when
it may actually require a stronger encoding/layout change first.

## Planned Work

1. extend the study seam with an ADR-030 comparison mode on the current
   no-QJL `1536x4-bit` lane
2. measure how closely that grouped scorer tracks the exact scorer on the real
   corpus
3. compare its correlation / survivor capture against the ADR-031 binary mode
4. keep this slice out of ordered-scan runtime

## Exit Criteria

- the packet records whether ADR-030 is directly compatible with tqvector's
  current scalar-coded format or only as an approximate feasibility seam
- the packet records real-corpus comparison data against the exact scorer
- the required checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
