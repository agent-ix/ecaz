# Task 85 Review Request: Reopen Comprehensive Plan

## Request

Review the Task 85 scope correction.

Packet 007 correctly rejected the measured Task 85 options under the
same-recall latency bar, but it incorrectly closed the task instead of turning
the identified next research directions into first-class Task 85 work. That
was a process and scope error: the user asked for a comprehensive SPIRE
optimization plan, not a sequence of small rejected slices followed by a vague
future note.

## Correction

`plan/tasks/85-spire-product-scale-pareto-program.md` now reopens Task 85 and
adds a comprehensive optimization program covering:

- object-read and physical layout work;
- summary-scoring CPU work;
- candidate-set-preserving rerank locality;
- candidate-surface redesign only when recall is preserved;
- benchmark harness and evidence extensions;
- comparator and final product/default policy gates.

The previous closeout packet is now explicitly documented as premature. It
remains useful negative evidence, but it is not the Task 85 final closeout.

## Guardrail

The acceptance bar is unchanged and explicit:

- latency wins must retain or improve the Task 79/81 recall level;
- lower recall, lower rerank width, or hidden candidate growth does not count;
- same-suite AWS 1M/q500 controls are required for product claims;
- comparator context remains part of the final decision.

## Validation

No tests or benchmark suites were run. This packet is a planning/process
correction only.
