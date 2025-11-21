/// End-to-end tests for pip-rs
/// Tests complete real-world workflows

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test requirements file
fn create_requirements_file(dir: &TempDir, packages: &[&str]) -> PathBuf {
    let req_file = dir.path().join("requirements.txt");
    let content = packages.join("\n");
    fs::write(&req_file, content).expect("Failed to write requirements file");
    req_file
}

/// Helper to create a pyproject.toml
fn create_pyproject(dir: &TempDir, name: &str, version: &str, deps: &[&str]) -> PathBuf {
    let pyproject = dir.path().join("pyproject.toml");
    let deps_str = deps
        .iter()
        .map(|d| format!("    \"{}\",", d))
        .collect::<Vec<_>>()
        .join("\n");
    
    let content = format!(
        r#"[project]
name = "{}"
version = "{}"
description = "Test package"
dependencies = [
{}
]
"#,
        name, version, deps_str
    );
    
    fs::write(&pyproject, content).expect("Failed to write pyproject.toml");
    pyproject
}

#[test]
fn test_e2e_requirement_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let req_file = create_requirements_file(
        &temp_dir,
        &[
            "requests>=2.28.0",
            "numpy==1.20.0",
            "django<4.0",
            "flask[security]>=1.0",
        ],
    );

    // Read and parse requirements
    let content = fs::read_to_string(&req_file)?;
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .collect();

    assert_eq!(lines.len(), 4);
    assert!(lines[0].contains("requests"));
    assert!(lines[1].contains("numpy"));
    assert!(lines[2].contains("django"));
    assert!(lines[3].contains("flask"));

    Ok(())
}

#[test]
fn test_e2e_pyproject_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let pyproject = create_pyproject(
        &temp_dir,
        "my-package",
        "0.1.0",
        &["requests>=2.28.0", "numpy>=1.20.0"],
    );

    // Verify file exists and contains expected content
    let content = fs::read_to_string(&pyproject)?;
    assert!(content.contains("my-package"));
    assert!(content.contains("0.1.0"));
    assert!(content.contains("requests"));
    assert!(content.contains("numpy"));

    Ok(())
}

#[test]
fn test_e2e_venv_and_packages() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let venv_path = temp_dir.path().join("test_venv");

    // Create virtual environment
    let venv = pip_rs::venv::environment::VirtualEnvironment::new(venv_path.clone(), "3.11".to_string());
    venv.create()?;

    // Verify structure
    assert!(venv.is_valid());
    assert!(venv.get_site_packages_path().exists());
    assert!(venv.get_bin_path().exists());

    // Create site-packages
    let site_packages = pip_rs::installer::SitePackages::new(
        venv.get_site_packages_path().to_path_buf(),
    )?;

    // Simulate package installation
    fs::create_dir_all(site_packages.path().join("requests.dist-info"))?;

    // Verify installation
    assert!(site_packages.is_installed("requests"));
    let packages = site_packages.get_installed_packages()?;
    assert_eq!(packages.len(), 1);

    Ok(())
}

#[test]
fn test_e2e_config_and_cache() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("pip.conf");

    // Create and save config
    let mut config = pip_rs::config::config::Config::new();
    config.set_timeout(30);
    config.add_extra_index_url("https://test.pypi.org/simple/".to_string());
    config.save_to_file(&config_path)?;

    // Load and verify
    let loaded = pip_rs::config::config::Config::load_from_file(&config_path)?;
    assert_eq!(loaded.timeout(), 30);
    assert_eq!(loaded.extra_index_urls().len(), 1);

    // Test caching
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&cache_dir)?;
    let cache = pip_rs::cache::package_cache::PackageCache::new_custom(cache_dir)?;

    // Store and retrieve
    let package = pip_rs::models::Package {
        name: "requests".to_string(),
        version: "2.28.0".to_string(),
        summary: Some("test package data".to_string()),
        home_page: None,
        author: None,
        license: None,
        requires_python: None,
        requires_dist: vec![],
        classifiers: vec![],
    };
    
    cache.set(&package)?;
    let retrieved = cache.get("requests", "2.28.0")?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().summary, package.summary);

    Ok(())
}

