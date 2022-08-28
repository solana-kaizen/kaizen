
use cfg_if::cfg_if;
use solana_program::clock::UnixTimestamp as SolanaUnixTimestamp;
use workflow_allocator::result::Result;
use borsh::*;

cfg_if! {
    if #[cfg(target_arch = "bpf")] {
        use solana_program::clock::Clock;
        use solana_program::sysvar::Sysvar;
    } else {
        use std::time::SystemTime;
    }
}

#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Instant(pub u64);

impl Instant {
    // pub const ZERO: Instant = Instant(0);

    pub fn now() -> Result<Instant> {

        cfg_if! {
            if #[cfg(target_arch = "bpf")] {
                let unix_timestamp = Clock::get()?.unix_timestamp;
            } else {
                let unix_timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
            }
        }

        Ok(Instant(unix_timestamp as u64))
    }

    pub fn elapsed_since(&self, timestamp : &Instant) -> Duration {
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

#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Duration(pub u64);

impl Duration {
    pub const HOUR: Duration = Duration(3600);
    pub const DAY: Duration = Duration(3600 * 24);
    pub const WEEK: Duration = Duration(3600 * 24 * 7);
}
// let clock = Clock::get()