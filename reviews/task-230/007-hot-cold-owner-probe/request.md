---
task: 230
packet: 007-hot-cold-owner-probe
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 01
---

# Task 230 hot/cold remote-owner probe correction

Review the standalone harness correction at `93615542d`. Packet 004's clean
restart completed its first 10k row-heap control, but the first hot/cold arm
stopped after topology/serving and before benchmark measurement because the
remote-owner placement proof queried logical `source_id` and `source` columns
on the compact hot relation.

## Root cause

The standard logical fixture is
`dm(id, source_id, source, embedding, payload_note)`. In row-heap layout the
generation relation preserves those logical names. In hot/cold layout the hot
relation preserves source attnums instead: identity attnum 2 is `a_2`, while
the raw query vector `source` is not a hot column.

The placement proof correctly obtains its sample from the owner generation
relation, but it had only ever been exercised against row-heap layout and
hardcoded:

```sql
SELECT source_id::text || '|' || source::text FROM <generation-row-relation>
```

That fails closed on the hot relation with `column "source_id" does not exist`.

## Correction

- Select the physical identity column by layout: `source_id` for row heap,
  `a_2` for the compact hot tier.
- Keep ownership proof anchored in the internal generation relation, then join
  its identity to logical `dm.source_id` only to obtain the raw query vector.
- Use the same physical identity column for both the exact pinned owner probe
  and the returned-candidate ownership count.
- Add a focused test pinning both layout-to-column mappings.

No suite config, threshold, release guard, result parser, or decision rule
changes. The stopped hot/cold arm emitted no benchmark result. Because the CLI
changed after the successful row-heap arm, Packet 004 will not resume that arm;
after review closure it restarts from step 1 with an empty result surface and
all fixtures rebuilt at one accepted head.

## Validation

- `cargo fmt --check`: exit 0.
- `cargo test -p ecaz-cli
  task230_remote_owner_identity_column_tracks_physical_layout`: 1 passed,
  0 failed.
- `cargo clippy -p ecaz-cli --all-targets`: exit 0; baseline unchanged at
  77 binary / 78 test warnings.

## Review request

Please verify the physical-column mapping, that the join does not weaken the
owner-placement proof, and that all three internal-relation identity probes use
the selected column. If DONE, authorize a fresh Packet 004 step-1 restart after
release reinstall and matching CLI rebuild at the accepted head.
