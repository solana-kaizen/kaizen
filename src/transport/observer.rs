use crate::prelude::*;
use workflow_core::id::Id;
use crate::transport::transaction::Transaction;
use crate::error::Error;

pub trait Observer : Send + Sync {
    fn tx_chain_created(&self, id : &Id);
    fn tx_chain_complete(&self, id : &Id);

    fn tx_created(&self, tx_chain_id : &Id, transaction: &Transaction);
    fn tx_success(&self, tx_chain_id : &Id, transaction: &Transaction);
    fn tx_timeout(&self, tx_chain_id : &Id, transaction: &Transaction);
    fn tx_failure(&self, tx_chain_id : &Id, transaction: &Transaction, error: &Error);
    // fn get_tx_list(&self) -> Vec<Transaction>;
}

pub struct BasicObserver {

}

impl Observer for BasicObserver {
    fn tx_chain_created(&self, tx_chain_id: &Id) {
        log_trace!("NativeObserver::tx_set_created {}", tx_chain_id);
    }

    fn tx_chain_complete(&self, tx_chain_id : &Id) {
        log_trace!("NativeObserver::tx_created {}", tx_chain_id);
    }

    fn tx_created(&self, tx_chain_id : &Id, transaction: &Transaction) {
        log_trace!("NativeObserver::tx_created {} {:#?}", tx_chain_id, transaction);
    }

    fn tx_success(&self, tx_chain_id : &Id, transaction: &Transaction) {
        log_trace!("NativeObserver::tx_success {} {:#?}", tx_chain_id, transaction);
    }

    fn tx_timeout(&self, tx_chain_id : &Id, transaction: &Transaction) {
        log_trace!("NativeObserver::tx_timeout {} {:#?}", tx_chain_id, transaction);
    }

    fn tx_failure(&self, tx_chain_id : &Id, transaction: &Transaction, error: &Error) {
        log_trace!("NativeObserver::tx_failure {} {:#?} {:#?}", tx_chain_id, error, transaction);
    }

}