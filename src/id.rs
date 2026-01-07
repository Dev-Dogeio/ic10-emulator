//! Global ID allocation for devices and items

use std::sync::atomic::{AtomicI32, Ordering};

/// Global ID counter for devices and items
static GLOBAL_ID_COUNTER: AtomicI32 = AtomicI32::new(1);

/// Allocate a new unique global ID for devices and items
pub fn allocate_global_id() -> i32 {
    GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Reset the global ID counter back to its initial value.
/// Primarily intended for tests and global simulation reset.
pub fn reset_global_id_counter() {
    GLOBAL_ID_COUNTER.store(1, Ordering::SeqCst);
}
