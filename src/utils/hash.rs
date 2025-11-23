/// Hash verification utilities
use anyhow::Result;
use std::path::Path;
use std::fs::File;
use std::io::Read;

/// Verify file hash using the specified algorithm
pub async fn verify_hash(file_path: &Path, expected_hash: &str, algorithm: &str) -> Result<bool> {
    let computed_hash = compute_hash(file_path, algorithm).await?;
    Ok(computed_hash.eq_ignore_ascii_case(expected_hash))
}

/// Compute hash of a file
pub async fn compute_hash(file_path: &Path, algorithm: &str) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    match algorithm.to_lowercase().as_str() {
        "sha256" => {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            let result = hasher.finalize();
            Ok(format!("{:x}", result))
        }
        "sha1" => {
            use sha1::{Sha1, Digest};
            let mut hasher = Sha1::new();
            hasher.update(&buffer);
            let result = hasher.finalize();
            Ok(format!("{:x}", result))
        }
        "md5" => {
            use md5;
            let digest = md5::compute(&buffer);
            Ok(format!("{:x}", digest))
        }
        _ => Err(anyhow::anyhow!("Unsupported hash algorithm: {}", algorithm)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_verify_hash_sha256() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        
        // Compute the actual hash first
        let computed = compute_hash(temp_file.path(), "sha256").await.unwrap();
        
        // Verify with the computed hash
        let result = verify_hash(temp_file.path(), &computed, "sha256").await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_verify_hash_invalid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        
        let result = verify_hash(temp_file.path(), "invalid_hash", "sha256").await.unwrap();
        assert!(!result);
    }
}

#[allow(dead_code)]
pub fn parse_hash_string(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid hash format"));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
