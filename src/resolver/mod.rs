/// Dependency resolution
pub mod resolver;
pub mod extras;
pub mod lockfile;

pub use resolver::*;
pub use extras::{resolve_extras, get_available_extras};
pub use lockfile::{LockFile, LockedPackage};
