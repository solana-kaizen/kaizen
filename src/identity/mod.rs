//!
//! User Identity proxy account interface.
//!
//! User Identity allows:
//!     - Binding multiple user wallets to the same user account
//!     - Track various user properties via a single identity account
//!

pub mod program;

#[cfg(not(target_os = "solana"))]
pub mod client;

#[cfg(not(target_arch = "wasm32"))]
pub mod tests;
