pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use std::sync::Arc;
pub use cfg_if::cfg_if;

pub use std::convert::TryInto;
pub use std::convert::TryFrom;
pub use solana_program::entrypoint::ProgramResult;
pub use solana_program::program_error::ProgramError;
pub use solana_program::account_info::{AccountInfo,IntoAccountInfo};
pub use solana_program::pubkey::Pubkey;
pub use solana_program::system_instruction::SystemInstruction;
pub use solana_program::instruction::{ Instruction, AccountMeta };
pub use solana_program::entrypoint::ProcessInstruction;

pub use crate::accounts::{ AllocationPayer,LamportAllocation,IsSigner,Access };
pub use crate::address::AddressDomain;
pub use crate::context::{ Context, ContextReference, HandlerFn, HandlerFnCPtr, AccountAllocationArgs };
pub use crate::payload::Payload;
pub use crate::rent::RentCollector;
pub use crate::hash::PubkeyHashMap;
pub use crate::date::*;
pub use crate::container::segment::{Segment, SegmentStore, Layout};
pub use crate::container::array::Array;
pub use crate::container::collection::{
    PubkeyCollection,
    PubkeyCollectionReference,
    PubkeyCollectionMeta,
    PubkeyCollectionStore,

    PdaCollectionInterface,
    PdaProxyCollectionInterface,
    PdaCollection,
    PdaCollectionReference,
    PdaProxyCollection,
    PdaProxyCollectionReference,
    PdaCollectionMeta,
};
pub use crate::container::ContainerHeader;
pub use crate::identity::program::Identity;

pub use workflow_log::{log_trace, log_info, log_debug, log_warning, log_error};
pub use workflow_log;

pub use workflow_allocator::error_code;
pub use workflow_allocator::error::ErrorCode;

// #[cfg(not(target_os = "solana"))]
// pub use crate::tokens::{get_tokens, get_tokens_info, get_tokens_info_array, TokenInfo};

pub use workflow_allocator_macros::{
    declare_handlers,
    declare_interface,
    declare_program,
    container,
    // seal,
    Meta,
};

cfg_if::cfg_if! {
    if #[cfg(not(target_os = "solana"))] {

        pub use workflow_core::workflow_async_trait;

        pub use workflow_allocator::builder::{
            Gather,
            InstructionBuilder,
            InstructionBuilderConfig,
        };
        pub use workflow_allocator::utils::generate_random_pubkey;
        pub use workflow_allocator::accounts::{AccountData,AccountDataReference};
        pub use workflow_allocator::transport::*;

        pub use workflow_allocator::sequencer::Sequencer;
        pub use workflow_allocator::client::Client;
        pub use workflow_allocator::container::{ 
            ContainerReference, 
        };
        pub use workflow_allocator_macros::declare_client;
        pub use async_std;
    }

}

cfg_if! {
    if #[cfg(not(any(target_os = "solana",target_arch = "wasm32")))] {
        pub use workflow_allocator::inventory;
    }
}