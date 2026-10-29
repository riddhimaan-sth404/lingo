//! Prototype C & D: Ownership, References, and Cyclic Data Policies
//! Validates deterministic destruction and weak reference back-edges to prevent memory retention leaks.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

struct Node {
    pub value: i64,
    pub next: Option<Rc<RefCell<Node>>>,
    pub prev: Option<Weak<RefCell<Node>>>,
}

impl Drop for Node {
    fn drop(&mut self) {
        DROP_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_backlink_deterministic_cleanup() {
        DROP_COUNT.store(0, Ordering::SeqCst);

        {
            let first = Rc::new(RefCell::new(Node {
                value: 1,
                next: None,
                prev: None,
            }));

            let second = Rc::new(RefCell::new(Node {
                value: 2,
                next: None,
                prev: Some(Rc::downgrade(&first)),
            }));

            first.borrow_mut().next = Some(Rc::clone(&second));
        }

        // Both nodes must have been dropped deterministically without leaking
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 2);
    }
}
