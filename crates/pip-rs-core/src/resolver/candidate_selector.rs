/// Candidate selection logic for dependency resolution
/// 
/// This module implements improved candidate selection that reuses
/// already installed candidates when appropriate.

use std::collections::HashMap;
use crate::models::Package;

/// Candidate information
#[derive(Clone, Debug)]
pub struct Candidate {
    pub package: Package,
    pub is_installed: bool,
    pub is_editable: bool,
    pub link_url: Option<String>,
}

/// Candidate selection strategy
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionStrategy {
    /// Prefer installed candidates
    PreferInstalled,
    /// Prefer latest version
    PreferLatest,
    /// Prefer compatible version
    PreferCompatible,
}

/// Candidate selector for improved resolution
pub struct CandidateSelector {
    installed_candidates: HashMap<String, Candidate>,
    strategy: SelectionStrategy,
}

impl CandidateSelector {
    pub fn new(strategy: SelectionStrategy) -> Self {
        Self {
            installed_candidates: HashMap::new(),
            strategy,
        }
    }

    /// Register an installed candidate
    pub fn register_installed(&mut self, package: Package, is_editable: bool) {
        let key = format!("{}=={}", package.name.to_lowercase(), package.version);
        let candidate = Candidate {
            package,
            is_installed: true,
            is_editable,
            link_url: None,
        };
        self.installed_candidates.insert(key, candidate);
    }

    /// Check if candidate can be reused
    pub fn can_reuse_installed(
        &self,
        package_name: &str,
        version: &str,
        link_url: Option<&str>,
        is_editable: bool,
    ) -> bool {
        let key = format!("{}=={}", package_name.to_lowercase(), version);

        if let Some(installed) = self.installed_candidates.get(&key) {
            // Can reuse if:
            // 1. Link URL matches (or both are None)
            // 2. Editable status matches
            let link_matches = match (link_url, &installed.link_url) {
                (None, None) => true,
                (Some(a), Some(b)) => a == b,
                _ => false,
            };

            let editable_matches = is_editable == installed.is_editable;

            link_matches && editable_matches
        } else {
            false
        }
    }

    /// Select best candidate from available options
    pub fn select_best(&self, candidates: &[Candidate]) -> Option<Candidate> {
        if candidates.is_empty() {
            return None;
        }

        match self.strategy {
            SelectionStrategy::PreferInstalled => {
                // First, try to find an installed candidate
                candidates
                    .iter()
                    .find(|c| c.is_installed)
                    .cloned()
                    .or_else(|| candidates.first().cloned())
            }
            SelectionStrategy::PreferLatest => {
                // Select the latest version
                candidates
                    .iter()
                    .max_by(|a, b| {
                        // Simple version comparison (would use proper PEP 440 in production)
                        a.package.version.cmp(&b.package.version)
                    })
                    .cloned()
            }
            SelectionStrategy::PreferCompatible => {
                // Prefer installed if compatible, otherwise latest
                candidates
                    .iter()
                    .find(|c| c.is_installed)
                    .cloned()
                    .or_else(|| {
                        candidates
                            .iter()
                            .max_by(|a, b| {
                                a.package.version.cmp(&b.package.version)
                            })
                            .cloned()
                    })
            }
        }
    }

    /// Get installed candidates
    pub fn get_installed_candidates(&self) -> Vec<Candidate> {
        self.installed_candidates
            .values()
            .filter(|c| c.is_installed)
            .cloned()
            .collect()
    }

    /// Clear installed candidates
    pub fn clear_installed(&mut self) {
        self.installed_candidates.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> CandidateStats {
        let total = self.installed_candidates.len();
        let installed = self
            .installed_candidates
            .values()
            .filter(|c| c.is_installed)
            .count();
        let editable = self
            .installed_candidates
            .values()
            .filter(|c| c.is_editable)
            .count();

        CandidateStats {
            total,
            installed,
            editable,
        }
    }
}

impl Default for CandidateSelector {
    fn default() -> Self {
        Self::new(SelectionStrategy::PreferCompatible)
    }
}

/// Candidate statistics
#[derive(Debug, Clone)]
pub struct CandidateStats {
    pub total: usize,
    pub installed: usize,
    pub editable: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_package(name: &str, version: &str) -> Package {
        Package {
            name: name.to_string(),
            version: version.to_string(),
            summary: None,
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        }
    }

