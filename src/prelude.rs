pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use std::sync::Arc;

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
pub use crate::container::segment::{Segment, SegmentStore, Layout};
pub use crate::container::array::Array;
pub use crate::container::collection::{
    KeystoreCollection,
    OrderedCollectionMeta,
    OrderedCollectionStore,
    AccountCollection,
    AccountReferenceCollection,
    TsPubkey,
};
pub use crate::container::ContainerHeader;
pub use crate::identity::Identity;

pub use workflow_log::log_trace;

pub use workflow_allocator::error_code;
pub use workflow_allocator::error::ErrorCode;

// #[cfg(not(target_arch = "bpf"))]
// pub use crate::tokens::{get_tokens, get_tokens_info, get_tokens_info_array, TokenInfo};

pub use workflow_allocator_macros::{
    // describe_enum,
    declare_handlers,
    declare_interface,
    declare_program,
    container,
    // seal,
    Meta,
};

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {
        pub use workflow_allocator::builder::{
            InstructionBuilder,
            InstructionBuilderConfig,
            // CreateInstruction,
        };
        pub use workflow_allocator::utils::generate_random_pubkey;
        pub use workflow_allocator::accounts::{AccountData,AccountDataReference};
        pub use workflow_allocator::transport::*;

        pub use workflow_allocator::sequencer::Sequencer;
        // pub use workflow_allocator::identity::client::IdentityReference;
        pub use workflow_allocator::client::Client;
        pub use workflow_allocator::container::ContainerReference;
        pub use workflow_allocator_macros::declare_client;
        
    }

}
