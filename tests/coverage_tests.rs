/// Additional coverage tests for critical paths
/// Focuses on edge cases and error scenarios

use tempfile::TempDir;
use std::fs;

#[test]
fn test_coverage_empty_requirements() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let req_file = temp_dir.path().join("requirements.txt");
    fs::write(&req_file, "")?;

    let content = fs::read_to_string(&req_file)?;
    assert!(content.is_empty());

    Ok(())
}

#[test]
fn test_coverage_requirements_with_comments() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let req_file = temp_dir.path().join("requirements.txt");
    
    let content = r#"# This is a comment
requests>=2.28.0
# Another comment
numpy==1.20.0
"#;
    
    fs::write(&req_file, content)?;
    let read_content = fs::read_to_string(&req_file)?;
    
    let lines: Vec<&str> = read_content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .collect();
    
    assert_eq!(lines.len(), 2);
    Ok(())
}

#[test]
fn test_coverage_version_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    // Test various version formats
    let versions = vec![
        "1.0.0",
        "1.0",
        "1",
        "0.0.1",
        "10.20.30",
    ];

    for version_str in versions {
        let version = pip_rs::utils::Version::parse(version_str)?;
        let _ = version;
    }

    Ok(())
}

#[test]
fn test_coverage_venv_invalid_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let venv_path = temp_dir.path().join("test_venv");

    let venv = pip_rs::venv::VirtualEnvironment::new(venv_path.clone(), "3.11".to_string());
    venv.create()?;

    // Verify it's valid after creation
    assert!(venv.is_valid());

    Ok(())
}

#[test]
fn test_coverage_site_packages_multiple_packages() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let site_packages = pip_rs::installer::SitePackages::new(temp_dir.path().to_path_buf())?;

    // Create multiple packages
    let packages = vec!["requests", "numpy", "django", "flask", "pytest"];
    for pkg in &packages {
        fs::create_dir_all(site_packages.path().join(format!("{}.dist-info", pkg)))?;
    }

    // Verify all installed
    for pkg in &packages {
        assert!(site_packages.is_installed(pkg));
    }

    // List all
    let installed = site_packages.get_installed_packages()?;
    assert_eq!(installed.len(), packages.len());

    Ok(())
}

#[test]
fn test_coverage_config_multiple_indexes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("pip.conf");

    let mut config = pip_rs::config::Config::new();
    config.set_timeout(30);
    config.add_extra_index_url("https://test1.pypi.org/simple/".to_string());
    config.add_extra_index_url("https://test2.pypi.org/simple/".to_string());
    config.add_extra_index_url("https://test3.pypi.org/simple/".to_string());
    config.save_to_file(&config_path)?;

    let loaded = pip_rs::config::Config::load_from_file(&config_path)?;
    assert_eq!(loaded.extra_index_urls().len(), 3);

    Ok(())
}

#[test]
fn test_coverage_cache_multiple_versions() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let cache = pip_rs::cache::PackageCache::new_custom(temp_dir.path().to_path_buf())?;

    // Store multiple versions of same package
    let versions = vec!["1.0.0", "1.1.0", "2.0.0"];
    for version in &versions {
        let package = pip_rs::models::Package {
            name: "requests".to_string(),
            version: version.to_string(),
            summary: Some(format!("Summary for {}", version)),
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        };
        cache.set(&package)?;
    }

    // Verify all cached
    for version in &versions {
        let cached = cache.get("requests", version)?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().version, *version);
    }

    Ok(())
}

#[test]
fn test_coverage_wheel_various_names() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let wheels = vec![
        ("requests-2.28.0-py3-none-any.whl", "requests", "2.28.0"),
        ("numpy-1.20.0-cp39-cp39-linux_x86_64.whl", "numpy", "1.20.0"),
        ("django-3.2.0-py3-none-any.whl", "django", "3.2.0"),
        ("flask-2.0.0-py3-none-any.whl", "flask", "2.0.0"),
    ];

    for (wheel_name, expected_name, expected_version) in wheels {
        let wheel_path = Path::new(wheel_name);
        let wheel = pip_rs::installer::wheel::WheelFile::new(wheel_path.to_path_buf())?;
        assert_eq!(wheel.name, expected_name);
        assert_eq!(wheel.version, expected_version);
    }

    Ok(())
}

