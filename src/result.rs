//! [`Result`] type used by the Kaizen framework (client-side)

pub type Result<T> = std::result::Result<T, crate::error::Error>;
