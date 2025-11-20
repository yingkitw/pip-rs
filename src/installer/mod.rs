/// Package installation logic
pub mod wheel;
pub mod installer;
pub mod site_packages;
#[allow(dead_code)]
pub mod entry_point;
#[allow(dead_code)]
pub mod editable;

pub use installer::PackageInstaller;
pub use site_packages::SitePackages;
