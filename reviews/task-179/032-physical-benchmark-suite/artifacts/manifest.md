# Artifact manifest

- Head SHA: `4570ac008fe17770d6779ec4ee6be4e2aee79da4`
- Task bucket: `reviews/task-179/`
- Packet: `032-physical-benchmark-suite`
- Lane / fixture: local PG18 CLI surface validation; the quoted 10k probe is explicitly non-promoted development output
- Storage format / rerank mode: ec_distann physical benchmark surface / standard rerank
- Timestamp: `2026-07-12T05:47:06-07:00`
- Isolation: CLI tests were isolated; the development probe compared physical and single-instance surfaces but is not accepted benchmark evidence

## `validation.log`

- SHA-256: `2687d79bcde9ece2d69235ef1f5f97792bd01f0966c37c6d26bbef5208c08899`
- Command: `cargo check -p ecaz-cli`
- Result: PASS (one pre-existing `dead_code` warning)
- Command: `cargo test -p ecaz-cli distann_physical_topology_and_gate_are_structured -- --nocapture`
- Result: PASS (`1 passed; 0 failed; 427 filtered out`)
- Development-probe context: 10k, 10 queries, top-k 10, sweep 32
- Cited probe result: physical and single recall@10 `1.0000`; physical mean `10727.91 ms`; single mean `1043.92 ms`

## Accounting disposition

The 10k development probe predates decision-grade NFR-018 accounting. It did
not emit an explicit aggregate `control_index_bytes` field or heap-versus-TOAST
breakdown, so those numbers must not be inferred from this packet. The gap is
retained here explicitly; the closeout benchmark packet must recapture those
fields through `ecaz bench suite` before Task 179 is closed.
