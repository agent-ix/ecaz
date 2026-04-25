//! ec_ivf page layout: metadata page now, posting-list pages later.

use std::ptr;

use pgrx::pg_sys;

use super::options::{EcIvfOptions, RerankMode, StorageFormat};
use super::P_NEW;
use crate::storage::wal;

pub(super) const METADATA_BLOCK_NUMBER: pg_sys::BlockNumber = 0;
pub(super) const FIRST_DATA_BLOCK_NUMBER: pg_sys::BlockNumber = 1;
pub(super) const INDEX_FORMAT_VERSION: u16 = 1;

const METADATA_MAGIC: u32 = 0x5649_4345; // "ECIV" as little-endian bytes.
const METADATA_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MetadataPage {
    pub(super) format_version: u16,
    pub(super) nlists: u32,
    pub(super) nprobe: u32,
    pub(super) training_sample_rows: u32,
    pub(super) seed: u64,
    pub(super) storage_format: StorageFormat,
    pub(super) rerank: RerankMode,
}

impl MetadataPage {
    pub(super) fn empty(options: EcIvfOptions) -> Self {
        Self {
            format_version: INDEX_FORMAT_VERSION,
            nlists: u32::try_from(options.nlists).expect("validated nlists should fit in u32"),
            nprobe: u32::try_from(options.nprobe).expect("validated nprobe should fit in u32"),
            training_sample_rows: u32::try_from(options.training_sample_rows)
                .expect("validated training_sample_rows should fit in u32"),
            seed: u64::try_from(options.seed).expect("validated seed should fit in u64"),
            storage_format: options.storage_format,
            rerank: options.rerank,
        }
    }

    pub(super) fn encode(&self) -> [u8; METADATA_BYTES] {
        let mut out = [0_u8; METADATA_BYTES];
        out[0..4].copy_from_slice(&METADATA_MAGIC.to_le_bytes());
        out[4..6].copy_from_slice(&self.format_version.to_le_bytes());
        out[8..12].copy_from_slice(&self.nlists.to_le_bytes());
        out[12..16].copy_from_slice(&self.nprobe.to_le_bytes());
        out[16..20].copy_from_slice(&self.training_sample_rows.to_le_bytes());
        out[24..32].copy_from_slice(&self.seed.to_le_bytes());
        out[32] = self.storage_format as u8;
        out[33] = self.rerank as u8;
        out
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < METADATA_BYTES {
            return Err(format!(
                "ec_ivf metadata length mismatch: got {}, expected at least {METADATA_BYTES}",
                bytes.len()
            ));
        }
        let magic = u32::from_le_bytes(
            bytes[0..4]
                .try_into()
                .expect("metadata magic slice should be 4 bytes"),
        );
        if magic != METADATA_MAGIC {
            return Err(format!("invalid ec_ivf metadata magic: {magic:#x}"));
        }
        let format_version = u16::from_le_bytes(
            bytes[4..6]
                .try_into()
                .expect("metadata format slice should be 2 bytes"),
        );
        if format_version != INDEX_FORMAT_VERSION {
            return Err(format!(
                "unsupported ec_ivf metadata format version: {format_version}"
            ));
        }
        Ok(Self {
            format_version,
            nlists: u32::from_le_bytes(
                bytes[8..12]
                    .try_into()
                    .expect("metadata nlists slice should be 4 bytes"),
            ),
            nprobe: u32::from_le_bytes(
                bytes[12..16]
                    .try_into()
                    .expect("metadata nprobe slice should be 4 bytes"),
            ),
            training_sample_rows: u32::from_le_bytes(
                bytes[16..20]
                    .try_into()
                    .expect("metadata training sample slice should be 4 bytes"),
            ),
            seed: u64::from_le_bytes(
                bytes[24..32]
                    .try_into()
                    .expect("metadata seed slice should be 8 bytes"),
            ),
            storage_format: decode_storage_format(bytes[32])?,
            rerank: decode_rerank(bytes[33])?,
        })
    }
}

fn decode_storage_format(value: u8) -> Result<StorageFormat, String> {
    match value {
        value if value == StorageFormat::Auto as u8 => Ok(StorageFormat::Auto),
        value if value == StorageFormat::TurboQuant as u8 => Ok(StorageFormat::TurboQuant),
        value if value == StorageFormat::PqFastScan as u8 => Ok(StorageFormat::PqFastScan),
        value if value == StorageFormat::RaBitQ as u8 => Ok(StorageFormat::RaBitQ),
        other => Err(format!("invalid ec_ivf storage format code: {other}")),
    }
}

fn decode_rerank(value: u8) -> Result<RerankMode, String> {
    match value {
        value if value == RerankMode::Auto as u8 => Ok(RerankMode::Auto),
        value if value == RerankMode::Off as u8 => Ok(RerankMode::Off),
        value if value == RerankMode::HeapF32 as u8 => Ok(RerankMode::HeapF32),
        value if value == RerankMode::SourceColumn as u8 => Ok(RerankMode::SourceColumn),
        other => Err(format!("invalid ec_ivf rerank code: {other}")),
    }
}

pub(super) unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: MetadataPage,
) {
    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        METADATA_BLOCK_NUMBER
    };
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("ec_ivf failed to allocate metadata buffer");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

pub(super) unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> MetadataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("ec_ivf failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
    let metadata_ptr = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let metadata_bytes = unsafe { std::slice::from_raw_parts(metadata_ptr, METADATA_BYTES) };
    let metadata = MetadataPage::decode(metadata_bytes).unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    metadata
}

#[cfg(test)]
mod tests {
    use super::{MetadataPage, RerankMode, StorageFormat, INDEX_FORMAT_VERSION};
    use crate::am::ec_ivf::options::EcIvfOptions;

    #[test]
    fn metadata_roundtrip() {
        let metadata = MetadataPage::empty(EcIvfOptions {
            nlists: 128,
            nprobe: 8,
            training_sample_rows: 10_000,
            seed: 7,
            storage_format: StorageFormat::RaBitQ,
            rerank: RerankMode::HeapF32,
        });

        let decoded = MetadataPage::decode(&metadata.encode()).unwrap();

        assert_eq!(decoded, metadata);
        assert_eq!(decoded.format_version, INDEX_FORMAT_VERSION);
    }
}
