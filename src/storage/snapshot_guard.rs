//! Snapshot guards split PostgreSQL's two ownership patterns:
//! `RegisteredSnapshotGuard` owns register/unregister only, while
//! `ActiveSnapshotGuard` also pushes/pops the backend active-snapshot stack.

use pgrx::pg_sys;

pub(crate) struct RegisteredSnapshotGuard {
    snapshot: pg_sys::Snapshot,
}

impl RegisteredSnapshotGuard {
    pub(crate) fn latest() -> Option<Self> {
        // SAFETY: `GetLatestSnapshot` returns a PostgreSQL snapshot pointer
        // valid for registration in the current backend context.
        let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
        if snapshot.is_null() {
            return None;
        }
        Some(Self { snapshot })
    }

    pub(crate) fn transaction() -> Option<Self> {
        // SAFETY: `GetTransactionSnapshot` returns a PostgreSQL snapshot
        // pointer valid for registration in the current backend context.
        let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetTransactionSnapshot()) };
        if snapshot.is_null() {
            return None;
        }
        Some(Self { snapshot })
    }

    pub(crate) fn as_ptr(&self) -> pg_sys::Snapshot {
        self.snapshot
    }
}

impl Drop for RegisteredSnapshotGuard {
    fn drop(&mut self) {
        // SAFETY: `snapshot` was returned by `RegisterSnapshot`; this guard
        // owns the matching unregister.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::UnregisterSnapshot(self.snapshot) };
    }
}

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

    #[cfg(any(test, feature = "pg_test"))]
    pub(crate) fn latest_after_command_counter() -> Option<Self> {
        // SAFETY: `CommandCounterIncrement` is valid before acquiring a fresh
        // backend snapshot in PostgreSQL helper entry points.
        unsafe { pg_sys::CommandCounterIncrement() };
        Self::latest()
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
