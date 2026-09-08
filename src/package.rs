//! S+N++ package manifest and local dependency resolution.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub dependencies: BTreeMap<String, Dependency>,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut section = String::new();
        let mut name = None;
        let mut version = None;
        let mut entry = None;
        let mut dependencies = BTreeMap::new();
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() { continue; }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len()-1].trim().to_string();
                if section != "package" && section != "dependencies" {
                    return Err(format!("unknown manifest section `{section}` at line {}", line_no + 1));
                }
                continue;
            }
            let (key, value) = line.split_once('=').ok_or_else(|| format!("expected key = value at manifest line {}", line_no + 1))?;
            let key = key.trim();
            let value = value.trim().trim_matches('"').to_string();
            match section.as_str() {
                "package" => match key {
                    "name" => name = Some(value),
                    "version" => version = Some(value),
                    "entry" => entry = Some(value),
                    _ => return Err(format!("unknown package field `{key}` at line {}", line_no + 1)),
                },
                "dependencies" => {
                    if key.is_empty() || value.is_empty() { return Err(format!("invalid dependency at line {}", line_no + 1)); }
                    dependencies.insert(key.to_string(), Dependency { name: key.to_string(), source: value });
                }
                _ => return Err(format!("manifest field outside a section at line {}", line_no + 1)),
            }
        }
        let name = name.ok_or("manifest is missing [package] name")?;
        let version = version.ok_or("manifest is missing [package] version")?;
        Ok(Self { name, version, entry: entry.unwrap_or_else(|| "src/main.snp".into()), dependencies })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Self::parse(&text)
    }
}

pub fn resolve_local(root: impl AsRef<Path>, manifest: &Manifest) -> Result<Vec<PathBuf>, String> {
    let root = root.as_ref();
    let mut resolved = Vec::new();
    for dependency in manifest.dependencies.values() {
        let source = Path::new(&dependency.source);
        let path = if source.is_absolute() { source.to_path_buf() } else { root.join(source) };
        if !path.is_dir() { return Err(format!("dependency `{}` path does not exist: {}", dependency.name, path.display())); }
        let dependency_manifest = path.join("snp.toml");
        if !dependency_manifest.is_file() { return Err(format!("dependency `{}` has no snp.toml: {}", dependency.name, path.display())); }
        let dep = Manifest::load(&dependency_manifest)?;
        if dep.name != dependency.name { return Err(format!("dependency `{}` resolves to package `{}`", dependency.name, dep.name)); }
        resolved.push(path);
    }
    Ok(resolved)
}

pub fn init_project(root: impl AsRef<Path>, name: &str) -> Result<(), String> {
    let root = root.as_ref();
    fs::create_dir_all(root.join("src")).map_err(|e| format!("cannot create project: {e}"))?;
    let manifest = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nentry = \"src/main.snp\"\n\n[dependencies]\n");
    fs::write(root.join("snp.toml"), manifest).map_err(|e| format!("cannot write snp.toml: {e}"))?;
    let entry = "fn main() {\n    print(\"Hello from S+N++\");\n}\n";
    let entry_path = root.join("src/main.snp");
    if !entry_path.exists() { fs::write(entry_path, entry).map_err(|e| format!("cannot write entry file: {e}"))?; }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_manifest_and_dependencies() {
        let m = Manifest::parse("[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nutil = \"../util\"\n").unwrap();
        assert_eq!(m.name, "demo");
        assert_eq!(m.dependencies["util"].source, "../util");
    }
    #[test]
    fn rejects_missing_package_name() {
        assert!(Manifest::parse("[package]\nversion = \"0.1.0\"\n").is_err());
    }
}

pub fn validate_imports(imports: &[String], manifest: &Manifest) -> Result<(), String> {
    let std_imports: Vec<String> = imports.iter().filter(|name| name.starts_with("std.")).cloned().collect();
    crate::stdlib::resolve(&std_imports)?;
    for import in imports.iter().filter(|name| !name.starts_with("std.")) {
        let root = import.split('.').next().unwrap_or(import);
        if !manifest.dependencies.contains_key(root) {
            return Err(format!("unknown package namespace `{root}` in import `{import}`; add it to [dependencies]"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod resolver_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn validates_std_and_declared_package_imports() {
        let manifest = Manifest::parse("[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\nutil = \"../util\"\n").unwrap();
        assert!(validate_imports(&["std.io".into(), "util.math".into()], &manifest).is_ok());
        assert!(validate_imports(&["missing.math".into()], &manifest).is_err());
    }

    #[test]
    fn resolves_a_local_dependency_manifest() {
        let base = std::env::temp_dir().join(format!("snp-pkg-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let dep = base.join("util");
        fs::create_dir_all(&dep).unwrap();
        fs::write(dep.join("snp.toml"), "[package]\nname = \"util\"\nversion = \"0.1.0\"\n").unwrap();
        let manifest = Manifest::parse("[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\nutil = \"util\"\n").unwrap();
        assert_eq!(resolve_local(&base, &manifest).unwrap(), vec![dep]);
        let _ = fs::remove_dir_all(base);
    }
}