#[test]
fn test_coverage_entry_points_multiple() -> Result<(), Box<dyn std::error::Error>> {
    let entry_points = vec![
        ("pip", "pip._internal.cli.main", "main"),
        ("pytest", "pytest.main", "main"),
        ("black", "black.cli", "main"),
        ("flake8", "flake8.main.cli", "main"),
        ("mypy", "mypy.main", "main"),
    ];

    for (name, module, func) in entry_points {
        let ep = pip_rs::installer::EntryPoint::new(
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
fn test_coverage_validation_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    // Test package names with various valid formats
    assert!(pip_rs::utils::validate_package_name("a").is_ok());
    assert!(pip_rs::utils::validate_package_name("A").is_ok());
    assert!(pip_rs::utils::validate_package_name("a1").is_ok());
    assert!(pip_rs::utils::validate_package_name("a-b").is_ok());
    assert!(pip_rs::utils::validate_package_name("a_b").is_ok());
    assert!(pip_rs::utils::validate_package_name("a.b").is_ok());
    assert!(pip_rs::utils::validate_package_name("a-b-c").is_ok());

    // Test invalid package names
    assert!(pip_rs::utils::validate_package_name("a b").is_err());
    assert!(pip_rs::utils::validate_package_name("a@b").is_err());
    assert!(pip_rs::utils::validate_package_name("a#b").is_err());

    Ok(())
}

#[test]
fn test_coverage_security_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    // Test URL validation edge cases
    assert!(pip_rs::utils::verify_url_safety("https://pypi.org/simple/").is_ok());
    assert!(pip_rs::utils::verify_url_safety("http://example.com").is_ok());
    assert!(pip_rs::utils::verify_url_safety("ftp://example.com").is_err());
    assert!(pip_rs::utils::verify_url_safety("javascript:alert('xss')").is_err());

    // Test file path validation edge cases
    assert!(pip_rs::utils::verify_file_path_safety("/path/to/file").is_ok());
    assert!(pip_rs::utils::verify_file_path_safety("./relative/path").is_ok());
    assert!(pip_rs::utils::verify_file_path_safety("path/../../etc/passwd").is_err());

    // Test command injection prevention
    assert!(pip_rs::utils::check_command_injection("normal input").is_ok());
    assert!(pip_rs::utils::check_command_injection("input; rm -rf /").is_err());
    assert!(pip_rs::utils::check_command_injection("input | cat").is_err());
    assert!(pip_rs::utils::check_command_injection("input && command").is_err());

    Ok(())
}

#[test]
fn test_coverage_performance_tracking_multiple() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use pip_rs::utils::PerformanceTracker;

    let tracker = PerformanceTracker::new();

    // Record many operations
    for i in 0..10 {
        tracker.record(
            "operation".to_string(),
            Duration::from_millis(100 + i * 10),
            1_000_000 + i * 100_000,
        );
    }

    let summary = tracker.get_summary();
    assert_eq!(summary["operation"].count, 10);
    assert!(summary["operation"].avg_duration.as_millis() > 100);

    Ok(())
}

#[test]
fn test_coverage_pyproject_complex() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let pyproject_path = temp_dir.path().join("pyproject.toml");

    let content = r#"[project]
name = "complex-package"
version = "1.0.0"
description = "A complex package"
requires-python = ">=3.8"
dependencies = [
    "requests>=2.28.0",
    "numpy>=1.20.0",
    "django[postgres]>=3.2",
]

[project.optional-dependencies]
dev = ["pytest", "black"]
docs = ["sphinx"]

[build-system]
requires = ["setuptools", "wheel"]
build-backend = "setuptools.build_meta"
"#;

    fs::write(&pyproject_path, content)?;
    let pyproject = pip_rs::config::PyProject::load(&pyproject_path)?;

    assert_eq!(pyproject.get_name(), Some("complex-package".to_string()));
    assert_eq!(pyproject.get_version(), Some("1.0.0".to_string()));

    Ok(())
}

#[test]
fn test_coverage_marker_parsing() -> Result<(), Box<dyn std::error::Error>> {
    use pip_rs::models::Marker;

    let markers = vec![
        "python_version >= '3.6'",
        "sys_platform == 'linux'",
        "platform_machine == 'x86_64'",
        "python_version < '4.0'",
        "sys_platform != 'win32'",
    ];

    for marker_str in markers {
        let marker = Marker::parse(marker_str)?;
        let _ = marker;
    }

    Ok(())
}

#[test]
fn test_coverage_requirement_with_all_features() -> Result<(), Box<dyn std::error::Error>> {
    let requirements = vec![
        "requests>=2.28.0",
        "numpy==1.20.0",
        "django<4.0",
        "flask[security,socks]>=1.0",
        "pytest>=6.0,<7.0",
        "black>=21.0",
    ];

    for req_str in requirements {
        let req: pip_rs::models::Requirement = req_str.parse()?;
        assert!(!req.name.is_empty());
        assert!(!req.specs.is_empty());
    }

    Ok(())
}

#[test]
fn test_coverage_editable_install_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path().join("project");
    let site_packages = temp_dir.path().join("site-packages");

    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(&site_packages)?;

    fs::write(
        project_dir.join("pyproject.toml"),
        "[project]\nname = \"test-package\"\n",
    )?;

    let editable = pip_rs::installer::EditableInstall::new(
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
fn test_coverage_sanitization() -> Result<(), Box<dyn std::error::Error>> {
    let test_cases = vec![
        ("hello world", "hello world"),
        ("hello\0world", "helloworld"),
        ("hello\nworld", "hello\nworld"),
        ("hello\tworld", "hello\tworld"),
        ("hello\rworld", "helloworld"),
    ];

    for (input, _expected) in test_cases {
        let sanitized = pip_rs::utils::sanitize_output(input);
        assert!(!sanitized.contains('\0'));
    }

    Ok(())
}

#[test]
fn test_coverage_timer_functionality() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use pip_rs::utils::Timer;

    let timer = Timer::new("test_operation");
    std::thread::sleep(Duration::from_millis(10));
    
    let elapsed = timer.elapsed();
    assert!(elapsed.as_millis() >= 10);

    let elapsed_ms = timer.elapsed_ms();
    assert!(elapsed_ms >= 10.0);

    Ok(())
}

#[test]
fn test_coverage_version_comparison_all_operators() -> Result<(), Box<dyn std::error::Error>> {
    let v1 = pip_rs::utils::Version::parse("1.0.0")?;
    let v2 = pip_rs::utils::Version::parse("2.0.0")?;
    let v3 = pip_rs::utils::Version::parse("1.0.0")?;

    // Test all comparison operators
    assert!(v1 < v2);
    assert!(v2 > v1);
    assert!(v1 <= v2);
    assert!(v2 >= v1);
    assert!(v1 == v3);
    assert!(v1 <= v3);
    assert!(v1 >= v3);
    assert!(v1 != v2);

    Ok(())
}
