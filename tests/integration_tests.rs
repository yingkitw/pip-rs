use tempfile::TempDir;

#[test]
fn test_venv_creation_and_activation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let venv_path = temp_dir.path().join("test_venv");

    // Create virtual environment
    let venv = pip_rs::venv::VirtualEnvironment::new(venv_path.clone(), "3.11".to_string());
    venv.create()?;

    // Verify structure
    assert!(venv.is_valid());
    assert!(venv.get_site_packages_path().exists());
    assert!(venv.get_bin_path().exists());

    // Generate activation script
    let activation = pip_rs::venv::ActivationScript::new(venv_path);
    let bash_script = activation.generate_bash();
    assert!(bash_script.contains("VIRTUAL_ENV="));
    assert!(bash_script.contains("deactivate"));

    Ok(())
}

#[test]
fn test_config_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("pip.conf");

    // Create and save config
    let mut config = pip_rs::config::Config::new();
    config.set_timeout(30);
    config.add_extra_index_url("https://test.pypi.org/simple/".to_string());
    config.save_to_file(&config_path)?;

    // Load and verify
    let loaded = pip_rs::config::Config::load_from_file(&config_path)?;
    assert_eq!(loaded.timeout(), 30);
    assert_eq!(loaded.extra_index_urls().len(), 1);

    Ok(())
}

#[test]
fn test_editable_install_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path().join("project");
    let site_packages = temp_dir.path().join("site-packages");

    std::fs::create_dir_all(&project_dir)?;
    std::fs::create_dir_all(&site_packages)?;

    // Create pyproject.toml
    std::fs::write(
        project_dir.join("pyproject.toml"),
        "[project]\nname = \"test-package\"\n",
    )?;

    // Install in editable mode
    let editable = pip_rs::installer::EditableInstall::new(project_dir.clone(), site_packages.clone());
    editable.install()?;

    // Verify .pth file exists
    let pth_files: Vec<_> = std::fs::read_dir(&site_packages)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pth"))
        .collect();
    assert!(!pth_files.is_empty());

    // Uninstall
    editable.uninstall()?;

    Ok(())
}

#[test]
fn test_requirement_parsing_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let requirements = vec![
        "requests>=2.28.0",
        "numpy==1.20.0",
        "django<4.0",
        "flask[security]>=1.0",
    ];

    for req_str in requirements {
        let req: pip_rs::models::Requirement = req_str.parse()?;
        assert!(!req.name.is_empty());
        assert!(!req.specs.is_empty());
    }

    Ok(())
}

#[test]
fn test_version_comparison_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let v1 = pip_rs::utils::Version::parse("2.28.0")?;
    let v2 = pip_rs::utils::Version::parse("2.29.0")?;
    let v3 = pip_rs::utils::Version::parse("2.28.0")?;

    assert!(v2 > v1);
    assert!(v1 == v3);
    assert!(v1 < v2);

    Ok(())
}

#[test]
fn test_wheel_parsing_workflow() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let wheel_name = "requests-2.28.0-py3-none-any.whl";
    let wheel_path = Path::new(wheel_name);

    let wheel = pip_rs::installer::wheel::WheelFile::new(wheel_path.to_path_buf())?;
    assert_eq!(wheel.name, "requests");
    assert_eq!(wheel.version, "2.28.0");

    Ok(())
}

#[test]
fn test_pyproject_parsing_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let pyproject_path = temp_dir.path().join("pyproject.toml");

    let content = r#"
[project]
name = "test-package"
version = "0.1.0"
description = "A test package"
dependencies = [
    "requests>=2.28.0",
    "numpy>=1.20.0",
]
"#;

    std::fs::write(&pyproject_path, content)?;
    let pyproject = pip_rs::config::PyProject::load(&pyproject_path)?;

    assert_eq!(pyproject.get_name(), Some("test-package".to_string()));
    assert_eq!(pyproject.get_version(), Some("0.1.0".to_string()));

    Ok(())
}

#[test]
fn test_site_packages_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let site_packages = pip_rs::installer::SitePackages::new(temp_dir.path().to_path_buf())?;

    // Create a dist-info directory
    std::fs::create_dir_all(site_packages.path().join("requests.dist-info"))?;

    // Verify installation
    assert!(site_packages.is_installed("requests"));
    assert!(!site_packages.is_installed("numpy"));

    // List packages
    let packages = site_packages.get_installed_packages()?;
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0], "requests");

    Ok(())
}

#[test]
fn test_cache_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let cache = pip_rs::cache::PackageCache::new(temp_dir.path().to_path_buf())?;

    // Store data
    let data = b"test package data";
    cache.store("requests", "2.28.0", data)?;

    // Verify cached
    assert!(cache.is_cached("requests", "2.28.0"));

    // Retrieve data
    let retrieved = cache.retrieve("requests", "2.28.0")?;
    assert_eq!(retrieved, data);

    Ok(())
}

#[test]
fn test_entry_point_generation_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let ep = pip_rs::installer::EntryPoint::new(
        "pip".to_string(),
        "pip._internal.cli.main".to_string(),
        "main".to_string(),
    );

    let script = ep.generate_script();
    assert!(script.contains("pip._internal.cli.main"));
    assert!(script.contains("main()"));

    Ok(())
}
