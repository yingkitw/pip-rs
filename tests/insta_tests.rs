use insta::assert_debug_snapshot;
use pip_rs::commands::upgrade::detector::compare_versions;
use std::cmp::Ordering;

#[test]
fn test_version_comparisons_snapshot() {
    let versions = vec![
        ("1.0.0", "1.0.0"),
        ("1.0.0", "1.0.1"),
        ("1.0.0", "1.0.0a1"),
        ("1.0.0", "1.0.0.post1"),
        ("1.0.0a1", "1.0.0b1"),
        ("1.0.0b1", "1.0.0rc1"),
    ];

    let results: Vec<((&str, &str), Ordering)> = versions
        .into_iter()
        .map(|(v1, v2)| ((v1, v2), compare_versions(v1, v2)))
        .collect();

    assert_debug_snapshot!(results);
}
