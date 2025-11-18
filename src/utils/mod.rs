/// Utility functions
pub mod version;
pub mod hash;

pub use version::Version;
pub use hash::{verify_hash, parse_hash_string};
