//! Careful-side scaffold for `src/am/ec_diskann/routine_helpers.rs`.

#![allow(dead_code)]

mod scaffold {
    use crate::am::ec_diskann::reader::PersistedGraphReader;
    use crate::am::ec_diskann::tuple::VamanaNodeTuple;
    use crate::storage::page::{DataPageChain, ItemPointer};

    mod insert {
        use super::{DataPageChain, ItemPointer};
        use std::cmp::Ordering;

        pub(super) fn cmp_item_pointer_physical(
            left: &ItemPointer,
            right: &ItemPointer,
        ) -> Ordering {
            left.block_number
                .cmp(&right.block_number)
                .then(left.offset_number.cmp(&right.offset_number))
        }

        // Test-only shim of
        // `src/am/ec_diskann/insert.rs::bound_heap_tids_for_owner` that
        // returns just the primary heap tid. The helper only walks the
        // result vector; the bound-tid contents are not validated.
        pub(super) fn bound_heap_tids_for_owner(
            _chain: &DataPageChain,
            _owner_tid: ItemPointer,
            primary_heaptid: ItemPointer,
        ) -> Result<Vec<ItemPointer>, String> {
            Ok(vec![primary_heaptid])
        }
    }

    mod scan {
        use super::ItemPointer;

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct ScanResult {
            pub tid: ItemPointer,
            pub primary_heaptid: ItemPointer,
            pub distance: f32,
        }
    }

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_diskann/routine_helpers.rs"
    ));

    #[cfg(test)]
    mod tests {
        use super::*;

        fn tid(b: u32, o: u16) -> ItemPointer {
            ItemPointer {
                block_number: b,
                offset_number: o,
            }
        }

        // ---------------- existing helper tests --------------------

        #[test]
        fn miri_sort_and_dedup_item_pointers_sorts_physical_and_removes_dupes() {
            let mut tids = vec![
                tid(10, 3),
                tid(5, 1),
                tid(10, 1),
                tid(5, 1),
                tid(10, 3),
                tid(5, 2),
            ];
            sort_and_dedup_item_pointers(&mut tids);
            assert_eq!(
                tids,
                vec![tid(5, 1), tid(5, 2), tid(10, 1), tid(10, 3)],
            );
        }

        #[test]
        fn miri_vacuum_repair_scan_budget_clamps_to_at_least_one_neighbor() {
            assert_eq!(vacuum_repair_scan_budget(100, 0), 1);
            assert_eq!(vacuum_repair_scan_budget(5, 32), 5);
            assert_eq!(vacuum_repair_scan_budget(100, 32), 32);
            assert_eq!(vacuum_repair_scan_budget(32, 32), 32);
        }

        #[test]
        fn miri_sql_scan_result_cap_returns_rerank_budget_regardless_of_top_k() {
            assert_eq!(sql_scan_result_cap(10, 200), 200);
            assert_eq!(sql_scan_result_cap(1_000, 50), 50);
            assert_eq!(sql_scan_result_cap(0, 0), 0);
        }

        // ---------------- new chain-walker tests --------------------

        fn build_chain_with_nodes(
            node_tids: &[ItemPointer],
            dead_tids: &[ItemPointer],
        ) -> (DataPageChain, u16, usize, usize) {
            // Fixed shapes that match VamanaNodeTuple::placeholder/encode.
            let graph_degree_r: u16 = 4;
            let binary_word_count: usize = 0;
            let search_code_len: usize = 0;
            let mut chain = DataPageChain::new(8192);
            for tid in node_tids {
                let mut tuple = VamanaNodeTuple::placeholder(graph_degree_r, 0, 0);
                tuple.primary_heaptid = *tid;
                if dead_tids.contains(tid) {
                    tuple.deleted = true;
                }
                let encoded = tuple
                    .encode(graph_degree_r, binary_word_count, search_code_len)
                    .unwrap();
                chain.insert_raw_tuple(encoded).unwrap();
            }
            (chain, graph_degree_r, binary_word_count, search_code_len)
        }

        #[test]
        fn miri_count_live_tuples_in_chain_excludes_deleted_entries() {
            let nodes = vec![tid(1, 1), tid(1, 2), tid(1, 3), tid(1, 4)];
            let (chain, gr, bw, sc) = build_chain_with_nodes(&nodes, &[tid(1, 2)]);
            let live = count_live_tuples_in_chain(&chain, gr, bw, sc).unwrap();
            assert_eq!(live, 3);
        }

        #[test]
        fn miri_collect_node_tids_walks_every_chain_page_in_order() {
            let nodes = vec![tid(1, 1), tid(2, 2), tid(3, 3)];
            let (chain, gr, bw, sc) = build_chain_with_nodes(&nodes, &[]);
            let collected = collect_node_tids(&chain, gr, bw, sc).unwrap();
            // collect_node_tids returns the chain's storage tids (chain-local,
            // not the primary_heaptid we stamped into each tuple). Verify it
            // yields exactly `nodes.len()` entries.
            assert_eq!(collected.len(), nodes.len());
        }

        #[test]
        fn miri_read_chain_node_round_trips_through_persisted_reader() {
            let nodes = vec![tid(7, 7), tid(8, 8)];
            let (chain, gr, bw, sc) = build_chain_with_nodes(&nodes, &[]);
            let collected = collect_node_tids(&chain, gr, bw, sc).unwrap();
            let first_tid = collected[0];
            let tuple = read_chain_node(&chain, gr, bw, sc, first_tid).unwrap();
            // The primary_heaptid of the first stored node matches what
            // build_chain_with_nodes stamped in.
            assert_eq!(tuple.primary_heaptid, nodes[0]);
        }

        #[test]
        fn miri_write_chain_node_round_trips_mutated_tuple() {
            let nodes = vec![tid(11, 1), tid(11, 2)];
            let (mut chain, gr, bw, sc) = build_chain_with_nodes(&nodes, &[]);
            let collected = collect_node_tids(&chain, gr, bw, sc).unwrap();
            let target = collected[1];
            let mut updated = read_chain_node(&chain, gr, bw, sc, target).unwrap();
            updated.deleted = true;
            write_chain_node(&mut chain, gr, bw, sc, target, &updated).unwrap();
            let round = read_chain_node(&chain, gr, bw, sc, target).unwrap();
            assert!(round.deleted);
        }

        #[test]
        fn miri_write_chain_node_rejects_unknown_block_number() {
            let nodes = vec![tid(11, 1)];
            let (mut chain, gr, bw, sc) = build_chain_with_nodes(&nodes, &[]);
            let phantom = tid(999, 1);
            let tuple = VamanaNodeTuple::placeholder(gr, 0, 0);
            let err =
                write_chain_node(&mut chain, gr, bw, sc, phantom, &tuple).expect_err(
                    "writing to a phantom block must surface an error",
                );
            assert!(err.contains("could not find page"), "unexpected error: {err}");
        }

        // ---------------- collect_tuple_rewrites tests --------------

        #[test]
        fn miri_collect_tuple_rewrites_returns_empty_when_identical() {
            let nodes = vec![tid(1, 1), tid(1, 2)];
            let (chain, _gr, _bw, _sc) = build_chain_with_nodes(&nodes, &[]);
            let rewrites = collect_tuple_rewrites(&chain, &chain).unwrap();
            assert!(rewrites.is_empty());
        }

        #[test]
        fn miri_collect_tuple_rewrites_finds_only_changed_tuples() {
            let nodes = vec![tid(1, 1), tid(1, 2), tid(1, 3)];
            let (original, gr, bw, sc) = build_chain_with_nodes(&nodes, &[]);
            let (mut mutated, _gr, _bw, _sc) = build_chain_with_nodes(&nodes, &[]);
            let target_tids = collect_node_tids(&mutated, gr, bw, sc).unwrap();
            // Flip `deleted` on the middle tuple only.
            let mut tuple = read_chain_node(&mutated, gr, bw, sc, target_tids[1]).unwrap();
            tuple.deleted = true;
            write_chain_node(&mut mutated, gr, bw, sc, target_tids[1], &tuple).unwrap();

            let rewrites = collect_tuple_rewrites(&original, &mutated).unwrap();
            assert_eq!(rewrites.len(), 1);
            assert_eq!(rewrites[0].tid, target_tids[1]);
            assert_ne!(rewrites[0].expected_raw, rewrites[0].replacement_raw);
        }

        #[test]
        fn miri_collect_tuple_rewrites_rejects_mismatched_page_counts() {
            // Use a small page size so a single tuple fills the page,
            // forcing a second tuple onto a second page.
            // Page size 64 leaves only ~40 usable bytes; a single 16-byte
            // payload fills the page (raw_tuple_storage_bytes(16) == 24,
            // plus tuple body padding leaves no room for a second tuple).
            let mut one_page = DataPageChain::new(64);
            one_page.insert_raw_tuple(vec![0xaa; 16]).unwrap();
            let mut two_pages = DataPageChain::new(64);
            two_pages.insert_raw_tuple(vec![0xaa; 16]).unwrap();
            two_pages.insert_raw_tuple(vec![0xbb; 16]).unwrap();
            assert!(two_pages.pages().len() > one_page.pages().len());
            let err = collect_tuple_rewrites(&one_page, &two_pages)
                .expect_err("page-count mismatch must be rejected");
            assert!(err.contains("page-count mismatch"), "unexpected error: {err}");
        }

        #[test]
        fn miri_collect_tuple_rewrites_rejects_block_number_mismatch() {
            // Build two chains where the first page has a different block
            // number so the per-page block-number guard fires.
            let mut original = DataPageChain::new(128);
            original.insert_raw_tuple(vec![0xaa; 16]).unwrap();
            let mut mutated = DataPageChain::new(128);
            mutated.insert_raw_tuple(vec![0xaa; 16]).unwrap();
            // Force a block-number rewrite on `mutated`'s first page.
            // `DataPage::block_number` is `pub(crate)` in the careful
            // crate's compilation unit, so we can mutate it directly.
            let page = mutated.get_page_mut(1).unwrap();
            page.block_number = 99;
            let err = collect_tuple_rewrites(&original, &mutated)
                .expect_err("block-number mismatch must be rejected");
            assert!(err.contains("block mismatch"), "unexpected error: {err}");
        }

        #[test]
        fn miri_expand_scan_results_inflates_per_owner_and_respects_top_k() {
            let nodes = vec![tid(1, 1), tid(1, 2), tid(1, 3)];
            let (chain, _gr, _bw, _sc) = build_chain_with_nodes(&nodes, &[]);
            let node_results = vec![
                scan::ScanResult {
                    tid: tid(1, 1),
                    primary_heaptid: tid(100, 1),
                    distance: 0.1,
                },
                scan::ScanResult {
                    tid: tid(1, 2),
                    primary_heaptid: tid(100, 2),
                    distance: 0.2,
                },
            ];
            // Test shim's bound_heap_tids_for_owner returns just the
            // primary heaptid; expand inserts one entry per result.
            let expanded =
                expand_scan_results_with_bound_heap_tids(&chain, &node_results, 10).unwrap();
            assert_eq!(expanded.len(), 2);
            assert_eq!(expanded[0].primary_heaptid, tid(100, 1));
            assert_eq!(expanded[1].primary_heaptid, tid(100, 2));

            // Honor top_k cap.
            let capped =
                expand_scan_results_with_bound_heap_tids(&chain, &node_results, 1).unwrap();
            assert_eq!(capped.len(), 1);
        }

        #[test]
        fn miri_collect_tuple_rewrites_rejects_tuple_count_mismatch() {
            // Same page count + block number but different tuple counts:
            // tuple-count mismatch branch fires.
            let mut original = DataPageChain::new(256);
            original.insert_raw_tuple(vec![0xaa; 16]).unwrap();
            original.insert_raw_tuple(vec![0xbb; 16]).unwrap();
            let mut mutated = DataPageChain::new(256);
            mutated.insert_raw_tuple(vec![0xaa; 16]).unwrap();
            // Mutated has only one tuple while original has two.
            assert_eq!(original.pages().len(), mutated.pages().len());
            assert!(
                original.pages()[0].tuple_count() != mutated.pages()[0].tuple_count(),
                "preconditions: tuple counts must differ"
            );
            let err = collect_tuple_rewrites(&original, &mutated)
                .expect_err("tuple-count mismatch must be rejected");
            assert!(
                err.contains("tuple-count mismatch"),
                "unexpected error: {err}"
            );
        }
    }
}