    #[test]
    fn test_candidate_selector_new() {
        let selector = CandidateSelector::new(SelectionStrategy::PreferLatest);
        assert_eq!(selector.strategy, SelectionStrategy::PreferLatest);
    }

    #[test]
    fn test_register_installed() {
        let mut selector = CandidateSelector::new(SelectionStrategy::PreferInstalled);
        let pkg = create_test_package("requests", "2.28.0");
        selector.register_installed(pkg, false);

        let stats = selector.stats();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.installed, 1);
    }

    #[test]
    fn test_can_reuse_installed() {
        let mut selector = CandidateSelector::new(SelectionStrategy::PreferInstalled);
        let pkg = create_test_package("requests", "2.28.0");
        selector.register_installed(pkg, false);

        assert!(selector.can_reuse_installed("requests", "2.28.0", None, false));
        assert!(!selector.can_reuse_installed("requests", "2.28.0", None, true));
        assert!(!selector.can_reuse_installed("requests", "2.27.0", None, false));
    }

    #[test]
    fn test_select_best_prefer_installed() {
        let selector = CandidateSelector::new(SelectionStrategy::PreferInstalled);

        let candidates = vec![
            Candidate {
                package: create_test_package("pkg", "1.0.0"),
                is_installed: false,
                is_editable: false,
                link_url: None,
            },
            Candidate {
                package: create_test_package("pkg", "2.0.0"),
                is_installed: true,
                is_editable: false,
                link_url: None,
            },
        ];

        let selected = selector.select_best(&candidates).unwrap();
        assert_eq!(selected.package.version, "2.0.0");
        assert!(selected.is_installed);
    }

    #[test]
    fn test_select_best_prefer_latest() {
        let selector = CandidateSelector::new(SelectionStrategy::PreferLatest);

        let candidates = vec![
            Candidate {
                package: create_test_package("pkg", "1.0.0"),
                is_installed: true,
                is_editable: false,
                link_url: None,
            },
            Candidate {
                package: create_test_package("pkg", "2.0.0"),
                is_installed: false,
                is_editable: false,
                link_url: None,
            },
        ];

        let selected = selector.select_best(&candidates).unwrap();
        assert_eq!(selected.package.version, "2.0.0");
    }

    #[test]
    fn test_select_best_empty() {
        let selector = CandidateSelector::new(SelectionStrategy::PreferLatest);
        let candidates: Vec<Candidate> = vec![];
        assert!(selector.select_best(&candidates).is_none());
    }

    #[test]
    fn test_get_installed_candidates() {
        let mut selector = CandidateSelector::new(SelectionStrategy::PreferInstalled);
        let pkg1 = create_test_package("pkg1", "1.0.0");
        let pkg2 = create_test_package("pkg2", "2.0.0");

        selector.register_installed(pkg1, false);
        selector.register_installed(pkg2, true);

        let installed = selector.get_installed_candidates();
        assert_eq!(installed.len(), 2);
    }

    #[test]
    fn test_candidate_stats() {
        let mut selector = CandidateSelector::new(SelectionStrategy::PreferInstalled);
        let pkg1 = create_test_package("pkg1", "1.0.0");
        let pkg2 = create_test_package("pkg2", "2.0.0");

        selector.register_installed(pkg1, false);
        selector.register_installed(pkg2, true);

        let stats = selector.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.installed, 2);
        assert_eq!(stats.editable, 1);
    }

    #[test]
    fn test_default_strategy() {
        let selector = CandidateSelector::default();
        assert_eq!(selector.strategy, SelectionStrategy::PreferCompatible);
    }
}
