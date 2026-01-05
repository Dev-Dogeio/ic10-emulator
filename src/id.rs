use std::sync::atomic::{AtomicI32, Ordering};

/// Global ID counter shared by devices and items
static GLOBAL_ID_COUNTER: AtomicI32 = AtomicI32::new(1);

/// Allocate a new unique global ID for devices and items
pub fn allocate_global_id() -> i32 {
    GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}
