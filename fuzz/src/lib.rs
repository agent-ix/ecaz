#[path = "../../src/storage/page.rs"]
pub mod storage_page;

pub mod storage {
    pub mod page {
        pub use crate::storage_page::*;
    }
}

#[path = "../../src/am/ec_hnsw/page.rs"]
pub mod hnsw_page;

pub mod am {
    pub mod page {
        pub use crate::hnsw_page::*;
    }
}

#[path = "../../src/am/ec_diskann/page.rs"]
pub mod diskann_page;

#[allow(dead_code)]
#[path = "../../src/quant/mod.rs"]
pub mod quant;

use quant::prod::payload_len;

pub const HEADER_BYTES: usize = 2;
pub const MIN_BINARY_BYTES: usize = HEADER_BYTES + 4;
pub(crate) const DEFAULT_QUANT_BITS: u8 = 4;
pub(crate) const DEFAULT_QUANT_SEED: u64 = 42;

fn validate_bits(bits: u8) -> Result<(), String> {
    if !(2..=8).contains(&bits) {
        return Err(format!("bits must be between 2 and 8, got {bits}"));
    }
    Ok(())
}

fn validate_tqvector_seed(seed: u64) -> Result<(), String> {
    if seed != DEFAULT_QUANT_SEED {
        return Err(format!(
            "tqvector seed must use the canonical default ({DEFAULT_QUANT_SEED}), got {seed}"
        ));
    }
    Ok(())
}

fn validate_tqvector_bits(bits: u8) -> Result<(), String> {
    if bits != DEFAULT_QUANT_BITS {
        return Err(format!(
            "tqvector currently stores {DEFAULT_QUANT_BITS}-bit codes, got {bits}"
        ));
    }
    Ok(())
}

pub fn parse_text(s: &str) -> Result<(u16, u8, u64, f32, Vec<u8>), String> {
    let s = s.trim();
    if !s.starts_with('[') {
        return Err("missing '['".into());
    }
    let bracket_end = s.find(']').ok_or("missing ']'")?;
    let header = &s[1..bracket_end];
    let rest = s[bracket_end + 1..]
        .strip_prefix(':')
        .ok_or("missing ':' after header")?;

    let mut dim: Option<u16> = None;
    let mut bits: Option<u8> = None;
    let mut seed: u64 = DEFAULT_QUANT_SEED;
    let mut gamma: f32 = 0.0;

    for part in header.split(',') {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| format!("bad header field: {part}"))?;
        match key.trim() {
            "dim" => dim = Some(value.trim().parse().map_err(|e| format!("dim: {e}"))?),
            "bits" => bits = Some(value.trim().parse().map_err(|e| format!("bits: {e}"))?),
            "seed" => seed = value.trim().parse().map_err(|e| format!("seed: {e}"))?,
            "gamma" => gamma = value.trim().parse().map_err(|e| format!("gamma: {e}"))?,
            other => return Err(format!("unknown header field: {other}")),
        }
    }

    let dim = dim.ok_or("missing dim")?;
    let bits = bits.ok_or("missing bits")?;
    validate_bits(bits)?;
    validate_tqvector_bits(bits)?;
    validate_tqvector_seed(seed)?;

    let codes = hex::decode(rest).map_err(|e| format!("hex decode: {e}"))?;
    let expected = payload_len(dim as usize, bits) - 4;
    if codes.len() != expected {
        return Err(format!(
            "code length mismatch: got {} bytes, expected {expected}",
            codes.len()
        ));
    }

    Ok((dim, bits, seed, gamma, codes))
}

pub mod bench_api {
    pub use crate::diskann_page::{VamanaMetadataPage, INDEX_FORMAT_V3_DISKANN};
    pub use crate::hnsw_page::{
        CurrentFormatMetadata, MetadataPage, TqElementTuple, TqNeighborTuple,
    };
    pub use crate::parse_text;
    pub use crate::quant::prod::{unpack_mse_indices, ProdQuantizer};
    pub use crate::quant::Quantizer;
    pub use crate::storage::page::ItemPointer;
}
