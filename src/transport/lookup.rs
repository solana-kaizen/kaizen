//!
//! Account lookup synchronizer combining multiple pending async lookups for the same account into a single future.
//! 

use crate::result::Result;
use ahash::AHashMap;
use async_std::sync::Mutex;
use std::cmp::Eq;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use workflow_core::channel::*;
pub type LookupResult<T> = Result<Option<T>>;
pub enum RequestType<T> {
    New(Receiver<LookupResult<T>>),
    Pending(Receiver<LookupResult<T>>),
}

pub type SenderList<T> = Vec<Sender<LookupResult<T>>>;

pub struct LookupHandler<K, T> {
    pub map: Arc<Mutex<AHashMap<K, SenderList<T>>>>,
    pending: AtomicUsize,
}

impl<K, T> Default for LookupHandler<K, T>
where
    T: Clone,
    K: Clone + Eq + Hash + Display,
{
    fn default() -> Self {
        LookupHandler::<K, T>::new()
    }
}

impl<K, T> LookupHandler<K, T>
where
    T: Clone,
    K: Clone + Eq + Hash + Display,
{
    pub fn new() -> Self {
        LookupHandler {
            map: Arc::new(Mutex::new(AHashMap::new())),
            pending: AtomicUsize::new(0),
        }
    }

    pub fn pending(&self) -> usize {
        self.pending.load(Ordering::SeqCst)
    }

    pub async fn queue(&self, key: &K) -> RequestType<T> {
        let mut pending = self.map.lock().await;
        let (sender, receiver) = oneshot::<LookupResult<T>>();

        if let Some(list) = pending.get_mut(key) {
            list.push(sender);
            RequestType::Pending(receiver)
        } else {
            let list = vec![sender];
            pending.insert(key.clone(), list);
            self.pending.fetch_add(1, Ordering::Relaxed);
            RequestType::New(receiver)
        }
    }

    pub async fn complete(&self, key: &K, result: LookupResult<T>) {
        let mut pending = self.map.lock().await;

        if let Some(list) = pending.remove(key) {
            self.pending.fetch_sub(1, Ordering::Relaxed);
            for sender in list {
                sender
                    .send(result.clone())
                    .await
                    .expect("Unable to complete lookup result");
            }
        } else {
            panic!("Lookup handler failure while processing account `{key}`")
        }
    }
}

#[cfg(not(target_os = "solana"))]
#[cfg(any(test, feature = "test"))]
mod tests {
    use super::LookupHandler;
    use super::RequestType;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    use super::Result;
    use ahash::AHashMap;
    use async_std::task::sleep;
    use futures::join;
    use wasm_bindgen::prelude::*;
    use workflow_log::log_trace;

    #[derive(Debug, Eq, PartialEq)]
    enum RequestTypeTest {
        New = 0,
        Pending = 1,
    }

    struct LookupHandlerTest {
        pub lookup_handler: LookupHandler<u32, u32>,
        pub map: Arc<Mutex<AHashMap<u32, u32>>>,
        pub request_types: Arc<Mutex<Vec<RequestTypeTest>>>,
    }

    impl LookupHandlerTest {
        pub fn new() -> Self {
            Self {
                lookup_handler: LookupHandler::new(),
                map: Arc::new(Mutex::new(AHashMap::new())),
                request_types: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn insert(self: &Arc<Self>, key: u32, value: u32) -> Result<()> {
            let mut map = self.map.lock()?;
            map.insert(key, value);
            Ok(())
        }

        pub async fn lookup_remote_impl(self: &Arc<Self>, key: &u32) -> Result<Option<u32>> {
            log_trace!("[lh] lookup sleep...");
            sleep(Duration::from_millis(100)).await;
            log_trace!("[lh] lookup wake...");
            let map = self.map.lock()?;
            Ok(map.get(&key).cloned())
        }

        pub async fn lookup_handler_request(self: &Arc<Self>, key: &u32) -> Result<Option<u32>> {
            let request_type = self.lookup_handler.queue(key).await;
            match request_type {
                RequestType::New(receiver) => {
                    self.request_types
                        .lock()
                        .unwrap()
                        .push(RequestTypeTest::New);
                    log_trace!("[lh] new request");
                    let response = self.lookup_remote_impl(key).await;
                    log_trace!("[lh] completing initial request");
                    self.lookup_handler.complete(key, response).await;
                    receiver.recv().await?
                }
                RequestType::Pending(receiver) => {
                    self.request_types
                        .lock()
                        .unwrap()
                        .push(RequestTypeTest::Pending);
                    log_trace!("[lh] pending request");
                    receiver.recv().await?
                }
            }
        }
    }

    #[wasm_bindgen]
    pub async fn lookup_handler_test() -> Result<()> {
        let lht = Arc::new(LookupHandlerTest::new());
        lht.insert(0xc0fee, 0xdecaf)?;

        let v0 = lht.lookup_handler_request(&0xc0fee);
        let v1 = lht.lookup_handler_request(&0xc0fee);
        let v2 = lht.lookup_handler_request(&0xc0fee);
        let f = join!(v0, v1, v2);

        log_trace!("[lh] results: {:?}", f);
        let f = (
            f.0.unwrap().unwrap(),
            f.1.unwrap().unwrap(),
            f.2.unwrap().unwrap(),
        );
        assert_eq!(f, (0xdecaf, 0xdecaf, 0xdecaf));

        let request_types = lht.request_types.lock().unwrap();
        log_trace!("[lh] request types: {:?}", request_types);
        assert_eq!(
            request_types[..],
            [
                RequestTypeTest::New,
                RequestTypeTest::Pending,
                RequestTypeTest::Pending
            ]
        );
        log_trace!("all looks good ... 😎");

        Ok(())
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
    #[cfg(test)]
    mod tests {
        use super::*;

        #[async_std::test]
        pub async fn lookup_handler_test() -> Result<()> {
            super::lookup_handler_test().await
        }
    }
}