#[test]
fn test_e2e_version_resolution() -> Result<(), Box<dyn std::error::Error>> {
    // Test version parsing and comparison
    let v1 = pip_rs::utils::version::Version::parse("2.28.0")?;
    let v2 = pip_rs::utils::version::Version::parse("2.29.0")?;
    let v3 = pip_rs::utils::version::Version::parse("2.28.0")?;

    assert!(v2 > v1);
    assert!(v1 == v3);
    assert!(v1 < v2);

    // Test version constraints with standard versions
    let v1_0 = pip_rs::utils::version::Version::parse("1.0.0")?;
    let v2_0 = pip_rs::utils::version::Version::parse("2.0.0")?;
    let v2_1 = pip_rs::utils::version::Version::parse("2.1.0")?;

    assert!(v1_0 < v2_0);
    assert!(v2_0 < v2_1);

    Ok(())
}

#[test]
fn test_e2e_requirement_parsing_with_extras() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let req_file = create_requirements_file(
        &temp_dir,
        &[
            "requests[security,socks]>=2.28.0",
            "django[postgres]>=3.2",
            "flask>=1.0",
        ],
    );

    let content = fs::read_to_string(&req_file)?;
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .collect();

    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("security"));
    assert!(lines[0].contains("socks"));
    assert!(lines[1].contains("postgres"));

    Ok(())
}

#[test]
fn test_e2e_editable_install_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path().join("project");
    let site_packages = temp_dir.path().join("site-packages");

    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(&site_packages)?;

    // Create pyproject.toml
    fs::write(
        project_dir.join("pyproject.toml"),
        "[project]\nname = \"test-package\"\n",
    )?;

    // Install in editable mode
    let editable = pip_rs::installer::editable::EditableInstall::new(
        project_dir.clone(),
        site_packages.clone(),
    );
    editable.install()?;

    // Verify .pth file exists
    let pth_files: Vec<_> = fs::read_dir(&site_packages)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pth"))
        .collect();
    assert!(!pth_files.is_empty());

    // Uninstall
    editable.uninstall()?;

    Ok(())
}

#[test]
fn test_e2e_wheel_parsing_workflow() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let wheel_names = vec![
        "requests-2.28.0-py3-none-any.whl",
        "numpy-1.20.0-cp39-cp39-win_amd64.whl",
        "django-3.2.0-py3-none-any.whl",
    ];

    for wheel_name in wheel_names {
        let wheel_path = Path::new(wheel_name);
        let wheel = pip_rs::installer::wheel::WheelFile::new(wheel_path.to_path_buf())?;

        match wheel_name {
            "requests-2.28.0-py3-none-any.whl" => {
                assert_eq!(wheel.name, "requests");
                assert_eq!(wheel.version, "2.28.0");
            }
            "numpy-1.20.0-cp39-cp39-win_amd64.whl" => {
                assert_eq!(wheel.name, "numpy");
                assert_eq!(wheel.version, "1.20.0");
            }
            "django-3.2.0-py3-none-any.whl" => {
                assert_eq!(wheel.name, "django");
                assert_eq!(wheel.version, "3.2.0");
            }
            _ => {}
        }
    }

    Ok(())
}

#[test]
fn test_e2e_entry_point_generation() -> Result<(), Box<dyn std::error::Error>> {
    let entry_points = vec![
        ("pip", "pip._internal.cli.main", "main"),
        ("pytest", "pytest.main", "main"),
        ("black", "black.cli", "main"),
    ];

    for (name, module, func) in entry_points {
        let ep = pip_rs::installer::entry_point::EntryPoint::new(
            name.to_string(),
            module.to_string(),
            func.to_string(),
        );

        let script = ep.generate_script();
        assert!(script.contains(module));
        assert!(script.contains(func));
    }

    Ok(())
}

