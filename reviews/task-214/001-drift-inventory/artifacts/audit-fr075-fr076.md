# Audit: FR-075 (AM surface) + FR-076 (record format) vs code

Task 214 P0 slice. Auditor: parallel subagent, 2026-08-01, worktree
`.worktrees/task-203` @ `baf81d498`.

## Verified conformant (no finding)
- AM registration, handler, operator classes identical to ec_diskann (`bootstrap.sql:1629,1674-1683`; `routine.rs:745`).
- Reloptions `graph_degree`/`build_list_size`/`alpha`/`neighbor_code_format`/`closure_epsilon`/`head_index_cap`/`source_identity`/`distributed_control` with range validation (`options.rs:872-977`); defaults R=32 L=100 alpha=1.2 C=4096 (`mod.rs:220-247`).
- Routine callbacks complete (`routine.rs:26-81`, `cost.rs:17`).
- v4 metadata 97 bytes byte-for-byte; v5 control = 97-byte prefix + flags bit 0 + UUID at 97 (113 total); decoder rejects unknown flags/zero UUID/local graph state in control root (`page.rs:21-27,43-75,265-397`).
- `EC_CONTROL_PERSISTENCE`, CIC rejection, REINDEX control reset + fresh UUID (`ambuild.rs:486-616`).
- Control index rejects ordinary scans/inserts (`routine.rs:107-111,222-226`; `mod.rs:208-218`).
- FR-076 record: 20-byte header exact offsets, total `20 + S + R×8 + R×S`, ItemPointer LE encoding; physical-v1 decoder rejects legacy `(0x09,0x00)` prefix, enforces zero padding and neighbor_count ≤ R (`tuple.rs:42-344`); tombstones traversable-but-excluded (`expand.rs:114-177`, `scan.rs:490,793`).
- Handoff entry/batch wire formats field-for-field (offsets, domains, strictly-increasing vec_ids, zero graph_flags, NULL bitmap LSB-first, no TIDs/conninfo in envelopes) (`handoff_wire.rs`).
- Vamana core shared with ec_diskann, not forked (`ambuild.rs:1008,1484`, `shard_build.rs:58-61,961,1171`).
- Vec_id pinned deterministic hash + anti-drift test (`identity.rs:110-125`).

## Findings

### F1 — EXPLAIN counters not implemented (high, specified-but-changed)
FR-075 Outputs promise NFR-019 traversal counters in EXPLAIN (also FR-081-AC-5, NFR-019:39). `custom_scan.rs:66` — `ExplainCustomScan: None`; counters observable only via off-by-default NOTICE GUC `ec_distann.scan_profile_notice` (`routine.rs:614-628`, `options.rs:434-441`). Cross-ref NFR audit F7.

### F2 — Deployment mode is session-GUC state on the default lane (high, specified-but-changed)
FR-075: "deployment mode is determined by the published epoch manifest's node roster … no session state or GUC overrides it." Code: legacy (v4, default) lane derives multinode-ness and node set from the `ec_distann.roster` userset GUC (`roster.rs:36-70`; `custom_scan.rs:87-91`; `routine.rs:419-481`), and `scan_epoch` falls back to the `ec_distann.epoch` GUC when metadata is not Published (`roster.rs:87-93`). Only the physical control lane reads a published manifest. The spec describes the M2 lane as already replaced; it ships and drives multi-node scans.

### F3 — GUC surface massively understated (medium, shipped-but-unspecified)
FR-075 Inputs list two session GUCs (BW, H). Code registers ~17 production + 8 debug-fault GUCs (`options.rs:291-577`, `roster.rs:42-71`, `scan_registry.rs:198-218`). Only `candidate_heap_limit` (FR-079) and `max_scan_pins`/`max_retire_fences` (FR-082) are named in any spec. Notably `ec_distann.top_k` changes scan semantics (D9 early-exit bar, deepening trigger) and is spec-invisible.

### F4 — Task 210 head-topology GUCs unspecified (high, shipped-but-unspecified)
`shard_head_storage` (default true), `sharded_head_search` (default true), `head_replica_count`, `gateway_copy_capacity`, `allow_nonconforming_replica` (`options.rs:39-59,370-413`) — the shipped default head architecture — appear in no spec; code cites "NFR-021 clause 3/4"/"TRAV-30" but NFR-021 contains none of the mechanisms or GUC names. Cross-ref FR-080 audit.

### F5 — `build_shards` reloption unspecified (medium, shipped-but-unspecified)
`options.rs:913-923` (0=auto/1=monolithic/≥2=sharded), `mod.rs:250-256`. FR-075's reloption list includes companion `closure_epsilon` but not the switch that activates sharded build. Auto policy also unspecified (cross-ref FR-077/078 audit F4).

### F6 — Default vec_id derivation is a local heap-TID hash (medium, specified-but-changed)
FR-076 describes only the global source-identity derivation. Default (`source_identity` absent): `vec_id_from_local_heap_tid` (`identity.rs:44-49`) — stable across index rebuilds but not table rewrites, unusable for multinode placement. Control mode forces `source_identity='include'` (`ambuild.rs:620-624`), confining the gap to the single-node lane — which the spec never describes.

### F7 — Handoff identity pinned to exactly 16 bytes (low, specified-but-changed)
Spec: length-prefixed, unconstrained. Code rejects any identity ≠ 16 bytes (`handoff_wire.rs:126-131`, ADR-063 canonical payload).

### F8 — Record is format-versioned, not epoch-versioned (low, specified-but-changed)
FR-076 "self-describing and epoch-versioned": the record (`tuple.rs:65-73`) carries no epoch field; epoch binding lives at generation/manifest level. Stale wording, not a correctness gap (D10 immutability).

### F9 — Default lane still writes the legacy `(0x09,0)` record prefix (low, specified-but-changed)
`tuple.rs:186-192,282-287`: default lane's encode/decode use the legacy prefix; only physical generations carry `record_version=1`. FR-076 mentions the legacy prefix only as something the physical decoder rejects.

### F10 — String reloptions not validated at amoptions time (low, specified-but-changed)
`options.rs:953-974`: `neighbor_code_format`/`source_identity` registered with `None` validators; `ALTER INDEX ... SET (neighbor_code_format='bogus')` is accepted and fails at next use, which FR-075's "validate at amoptions time" does not allow. CREATE INDEX still fails immediately (ambuild parses), so FR-075-AC-2 holds in practice.

## Behaviors in NO distann spec (grep-verified)
- Task 210 head topology (F4).
- `build_shards` + auto-shard policy (F5).
- `ec_distann.top_k` + its role as deepening trigger (`routine.rs:196-211,654-662`, `mod.rs:277-280`).
- Per-backend physical epoch/head cache + `physical_epoch_cache` kill switch (`options.rs:70,442-449`).
- Remote RPC timeout budgets (`remote_connect_timeout_ms` 5000, `remote_statement_timeout_ms` 120000; `options.rs:98-104,489-508`).
- `replica_control_password_file` (SIGHUP-context credential path; `options.rs:105-106,450-457`).
- The 8 NFR-020-style debug fault-injection GUCs (`options.rs:108-157,509-576`) — code cites NFR-020/TC-042; NFR-020 names none.
- Benchmark-only GUC family behind `distann-head-attribution-benchmark` (`options.rs:71-94,312-369`) — compile-gated; lower priority.
