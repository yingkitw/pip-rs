/// Performance benchmarks for pip-rs
use std::time::Instant;

fn benchmark_version_parsing() {
    let versions = vec![
        "1.0.0",
        "2.28.0",
        "3.11.5",
        "0.0.1",
        "10.20.30",
    ];

    let start = Instant::now();
    for _ in 0..1000 {
        for v in &versions {
            let _ = pip_rs::utils::Version::parse(v);
        }
    }
    let elapsed = start.elapsed();

    println!("Version parsing (5000 iterations): {:?}", elapsed);
    println!("Average per parse: {:?}", elapsed / 5000);
}

fn benchmark_requirement_parsing() {
    let requirements = vec![
        "requests>=2.28.0",
        "numpy==1.20.0",
        "django<4.0",
        "flask[security]>=1.0",
        "pytest>=6.0,<7.0",
    ];

    let start = Instant::now();
    for _ in 0..1000 {
        for req in &requirements {
            let _ = req.parse::<pip_rs::models::Requirement>();
        }
    }
    let elapsed = start.elapsed();

    println!("Requirement parsing (5000 iterations): {:?}", elapsed);
    println!("Average per parse: {:?}", elapsed / 5000);
}

fn benchmark_config_creation() {
    let start = Instant::now();
    for _ in 0..1000 {
        let mut config = pip_rs::config::Config::new();
        config.set_timeout(30);
        config.add_extra_index_url("https://test.pypi.org/simple/".to_string());
    }
    let elapsed = start.elapsed();

    println!("Config creation (1000 iterations): {:?}", elapsed);
    println!("Average per creation: {:?}", elapsed / 1000);
}

fn benchmark_venv_structure() {
    let start = Instant::now();
    for i in 0..100 {
        let path = std::path::PathBuf::from(format!("/tmp/venv{}", i));
        let venv = pip_rs::venv::VirtualEnvironment::new(path, "3.11".to_string());
        let _ = venv.get_site_packages_path();
        let _ = venv.get_bin_path();
    }
    let elapsed = start.elapsed();

    println!("Virtual environment path operations (100 iterations): {:?}", elapsed);
    println!("Average per operation: {:?}", elapsed / 100);
}

fn main() {
    println!("=== pip-rs Performance Benchmarks ===\n");

    println!("--- Version Parsing ---");
    benchmark_version_parsing();

    println!("\n--- Requirement Parsing ---");
    benchmark_requirement_parsing();

    println!("\n--- Configuration Creation ---");
    benchmark_config_creation();

    println!("\n--- Virtual Environment Structure ---");
    benchmark_venv_structure();

    println!("\n=== Benchmarks Complete ===");
}
