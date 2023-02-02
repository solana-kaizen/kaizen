use borsh::*;
use cfg_if::cfg_if;
use kaizen::result::Result;
use serde::*;
use solana_program::clock::UnixTimestamp as SolanaUnixTimestamp;

cfg_if! {
    if #[cfg(target_os = "solana")] {
        use solana_program::clock::Clock;
        use solana_program::sysvar::Sysvar;
    } else if #[cfg(target_arch = "wasm32")] {
        use js_sys::Date;
    } else {
        use std::time::SystemTime;
    }
}

/// Instant-like struct compatible with Native, Wasm32, BPF platforms.
/// This structure keeps internal time in seconds and supports Borsh serialization.
#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy,
)]
#[repr(transparent)]
pub struct Instant(pub u64);

impl Instant {
    // pub const ZERO: Instant = Instant(0);

    pub fn now() -> Result<Instant> {
        cfg_if! {
            if #[cfg(target_os = "solana")] {
                let unix_timestamp = Clock::get()?.unix_timestamp;
                Ok(Instant(unix_timestamp as u64))
            } else if #[cfg(target_arch = "wasm32")] {
                let unix_timestamp = Date::now() / 1000.0;
                Ok(Instant(unix_timestamp as u64))
            } else {
                let unix_timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
                Ok(Instant(unix_timestamp))
            }
        }

        
    }

    pub fn elapsed_since(&self, timestamp: &Instant) -> Duration {
        Duration(timestamp.0 - self.0)
    }

    pub fn elapsed(&self) -> Result<Duration> {
        Ok(self.elapsed_since(&Instant::now()?))
    }
}

impl From<SolanaUnixTimestamp> for Instant {
    fn from(timestamp: SolanaUnixTimestamp) -> Self {
        Self(timestamp as u64)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Duration(pub u64);

impl Duration {
    pub const HOUR: Duration = Duration(3600);
    pub const DAY: Duration = Duration(3600 * 24);
    pub const WEEK: Duration = Duration(3600 * 24 * 7);
}
