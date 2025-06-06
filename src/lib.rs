pub mod async_io;
pub mod net;

#[cfg(feature = "macros")]
pub mod macros;

pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;

#[cfg(feature = "macros")]
pub use macros::main;