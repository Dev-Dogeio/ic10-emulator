//! Global ID allocation for devices and items

use std::collections::HashSet;
use std::sync::{
    Mutex,
    atomic::{AtomicI32, Ordering},
};

/// Global ID counter for devices and items
static GLOBAL_ID_COUNTER: AtomicI32 = AtomicI32::new(1);

/// Global set of allocated IDs to prevent duplicates
static ALLOCATED_IDS: Mutex<Option<HashSet<i32>>> = Mutex::new(None);

/// Initialize the allocated IDs tracking set
fn ensure_allocated_ids_initialized() {
    let mut ids = ALLOCATED_IDS.lock().unwrap_or_else(|e| e.into_inner());
    if ids.is_none() {
        *ids = Some(HashSet::new());
    }
}

/// Allocate a new unique global ID for devices and items
pub fn allocate_global_id() -> i32 {
    ensure_allocated_ids_initialized();
    let id = GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    let mut ids = ALLOCATED_IDS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(set) = ids.as_mut() {
        set.insert(id);
    }

    id
}

/// Reserve a specific ID, panicking if it's already been allocated
pub fn reserve_global_id(requested_id: i32) -> i32 {
    ensure_allocated_ids_initialized();

    let mut ids = ALLOCATED_IDS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(set) = ids.as_mut() {
        if set.contains(&requested_id) {
            panic!(
                "ID {} has already been allocated and cannot be reserved.",
                requested_id
            );
        }
        set.insert(requested_id);

        // Update counter if the reserved ID is higher than current counter
        let current = GLOBAL_ID_COUNTER.load(Ordering::SeqCst);
        if requested_id >= current {
            GLOBAL_ID_COUNTER.store(requested_id + 1, Ordering::SeqCst);
        }
    }

    requested_id
}

/// Reset the global ID counter back to its initial value.
pub fn reset_global_id_counter() {
    GLOBAL_ID_COUNTER.store(1, Ordering::SeqCst);
    let mut ids = ALLOCATED_IDS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(set) = ids.as_mut() {
        set.clear();
    }
}
