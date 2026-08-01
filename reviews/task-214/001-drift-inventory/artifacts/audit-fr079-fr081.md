# Audit: FR-079 (remote expansion protocol) + FR-081 (query orchestration) vs code

Task 214 P0 slice. Auditor: parallel subagent, 2026-08-01, worktree
`.worktrees/task-203` @ `baf81d498`.

## Findings

### F1 — Physical expansion wire is an 8-parameter overload with query-digest session state (high, specified-but-changed)
FR-079 declares the 6-parameter `ec_distann_expand_nodes(...)` a frozen wire contract. Production physical wire calls an 8-parameter overload adding `query_digest bytea` and `skip_neighbor_vec_ids bigint[]` (`lib.rs:808-816`; `remote_transport.rs:567-571`; `generation_read.rs:2327-2356`). The digest enables stateful semantics unmentioned by the spec: after hop 1 the coordinator sends empty `query` + digest and the owner reuses a per-connection cached query vector (`remote_transport.rs:1141-1153`).

### F2 — Legacy oid-signature expansion appends telemetry columns (medium, shipped-but-unspecified)
`remote_endpoint.rs:58,168-199` — `owner_total_ns`, `owner_open_validate_ns` on every row in all builds; the spec's 5-column response has no telemetry columns. Only the regclass SQL wrappers project the spec shape.

### F3 — Legacy materialization accepts caller-supplied send-function names — the shape FR-079 prohibits (high, specified-but-changed)
`remote_endpoint.rs:463-488` takes `payload_send_functions text[]`, interpolated into owner-side SQL (`build_payload_sql` :619-666, mitigated by an identifier-shape validator); used in production by the non-distributed-control multi-node CustomScan lane (`custom_scan.rs:889` → `remote_transport.rs:2156-2159`). FR-079-AC-9 forbids exactly this. Only the physical overload (attnums + schema fingerprint from the retained row-schema descriptor, `generation_read.rs:1608-1634`) conforms.

### F4 — `EC_VECTOR_MISSING` for missing row-tier tuples is never raised (high, specified-but-changed)
Owners return a per-row `tuple_payload_missing` flag (extra column absent from the spec schema; `lib.rs:818-825`, `remote_endpoint.rs:445,654-665`, `generation_read.rs:1655-1664`). The physical coordinator converts it to the wrong category (`EC_GENERATION_MISSING`, `generation_read.rs:3911-3914`); the legacy CustomScan lane **silently drops** the row as a benign race (`custom_scan.rs:1369-1372,1166`), where the spec declares it "always corruption or co-placement drift, never a vacuum race."

### F5 — Error taxonomy diverges on 4 of 9 FR-079 rows (medium, specified-but-changed)
`expand_error.rs:14-117`: schema mismatch → `EC_BAD_INPUT` (`generation_read.rs:1591-1607`); `EC_UNSUPPORTED_PROJECTION` only a coordinator plan-time string (`custom_scan.rs:321`); `EC_EPOCH_FINGERPRINT_VERSION` only message text inside another class (`manifest_v2.rs:558-566`); `EC_REMOTE_INTERNAL` spelled `EC_INTERNAL`. Conversely `EC_GENERATION_MISSING` (42704) ships but is not in the FR-079 table.

### F6 — Three additional endpoints in the FR-079 class unspecified (medium, shipped-but-unspecified)
`ec_distann_list_directory` (:94-108, full vec_id/ctid/tombstone enumeration), `ec_distann_materialize_rows` (:325-342, ctid-shipping, loopback-only validity, overlaps the specified endpoint with weaker semantics), `ec_distann_epoch_fingerprint` (:206-224). All captured by the fail-closed privilege block; none in any FR.

### F7 — Entire TRAV-30 gateway-copy mechanism unspecified (high, shipped-but-unspecified)
`gateway_copy.rs` (copy set, `fill_gateway_rows`, stats fn); `generation_read.rs:2000-2042` (`ec_distann_gateway_routing_export`), :3955-4050 (populate), :4110-4210 (skip-mask + coordinator rescore + batch-L re-application); `options.rs:370-379` (`gateway_copy_capacity`, discard-on-change); `remote_transport.rs:739-742` (`skip_neighbor_vec_ids`). Reachable in production builds; materially alters the FR-079 response contract (owners return deliberately empty neighbour arrays) and the FR-081 merge step. Needs normative text, not ADR-086's rejected-alternatives note. Cross-ref FR-084/ADR audit F-2.

