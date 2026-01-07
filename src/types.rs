//! Shared reference types for interior mutability

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Shared alias: reference-counted `RefCell` for interior mutability
pub type Shared<T> = Rc<RefCell<T>>;

/// Optional `Shared` reference
pub type OptShared<T> = Option<Shared<T>>;

/// Weak reference to a `Shared` wrapper
pub type WeakShared<T> = Weak<RefCell<T>>;

/// Optional weak shared reference
pub type OptWeakShared<T> = Option<WeakShared<T>>;

/// Create a new `Shared` wrapper
pub fn shared<T>(value: T) -> Shared<T> {
    Rc::new(RefCell::new(value))
}
