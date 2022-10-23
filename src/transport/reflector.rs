use std::sync::{Arc, Mutex};
use workflow_allocator::result::Result;

pub type ReflectPendingFn = Arc<Box<(dyn Fn(usize) + Sync + Send)>>;

pub struct PendingReflector {
    pub transactions: Arc<Mutex<Option<ReflectPendingFn>>>,
    pub lookups: Arc<Mutex<Option<ReflectPendingFn>>>,
}

impl PendingReflector {

    pub fn new() -> PendingReflector {
        PendingReflector {
            transactions: Arc::new(Mutex::new(None)),
            lookups: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init(&self, lookups: Option<ReflectPendingFn>, transactions : Option<ReflectPendingFn>) -> Result<()>{
        *self.lookups.lock()? = lookups;
        *self.transactions.lock()? = transactions;
        Ok(())
    }

    pub fn update_lookups(&self, pending: usize) {
        let handler = {
            let handler = self.lookups.lock().unwrap();
            handler.as_ref().cloned()
        };
        if let Some(handler) = handler {
            handler(pending);
        }
    }

    pub fn update_transactions(&self, pending: usize) {
        let handler = {
            let handler = self.transactions.lock().unwrap();
            handler.as_ref().cloned()
        };
        if let Some(handler) = handler {
            handler(pending);
        }
    }

}