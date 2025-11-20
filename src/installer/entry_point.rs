/// Entry point generation for console scripts
use anyhow::Result;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EntryPoint {
    pub name: String,
    pub module: String,
    pub function: String,
}

impl EntryPoint {
    pub fn new(name: String, module: String, function: String) -> Self {
        Self {
            name,
            module,
            function,
        }
    }

    /// Generate a console script wrapper
    pub fn generate_script(&self) -> String {
        if cfg!(target_os = "windows") {
            self.generate_windows_script()
        } else {
            self.generate_unix_script()
        }
    }

    fn generate_unix_script(&self) -> String {
        format!(
            "#!/usr/bin/env python\n\
             import sys\n\
             from {} import {}\n\
             if __name__ == '__main__':\n\
             \tsys.exit({}())\n",
            self.module, self.function, self.function
        )
    }

    fn generate_windows_script(&self) -> String {
        format!(
            "#!python\n\
             # -*- coding: utf-8 -*-\n\
             import re\n\
             import sys\n\
             from {} import {}\n\
             if __name__ == '__main__':\n\
             \tsys.exit({}())\n",
            self.module, self.function, self.function
        )
    }

    pub fn install(&self, scripts_dir: &PathBuf) -> Result<PathBuf> {
        fs::create_dir_all(scripts_dir)?;
        
        let script_path = if cfg!(target_os = "windows") {
            scripts_dir.join(format!("{}.exe", self.name))
        } else {
            scripts_dir.join(&self.name)
        };

        let script_content = self.generate_script();
        fs::write(&script_path, script_content)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        Ok(script_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_point_creation() {
        let ep = EntryPoint::new(
            "pip".to_string(),
            "pip._internal.cli.main".to_string(),
            "main".to_string(),
        );
        assert_eq!(ep.name, "pip");
        assert_eq!(ep.module, "pip._internal.cli.main");
    }

    #[test]
    fn test_script_generation() {
        let ep = EntryPoint::new(
            "pip".to_string(),
            "pip._internal.cli.main".to_string(),
            "main".to_string(),
        );
        let script = ep.generate_script();
        assert!(script.contains("pip._internal.cli.main"));
        assert!(script.contains("main()"));
    }
}
