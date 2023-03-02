#[cfg(target_arch = "wasm32")]
pub use solana_web3_sys::prelude::*;

#[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
mod native;

#[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
pub use native::*;
