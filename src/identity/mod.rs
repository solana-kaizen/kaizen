pub mod program;

#[cfg(not(target_arch = "bpf"))]
pub mod client;

#[cfg(not(target_arch = "wasm32"))]
pub mod tests;
