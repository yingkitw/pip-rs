/// Package installation logic
pub mod wheel;
pub mod installer;
pub mod site_packages;
#[allow(dead_code)]
pub mod entry_point;
#[allow(dead_code)]
pub mod editable;
pub mod editable_cache;
pub mod egg_link_handler;

pub use installer::PackageInstaller;
pub use site_packages::SitePackages;
pub use editable_cache::EditableCache;
pub use egg_link_handler::{EggLinkInfo, EggLinkHandler};
