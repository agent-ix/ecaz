use std::sync::atomic::{AtomicI64, Ordering};

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};

static FAULT_PALLOC_NTH_GUC: GucSetting<i32> = GucSetting::<i32>::new(-1);
static FAULT_PALLOC_COUNTER: AtomicI64 = AtomicI64::new(0);

pub(crate) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ecaz.fault_palloc_nth",
        c"Fail the Nth instrumented ECAZ palloc site.",
        c"Task 38 live-fault knob. -1 disables injection; positive values raise ERROR on the Nth and later instrumented ECAZ palloc sites in the backend.",
        &FAULT_PALLOC_NTH_GUC,
        -1,
        i32::MAX,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(crate) fn reset_palloc_counter() {
    FAULT_PALLOC_COUNTER.store(0, Ordering::SeqCst);
}

pub(crate) fn maybe_fail_palloc(label: &str) {
    if cfg!(test) {
        return;
    }
    let nth = FAULT_PALLOC_NTH_GUC.get();
    if nth <= 0 {
        return;
    }
    let count = FAULT_PALLOC_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    if count >= i64::from(nth) {
        pgrx::error!("ecaz fault injection palloc failure at {label} (allocation #{count})");
    }
}
