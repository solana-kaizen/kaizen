use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub timeout : Duration,
    pub confirm_transaction_initial_timeout : Duration,
    pub retries : usize
}

impl TransportConfig {
    pub fn new(timeout: Duration, confirm_transaction_initial_timeout: Duration, retries: usize) -> TransportConfig {
        TransportConfig {
            timeout,
            confirm_transaction_initial_timeout,
            retries
        }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        TransportConfig {
            timeout : Duration::from_secs(60u64),
            confirm_transaction_initial_timeout : Duration::from_secs(5u64),
            retries : 2
        }
    }
}
