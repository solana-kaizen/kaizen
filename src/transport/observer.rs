//!
//! Transaction processing observer traits for client-side application tracking.
//!

use crate::error::Error;
use crate::prelude::*;
use crate::transport::transaction::Transaction;
use async_trait::async_trait;
use wasm_bindgen::prelude::*;

/// # Observer
///
/// The observer trait is meant to be implemented client-side and
/// registered with TransactionQueue to observe transaction processing.
///
/// Observer is meant to be reactive. i.e. its methods should create
/// and destroy UI elements *only* in response to trait function calls.
///
/// User Interface can use `TransactionChain::id` and `Transaction::id` to
/// maintain references to the chain and transaction instances.
///
/// ## Chain notifications
///
/// Chain represents a chain of related transactions that are meant to be
/// executed in a specific order.
///
/// ## Transaction notifications
///
/// Transaction represents a single transaction unit. Transaction always
/// has a chain, even if it is a single transaction.
///
/// # Dangling Chains
///
/// A chain results in a dangling state when a transaction failure occurs.
/// The idea behind this is that User Interface will reflect the transaction
/// failure and present user with Retry or Discard mechanism.  If user chooses
/// a retry on the transaction, it can be re-queued with TransactionQueue
/// once again, at which point it will detect the existing chain in which
/// this transaction is present and restart the chain processing.
///
/// Otherwise User Interface can offer user to discard the chain (whole
/// transaction chain), at which point you should call `TransactionQueue::discard_chain()`
/// that will result in `Observer::tx_chain_discarded()` being called
/// allowing you to destroy associated User Interface elements.
///

#[async_trait]
pub trait Observer: Send + Sync {
    /// Called when a new transaction is added to the queue.
    async fn tx_chain_created(&self, tx_chain: Arc<TransactionChain>);
    /// Called when transaction execution has completed successfully.
    async fn tx_chain_complete(&self, tx_chain: Arc<TransactionChain>);
    /// Called in response to `TransactionQueue::discard_chain()`.
    async fn tx_chain_discarded(&self, tx_chain: Arc<TransactionChain>);

    /// Called when transaction is posted in the queue (chain).
    async fn tx_created(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>);
    /// Called when transaction processing begins.
    async fn tx_processing(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>);
    /// Called on successful completion of a transaction
    async fn tx_success(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>);
    /// Called on transaction timeout (chain is left dangling)
    async fn tx_timeout(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>);
    /// Called on transaction failure (chain is left dangling)
    async fn tx_failure(
        &self,
        tx_chain: Arc<TransactionChain>,
        transaction: Arc<Transaction>,
        error: Error,
    );
}

pub struct BasicObserver {}

#[async_trait]
impl Observer for BasicObserver {
    async fn tx_chain_created(&self, tx_chain: Arc<TransactionChain>) {
        log_trace!("BasicObserver::tx_chain_created {}", tx_chain.id);
    }

    async fn tx_chain_complete(&self, tx_chain: Arc<TransactionChain>) {
        log_trace!("BasicObserver::tx_chain_complete {}", tx_chain.id);
    }

    async fn tx_chain_discarded(&self, tx_chain: Arc<TransactionChain>) {
        log_trace!("BasicObserver::tx_chain_discarded {}", tx_chain.id);
    }

    async fn tx_created(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) {
        log_trace!(
            "BasicObserver::tx_created {} {:#?}",
            tx_chain.id,
            transaction
        );
    }

    async fn tx_processing(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) {
        log_trace!(
            "BasicObserver::tx_processing {} {:#?}",
            tx_chain.id,
            transaction
        );
    }

    async fn tx_success(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) {
        log_trace!(
            "BasicObserver::tx_success {} {:#?}",
            tx_chain.id,
            transaction
        );
    }

    async fn tx_timeout(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) {
        log_trace!(
            "BasicObserver::tx_timeout {} {:#?}",
            tx_chain.id,
            transaction
        );
    }

    async fn tx_failure(
        &self,
        tx_chain: Arc<TransactionChain>,
        transaction: Arc<Transaction>,
        err: Error,
    ) {
        log_trace!(
            "BasicObserver::tx_failure {} {:#?} {:#?}",
            tx_chain.id,
            err,
            transaction
        );
    }
}

mod wasm {
    //!
    //! WASM interface implementation of [`Observer`](super::Observer).
    //!
    use super::*;
    use crate::result::Result;
    use js_sys::{Function, Object};
    use serde::Serialize;
    use serde_wasm_bindgen::*;
    use std::sync::{Arc, Mutex};
    use workflow_core::id::Id;
    use workflow_wasm::prelude::*;

