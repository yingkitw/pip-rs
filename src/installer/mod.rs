/// Package installation logic
pub mod wheel;
pub mod installer;
pub mod site_packages;
pub mod entry_point;
pub mod editable;

pub use installer::PackageInstaller;
pub use site_packages::SitePackages;
pub use entry_point::EntryPoint;
pub use editable::EditableInstall;
