/// Dependency resolution
pub mod resolver;
pub mod extras;
pub mod lockfile;
pub mod dependency_cache;
pub mod direct_url;
pub mod candidate_selector;

pub use resolver::*;
pub use lockfile::LockFile;
pub use dependency_cache::DependencyCache;
pub use direct_url::{DirectUrl, DirectUrlType, DirectUrlConflictDetector};
pub use candidate_selector::{CandidateSelector, SelectionStrategy, Candidate};
