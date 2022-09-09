use crate::prelude::*;
use workflow_core::id::Id;
use crate::transport::transaction::Transaction;
use crate::error::Error;

pub trait Observer : Send + Sync {
    fn transaction_set_created(&self, id : &Id);
    fn transaction_set_complete(&self, id : &Id);

    fn transaction_created(&self, tx_set_id : &Id, transaction: &Transaction);
    fn transaction_success(&self, tx_set_id : &Id, transaction: &Transaction);
    fn transaction_timeout(&self, tx_set_id : &Id, transaction: &Transaction);
    fn transaction_failure(&self, tx_set_id : &Id, transaction: &Transaction, error: &Error);
    // fn get_transaction_list(&self) -> Vec<Transaction>;
}

pub struct BasicObserver {

}

impl Observer for BasicObserver {
    fn transaction_set_created(&self, tx_set_id: &Id) {
        log_trace!("NativeObserver::transaction_set_created {}", tx_set_id);
    }

    fn transaction_set_complete(&self, tx_set_id : &Id) {
        log_trace!("NativeObserver::transaction_created {}", tx_set_id);
    }

    fn transaction_created(&self, tx_set_id : &Id, transaction: &Transaction) {
        log_trace!("NativeObserver::transaction_created {} {:#?}", tx_set_id, transaction);
    }

    fn transaction_success(&self, tx_set_id : &Id, transaction: &Transaction) {
        log_trace!("NativeObserver::transaction_success {} {:#?}", tx_set_id, transaction);
    }

    fn transaction_timeout(&self, tx_set_id : &Id, transaction: &Transaction) {
        log_trace!("NativeObserver::transaction_timeout {} {:#?}", tx_set_id, transaction);
    }

    fn transaction_failure(&self, tx_set_id : &Id, transaction: &Transaction, error: &Error) {
        log_trace!("NativeObserver::transaction_failure {} {:#?} {:#?}", tx_set_id, error, transaction);
    }

}