    #[derive(Clone, Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum NotificationType {
        ChainCreated,
        ChainComplete,
        ChainDiscarded,
        TransactionCreated,
        TransactionProcessing,
        TransactionSuccess,
        TransactionTimeout,
        TransactionFailure,
    }

    #[derive(Clone, Serialize)]
    pub struct TransactionNotification {
        /// Transaction chain info
        #[serde(rename = "txChain")]
        tx_chain: Arc<TransactionChain>,

        /// Transaction info
        transaction: Arc<Transaction>,

        /// Error string (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    }

    impl TransactionNotification {
        pub fn new(tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) -> Self {
            TransactionNotification {
                tx_chain,
                transaction,
                error: None,
            }
        }
        pub fn new_with_error(
            tx_chain: Arc<TransactionChain>,
            transaction: Arc<Transaction>,
            error: kaizen::error::Error,
        ) -> Self {
            TransactionNotification {
                tx_chain,
                transaction,
                error: Some(error.to_string()),
            }
        }
    }

    #[derive(Default)]
    pub struct TransactionObserverInner {
        notification_callback: Arc<Mutex<Option<Sendable<Function>>>>,
    }

    impl TransactionObserverInner {
        pub async fn set_handler(&self, callback: JsValue) -> Result<()> {
            if callback.is_function() {
                let fn_callback: Function = callback.into();
                self.notification_callback
                    .lock()
                    .unwrap()
                    .replace(fn_callback.into());
            } else {
                self.remove_handler();
            }
            Ok(())
        }

        pub fn remove_handler(&self) {
            *self.notification_callback.lock().unwrap() = None;
        }

        fn post_notification<Op, T>(&self, op: Op, payload: T)
        where
            T: Serialize,
            Op: Serialize,
        {
            if let Some(callback) = self.notification_callback.lock().unwrap().as_ref() {
                let object = Object::new();
                object
                    .set("event", &to_value(&op).unwrap())
                    .expect("TransactionObserver::post_notification() event serialization failure");
                object
                    .set("data", &to_value(&payload).unwrap())
                    .expect("TransactionObserver::post_notification() event serialization failure");
                if let Err(err) = callback.0.call1(&JsValue::undefined(), &object) {
                    log_error!("Error while executing notification callback: {:?}", err);
                }
            }
        }
    }

    #[async_trait]
    impl Observer for TransactionObserverInner {
        async fn tx_chain_created(&self, tx_chain: Arc<TransactionChain>) {
            self.post_notification(NotificationType::ChainCreated, tx_chain);
        }

        async fn tx_chain_complete(&self, tx_chain: Arc<TransactionChain>) {
            self.post_notification(NotificationType::ChainComplete, tx_chain);
        }

        async fn tx_chain_discarded(&self, tx_chain: Arc<TransactionChain>) {
            self.post_notification(NotificationType::ChainDiscarded, tx_chain);
        }

        async fn tx_created(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) {
            self.post_notification(
                NotificationType::TransactionCreated,
                TransactionNotification::new(tx_chain, transaction),
            );
        }

        async fn tx_processing(
            &self,
            tx_chain: Arc<TransactionChain>,
            transaction: Arc<Transaction>,
        ) {
            self.post_notification(
                NotificationType::TransactionProcessing,
                TransactionNotification::new(tx_chain, transaction),
            );
        }

        async fn tx_success(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) {
            self.post_notification(
                NotificationType::TransactionSuccess,
                TransactionNotification::new(tx_chain, transaction),
            );
        }

        async fn tx_timeout(&self, tx_chain: Arc<TransactionChain>, transaction: Arc<Transaction>) {
            self.post_notification(
                NotificationType::TransactionTimeout,
                TransactionNotification::new(tx_chain, transaction),
            );
        }

        async fn tx_failure(
            &self,
            tx_chain: Arc<TransactionChain>,
            transaction: Arc<Transaction>,
            err: kaizen::error::Error,
        ) {
            self.post_notification(
                NotificationType::TransactionFailure,
                TransactionNotification::new_with_error(tx_chain, transaction, err),
            );
        }
    }

    /// [`TransactionObserver`] is an [`Observer`](super::Observer) implementation
    /// meant to be used in WASM. TransactionObserver exposes two main functions:
    ///
    /// - [`set_handler()`] (`setHandler()` in JavaScript)
    /// - [`remove_handler()`] (`removeHandler()` in JavaScript)
    ///
    /// When [`set_handler()`] is invoked, the observer instance auto-registers itself
    /// with the global [`Transport`]. When [`remove_handler()`] is invoked, it
    /// unregisters itself from the global [`Transport`].
    ///
    /// While registered, the supplied handler will receive [`TransactionNotification`]
    /// events as a single object containing two fields:
    ///
    /// `{ event : "...", data : "..." }`
    ///
    /// The incoming `event` value contains kebab representation of the [`NotificationType`]
    /// enum. The `data` contains the [`TransactionNotification`] object.
    ///
    #[wasm_bindgen]
    #[derive(Default)]
    pub struct TransactionObserver {
        observer_id: Id,
        inner: Arc<TransactionObserverInner>,
    }

    #[wasm_bindgen]
    impl TransactionObserver {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            TransactionObserver {
                observer_id: Id::new(),
                inner: Arc::new(TransactionObserverInner::default()),
            }
        }

        #[wasm_bindgen(js_name = "setHandler")]
        pub async fn set_handler(&self, callback: JsValue) -> Result<()> {
            self.inner.set_handler(callback).await?;

            let transport = Transport::global().unwrap_or_else(|err| {
                panic!("TransactionObserver - missing global transport: {err}");
            });

            transport
                .queue
                .register_observer(&self.observer_id, self.inner.clone())
                .unwrap_or_else(|err| {
                    panic!(
                        "TransactionObserver - unable to register observer with transport: {err}"
                    );
                });

            Ok(())
        }

        #[wasm_bindgen(js_name = "removeHandler")]
        pub fn remove_handler(&self) {
            let transport = Transport::global().unwrap_or_else(|err| {
                panic!("TransactionObserver - missing global transport: {err}");
            });

            transport
                .queue
                .unregister_observer(&self.observer_id)
                .unwrap_or_else(|err| {
                    panic!(
                        "TransactionObserver - unable to unregister observer with transport: {err}"
                    );
                });

            self.inner.remove_handler();
        }
    }
}
