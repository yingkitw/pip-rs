/// Network operations and PyPI communication
pub mod pypi;
pub mod client;
pub mod index;
pub mod lazy_client;

pub use pypi::*;
pub use client::PackageClient;
pub use lazy_client::get_client;

use once_cell::sync::Lazy;

/// Global HTTP client for all PyPI requests - lazily initialized
/// This avoids startup overhead when commands don't need network access
static GLOBAL_CLIENT: Lazy<client::PackageClient> = Lazy::new(client::PackageClient::new);

/// Global package cache - lazily initialized
static PACKAGE_CACHE: Lazy<std::sync::Mutex<crate::cache::package_cache::PackageCache>> = 
    Lazy::new(|| std::sync::Mutex::new(crate::cache::package_cache::PackageCache::new().unwrap_or_default()));

/// Get the global package client (lazy initialization)
pub fn global_client() -> &'static client::PackageClient {
    &GLOBAL_CLIENT
}

/// Get the global package cache (lazy initialization)
pub fn global_cache() -> &'static std::sync::Mutex<crate::cache::package_cache::PackageCache> {
    &PACKAGE_CACHE
}
