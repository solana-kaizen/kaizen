//!
//! Program and Application prelude containing general-purpose imports.
//!

pub use cfg_if::cfg_if;
pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use std::sync::Arc;

pub use borsh::*;
pub use solana_program::account_info::{AccountInfo, IntoAccountInfo};
pub use solana_program::entrypoint::ProcessInstruction;
pub use solana_program::entrypoint::ProgramResult;
pub use solana_program::instruction::{AccountMeta, Instruction};
pub use solana_program::program_error::ProgramError;
pub use solana_program::pubkey::Pubkey;
pub use solana_program::system_instruction::SystemInstruction;
pub use std::convert::TryFrom;
pub use std::convert::TryInto;

pub use crate::accounts::{Access, AllocationPayer, IsSigner, LamportAllocation};
pub use crate::address::AddressDomain;
pub use crate::container::array::Array;
pub use crate::container::collection::{
    PdaCollection, PdaCollectionInterface, PdaCollectionMeta, PdaCollectionReference,
    PdaProxyCollection, PdaProxyCollectionInterface, PdaProxyCollectionReference, PubkeyCollection,
    PubkeyCollectionMeta, PubkeyCollectionReference, PubkeyCollectionStore,
};
pub use crate::container::segment::{Layout, Segment, SegmentStore};
pub use crate::container::ContainerHeader;
pub use crate::context::{
    AccountAllocationArgs, Context, ContextReference, HandlerFn, HandlerFnCPtr,
};
pub use crate::date::*;
pub use crate::hash::PubkeyHashMap;
pub use crate::identity::program::Identity;
pub use crate::payload::Payload;
pub use crate::rent::RentCollector;

pub use workflow_log;
pub use workflow_log::{log_debug, log_error, log_info, log_trace, log_warning};

pub use kaizen::error::ErrorCode;
pub use kaizen::error_code;

// #[cfg(not(target_os = "solana"))]
// pub use crate::tokens::{get_tokens, get_tokens_info, get_tokens_info_array, TokenInfo};

pub use kaizen_macros::{
    container,
    declare_handlers,
    declare_interface,
    declare_program,
    // seal,
    Meta,
};

cfg_if::cfg_if! {
    if #[cfg(not(target_os = "solana"))] {

        pub use workflow_core::workflow_async_trait;

        pub use kaizen::builder::{
            Gather,
            InstructionBuilder,
            InstructionBuilderConfig,
        };
        pub use kaizen::pubkey::*;
        pub use kaizen::accounts::{AccountData,AccountDataReference};
        pub use kaizen::transport::*;

        pub use kaizen::sequencer::Sequencer;
        pub use kaizen::client::Client;
        pub use kaizen::container::{
            ContainerReference,
        };
        pub use kaizen_macros::declare_client;
        pub use async_std;
    }

}

cfg_if! {
    if #[cfg(not(any(target_os = "solana",target_arch = "wasm32")))] {
        pub use kaizen::inventory;
    }
}
