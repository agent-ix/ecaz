---
task: 190
packet: 001-entry-and-residual-case
role: coder
date: 2026-07-23
status: review_requested
---

# Review request: architecture entry and residual case

Task 190 is activated as a latency-only decision after Task 194 completed the
missing nine-way attribution and Tasks 195--197 established the retained
production point and benchmark integrity.

The accepted evidence isolates ten sequential traversal rounds and
4.078--5.013 ms/scan of transport wait, against a retained 100k release point
of 0.9625 recall / 19.90 ms warm mean. Encode/decode/connection work is only
0.071 ms and logical traffic is small. BW8/H50 reduced rounds and wait but did
not move end-to-end mean usefully and regressed p95.

That establishes a material architecture-addressable latency residual. It does
not close or supersede the independent Tasks 185/186/188/189 recall branches.
No 1m run was triggered because Task 194's 100k candidate failed its
pre-registered usefulness gate.

Please review whether the immutable evidence and scope boundary are sufficient
to enter Task 190's two-family comparison.
