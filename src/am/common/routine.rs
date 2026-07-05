use pgrx::{pg_sys, AllocatedByRust, PgBox};

pub(crate) type IndexAmRoutineBox = PgBox<pg_sys::IndexAmRoutine, AllocatedByRust>;

pub(crate) fn alloc_index_am_routine() -> IndexAmRoutineBox {
    // SAFETY: `IndexAmRoutine` is a PostgreSQL Node type and must be allocated
    // with the matching `T_IndexAmRoutine` node tag.
    unsafe { PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) }
}
