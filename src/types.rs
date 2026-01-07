//! Shared reference types for interior mutability

use std::cell::RefCell;
use std::rc::Rc;

/// Shared alias: reference-counted `RefCell` for interior mutability
pub type Shared<T> = Rc<RefCell<T>>;

/// Optional `Shared` reference
pub type OptShared<T> = Option<Shared<T>>;

/// Create a new `Shared` wrapper
pub fn shared<T>(value: T) -> Shared<T> {
    Rc::new(RefCell::new(value))
}
