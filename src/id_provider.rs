use std::sync::atomic::AtomicU32;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct IdProvider {
    inner: Arc<IdProviderInner>,
}

impl IdProvider {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(IdProviderInner {
                next_id: AtomicU32::new(1),
            }),
        }
    }

    pub fn next(&self) -> u32 {
        self.inner
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

#[derive(Debug)]
struct IdProviderInner {
    next_id: AtomicU32,
}
