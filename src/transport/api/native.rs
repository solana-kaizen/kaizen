pub use {
    crate::result::Result,
    solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig},
    solana_client::{
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::Memcmp,
    },
    solana_rpc_client_api::filter::{MemcmpEncodedBytes, RpcFilterType},
    solana_sdk::{clock::Slot, commitment_config::CommitmentConfig},
    workflow_log::log_trace,
};

#[derive(Debug, Clone, Default)]
pub struct GetProgramAccountsConfig {
    pub filters: Option<Vec<GetProgramAccountsFilter>>,
    pub encoding: Option<UiAccountEncoding>,
    pub data_slice: Option<UiDataSliceConfig>,
    pub commitment: Option<CommitmentConfig>,
    pub min_context_slot: Option<Slot>,
    pub with_context: Option<bool>,
}

impl GetProgramAccountsConfig {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_filters(mut self, filters: Vec<GetProgramAccountsFilter>) -> Result<Self> {
        self.filters = Some(filters);
        Ok(self)
    }

    pub fn encoding(mut self, encoding: UiAccountEncoding) -> Result<Self> {
        self.encoding = Some(encoding);
        Ok(self)
    }

    pub fn data_slice(mut self, data_slice: UiDataSliceConfig) -> Result<Self> {
        self.data_slice = Some(data_slice);
        Ok(self)
    }

    pub fn commitment(mut self, commitment: CommitmentConfig) -> Result<Self> {
        self.commitment = Some(commitment);
        Ok(self)
    }

    pub fn min_context_slot(mut self, min_context_slot: Slot) -> Result<Self> {
        self.min_context_slot = Some(min_context_slot);
        Ok(self)
    }

    pub fn with_context(mut self, with_context: bool) -> Result<Self> {
        self.with_context = Some(with_context);
        Ok(self)
    }

    pub fn build(self) -> Result<RpcProgramAccountsConfig> {
        let filters = match self.filters {
            Some(filters) => {
                let mut list = vec![];
                for filter in filters {
                    list.push(filter.try_into()?);
                }

                Some(list)
            }
            None => None,
        };

        Ok(RpcProgramAccountsConfig {
            filters,
            account_config: RpcAccountInfoConfig {
                encoding: self.encoding,
                data_slice: self.data_slice,
                commitment: self.commitment,
                min_context_slot: self.min_context_slot,
            },
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone)]
pub enum GetProgramAccountsFilter {
    /// Memory comparison filter using offset and base58 encoded string
    MemcmpEncodedBase58(usize, String),

    /// Memory comparison filter using offset and base64 encoded string
    MemcmpEncodedBase64(usize, String),

    /// Memory comparison filter using offset and bytes which will be encoded as base58
    MemcmpEncodeBase58(usize, Vec<u8>),

    /// Memory comparison filter using offset and bytes which will be encoded as base64
    MemcmpEncodeBase64(usize, Vec<u8>),

    /// Data size comparison filter
    DataSize(usize),
}

impl TryFrom<GetProgramAccountsFilter> for RpcFilterType {
    type Error = crate::error::Error;
    fn try_from(filter: GetProgramAccountsFilter) -> std::result::Result<Self, Self::Error> {
        Ok(match filter {
            GetProgramAccountsFilter::MemcmpEncodedBase58(offset, encoded_string) => {
                //log_trace!("encoded_string: {encoded_string:?}");
                let bytes = MemcmpEncodedBytes::Base58(encoded_string);
                RpcFilterType::Memcmp(Memcmp::new(offset, bytes))
            }
            GetProgramAccountsFilter::MemcmpEncodedBase64(offset, encoded_string) => {
                //log_trace!("encoded_string: {encoded_string:?}");
                let bytes = MemcmpEncodedBytes::Base64(encoded_string);
                RpcFilterType::Memcmp(Memcmp::new(offset, bytes))
            }

            GetProgramAccountsFilter::MemcmpEncodeBase58(offset, bytes) => {
                //log_trace!("data: {bytes:?}");
                RpcFilterType::Memcmp(Memcmp::new_base58_encoded(offset, &bytes))
            }

            GetProgramAccountsFilter::MemcmpEncodeBase64(offset, bytes) => {
                //log_trace!("bytes: {bytes:?}");
                let bytes = MemcmpEncodedBytes::Base64(base64::encode(bytes));
                RpcFilterType::Memcmp(Memcmp::new(offset, bytes))
            }

            GetProgramAccountsFilter::DataSize(data_size) => {
                RpcFilterType::DataSize(data_size as u64)
            }
        })
    }
}
