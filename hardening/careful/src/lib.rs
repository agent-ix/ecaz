#[path = "../../../src/storage/page.rs"]
pub mod careful_storage_page;

#[path = "../../../src/am/ec_diskann/tuple.rs"]
pub mod careful_diskann_tuple;

#[path = "../../../src/am/ec_diskann/page.rs"]
pub mod careful_diskann_page;

#[path = "../../../src/am/ec_diskann/vacuum.rs"]
pub mod careful_diskann_vacuum;

#[path = "../../../src/am/ec_diskann/vamana.rs"]
pub mod careful_diskann_vamana;

#[path = "../../../src/am/ec_hnsw/search.rs"]
pub mod careful_hnsw_search;

#[path = "../../../src/am/ec_hnsw/page.rs"]
pub mod careful_hnsw_page;

#[path = "../../../src/am/ec_ivf/page.rs"]
pub mod careful_ivf_page;

#[path = "../../../src/am/common/cost.rs"]
pub mod careful_common_cost;

extern crate self as pgrx;

pub use crate::careful_pg_guards::pg_sys;

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        panic!($($arg)*)
    };
}

#[allow(dead_code, unused_imports)]
#[path = "pg_guards.rs"]
pub mod careful_pg_guards;

#[allow(dead_code, unused_imports)]
#[path = "spire.rs"]
pub mod careful_spire;

#[allow(dead_code)]
#[path = "../../../src/am/common/training.rs"]
pub mod careful_common_training;

#[path = "../../../src/am/ec_diskann/persist.rs"]
pub mod careful_diskann_persist;

#[path = "../../../src/am/ec_diskann/reader.rs"]
pub mod careful_diskann_reader;

#[path = "../../../src/am/ec_diskann/build.rs"]
pub mod careful_diskann_build;

#[path = "../../../src/am/ec_diskann/scan.rs"]
pub mod careful_diskann_scan;

pub mod storage {
    pub use crate::careful_pg_guards::buffer_guard;
    pub use crate::careful_pg_guards::wal;
    pub use crate::careful_storage_page as page;
}

pub mod am {
    pub(crate) mod page {
        pub(crate) const INDEX_FORMAT_V1_SCALAR: u16 = 1;
        pub(crate) const INDEX_FORMAT_V2_GROUPED: u16 = 2;
    }

    pub mod common {
        pub use crate::careful_common_cost as cost;
        pub use crate::careful_common_training as training;
    }

    pub mod ec_diskann {
        pub(crate) fn maybe_check_for_interrupts() {}

        pub use crate::careful_diskann_build as build;
        pub use crate::careful_diskann_page as page;
        pub use crate::careful_diskann_persist as persist;
        pub use crate::careful_diskann_reader as reader;
        pub use crate::careful_diskann_scan as scan;
        pub use crate::careful_diskann_tuple as tuple;
        pub use crate::careful_diskann_vacuum as vacuum;
        pub use crate::careful_diskann_vamana as vamana;
    }

    pub mod ec_hnsw {
        pub use crate::careful_hnsw_page as page;
        pub use crate::careful_hnsw_search as search;
    }

    pub mod ec_ivf {
        pub use crate::careful_ivf_page as page;
    }

    pub mod ec_spire {
        pub use crate::careful_spire::{assign, meta, page, storage};
    }
}

#[allow(dead_code)]
#[path = "../../../src/quant/mod.rs"]
mod quant;

#[cfg(test)]
mod tests {
    use super::am::ec_diskann::tuple::{VamanaCodebookTuple, VamanaNodeTuple};
    use super::am::ec_diskann::vacuum::repair_neighbors;
    use super::storage::page::{
        align_up, aligned_tuple_bytes, raw_tuple_storage_bytes, DataPage, DataPageChain,
        ItemPointer, FIRST_DATA_BLOCK_NUMBER, PAGE_HEADER_BYTES,
    };
    use std::collections::HashSet;

    fn tid(b: u32, o: u16) -> ItemPointer {
        ItemPointer {
            block_number: b,
            offset_number: o,
        }
    }

    #[test]
    fn item_pointer_decode_rejects_short_payloads() {
        assert!(ItemPointer::decode(&[0; 5]).is_err());
    }

    #[test]
    fn item_pointer_decode_rejects_long_payloads() {
        assert!(ItemPointer::decode(&[0; 7]).is_err());
    }

    #[test]
    fn page_alignment_helpers_preserve_exact_and_round_up_cases() {
        assert_eq!(align_up(16, 8), 16);
        assert_eq!(align_up(17, 8), 24);
        assert_eq!(aligned_tuple_bytes(8), 16);
        assert_eq!(aligned_tuple_bytes(9), 24);
    }

    #[test]
    fn data_page_reports_layout_and_free_space() {
        let mut page = DataPage::new(9, 96);

        assert_eq!(page.block_number(), 9);
        assert_eq!(page.tuple_count(), 0);
        assert_eq!(page.free_bytes(), 96 - PAGE_HEADER_BYTES);
        assert!(page.can_fit_raw_tuple(16));

        let inserted = page.insert_raw_tuple(vec![0xaa; 16]).unwrap();
        assert_eq!(inserted, tid(9, 1));
        assert_eq!(page.tuples(), &[vec![0xaa; 16]]);
        assert_eq!(page.free_bytes(), 96 - PAGE_HEADER_BYTES - raw_tuple_storage_bytes(16));
    }

    #[test]
    fn data_page_rejects_invalid_tid_lookups_and_updates() {
        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, 128);
        let inserted = page.insert_raw_tuple(vec![1, 2, 3, 4]).unwrap();

