use std::cell::RefCell;
use std::rc::Rc;

pub type Shared<T> = Rc<RefCell<T>>;
pub type OptShared<T> = Option<Shared<T>>;

pub fn shared<T>(value: T) -> Shared<T> {
    Rc::new(RefCell::new(value))
}
