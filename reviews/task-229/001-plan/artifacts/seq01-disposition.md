# Seq-01 reviewer finding disposition

The revised request seq-03 addresses the first outside verdict at
`feedback/2026-08-26-01-reviewer.md`. No source implementation began.

| Finding | Revised disposition |
| --- | --- |
| P1-1 | Sidecar key changed from identity-latest `vec_id` to version-exact frozen row-tier `row_tid`; `vec_id` remains a checked echo. Replacement is append-only. |
| P1-2 | Missing entry split from corruption: same-snapshot row-tier invisibility returns the existing missing/skip marker; a visible row-tier tuple without its same-transaction sidecar is corruption. |
| P1-3 | Primary 100k warm-mean threshold, replicate/sign rules, smaller-scale/tail gates, and storage/build/DML ceilings are preregistered. |
| P1-4 | Primary recall/latency uses a same-generation feature-gated Userset read toggle on covered generations; fresh-build AB/BA remains only for build/storage/DML axes. |
| P2-1 | Entry reduced to null bitmap plus packed fixed-width values; no per-entry digest/count/offset/cover digest. `STORAGE PLAIN`, fillfactor 100, exact derived length, and checksum boundary are explicit. |
| P2-2 | One compact heap plus non-covering unique row-TID B-tree retained and justified against visibility-dependent, payload-duplicating INCLUDE. |
| P2-3 | Existing digest domains stay unchanged; legacy byte/digest round-trip fixtures are required. |
| P2-4 | Every fixed 303-byte Rust, lifecycle-wire, SQL, export, and fixture consumer is explicitly in packet 002. |
| P2-5 | Cache invalidation, replay, drop/reclaim, REINDEX, relation enumeration, and SQL uniqueness are explicit five-relation work. |
| P2-6 | CLI fixture and suite fields are a definite separately reviewed runner prerequisite. |
| P2-7 | Benchmark-feature owner telemetry extends the existing physical response telemetry and is aggregated at the coordinator; topology is scraped per node. |
| P2-8 | Only an **uncovered** qual-required attnum disqualifies; covered qual-only is explicitly eligible. |
| P2-9 | Bespoke-config reason, isolated tables/indexes, release profile/features, `allow_debug_extension=false`, and `results.jsonl` traceability are explicit packet-004 manifest requirements. |

The eight question rulings are incorporated: fixed-width bound rationale and
PG18 binary stability; deterministic naming; unchanged domains and re-bootstrap
posture; explicitly initial DML digest; tombstone retention derived from row-
tier retention; missing/corruption split; only `Frozen` local hits converted;
and same-generation primary read attribution.
