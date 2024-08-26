#[cfg(feature="hot")]
pub mod hot;
#[cfg(not(feature="hot"))]
pub mod not_hot;

#[cfg(feature="hot")]
pub use hot::watch_constants;
#[cfg(not(feature="hot"))]
pub use not_hot::watch_constants;

#[cfg(feature="hot")]
pub extern crate linkme;

#[cfg(feature="hot")]
pub use linkme::distributed_slice as hot_distributed_slice;