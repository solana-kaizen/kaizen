use crate::result::Result;
use cfg_if::cfg_if;
use solana_sdk::{clock::Slot, commitment_config::CommitmentConfig};
use workflow_log::log_trace;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use solana_web3_sys::prelude::*;
        use js_sys::{Array, Object, Reflect};
        use wasm_bindgen::prelude::*;
    }else{
        pub use {
            solana_account_decoder::{
                UiAccountEncoding as RpcAccountEncoding,
                UiDataSliceConfig as RpcDataSliceConfig
            },
            solana_client::{
                rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
                rpc_filter::Memcmp,
            },
            solana_rpc_client_api::filter::{MemcmpEncodedBytes, RpcFilterType},
        };
    }
}

#[derive(Debug, Clone)]
pub enum AccountEncoding {
    Base58,
    Base64,
}

impl From<AccountEncoding> for RpcAccountEncoding {
    fn from(value: AccountEncoding) -> Self {
        match value {
            AccountEncoding::Base58 => RpcAccountEncoding::Base58,
            AccountEncoding::Base64 => RpcAccountEncoding::Base64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountDataSliceConfig {
    pub offset: usize,
    pub length: usize,
}

impl From<AccountDataSliceConfig> for RpcDataSliceConfig {
    fn from(value: AccountDataSliceConfig) -> Self {
        Self {
            offset: value.offset,
            length: value.length,
        }
    }
}

pub type AccountCommitmentConfig = CommitmentConfig;

#[derive(Debug, Clone, Default)]
pub struct GetProgramAccountsConfig {
    pub filters: Option<Vec<AccountFilter>>,
    pub encoding: Option<AccountEncoding>,
    pub data_slice: Option<AccountDataSliceConfig>,
    pub commitment: Option<AccountCommitmentConfig>,
    pub min_context_slot: Option<Slot>,
    pub with_context: Option<bool>,
}

impl GetProgramAccountsConfig {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_filters(mut self, filters: Vec<AccountFilter>) -> Result<Self> {
        log_trace!("filters: {filters:?}");
        self.filters = Some(filters);
        Ok(self)
    }

    pub fn encoding(mut self, encoding: AccountEncoding) -> Result<Self> {
        self.encoding = Some(encoding);
        Ok(self)
    }

    pub fn data_slice(mut self, data_slice: AccountDataSliceConfig) -> Result<Self> {
        self.data_slice = Some(data_slice);
        Ok(self)
    }

    pub fn commitment(mut self, commitment: AccountCommitmentConfig) -> Result<Self> {
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
}

#[cfg(target_arch = "wasm32")]
impl TryFrom<GetProgramAccountsConfig> for RpcProgramAccountsConfig {
    type Error = crate::error::Error;
    fn try_from(this: GetProgramAccountsConfig) -> Result<Self> {
        let mut config = RpcProgramAccountsConfig::new();
        if let Some(filters) = this.filters {
            let list = Array::new();
            for filter in filters {
                list.push(&filter.try_into()?);
            }

            config = config.add_filters(list)?;
        };

        if let Some(value) = this.encoding {
            config = config.encoding(value.into())?;
        }
        if let Some(data_slice) = this.data_slice {
            config = config.data_slice(data_slice.into())?;
        }
        if let Some(commitment) = this.commitment {
            config = config.commitment(commitment)?;
        }
        if let Some(min_context_slot) = this.min_context_slot {
            config = config.min_context_slot(min_context_slot)?;
        }

        Ok(config)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<GetProgramAccountsConfig> for RpcProgramAccountsConfig {
    type Error = crate::error::Error;
    fn try_from(this: GetProgramAccountsConfig) -> Result<Self> {
        let filters = match this.filters {
            Some(filters) => {
                let mut list = vec![];
                for filter in filters {
                    list.push(filter.try_into()?);
                }

                Some(list)
            }
            None => None,
        };

        let config = RpcProgramAccountsConfig {
            filters,
            account_config: RpcAccountInfoConfig {
                encoding: this.encoding.map(|e| e.into()),
                data_slice: this.data_slice.map(|e| e.into()),
                commitment: this.commitment,
                min_context_slot: this.min_context_slot,
            },
            ..Default::default()
        };

        Ok(config)
    }
}

#[derive(Debug, Clone)]
pub enum AccountFilter {
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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        fn create_memcmp_filter(
            holder: &Object,
            offset: usize,
            data: String,
            encoding: &str,
        ) -> Result<()> {
            let memcmp = Object::new();
            Reflect::set(&memcmp, &JsValue::from("offset"), &JsValue::from(offset))?;
            Reflect::set(&memcmp, &JsValue::from("bytes"), &JsValue::from(data))?;
            Reflect::set(
                &memcmp,
                &JsValue::from("encoding"),
                &JsValue::from(encoding),
            )?;
            Reflect::set(holder, &JsValue::from("memcmp"), &memcmp.into())?;

            Ok(())
        }

        impl TryFrom<AccountFilter> for JsValue {
            type Error = crate::error::Error;
            fn try_from(value: AccountFilter) -> Result<Self> {
                let obj = Object::new();
                match value {
                    AccountFilter::MemcmpEncodedBase58(offset, data) => {
                        create_memcmp_filter(&obj, offset, data, "base58")?;
                    }
                    AccountFilter::MemcmpEncodedBase64(offset, data) => {
                        create_memcmp_filter(&obj, offset, data, "base64")?;
                    }
                    AccountFilter::MemcmpEncodeBase58(offset, bytes) => {
                        let data = bs58::encode(bytes).into_string();
                        create_memcmp_filter(&obj, offset, data, "base58")?;
                    }
                    AccountFilter::MemcmpEncodeBase64(offset, bytes) => {
                        let data = base64::encode(bytes);
                        create_memcmp_filter(&obj, offset, data, "base64")?;
                    }
                    AccountFilter::DataSize(data_size) => {
                        Reflect::set(&obj, &JsValue::from("dataSize"), &JsValue::from(data_size))?;
                    }
                }

                Ok(obj.into())
            }
        }
    }else{
        impl TryFrom<AccountFilter> for RpcFilterType {
            type Error = crate::error::Error;
            fn try_from(filter: AccountFilter) -> std::result::Result<Self, Self::Error> {
                Ok(match filter {
                    AccountFilter::MemcmpEncodedBase58(offset, encoded_string) => {
                        //log_trace!("encoded_string: {encoded_string:?}");
                        let bytes = MemcmpEncodedBytes::Base58(encoded_string);
                        RpcFilterType::Memcmp(Memcmp::new(offset, bytes))
                    }
                    AccountFilter::MemcmpEncodedBase64(offset, encoded_string) => {
                        //log_trace!("encoded_string: {encoded_string:?}");
                        let bytes = MemcmpEncodedBytes::Base64(encoded_string);
                        RpcFilterType::Memcmp(Memcmp::new(offset, bytes))
                    }

                    AccountFilter::MemcmpEncodeBase58(offset, bytes) => {
                        //log_trace!("data: {bytes:?}");
                        RpcFilterType::Memcmp(Memcmp::new_base58_encoded(offset, &bytes))
                    }

                    AccountFilter::MemcmpEncodeBase64(offset, bytes) => {
                        //log_trace!("bytes: {bytes:?}");
                        let bytes = MemcmpEncodedBytes::Base64(base64::encode(bytes));
                        RpcFilterType::Memcmp(Memcmp::new(offset, bytes))
                    }

                    AccountFilter::DataSize(data_size) => {
                        RpcFilterType::DataSize(data_size as u64)
                    }
                })
            }
        }

    }
}
