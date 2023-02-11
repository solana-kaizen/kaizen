//!
//! Rent collection utilities.
//!

use solana_program::account_info::AccountInfo;

#[derive(Debug, Copy, Clone)]
pub enum RentCollector<'info, 'refs> {
    Program,
    Account(&'refs AccountInfo<'info>),
}

impl<'info, 'refs> Default for RentCollector<'info, 'refs> {
    fn default() -> RentCollector<'info, 'refs> {
        RentCollector::Program
    }
}
