use crate::error::Error;
use crate::prelude::*;
use crate::transport::transaction::Transaction;
use async_trait::async_trait;

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
