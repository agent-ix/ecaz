# Packet 060 complete finding disposition

This table is the durable disposition for every P2 and P3 item recorded by
packet 060. It does not self-close any review request; the outside reviewer is
asked to verify the listed implementation and evidence packets.

## P2 findings

| Finding | Disposition | Durable checkpoint |
| --- | --- | --- |
| P2-1 transaction-fence `RefCell` borrow | Fixed: the borrow ends before PostgreSQL lock release and shared-registry cleanup. | `7bb215458`, packet 061 |
| P2-2 backend termination polling | Fixed: `ProcDiePending` is a prompt transport stop condition. | `7bb215458`, packet 061 |
| P2-3 portable interrupt globals | Fixed: production uses volatile reads of pgrx-bound PostgreSQL globals; repeated glibc-only `dlsym` was removed. | `7bb215458`, packet 061 |
| P2-4 CustomScan Rust-state leak on ERROR | Fixed: per-query memory-context callback drops Rust state exactly once on normal and abort cleanup. | `7bb215458`, packet 061 |
| P2-5 legacy materialize directory rebuild | Fixed despite the lane's deprecated status: the endpoint uses the cached directory entry. | `0b2d4fbab`, packet 062 |
| P2-6 owner fixed costs and search shape | Fixed/measured: retained scan and quantizer caches, prepared pooled statements, concurrent cold connects, persisted head reuse, shared descriptors, session query cache, and heap frontier landed. The default A/B, BW16/H25 fixed-product arm, and outside-roster arm all ran at 10k/50k/100k. | `4587c0d09`, `a7fa64895`, packets 063, 065, 066 |
| P2-7 per-entry prepared handoff inserts | Fixed: graph payloads are inserted as one checked `unnest` batch. | `0b2d4fbab`, packet 062 |
| P2-8 string-typed lifecycle and coordinator size | Fixed: typed state authorities and legal transitions plus T1/T2/T3/T4a/cancel phase modules. | `0043c3e74`, packet 064 |
| P2-9 registry concurrency and flat test file | Fixed: real two-backend preload contention test; lifecycle and registry tests split from the basic include. | `0043c3e74`, packet 064 |
| P2-10 control-index/heap-vs-TOAST evidence gap | Recorded without reconstruction: packet 032 now has a manifest that states the historical gap. Current suite evidence records `control_index_bytes`; heap-vs-TOAST is not inferred from aggregate storage. | `fc2237548`, packets 032, 064, 066 |

## P3 code and test findings

| Finding | Disposition | Durable checkpoint |
| --- | --- | --- |
| Unchecked Registered-to-Ready transition | Fixed with `RETURNING 1`, typed transition validation, and exactly-one-row enforcement. | `0b2d4fbab`, `0043c3e74`, packets 062, 064 |
| Missing `EC_DISTANN_CONTROL_SCAN` coverage | Fixed with a focused raw AM backstop test. | `4587c0d09`, packet 063 |
| Duplicated identifiers/digests/NULL bitmap/handoff shape/pool setup | Consolidated behind shared authorities. | `4587c0d09`, packet 063 |
| Swallowed SPI detail | Fixed at the named coordinator sites; underlying PostgreSQL detail is retained. | `0b2d4fbab`, `0043c3e74`, packets 062, 064 |
| Sequential cold connects | Fixed with concurrent pool establishment and identity setup. | `4587c0d09`, packet 063 |
| Per-backend head rebuild | Fixed by persisting and validating the bounded head graph. | `4587c0d09`, packet 063 |
| Descriptor clone, repeated `dlsym`, query reserialization, sorted-Vec beam | Fixed respectively with `Arc`, direct globals, session query caching, and a deterministic binary heap. | `7bb215458`, `4587c0d09`, packets 061, 063 |

## P3 evidence and hygiene findings

- **Packet 036 isolation:** packet 068 runs exact parent
  `9a0f21f0824c675d06e9e87747eb36a70859611f` versus transport checkpoint
  `ceb15f73ac69fcd98896457c9578fadae2ff0c09` at 10k/50k/100k with recall,
  latency, storage, and topology through `ecaz bench suite`. Final measured
  disposition: recall is unchanged, storage is unchanged apart from a 32 KiB
  10k page-allocation difference, warm p95 is 4.1-8.3% lower, and the broader
  recall-workload mean is mixed (-21.3%, +5.5%, +14.7%). Packet 068 does not
  turn that single-run evidence into a speedup or neutrality claim.
- **Packet 046 isolation:** no separate performance matrix is claimed. That
  checkpoint only rejects unsupported system-column projections before scan;
  it does not alter any supported query's traversal, scoring, posting,
  quantization, rerank, or storage behavior. An A/B on supported queries would
  exercise identical production code and would not demonstrate the new
  fail-closed branch. Its correctness coverage is the relevant evidence.
- **No 1m run:** packet 066 states the reason: the host's staged-current corpus
  contains 10k/50k/100k inputs but no staged 1m corpus. The required minimum
  matrix is complete; no 1m result is fabricated or inferred.
- **Outside-roster scale:** packet 066 measures all three remote owners at
  10k/50k/100k with topology, recall, latency, storage, and control bytes.
- **Packets 039-058:** all twenty requests still lack packet-local outside
  feedback. Packet 069 explicitly asks the reviewer to write a decision in
  each owning packet's `feedback/` directory; aggregate traceability alone is
  not treated as per-packet signoff.
- **Packet 032 manifest:** added by `fc2237548`; it records the historical
  artifact limitations instead of reconstructing missing columns.
- **Side branches:** retained as integrated historical worktrees, not active
  alternatives. `task-179-scan-registry` checkpoint `474382e93` is integrated
  by `eccc880de`; `task-179-participant-lifecycle` checkpoint `4aed07da2` by
  `907150c03`; and `task-179-publication-codecs` checkpoint `d47286759` by the
  main-lane codec checkpoint `c22c29ce8`, whose current content is a superset.
  They are deliberately annotated rather than destructively deleted.
- **Stale module header:** updated with the current physical-shard scope in
  `4587c0d09`.
- **`#[allow]` inventory:** the reviewer found the inventory benign. The
  formerly unexplained `options.rs` dead-code allowance now has a reasoned
  comment in `4587c0d09`.
