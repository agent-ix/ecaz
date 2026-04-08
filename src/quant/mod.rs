//! TurboQuant two-stage vector quantization.
//!
//! Extracted from TurboQuantDB and adapted for the pgrx extension context:
//! - `ndarray::Array1<f64>` replaced with `&[f32]` / `Vec<f32>`
//! - rayon removed (Postgres backends are single-threaded per query)
//! - AVX2+FMA SIMD retained, aarch64 NEON added
//!
//! ## Pipeline
//!
//! 1. [`rotation`] + [`mse`] — SRHT rotation + Lloyd-Max scalar codebook (b-1 bits/dim)
//! 2. [`qjl`] — residual projection, 1-bit quantized and bit-packed
//! 3. [`prod::ProdQuantizer`] — orchestrates both stages and exposes encode/decode/score APIs
//!
//! ## Storage format
//!
//! Packed code = `[mse_packed][qjl_packed]`
//! - MSE: `ceil(dim * (bits-1) / 8)` bytes
//! - QJL: `ceil(dim / 8)` bytes
//! - Total at 1536-dim, 4-bit: 576 + 192 = 768 bytes
//!
//! ## Scoring
//!
//! Query preparation (`prepare_ip_query`) computes a lookup table once.
//! Each candidate is scored with `score_ip_encoded` — O(n) with zero allocation,
//! AVX2+FMA or NEON accelerated.

pub mod codebook;
pub mod hadamard;
pub mod mse;
pub mod prod;
pub mod qjl;
pub mod rotation;
mod simd;

/// Index into the MSE codebook. Max 2^16 = 65536 centroids (bits <= 16).
pub type CodeIndex = u16;