### F8 — EXPLAIN counters absent; two spec-listed counters feature-gated (medium, specified-but-changed)
FR-081 requires counters via EXPLAIN + bench step (AC-5). Code: NOTICE behind `scan_profile_notice` only (`routine.rs:614-628`); `ExplainCustomScan: None` (`custom_scan.rs:53-67`); per-node batch sizes and pool reuse exist only behind `distann-head-attribution-benchmark`. Cross-ref FR-075 audit F1, NFR audit F7.

### F9 — Four RPCs bypass the deadline/interrupt wrapper (medium, specified-but-changed)
FR-081: every remote call has a nonzero client-side deadline + remote statement_timeout + interrupt checks. Head-shard search (`remote_transport.rs:821-853`), gateway-routing export (:924-958), head-shard export/import (:961-1054) issue bare awaits — no client timeout, no interrupt-select, no cancel token. Expansion/materialization/lifecycle conform via `await_remote`. A stalled owner during head search blocks the backend beyond budget.

### F10 — Transport hardwired NoTls; NFR-014 posture deferred (medium, specified-but-changed)
`remote_transport.rs:1-14,28` — module doc openly defers TLS/secret handling; FR-079 incorporates NFR-014 by reference as a SHALL. Pooling and prepared-statement batching conform. Spec should carry the loopback-substrate caveat or track the gap.

### F11 — Third heap_tid lane the spec taxonomy does not admit (low, shipped-but-unspecified)
`distributed_control=false` + multi-node session roster (the loopback fixture): every "owner" resolves heap_tid against a shared live base table (`routine.rs:444-516`, `expand.rs:176-192`). Reachable in production builds via roster GUCs; neither of the spec's two permitted lanes. Name it or forbid it.

### F12 — Sharded head search / replicas: shipped default, normatively undocumented (medium, shipped-but-unspecified)
4 endpoints, 3 GUCs, attestation table, per-owner bounded seed-merge protocol (`generation_read.rs:1972-2278`, `options.rs:380-405`, `remote_transport.rs:744-853`). NFR-021 clause 3 mandates the property; no functional spec defines the mechanism. Cross-ref FR-080 audit F6.

### F13 — Task 205 pushdown semantics verified conformant (verification note)
Threshold from L-th live retained candidate, `candidate_limit = L` verbatim, owner applies threshold per-row + L once across the merged batch, gateway path re-applies batch-L (`generation_read.rs:4203`) with owner-only-equivalence unit test (`gateway_copy.rs:309-367`). Checked, not assumed.

### F14 — Lazy-10 window policy verified conformant (verification note)
`PRODUCTION_MATERIALIZATION_BATCH_SIZE = 10` (`options.rs:96`), GUC override feature-gated (:593-601), proven-prefix cap, no re-request after deepening (`custom_scan.rs:1085-1190`). Nuance: partially consumed windows request `[output_index, aligned_end)` — still within contract.

### F15 — FR-079-AC-1 positional reassembly verified conformant (verification note)
Enforced at every layer: owner request-order rows, coordinator positional zip + count/order errors, multi-owner scatter into request-order slots with unfilled-slot check (`scan.rs:473-479`, `generation_read.rs:3898-3910,4084-4227`, `remote_endpoint.rs:349-423`); `gateway_copy.rs:96-107` keeps split requests order-stable.

## Slice behaviors in no distann spec (summary)
Gateway copies (F7); query-digest caching + `ec_distann_physical_seed_id_digest` (F1); sharded head search/replicas as shipped default (F12); auxiliary endpoints (F6); legacy loopback multi-node roster lane (F11). `scan_registry.rs` is NOT unspecified — it implements FR-082's fence/token text. Debug/test SQL functions are all feature-gated, satisfying FR-079's production-build rule; the fail-closed privilege DO-block (`lib.rs:874-921`) matches the spec including its three named public exceptions.
