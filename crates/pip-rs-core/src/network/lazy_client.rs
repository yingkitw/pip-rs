/// Lazy-initialized HTTP client for minimal startup overhead
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::time::Duration;

static GLOBAL_CLIENT: OnceCell<Client> = OnceCell::new();

/// Get or initialize the global HTTP client
/// This is lazy - the client is only created on first use
pub fn get_client() -> &'static Client {
    GLOBAL_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .user_agent("pip-rs/1.0")
            .build()
            .unwrap_or_else(|_| Client::new())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_client_initialization() {
        // First call initializes
        let client1 = get_client();
        // Second call returns same instance
        let client2 = get_client();
        // Should be the same pointer
        assert!(std::ptr::eq(client1, client2));
    }
}
