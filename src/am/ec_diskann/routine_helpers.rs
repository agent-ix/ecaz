// Pure helpers extracted from `routine.rs` so the hardening shadow
// crate can `include!` them. Like `coordinator/diagnostics_helpers.rs`,
// this file must stay free of pgrx FFI and `pg_sys` calls; anything
// that needs a live relation or memory context belongs in `routine.rs`.

#[derive(Debug, Clone, PartialEq, Eq)]
struct TupleRewrite {
    tid: ItemPointer,
    expected_raw: Vec<u8>,
    replacement_raw: Vec<u8>,
}

fn sort_and_dedup_item_pointers(tids: &mut Vec<ItemPointer>) {
    tids.sort_unstable_by(insert::cmp_item_pointer_physical);
    tids.dedup();
}

fn vacuum_repair_scan_budget(build_list_size: usize, graph_degree_r: usize) -> usize {
    build_list_size.min(graph_degree_r.max(1))
}

fn sql_scan_result_cap(reloption_top_k: usize, rerank_budget: usize) -> usize {
    // `LIMIT` is not visible to `amrescan`, so the SQL scan path must
    // materialize the full rerank window and let the executor truncate.
    // The reloption `top_k` remains a pure scan-shell knob rather than a
    // hard SQL result cap.
    let _ = reloption_top_k;
    rerank_budget
}

fn count_live_tuples_in_chain(
    chain: &DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
) -> Result<usize, String> {
    let reader =
        PersistedGraphReader::new(chain, graph_degree_r, binary_word_count, search_code_len);
    let live_count = reader
        .iter_live_tids()
        .try_fold(0usize, |count, item| item.map(|_| count + 1))?;
    Ok(live_count)
}

fn collect_node_tids(
    chain: &DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
) -> Result<Vec<ItemPointer>, String> {
    let reader =
        PersistedGraphReader::new(chain, graph_degree_r, binary_word_count, search_code_len);
    reader.iter_node_tids().collect()
}

fn read_chain_node(
    chain: &DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
    tid: ItemPointer,
) -> Result<VamanaNodeTuple, String> {
    let reader =
        PersistedGraphReader::new(chain, graph_degree_r, binary_word_count, search_code_len);
    reader.read_node(tid)
}

fn write_chain_node(
    chain: &mut DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
    tid: ItemPointer,
    tuple: &VamanaNodeTuple,
) -> Result<(), String> {
    let encoded = tuple.encode(graph_degree_r, binary_word_count, search_code_len)?;
    let page = chain.get_page_mut(tid.block_number).ok_or_else(|| {
        format!(
            "ec_diskann vacuum rewrite could not find page {} for ({},{})",
            tid.block_number, tid.block_number, tid.offset_number
        )
    })?;
    page.update_raw_tuple(tid, encoded)
}

fn expand_scan_results_with_bound_heap_tids(
    chain: &DataPageChain,
    node_results: &[scan::ScanResult],
    top_k: usize,
) -> Result<Vec<scan::ScanResult>, String> {
    let mut expanded = Vec::with_capacity(top_k.min(node_results.len()));
    for result in node_results {
        let bound_heap_tids =
            insert::bound_heap_tids_for_owner(chain, result.tid, result.primary_heaptid)?;
        for heap_tid in bound_heap_tids {
            expanded.push(scan::ScanResult {
                tid: result.tid,
                primary_heaptid: heap_tid,
                distance: result.distance,
            });
            if expanded.len() >= top_k {
                return Ok(expanded);
            }
        }
    }
    Ok(expanded)
}

fn collect_tuple_rewrites(
    original_chain: &DataPageChain,
    mutated_chain: &DataPageChain,
) -> Result<Vec<TupleRewrite>, String> {
    if original_chain.pages().len() != mutated_chain.pages().len() {
        return Err(format!(
            "ec_diskann vacuum rewrite page-count mismatch: original {}, mutated {}",
            original_chain.pages().len(),
            mutated_chain.pages().len()
        ));
    }

    let mut rewrites = Vec::new();
    for (original_page, mutated_page) in original_chain.pages().iter().zip(mutated_chain.pages()) {
        if original_page.block_number() != mutated_page.block_number() {
            return Err(format!(
                "ec_diskann vacuum rewrite block mismatch: original {}, mutated {}",
                original_page.block_number(),
                mutated_page.block_number()
            ));
        }
        if original_page.tuple_count() != mutated_page.tuple_count() {
            return Err(format!(
                "ec_diskann vacuum rewrite tuple-count mismatch on block {}: original {}, mutated {}",
                original_page.block_number(),
                original_page.tuple_count(),
                mutated_page.tuple_count()
            ));
        }

        for offset in 1..=original_page.tuple_count() {
            let tid = ItemPointer {
                block_number: original_page.block_number(),
                offset_number: offset as u16,
            };
            let expected_raw = original_page.raw_tuple(tid)?.to_vec();
            let replacement_raw = mutated_page.raw_tuple(tid)?.to_vec();
            if expected_raw != replacement_raw {
                rewrites.push(TupleRewrite {
                    tid,
                    expected_raw,
                    replacement_raw,
                });
            }
        }
    }
    Ok(rewrites)
}
