/// Wheel file handling
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct WheelFile {
    pub path: PathBuf,
    pub name: String,
    pub version: String,
}

impl WheelFile {
    pub fn new(path: PathBuf) -> Result<Self> {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid wheel filename"))?
            .to_string();

        // Parse wheel filename: {distribution}-{version}(-{build tag})?-{python tag}-{abi tag}-{platform tag}.whl
        let parts: Vec<&str> = filename.split('-').collect();
        if parts.len() < 5 {
            return Err(anyhow!("Invalid wheel filename format"));
        }

        Ok(WheelFile {
            path,
            name: parts[0].to_string(),
            version: parts[1].to_string(),
        })
    }

    pub fn extract(&self, target_dir: &Path) -> Result<()> {
        let file = fs::File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;

        let num_files = archive.len();
        for i in 0..num_files {
            let mut file = archive.by_index(i)?;
            let outpath = target_dir.join(file.name());

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    pub fn get_metadata(&self) -> Result<WheelMetadata> {
        let file = fs::File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;

        // Find METADATA file in dist-info directory
        let num_files = archive.len();
        for i in 0..num_files {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with("/METADATA") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                return Ok(WheelMetadata::parse(&contents)?);
            }
        }

        Err(anyhow!("METADATA file not found in wheel"))
    }
}

#[derive(Debug, Clone)]
pub struct WheelMetadata {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub requires_dist: Vec<String>,
}

impl WheelMetadata {
    pub fn parse(content: &str) -> Result<Self> {
        let mut name = String::new();
        let mut version = String::new();
        let mut summary = None;
        let mut requires_dist = Vec::new();

        for line in content.lines() {
            if line.starts_with("Name: ") {
                name = line[6..].to_string();
            } else if line.starts_with("Version: ") {
                version = line[9..].to_string();
            } else if line.starts_with("Summary: ") {
                summary = Some(line[9..].to_string());
            } else if line.starts_with("Requires-Dist: ") {
                requires_dist.push(line[15..].to_string());
            }
        }

        Ok(WheelMetadata {
            name,
            version,
            summary,
            requires_dist,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wheel_filename_parsing() {
        let path = PathBuf::from("requests-2.28.0-py3-none-any.whl");
        let wheel = WheelFile::new(path).unwrap();
        assert_eq!(wheel.name, "requests");
        assert_eq!(wheel.version, "2.28.0");
    }
}
