use crate::prelude::*;
use crate::transport::transaction::Transaction;
use crate::error::Error;

pub trait Observer : Send + Sync {
    fn transaction_created(&self, transaction: &Transaction);
    fn transaction_success(&self, transaction: &Transaction);
    fn transaction_timeout(&self, transaction: &Transaction);
    fn transaction_failure(&self, transaction: &Transaction, error: &Error);
    // fn get_transaction_list(&self) -> Vec<Transaction>;
}

pub struct BasicObserver {

}

impl Observer for BasicObserver {
    fn transaction_created(&self, transaction: &Transaction) {
        log_trace!("NativeObserver::transaction_created {:#?}", transaction);
    }

    fn transaction_success(&self, transaction: &Transaction) {
        log_trace!("NativeObserver::transaction_success {:#?}", transaction);
    }

    fn transaction_timeout(&self, transaction: &Transaction) {
        log_trace!("NativeObserver::transaction_timeout {:#?}", transaction);
    }

    fn transaction_failure(&self, transaction: &Transaction, error: &Error) {
        log_trace!("NativeObserver::transaction_failure {:#?} {:#?}", error, transaction);
    }

}