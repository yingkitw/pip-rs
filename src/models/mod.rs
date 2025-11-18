/// Data models for packages, requirements, and metadata
pub mod package;
pub mod requirement;
pub mod metadata;

pub use package::Package;
pub use requirement::{Requirement, VersionSpec, VersionOp};
