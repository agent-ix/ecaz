## Feedback: ADR-030 v2 Scan Storage Descriptor Seam

Read `scan_graph_storage: graph::GraphStorageDescriptor` on the scan state and the
use of `with_graph_storage_tuple` across `scan.rs`.

### What's right

- Carrying `GraphStorageDescriptor` through scan state (not deriving it at each tuple
  decode) means format-awareness is decided once at scan open and then applied
  consistently. No risk of one callsite treating the tuple as scalar while another
  treats it as grouped.
- The descriptor type carries layout (`code_len`, `binary_word_count`,
  `search_code_len`, `rerank_code_len`), so decode sites can compute slice offsets
  without reparsing metadata.

### Concerns

1. **`rerank_code_len` derives from `metadata.bits`, not `search_bits`.** Already
   flagged on packet 312, but worth repeating here because this is the packet that
   hard-wires the derivation into the descriptor. If metadata's `bits` field is ever
   redefined to mean something other than "rerank code bits per subvector," this
   will silently desync.

2. **Tests assert grouped-scalar disjointness?** Packet 327 has tests that assert
   scalar cache entries have no grouped hot payloads and vice versa. Good. But the
   descriptor itself should also carry an invariant: `Scalar` and `GroupedV2` are
   mutually exclusive and exhaustive. If a future variant is added, an exhaustive
   match on `GraphStorageDescriptor` should force every decode site to handle it.
   Confirm this is enforced (no `_ =>` arms).

### Observation

Passing a typed descriptor through scan state instead of raw metadata fields is the
correct abstraction. It's what makes 327's cache format-awareness clean, and it's what
lets 328's `grouped_score_input()` return `None` on scalar entries without branching
on metadata at each callsite.
