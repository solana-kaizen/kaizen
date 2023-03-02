use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(target_os = "solana"))] {
        mod components;
        pub use components::*;
    }
}
