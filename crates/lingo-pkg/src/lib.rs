use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub dependencies: HashMap<String, String>,
}

impl PackageManifest {
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut name = String::new();
        let mut version = "0.1.0".to_string();
        let mut entry = "src/main.ln".to_string();
        let mut dependencies = HashMap::new();

        let mut current_section = "";

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = &trimmed[1..trimmed.len() - 1];
                continue;
            }

            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');

                if current_section == "package" {
                    match key {
                        "name" => name = val.to_string(),
                        "version" => version = val.to_string(),
                        "entry" => entry = val.to_string(),
                        _ => {}
                    }
                } else if current_section == "dependencies" {
                    dependencies.insert(key.to_string(), val.to_string());
                }
            }
        }

        if name.is_empty() {
            return Err("Manifest missing package name".into());
        }

        Ok(Self {
            name,
            version,
            entry,
            dependencies,
        })
    }

    pub fn to_toml_string(&self) -> String {
        let mut out = String::new();
        out.push_str("[package]\n");
        out.push_str(&format!("name = \"{}\"\n", self.name));
        out.push_str(&format!("version = \"{}\"\n", self.version));
        out.push_str(&format!("entry = \"{}\"\n\n", self.entry));

        out.push_str("[dependencies]\n");
        for (dep, ver) in &self.dependencies {
            out.push_str(&format!("{dep} = \"{ver}\"\n"));
        }
        out
    }
}

pub struct PackageManager;

impl PackageManager {
    pub fn init_project(dir: &Path, name: &str) -> Result<PathBuf, String> {
        let manifest = PackageManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            entry: "src/main.ln".to_string(),
            dependencies: HashMap::new(),
        };

        fs::create_dir_all(dir.join("src")).map_err(|e| e.to_string())?;
        let manifest_path = dir.join("lingo.toml");
        fs::write(&manifest_path, manifest.to_toml_string()).map_err(|e| e.to_string())?;

        let main_path = dir.join("src/main.ln");
        let initial_src = "def main():\n    println(\"Hello from \" + \"".to_string() + name + "!\")\n";
        fs::write(&main_path, initial_src).map_err(|e| e.to_string())?;

        // Write initial lockfile
        let lockfile_path = dir.join("lingo.lock");
        let lock_content = format!("# Lingo lockfile\n[package]\nname = \"{}\"\nversion = \"{}\"\n", name, "0.1.0");
        fs::write(&lockfile_path, lock_content).map_err(|e| e.to_string())?;

        Ok(manifest_path)
    }

    pub fn load_manifest(dir: &Path) -> Result<PackageManifest, String> {
        let manifest_path = dir.join("lingo.toml");
        if !manifest_path.exists() {
            return Err(format!("No lingo.toml found in {:?}", dir));
        }
        let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        PackageManifest::parse(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse_and_serialize() {
        let toml = "[package]\nname = \"test_pkg\"\nversion = \"1.2.3\"\nentry = \"src/main.ln\"\n\n[dependencies]\ncore = \"0.1.0\"\n";
        let manifest = PackageManifest::parse(toml).unwrap();
        assert_eq!(manifest.name, "test_pkg");
        assert_eq!(manifest.version, "1.2.3");
        assert_eq!(manifest.dependencies.get("core"), Some(&"0.1.0".to_string()));
    }
}
