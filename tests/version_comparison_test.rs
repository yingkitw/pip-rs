/// Test to verify PEP 440 version comparison works correctly
use std::cmp::Ordering;

// Import the compare_versions function from the library
use pip_rs::commands::upgrade::detector::compare_versions;

#[test]
fn test_pep440_prerelease_comparison() {
    // Pre-releases should be less than final releases
    assert_eq!(compare_versions("1.0.0a1", "1.0.0"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0b1", "1.0.0"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0rc1", "1.0.0"), Ordering::Less);
    
    // Final release should be greater than pre-releases
    assert_eq!(compare_versions("1.0.0", "1.0.0a1"), Ordering::Greater);
    assert_eq!(compare_versions("1.0.0", "1.0.0b1"), Ordering::Greater);
    assert_eq!(compare_versions("1.0.0", "1.0.0rc1"), Ordering::Greater);
}

#[test]
fn test_pep440_postrelease_comparison() {
    // Post-releases should be greater than final releases
    assert_eq!(compare_versions("1.0.0", "1.0.0.post1"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0.post1", "1.0.0"), Ordering::Greater);
}

#[test]
fn test_pep440_dev_comparison() {
    // Dev releases should be less than final releases
    assert_eq!(compare_versions("1.0.0.dev1", "1.0.0"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0", "1.0.0.dev1"), Ordering::Greater);
}

#[test]
fn test_pep440_prerelease_ordering() {
    // Pre-releases should be ordered: alpha < beta < rc
    assert_eq!(compare_versions("1.0.0a1", "1.0.0b1"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0b1", "1.0.0rc1"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0a1", "1.0.0rc1"), Ordering::Less);
}

#[test]
fn test_basic_version_comparison() {
    // Basic version comparisons should still work
    assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
    assert_eq!(compare_versions("1.0.0", "1.0.1"), Ordering::Less);
    assert_eq!(compare_versions("1.0.1", "1.0.0"), Ordering::Greater);
    assert_eq!(compare_versions("1.0", "1.0.0"), Ordering::Equal);
    assert_eq!(compare_versions("1.9.9", "1.10.0"), Ordering::Less);
}

#[test]
fn test_complex_version_comparison() {
    // Complex version strings
    assert_eq!(compare_versions("2024.1.0", "2024.1.0a1"), Ordering::Greater);
    assert_eq!(compare_versions("2024.1.0a1", "2024.1.0"), Ordering::Less);
}

#[test]
fn test_version_with_multiple_prerelease_numbers() {
    // Multiple pre-release numbers
    assert_eq!(compare_versions("1.0.0a1", "1.0.0a2"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0rc1", "1.0.0rc2"), Ordering::Less);
}