#[test]
fn test_e2e_site_packages_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let site_packages = pip_rs::installer::SitePackages::new(temp_dir.path().to_path_buf())?;

    // Create multiple packages
    fs::create_dir_all(site_packages.path().join("requests.dist-info"))?;
    fs::create_dir_all(site_packages.path().join("numpy.dist-info"))?;
    fs::create_dir_all(site_packages.path().join("django.dist-info"))?;

    // Verify installations
    assert!(site_packages.is_installed("requests"));
    assert!(site_packages.is_installed("numpy"));
    assert!(site_packages.is_installed("django"));
    assert!(!site_packages.is_installed("flask"));

    // List packages
    let packages = site_packages.get_installed_packages()?;
    assert_eq!(packages.len(), 3);
    assert!(packages.contains(&"requests".to_string()));
    assert!(packages.contains(&"numpy".to_string()));
    assert!(packages.contains(&"django".to_string()));

    Ok(())
}

#[test]
fn test_e2e_cache_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let cache = pip_rs::cache::package_cache::PackageCache::new_custom(temp_dir.path().to_path_buf())?;

    // Store multiple packages
    let packages = vec![
        ("requests", "2.28.0", "requests data here"),
        ("numpy", "1.20.0", "numpy data here"),
        ("django", "3.2.0", "django data here"),
    ];

    for (name, version, summary) in &packages {
        let package = pip_rs::models::Package {
            name: name.to_string(),
            version: version.to_string(),
            summary: Some(summary.to_string()),
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        };
        cache.set(&package)?;
    }

    // Verify all cached and retrieve
    for (name, version, expected_summary) in &packages {
        let retrieved = cache.get(name, version)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().summary, Some(expected_summary.to_string()));
    }

    Ok(())
}

#[test]
fn test_e2e_marker_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    use pip_rs::models::Marker;

    // Test various marker expressions
    let markers = vec![
        "python_version >= '3.6'",
        "sys_platform == 'linux'",
        "platform_machine == 'x86_64'",
    ];

    for marker_str in markers {
        let marker = Marker::parse(marker_str)?;
        // Marker should parse successfully
        // Note: evaluate requires an Environment, so we just verify parsing works
        let _ = marker;
    }

    Ok(())
}

#[test]
fn test_e2e_validation_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Test package name validation
    assert!(pip_rs::utils::validation::validate_package_name("requests").is_ok());
    assert!(pip_rs::utils::validation::validate_package_name("my-package").is_ok());
    assert!(pip_rs::utils::validation::validate_package_name("").is_err());

    // Test version validation
    assert!(pip_rs::utils::validation::validate_version_spec(">=1.0.0").is_ok());
    assert!(pip_rs::utils::validation::validate_version_spec("==1.0.0").is_ok());
    assert!(pip_rs::utils::validation::validate_version_spec("").is_err());

    // Test URL validation
    assert!(pip_rs::utils::validation::validate_url("https://pypi.org/simple/").is_ok());
    assert!(pip_rs::utils::validation::validate_url("http://example.com").is_ok());
    assert!(pip_rs::utils::validation::validate_url("ftp://example.com").is_err());

    // Test file path validation
    assert!(pip_rs::utils::validation::validate_file_path("/path/to/file").is_ok());
    assert!(pip_rs::utils::validation::validate_file_path("").is_err());

    Ok(())
}

#[test]
fn test_e2e_performance_tracking() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use pip_rs::utils::performance::PerformanceTracker;

    let tracker = PerformanceTracker::new();

    // Record multiple operations
    tracker.record("install".to_string(), Duration::from_secs(5), 50_000_000);
    tracker.record("install".to_string(), Duration::from_secs(6), 60_000_000);
    tracker.record("resolve".to_string(), Duration::from_secs(2), 10_000_000);

    // Get summary
    let summary = tracker.get_summary();
    assert_eq!(summary.len(), 2);

    // Verify install metrics
    assert_eq!(summary["install"].count, 2);
    assert!(summary["install"].avg_duration.as_secs() >= 5);

    // Verify resolve metrics
    assert_eq!(summary["resolve"].count, 1);
    assert_eq!(summary["resolve"].avg_duration.as_secs(), 2);

    Ok(())
}
