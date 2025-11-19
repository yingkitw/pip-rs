/// Extras resolution - handles optional dependencies
use crate::models::{Package, Requirement};
use anyhow::Result;

/// Resolve extras for a package
pub fn resolve_extras(package: &Package, requested_extras: &[String]) -> Result<Vec<Requirement>> {
    let mut extra_deps = Vec::new();

    if requested_extras.is_empty() {
        return Ok(extra_deps);
    }

    // Parse requires_dist to find extras
    for dep_str in &package.requires_dist {
        if let Ok(req) = dep_str.parse::<Requirement>() {
            // Check if this dependency is conditional on extras
            if let Some(marker) = &req.marker {
                // Look for "extra ==" pattern in marker
                for extra in requested_extras {
                    if marker.contains(&format!("extra == '{}'", extra))
                        || marker.contains(&format!("extra == \"{}\"", extra))
                    {
                        extra_deps.push(req.clone());
                        break;
                    }
                }
            }
        }
    }

    Ok(extra_deps)
}

/// Get available extras for a package
pub fn get_available_extras(package: &Package) -> Vec<String> {
    let mut extras = std::collections::HashSet::new();

    for dep_str in &package.requires_dist {
        if let Ok(req) = dep_str.parse::<Requirement>() {
            if let Some(marker) = &req.marker {
                // Extract extra names from markers like "extra == 'security'"
                if let Some(start) = marker.find("extra == '") {
                    let rest = &marker[start + 10..];
                    if let Some(end) = rest.find('\'') {
                        extras.insert(rest[..end].to_string());
                    }
                } else if let Some(start) = marker.find("extra == \"") {
                    let rest = &marker[start + 10..];
                    if let Some(end) = rest.find('"') {
                        extras.insert(rest[..end].to_string());
                    }
                }
            }
        }
    }

    extras.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_extras() {
        let package = Package {
            name: "requests".to_string(),
            version: "2.28.0".to_string(),
            summary: Some("HTTP library".to_string()),
            home_page: Some("https://requests.readthedocs.io".to_string()),
            author: Some("Kenneth Reitz".to_string()),
            license: Some("Apache 2.0".to_string()),
            requires_python: None,
            requires_dist: vec![
                "charset-normalizer<3,>=2".to_string(),
                "idna<4,>=2.5; extra == 'security'".to_string(),
                "urllib3<2,>=1.21.1".to_string(),
                "certifi>=2017.4.17; extra == 'security'".to_string(),
            ],
            classifiers: vec![],
        };

        let extras = get_available_extras(&package);
        assert!(extras.contains(&"security".to_string()));
    }
}
