---
task: 193
packet: 001-preregistration-baseline
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 193 pre-registration / candidate audit

Source inspection finds that the current owner payload endpoint already issues
one set-returning SQL operation per owner window: `build_payload_sql` uses
`unnest($1::text[]) WITH ORDINALITY`, joins each requested TID, and restores
request order. The coordinator groups remote IDs by owner and sends one
request per owner. Therefore MAT-23/MAT-24's proposed batching is already
present, and MAT-19 (SPI-plan caching) would be a distinct candidate rather
than a batch-fetch change.

Task 193 will not duplicate the existing batching. Its decision packet will
record this audit and STOP unless a paired measurement identifies a remaining
per-row SPI-plan cost attributable to MAT-19.
