use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub(super) struct SharedQueue<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
    cap: usize,
}

impl<T> SharedQueue<T> {
    pub(super) fn enqueue(&self, elem: T) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.len() < self.cap {
            inner.push_front(elem);
            true
        } else {
            false
        }
    }

    pub(super) fn dequeue(&self) -> Option<T> {
        self.inner.lock().unwrap().pop_back()
    }

    pub(super) fn new(cap: usize) -> Self {
        let queue = VecDeque::with_capacity(cap);
        Self {
            inner: Arc::new(Mutex::new(queue)),
            cap,
        }
    }

    pub(super) fn clear(&self) {
        self.inner.lock().unwrap().clear()
    }
}
