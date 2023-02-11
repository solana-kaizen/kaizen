/*!

[<img alt="github" src="https://img.shields.io/badge/github-solana--kaizen/kaizen-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/solana-kaizen/kaizen)
[<img alt="crates.io" src="https://img.shields.io/crates/v/kaizen.svg?maxAge=2592000&style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/kaizen)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-kaizen-56c2a5?maxAge=2592000&style=for-the-badge&logo=rust" height="20">](https://docs.rs/kaizen)
<img alt="license" src="https://img.shields.io/crates/l/kaizen.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">
<img src="https://img.shields.io/badge/platform-native-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32/browser-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32/node.js-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-solana_os-informational?style=for-the-badge&color=50a0f0" height="20">

Solana OS Rust framework for platform-neutral application development.

 */

pub use cfg_if::cfg_if;

extern crate self as kaizen;

pub mod macros {
    //! Macros available via the Kaizen framework
    pub use kaizen_macros::*;
}

pub mod accounts;
pub mod address;
pub mod container;
pub mod context;
pub mod date;
pub mod error;
pub mod hash;
pub mod identity;
pub mod instruction;
pub mod payload;
pub mod prelude;
pub mod program;
pub mod realloc;
pub mod rent;
pub mod result;
pub mod time;
pub mod utils;

// pub use utils::generate_random_pubkey;

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
        pub fn init_kaizen(workflow: &JsValue, solana: &JsValue, mods:&JsValue) -> crate::result::Result<()> {

            crate::wasm::init_kaizen(workflow, solana, mods)?;
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
        /// [`inventory`] is used to register application containers in a client-side or kaizen emulator environment.
        pub use inventory;
    }
}
