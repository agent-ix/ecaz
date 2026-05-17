#[derive(Debug, Clone, Copy)]
struct SpireCustomScanExplainContext {
    remote_fanout: u64,
    nprobe: u64,
    rerank_width: i64,
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_explain_custom_scan(
    node: *mut pg_sys::CustomScanState,
    _ancestors: *mut pg_sys::List,
    es: *mut pg_sys::ExplainState,
) {
    if node.is_null() || es.is_null() {
        return;
    }

    // SAFETY: PostgreSQL calls this callback with a live CustomScanState whose
    // plan-private list was constructed by our planner callback.
    let custom_scan = unsafe { custom_scan_plan(node) };
    // SAFETY: `custom_scan` is the plan pointer from the live callback state.
    let index_oid = unsafe { custom_scan_index_oid_from_plan(custom_scan) };
    let context = custom_scan_explain_context(index_oid);

    // SAFETY: `es` is the non-null ExplainState supplied by PostgreSQL for the
    // duration of this callback; property names and values are static C strings.
    unsafe {
        pg_sys::ExplainPropertyText(
            c"node".as_ptr(),
            c"EcSpireDistributedScan".as_ptr(),
            es,
        );
        pg_sys::ExplainPropertyUInteger(
            c"remote_fanout".as_ptr(),
            std::ptr::null(),
            context.remote_fanout,
            es,
        );
        // Minimal Phase 12b contract: this is a stable shape marker, not a
        // live transport probe.
        pg_sys::ExplainPropertyText(c"tuple_transport_status".as_ptr(), c"ready".as_ptr(), es);
        pg_sys::ExplainPropertyUInteger(c"nprobe".as_ptr(), std::ptr::null(), context.nprobe, es);
        pg_sys::ExplainPropertyInteger(
            c"rerank_width".as_ptr(),
            std::ptr::null(),
            context.rerank_width,
            es,
        );
    }
}

fn custom_scan_explain_context(index_oid: pg_sys::Oid) -> SpireCustomScanExplainContext {
    if index_oid == pg_sys::InvalidOid {
        return SpireCustomScanExplainContext {
            remote_fanout: 0,
            nprobe: 0,
            rerank_width: 0,
        };
    }

    let Some(index_relation) =
        crate::storage::relation_guard::IndexRelationGuard::try_access_share(index_oid)
    else {
        return SpireCustomScanExplainContext {
            remote_fanout: 0,
            nprobe: 0,
            rerank_width: 0,
        };
    };

    // SAFETY: The relation pointer is owned by `IndexRelationGuard` and
    // remains open under AccessShareLock while these helpers read relation
    // metadata.
    let eligibility = unsafe { custom_scan_index_eligibility_row(index_relation.as_ptr()) };
    // SAFETY: Same open index relation; the helper only reads reloptions.
    let relation_options = unsafe { super::options::relation_options(index_relation.as_ptr()) };
    let configured_nlists = u32::try_from(relation_options.nlists).unwrap_or(0);
    let relation_nprobe = u32::try_from(relation_options.nprobe).unwrap_or(0);
    let nprobe = super::options::resolve_scan_nprobe(configured_nlists, relation_nprobe);
    let rerank_width = super::options::resolve_scan_rerank_width(relation_options.rerank_width);

    SpireCustomScanExplainContext {
        remote_fanout: eligibility.remote_available_node_count,
        nprobe: u64::from(nprobe.effective_nprobe),
        rerank_width: i64::from(rerank_width.effective_rerank_width),
    }
}
