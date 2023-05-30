use std::sync::{Arc};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CommitteeManager {
    inner: Arc<RwLock<CommitteeManagerInner>>
}

impl CommitteeManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CommitteeManagerInner {
            }))
        }
    }
}


#[derive(Debug)]
struct CommitteeManagerInner {

}
