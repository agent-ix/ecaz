---
id: ADR-001
title: "turbo-quant crate lacks code-to-code inner product"
status: SUPERSEDED
superseded_by: ADR-006
impact: HIGH for FR-005, FR-008, FR-009 (HNSW AM)
date: 2026-04-03
---
# ADR-001: turbo-quant crate lacks code-to-code inner product

**SUPERSEDED by ADR-006.** The turbo-quant crate was dropped entirely in favor of an own quantizer implementation. The own implementation includes both LUT-based scoring (`score_ip_encoded`) and code-to-code scoring (`score_ip_encoded_lite`). See FR-005, FR-017, and FR-015 for the scoring architecture.

## Original Context

The `turbo-quant` crate (v0.1) only exposed asymmetric scoring (`code × f32`), not symmetric code-to-code scoring. This was identified as a risk for HNSW graph traversal.

## Resolution

ADR-006 decided to drop the turbo-quant crate and implement the quantizer core directly. The own implementation includes `score_ip_encoded_lite` for code-to-code scoring, eliminating this concern entirely.
