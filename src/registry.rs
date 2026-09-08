//! S+N++ remote registry protocol primitives.
//!
//! Protocol version 1 is intentionally transport-neutral. An HTTP client can
//! fetch the URLs produced here, while the existing filesystem registry can be
//! used as a deterministic fixture during tests.

use crate::package::{Constraint, Version};

pub const PROTOCOL_VERSION: &str = "snp-registry-v1";
pub const INDEX_MEDIA_TYPE: &str = "application/vnd.snp.registry.index+json;v=1";
pub const METADATA_MEDIA_TYPE: &str = "application/vnd.snp.package.metadata+toml;v=1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryClientConfig {
    pub base_url: String,
    pub cache_dir: String,
    pub user_agent: String,
}

impl RegistryClientConfig {
    pub fn new(base_url: impl Into<String>, cache_dir: impl Into<String>) -> Self {
        Self { base_url: base_url.into().trim_end_matches('/').to_string(), cache_dir: cache_dir.into(), user_agent: format!("snp/{PROTOCOL_VERSION}") }
    }
    pub fn index_url(&self, package: &str) -> String { format!("{}/v1/index/{}.json", self.base_url, package) }
    pub fn metadata_url(&self, package: &str, version: &Version) -> String { format!("{}/v1/packages/{}/{}/snp.toml", self.base_url, package, version) }
    pub fn archive_url(&self, package: &str, version: &Version) -> String { format!("{}/v1/packages/{}/{}/download", self.base_url, package, version) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDocument {
    pub protocol: String,
    pub package: String,
    pub versions: Vec<IndexVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexVersion {
    pub version: Version,
    pub metadata_url: String,
    pub archive_url: String,
    pub checksum: String,
    pub dependencies: Vec<IndexDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDependency {
    pub name: String,
    pub constraint: Constraint,
}

impl IndexDocument {
    pub fn validate(&self) -> Result<(), String> {
        if self.protocol != PROTOCOL_VERSION { return Err(format!("unsupported registry protocol `{}`", self.protocol)); }
        if self.package.is_empty() { return Err("registry index package name is empty".into()); }
        if self.versions.is_empty() { return Err(format!("registry index for `{}` has no versions", self.package)); }
        for version in &self.versions {
            if version.metadata_url.is_empty() || version.archive_url.is_empty() { return Err(format!("registry index `{}` has incomplete version {}", self.package, version.version)); }
            if version.checksum.is_empty() { return Err(format!("registry index `{}` version {} has no checksum", self.package, version.version)); }
        }
        Ok(())
    }
    pub fn highest_matching(&self, constraint: &Constraint) -> Result<&IndexVersion, String> {
        self.validate()?;
        self.versions.iter().filter(|item| constraint.matches(&item.version)).max_by(|a, b| a.version.cmp(&b.version)).ok_or_else(|| format!("no registry version of `{}` satisfies {constraint}", self.package))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builds_versioned_endpoints() {
        let config = RegistryClientConfig::new("https://registry.example.test/", "/tmp/snp-cache");
        assert_eq!(config.index_url("demo"), "https://registry.example.test/v1/index/demo.json");
        assert_eq!(config.metadata_url("demo", &Version::parse("1.2.3").unwrap()), "https://registry.example.test/v1/packages/demo/1.2.3/snp.toml");
    }
    #[test]
    fn selects_highest_matching_index_version() {
        let index = IndexDocument { protocol: PROTOCOL_VERSION.into(), package: "demo".into(), versions: vec![
            IndexVersion { version: Version::parse("1.0.0").unwrap(), metadata_url: "m1".into(), archive_url: "a1".into(), checksum: "sha256:one".into(), dependencies: vec![] },
            IndexVersion { version: Version::parse("1.2.0").unwrap(), metadata_url: "m2".into(), archive_url: "a2".into(), checksum: "sha256:two".into(), dependencies: vec![] },
        ] };
        assert_eq!(index.highest_matching(&Constraint::Caret(Version::parse("1.0.0").unwrap())).unwrap().version, Version::parse("1.2.0").unwrap());
    }
}
