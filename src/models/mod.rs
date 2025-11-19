/// Data models for packages, requirements, and metadata
pub mod package;
pub mod requirement;
pub mod metadata;
pub mod marker;

pub use package::Package;
pub use requirement::{Requirement, VersionSpec, VersionOp};
pub use marker::{Marker, Environment};
