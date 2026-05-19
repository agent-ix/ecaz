// Pure helpers extracted from `routine.rs` so the hardening shadow
// crate can `include!` them. Like `coordinator/diagnostics_helpers.rs`,
// this file must stay free of pgrx FFI and `pg_sys` calls; anything
// that needs a live relation or memory context belongs in `routine.rs`.

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
