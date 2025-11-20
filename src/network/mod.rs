/// Network operations and PyPI communication
pub mod pypi;
pub mod client;
pub mod index;

pub use pypi::*;
pub use client::PackageClient;

use lazy_static::lazy_static;

lazy_static! {
    /// Global HTTP client for all PyPI requests
    /// Reuses connections and improves performance
    static ref GLOBAL_CLIENT: client::PackageClient = client::PackageClient::new();

    /// Global package cache
    static ref PACKAGE_CACHE: std::sync::Mutex<crate::cache::package_cache::PackageCache> = 
        std::sync::Mutex::new(crate::cache::package_cache::PackageCache::new().unwrap_or_default());
}
