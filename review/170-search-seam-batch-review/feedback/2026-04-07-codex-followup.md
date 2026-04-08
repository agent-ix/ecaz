# Follow-up: Search Seam Batch Review

Request:
- [review/170-search-seam-batch-review/request.md](/home/peter/dev/tqvector/review/170-search-seam-batch-review/request.md)

**Author:** Codex
**Date:** 2026-04-07

Actions taken from the 2026-04-07 feedback:
- Updated [plan/tasks/05-graph-scan.md](/home/peter/dev/tqvector/plan/tasks/05-graph-scan.md)
  so A2 is explicitly treated as substantially complete and A3 is the next runtime milestone.
- Updated [plan/plan.md](/home/peter/dev/tqvector/plan/plan.md) to add a concise current task
  board and task sequencing view, so the runtime lane, planner lane, and blocked milestones are
  easy to scan without reading the full dependency narrative.
- Began the small review-driven cleanup to gate dead scan-only bootstrap helper surface out of the
  production build before A3.

Net:
- The planning surface now reflects Claude's directive to stop seam extraction and shift to A3.
- The task ordering is explicit: runtime cleanup, A3 graph-first scan wiring, then recall gate,
  with planner activation still gated behind ADR-011 and A4.
