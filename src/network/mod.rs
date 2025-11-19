/// Network operations and PyPI communication
pub mod pypi;
pub mod client;
pub mod index;

pub use pypi::*;
pub use client::PackageClient;
pub use index::{IndexManager, IndexConfig, parse_index_config};

use lazy_static::lazy_static;

lazy_static! {
    /// Global HTTP client for all PyPI requests
    /// Reuses connections and improves performance
    static ref GLOBAL_CLIENT: client::PackageClient = client::PackageClient::new();
}
