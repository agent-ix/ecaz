//! Careful-side scaffold for `src/am/ec_diskann/routine_helpers.rs`.
//!
//! Like `spire_diagnostics_helpers`, this module sets up the minimal
//! ambient scope the production helpers expect (`ItemPointer` from the
//! crate's storage page, and `insert::cmp_item_pointer_physical`) and
//! `include!`s the production helpers verbatim so coverage attributes
//! back to the production file.

#![allow(dead_code)]

mod scaffold {
    use crate::storage::page::ItemPointer;

    mod insert {
        use super::ItemPointer;
        use std::cmp::Ordering;

        pub(super) fn cmp_item_pointer_physical(
            left: &ItemPointer,
            right: &ItemPointer,
        ) -> Ordering {
            left.block_number
                .cmp(&right.block_number)
                .then(left.offset_number.cmp(&right.offset_number))
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
            // graph_degree_r=0 must clamp to 1 so the scan still considers
            // the entry point.
            assert_eq!(vacuum_repair_scan_budget(100, 0), 1);
            // build_list_size <= graph_degree_r returns build_list_size.
            assert_eq!(vacuum_repair_scan_budget(5, 32), 5);
            // graph_degree_r < build_list_size returns graph_degree_r.
            assert_eq!(vacuum_repair_scan_budget(100, 32), 32);
            // Equal: returns either (they tie).
            assert_eq!(vacuum_repair_scan_budget(32, 32), 32);
        }

        #[test]
        fn miri_sql_scan_result_cap_returns_rerank_budget_regardless_of_top_k() {
            // The reloption top_k is ignored on the SQL path; only the
            // rerank budget caps how many rows are materialised.
            assert_eq!(sql_scan_result_cap(10, 200), 200);
            assert_eq!(sql_scan_result_cap(1_000, 50), 50);
            assert_eq!(sql_scan_result_cap(0, 0), 0);
        }
    }
}
