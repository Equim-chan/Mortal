mod akochan;
mod batchify;
mod mortal;
mod traits;
#[cfg(test)]
mod tsumogiri;

pub use akochan::AkochanAgent;
pub use batchify::BatchifiedAgent;
pub use mortal::MortalBatchAgent;
pub use traits::{Agent, BatchAgent};
#[cfg(test)]
pub use tsumogiri::Tsumogiri;
