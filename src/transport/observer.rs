use crate::prelude::*;
use crate::transport::transaction::WorkflowTransaction;

pub trait Observer {
    fn transaction_created(&self, transaction: &WorkflowTransaction);
    fn transaction_success(&self, transaction: &WorkflowTransaction);
    fn transaction_timeout(&self, transaction: &WorkflowTransaction);
    fn transaction_failure(&self, transaction: &WorkflowTransaction);
    // fn get_transaction_list(&self) -> Vec<Transaction>;
}

pub struct BasicObserver {

}

impl Observer for BasicObserver {
    fn transaction_created(&self, transaction: &WorkflowTransaction) {
        log_trace!("NativeObserver::transaction_created {:#?}", transaction);
    }

    fn transaction_success(&self, transaction: &WorkflowTransaction) {
        log_trace!("NativeObserver::transaction_success {:#?}", transaction);
    }

    fn transaction_timeout(&self, transaction: &WorkflowTransaction) {
        log_trace!("NativeObserver::transaction_timeout {:#?}", transaction);
    }

    fn transaction_failure(&self, transaction: &WorkflowTransaction) {
        log_trace!("NativeObserver::transaction_failure {:#?}", transaction);
    }

}