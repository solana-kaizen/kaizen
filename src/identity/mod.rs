pub mod program;

#[cfg(not(target_os = "solana"))]
pub mod client;

#[cfg(not(target_arch = "wasm32"))]
pub mod tests;
