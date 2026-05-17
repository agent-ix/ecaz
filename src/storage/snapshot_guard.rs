use pgrx::pg_sys;

pub(crate) struct ActiveSnapshotGuard {
    snapshot: pg_sys::Snapshot,
}

impl ActiveSnapshotGuard {
    pub(crate) fn latest() -> Option<Self> {
        // SAFETY: `GetLatestSnapshot` returns a PostgreSQL snapshot pointer
        // valid for registration in the current backend context.
        let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
        if snapshot.is_null() {
            return None;
        }
        // SAFETY: `snapshot` is registered above and remains registered until
        // this guard drops.
        unsafe { pg_sys::PushActiveSnapshot(snapshot) };
        Some(Self { snapshot })
    }

    pub(crate) fn as_ptr(&self) -> pg_sys::Snapshot {
        self.snapshot
    }
}

impl Drop for ActiveSnapshotGuard {
    fn drop(&mut self) {
        // SAFETY: `snapshot` was pushed by `ActiveSnapshotGuard::latest` and
        // remains registered until this drop runs.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe {
            pg_sys::PopActiveSnapshot();
            pg_sys::UnregisterSnapshot(self.snapshot);
        }
    }
}
