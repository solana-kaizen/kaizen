use {
    super::filter::RpcFilterType,
    serde::{Deserialize, Serialize},
    // solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig},
    solana_sdk::{
        //clock::{Epoch, Slot},
        clock::Slot,
        commitment_config::CommitmentConfig, // CommitmentLevel},
    },
    // solana_transaction_status::{TransactionDetails, UiTransactionEncoding},
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum UiAccountEncoding {
    Binary, // Legacy. Retained for RPC backwards compatibility
    Base58,
    Base64,
    JsonParsed,
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiDataSliceConfig {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountInfoConfig {
    pub encoding: Option<UiAccountEncoding>,
    pub data_slice: Option<UiDataSliceConfig>,
    #[serde(flatten)]
    pub commitment: Option<CommitmentConfig>,
    pub min_context_slot: Option<Slot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcProgramAccountsConfig {
    pub filters: Option<Vec<RpcFilterType>>,
    #[serde(flatten)]
    pub account_config: RpcAccountInfoConfig,
    pub with_context: Option<bool>,
}