        assert!(page.raw_tuple(tid(FIRST_DATA_BLOCK_NUMBER + 1, 1)).is_err());
        assert!(page.raw_tuple(tid(FIRST_DATA_BLOCK_NUMBER, 0)).is_err());
        assert!(page.raw_tuple(tid(FIRST_DATA_BLOCK_NUMBER, 2)).is_err());

        assert!(page
            .update_raw_tuple(tid(FIRST_DATA_BLOCK_NUMBER + 1, 1), vec![9; 4])
            .is_err());
        assert!(page
            .update_raw_tuple(tid(FIRST_DATA_BLOCK_NUMBER, 0), vec![9; 4])
            .is_err());
        assert!(page
            .update_raw_tuple(tid(FIRST_DATA_BLOCK_NUMBER, 2), vec![9; 4])
            .is_err());
        assert!(page.update_raw_tuple(inserted, vec![9; 3]).is_err());

        page.update_raw_tuple(inserted, vec![9; 4]).unwrap();
        assert_eq!(page.raw_tuple(inserted).unwrap(), &[9; 4]);
    }

    #[test]
    fn data_page_rejects_tuple_that_does_not_fit() {
        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, 48);
        assert!(page.insert_raw_tuple(vec![0; 24]).is_err());
    }

    #[test]
    fn page_chain_preserves_payloads_across_overflow() {
        let mut chain = DataPageChain::new(128);
        let first = chain.insert_raw_tuple(vec![1; 32]).unwrap();
        let second = chain.insert_raw_tuple(vec![2; 32]).unwrap();
        let third = chain.insert_raw_tuple(vec![3; 32]).unwrap();

        assert_eq!(first.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(second.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(third.block_number, FIRST_DATA_BLOCK_NUMBER + 1);
        assert_eq!(
            chain
                .get_page(third.block_number)
                .unwrap()
                .raw_tuple(third)
                .unwrap(),
            &[3; 32]
        );
    }

    #[test]
    fn data_page_chain_reports_size_and_mutates_existing_page() {
        let mut chain = DataPageChain::new(128);
        let tid = chain.insert_raw_tuple(vec![1; 16]).unwrap();

        assert_eq!(chain.page_size(), 128);
        assert!(chain.get_page(0).is_none());
        assert!(chain.get_page(FIRST_DATA_BLOCK_NUMBER + 1).is_none());

        let page = chain.get_page_mut(tid.block_number).unwrap();
        page.update_raw_tuple(tid, vec![2; 16]).unwrap();

        assert_eq!(
            chain
                .get_page(tid.block_number)
                .unwrap()
                .raw_tuple(tid)
                .unwrap(),
            &[2; 16]
        );
    }

    #[test]
    fn data_page_chain_zero_empty_pages_is_noop() {
        let mut chain = DataPageChain::new(128);

        assert_eq!(chain.append_empty_pages(0), None);
        assert_eq!(chain.pages().len(), 1);
    }

    #[test]
    fn data_page_chain_rejects_tuple_larger_than_page_payload_capacity() {
        let mut chain = DataPageChain::new(64);

        assert!(chain.insert_raw_tuple(vec![0; 40]).is_err());
        assert_eq!(chain.pages().len(), 1);
        assert_eq!(chain.pages()[0].tuple_count(), 0);
    }

    #[test]
    fn diskann_tuple_codebook_roundtrip() {
        let tuple = VamanaCodebookTuple {
            group_index: 3,
            nexttid: tid(42, 7),
            centroids: (0..64).map(|i| i as f32 * 0.25).collect(),
        };
        let encoded = tuple.encode();
        assert_eq!(encoded.len(), VamanaCodebookTuple::encoded_len(64));
        let decoded = VamanaCodebookTuple::decode(&encoded, 64).unwrap();
        assert_eq!(tuple, decoded);
    }

    #[test]
    fn diskann_vacuum_repair_neighbors_compacts_and_pads() {
        let mut tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        tuple.primary_heaptid = tid(100, 1);
        tuple.neighbor_count = 4;
        tuple.neighbors[0] = tid(1, 1);
        tuple.neighbors[1] = tid(2, 2);
        tuple.neighbors[2] = tid(3, 3);
        tuple.neighbors[3] = tid(4, 4);

        let dead = HashSet::from([tid(2, 2), tid(4, 4)]);
        assert_eq!(repair_neighbors(&mut tuple, &dead), 2);
        assert_eq!(tuple.neighbor_count, 2);
        assert_eq!(&tuple.neighbors[..2], &[tid(1, 1), tid(3, 3)]);
        assert_eq!(&tuple.neighbors[2..], &[ItemPointer::INVALID; 2]);
    }

    #[test]
    fn quant_family_default_and_names_are_stable() {
        assert_eq!(super::quant::Family::DEFAULT, super::quant::Family::TurboQuant);
        assert_eq!(super::quant::Family::TurboQuant.as_str(), "turboquant");
        assert_eq!(super::quant::Family::PqFastScan.as_str(), "pq_fastscan");
    }

    #[test]
    fn quant_family_reloption_parser_accepts_known_values() {
        assert_eq!(
            super::quant::Family::parse_reloption("turboquant").unwrap(),
            super::quant::Family::TurboQuant
        );
        assert_eq!(
            super::quant::Family::parse_reloption("pq_fastscan").unwrap(),
            super::quant::Family::PqFastScan
        );
    }

    #[test]
    fn quant_family_reloption_parser_rejects_unknown_values() {
        let error = super::quant::Family::parse_reloption("product_quant").unwrap_err();
        assert!(error.contains("invalid ec_hnsw storage_format reloption"));
        assert!(error.contains("product_quant"));
    }

    #[test]
    fn quant_simd_backend_name_is_reported() {
        assert!(!super::quant::simd_backend_name().is_empty());
    }
}
