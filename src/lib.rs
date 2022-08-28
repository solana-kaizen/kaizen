// extern crate self as allocator;
extern crate self as workflow_allocator;

pub mod macros {
    pub use workflow_allocator_macros::*;
}

// pub mod console;
pub mod error;
pub mod result;
pub mod address;
pub mod realloc;
pub mod accounts;
pub mod container;
pub mod utils;
pub mod rent;
pub mod time;
pub mod hash;
pub mod payload;
pub mod context;
pub mod program;
pub mod instruction;
// pub mod enums;
pub mod prelude;
pub mod btree;
// pub mod macros;
// pub mod log;
pub mod identity;
pub mod pgp;

pub use utils::generate_random_pubkey;

// #[cfg(not(any(target_arch = "bpf", target_arch = "wasm32")))]
// pub mod fsio;

// #[cfg(not(target_arch = "bpf"))]
// pub mod i18n;

// #[cfg(target_arch = "wasm32")]
#[cfg(not(target_arch = "bpf"))]
pub mod wasm;

// #[cfg(not(target_arch = "bpf"))]
// pub mod task;

#[cfg(not(target_arch = "bpf"))]
pub mod builder;

#[cfg(not(target_arch = "bpf"))]
pub mod client;

#[cfg(not(target_arch = "bpf"))]
pub mod transport;

// // #[cfg(all(feature = "websockets", not(target_arch = "bpf")))]
// #[cfg(not(target_arch = "bpf"))]
// // #[cfg(target_arch = "wasm32")]
// pub mod websocket;

// // #[cfg(target_arch = "wasm32")]
// #[cfg(not(target_arch = "bpf"))]
// pub mod rpc;

#[cfg(not(target_arch = "bpf"))]
pub mod store;

#[cfg(not(target_arch = "bpf"))]
pub mod cache;

// #[cfg(not(target_arch = "bpf"))]
// pub mod tokens;

#[cfg(target_arch = "bpf")]
pub mod solana;
#[cfg(target_arch = "bpf")]
pub use solana::{
    allocate_pda,
    allocate_multiple_pda,
    transfer_sol,
    transfer_spl,
};

#[cfg(not(target_arch = "bpf"))]
pub mod emulator;
#[cfg(not(target_arch = "bpf"))]
pub use emulator::{
    allocate_pda,
    allocate_multiple_pda,
    transfer_sol,
    transfer_spl,
};

#[cfg(not(target_arch = "bpf"))]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn bind(workflow: &JsValue, solana: &JsValue) -> crate::result::Result<()> {
        
    crate::wasm::bind(workflow,solana)?;
    crate::program::registry::wasm::load_program_registry(workflow)?;
    crate::container::registry::wasm::load_container_registry(workflow)?;

    Ok(())
}

#[cfg(not(any(target_arch = "bpf", target_arch = "wasm32")))]
pub fn init() -> crate::result::Result<()> {

    crate::program::registry::init()?;
    crate::container::registry::init()?;

    Ok(())
}