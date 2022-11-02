use cfg_if::cfg_if;

extern crate self as workflow_allocator;

pub mod macros {
    pub use workflow_allocator_macros::*;
}

pub mod error;
pub mod result;
pub mod address;
pub mod realloc;
pub mod accounts;
pub mod container;
pub mod utils;
pub mod rent;
pub mod time;
pub mod date;
pub mod hash;
pub mod payload;
pub mod context;
pub mod program;
pub mod instruction;
pub mod prelude;
pub mod identity;
// pub mod btree;
// pub mod pgp;

pub use utils::generate_random_pubkey;

cfg_if! {
    if #[cfg(not(target_os = "solana"))] {
        pub mod wasm;
        pub mod builder;
        pub mod sequencer;
        pub mod client;
        pub mod wallet;
        pub mod transport;
        pub mod store;
        pub mod cache;
        pub mod user;

        #[allow(unused_imports)]
        use wasm_bindgen::prelude::*;
    }
}

cfg_if! {
    if #[cfg(target_os = "solana")] {
        pub mod solana;
        pub use solana::{
            allocate_pda,
            allocate_multiple_pda,
            transfer_sol,
            transfer_spl,
        };
    } else {
        pub mod emulator;
        pub use emulator::{
            allocate_pda,
            allocate_multiple_pda,
            transfer_sol,
            transfer_spl,
        };
    }
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        
        #[wasm_bindgen]
        pub fn bind(workflow: &JsValue, solana: &JsValue, mods:&JsValue) -> crate::result::Result<()> {
                
            crate::wasm::bind(workflow, solana, mods)?;
            crate::program::registry::wasm::load_program_registry(workflow)?;
            crate::container::registry::wasm::load_container_registry(workflow)?;
        
            Ok(())
        }

    } else if #[cfg(not(target_os = "solana"))] {

        pub fn init() -> crate::result::Result<()> {
        
            crate::program::registry::init()?;
            crate::container::registry::init()?;
        
            Ok(())
        }

    }
}

cfg_if! {
    if #[cfg(not(any(target_os = "solana",target_arch = "wasm32")))] {
        pub use inventory;
    }
}