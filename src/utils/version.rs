/// Version comparison utilities
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    parts: Vec<u32>,
    pre: Option<String>,
    post: Option<String>,
}

impl Version {
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        
        // Split on pre-release markers
        let (version_part, pre) = if let Some(idx) = s.find("a") {
            let (v, p) = s.split_at(idx);
            (v, Some(p.to_string()))
        } else if let Some(idx) = s.find("b") {
            let (v, p) = s.split_at(idx);
            (v, Some(p.to_string()))
        } else if let Some(idx) = s.find("rc") {
            let (v, p) = s.split_at(idx);
            (v, Some(p.to_string()))
        } else {
            (s, None)
        };

        let parts: Result<Vec<u32>, _> = version_part
            .split('.')
            .map(|p| p.parse::<u32>().map_err(|_| "Invalid version".to_string()))
            .collect();

        Ok(Version {
            parts: parts?,
            pre,
            post: None,
        })
    }

    #[allow(dead_code)]
    pub fn compare(&self, other: &Version) -> Ordering {
        for i in 0..self.parts.len().max(other.parts.len()) {
            let a = self.parts.get(i).copied().unwrap_or(0);
            let b = other.parts.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }
}

impl std::str::FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.parts, vec![1, 2, 3]);
    }

    #[test]
    fn test_version_compare() {
        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::parse("1.2.4").unwrap();
        assert_eq!(v1.compare(&v2), Ordering::Less);
    }
}
