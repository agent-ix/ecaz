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
    unsafe {
        if node.is_null() || es.is_null() {
            return;
        }
        let custom_scan = custom_scan_plan(node);
        let index_oid = custom_scan_index_oid_from_plan(custom_scan);
        let context = custom_scan_explain_context(index_oid);

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

unsafe fn custom_scan_explain_context(index_oid: pg_sys::Oid) -> SpireCustomScanExplainContext {
    unsafe {
        if index_oid == pg_sys::InvalidOid {
            return SpireCustomScanExplainContext {
                remote_fanout: 0,
                nprobe: 0,
                rerank_width: 0,
            };
        }

        let index_relation =
            pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        if index_relation.is_null() {
            return SpireCustomScanExplainContext {
                remote_fanout: 0,
                nprobe: 0,
                rerank_width: 0,
            };
        }

        let eligibility = custom_scan_index_eligibility_row(index_relation);
        let relation_options = super::options::relation_options(index_relation);
        let configured_nlists = u32::try_from(relation_options.nlists).unwrap_or(0);
        let relation_nprobe = u32::try_from(relation_options.nprobe).unwrap_or(0);
        let nprobe = super::options::resolve_scan_nprobe(configured_nlists, relation_nprobe);
        let rerank_width = super::options::resolve_scan_rerank_width(relation_options.rerank_width);
        pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);

        SpireCustomScanExplainContext {
            remote_fanout: eligibility.remote_available_node_count,
            nprobe: u64::from(nprobe.effective_nprobe),
            rerank_width: i64::from(rerank_width.effective_rerank_width),
        }
    }
}
