use std::cell::RefCell;

thread_local! {
    static SCOPE: RefCell<Option<Scope>> = const { RefCell::new(None) };
}

pub struct Scope {
    depth: u32,
}

impl Scope {
    pub fn create() {
        SCOPE.with_borrow_mut(|scope| {
            assert!(scope.is_none());
            *scope = Some(Scope { depth: 0 });
        })
    }

    pub fn delete() {
        SCOPE.with_borrow_mut(|scope| {
            assert!(scope.is_some());
            *scope = None;
        })
    }

    pub fn get() -> Option<u32> {
        SCOPE.with_borrow(|scope| scope.as_ref().map(|scope| scope.depth))
    }

    pub fn increment_depth() {
        SCOPE.with_borrow_mut(|scope| {
            let depth = &mut scope.as_mut().unwrap().depth;
            *depth = depth.checked_add(1).unwrap();
        });
    }

    pub fn decrement_depth() {
        SCOPE.with_borrow_mut(|scope| {
            let depth = &mut scope.as_mut().unwrap().depth;
            *depth = depth.checked_sub(1).unwrap();
        });
    }
}
