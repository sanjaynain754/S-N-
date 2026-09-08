//! S+N++ remote registry protocol, HTTP transport and JSON index parsing.
use crate::package::{Constraint, Version};
use std::process::Command;

pub const PROTOCOL_VERSION: &str = "snp-registry-v1";
pub const INDEX_MEDIA_TYPE: &str = "application/vnd.snp.registry.index+json;v=1";
pub const METADATA_MEDIA_TYPE: &str = "application/vnd.snp.package.metadata+toml;v=1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryClientConfig { pub base_url: String, pub cache_dir: String, pub user_agent: String }
impl RegistryClientConfig {
    pub fn new(base_url: impl Into<String>, cache_dir: impl Into<String>) -> Self { Self { base_url: base_url.into().trim_end_matches('/').into(), cache_dir: cache_dir.into(), user_agent: format!("snp/{PROTOCOL_VERSION}") } }
    pub fn index_url(&self, package: &str) -> String { format!("{}/v1/index/{}.json", self.base_url, package) }
    pub fn metadata_url(&self, package: &str, version: &Version) -> String { format!("{}/v1/packages/{}/{}/snp.toml", self.base_url, package, version) }
    pub fn archive_url(&self, package: &str, version: &Version) -> String { format!("{}/v1/packages/{}/{}/download", self.base_url, package, version) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDocument { pub protocol: String, pub package: String, pub versions: Vec<IndexVersion> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexVersion { pub version: Version, pub metadata_url: String, pub archive_url: String, pub checksum: String, pub dependencies: Vec<IndexDependency> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDependency { pub name: String, pub constraint: Constraint }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError { HttpStatus(u16), Transport(String), Json(String), Protocol(String) }
impl std::fmt::Display for RegistryError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { Self::HttpStatus(s) => write!(f, "registry returned HTTP status {s}"), Self::Transport(e) => write!(f, "registry transport error: {e}"), Self::Json(e) => write!(f, "invalid registry JSON: {e}"), Self::Protocol(e) => write!(f, "registry protocol error: {e}") } } }
impl std::error::Error for RegistryError {}

pub struct HttpRegistryClient { config: RegistryClientConfig }
impl HttpRegistryClient {
    pub fn new(config: RegistryClientConfig) -> Self { Self { config } }
    pub fn config(&self) -> &RegistryClientConfig { &self.config }
    pub fn fetch_index(&self, package: &str) -> Result<IndexDocument, RegistryError> {
        let (status, body) = self.get(&self.config.index_url(package))?;
        if !(200..300).contains(&status) { return Err(RegistryError::HttpStatus(status)); }
        let index = parse_index_json(&body)?;
        if index.package != package { return Err(RegistryError::Protocol(format!("requested `{package}`, response contains `{}`", index.package))); }
        index.validate().map_err(RegistryError::Protocol)?;
        Ok(index)
    }
    fn get(&self, url: &str) -> Result<(u16, String), RegistryError> {
        let output = Command::new("curl").args(["--silent", "--show-error", "--location", "--max-time", "15", "--user-agent", &self.config.user_agent, "--header", &format!("Accept: {INDEX_MEDIA_TYPE}"), "--write-out", "\n%{http_code}", url]).output().map_err(|e| RegistryError::Transport(e.to_string()))?;
        if !output.status.success() { return Err(RegistryError::Transport(String::from_utf8_lossy(&output.stderr).trim().to_string())); }
        let text = String::from_utf8(output.stdout).map_err(|e| RegistryError::Transport(e.to_string()))?;
        let (body, status) = text.rsplit_once('\n').ok_or_else(|| RegistryError::Transport("HTTP response did not include status".into()))?;
        let status = status.parse::<u16>().map_err(|e| RegistryError::Transport(e.to_string()))?;
        Ok((status, body.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Json { Object(Vec<(String, Json)>), Array(Vec<Json>), String(String), Null }
struct JsonParser<'a> { chars: std::iter::Peekable<std::str::Chars<'a>> }
impl<'a> JsonParser<'a> {
    fn new(text: &'a str) -> Self { Self { chars: text.chars().peekable() } }
    fn parse(mut self) -> Result<Json, RegistryError> { let value = self.value()?; self.ws(); if self.chars.next().is_some() { return Err(RegistryError::Json("trailing JSON data".into())); } Ok(value) }
    fn ws(&mut self) { while self.chars.peek().is_some_and(|c| c.is_ascii_whitespace()) { self.chars.next(); } }
    fn value(&mut self) -> Result<Json, RegistryError> { self.ws(); match self.chars.peek().copied() { Some('{') => self.object(), Some('[') => self.array(), Some('"') => Ok(Json::String(self.string()?)), Some('n') => { for expected in "null".chars() { if self.chars.next() != Some(expected) { return Err(RegistryError::Json("invalid null".into())); } } Ok(Json::Null) }, _ => Err(RegistryError::Json("expected JSON value".into())) } }
    fn string(&mut self) -> Result<String, RegistryError> { if self.chars.next() != Some('"') { return Err(RegistryError::Json("expected string".into())); } let mut out = String::new(); while let Some(c) = self.chars.next() { match c { '"' => return Ok(out), '\\' => out.push(self.chars.next().ok_or_else(|| RegistryError::Json("unfinished escape".into()))?), c => out.push(c) } } Err(RegistryError::Json("unterminated string".into())) }
    fn object(&mut self) -> Result<Json, RegistryError> { self.chars.next(); let mut out = Vec::new(); self.ws(); if self.chars.peek() == Some(&'}') { self.chars.next(); return Ok(Json::Object(out)); } loop { self.ws(); let key = self.string()?; self.ws(); if self.chars.next() != Some(':') { return Err(RegistryError::Json("expected object colon".into())); } out.push((key, self.value()?)); self.ws(); match self.chars.next() { Some('}') => break, Some(',') => continue, _ => return Err(RegistryError::Json("expected object separator".into())) } } Ok(Json::Object(out)) }
    fn array(&mut self) -> Result<Json, RegistryError> { self.chars.next(); let mut out = Vec::new(); self.ws(); if self.chars.peek() == Some(&']') { self.chars.next(); return Ok(Json::Array(out)); } loop { out.push(self.value()?); self.ws(); match self.chars.next() { Some(']') => break, Some(',') => continue, _ => return Err(RegistryError::Json("expected array separator".into())) } } Ok(Json::Array(out)) }
}
fn field<'a>(object: &'a [(String, Json)], name: &str) -> Result<&'a Json, RegistryError> { object.iter().find(|(key, _)| key == name).map(|(_, value)| value).ok_or_else(|| RegistryError::Json(format!("missing field `{name}`"))) }
fn string_field(object: &[(String, Json)], name: &str) -> Result<String, RegistryError> { match field(object, name)? { Json::String(value) => Ok(value.clone()), _ => Err(RegistryError::Json(format!("field `{name}` must be a string"))) } }

pub fn parse_index_json(text: &str) -> Result<IndexDocument, RegistryError> {
    let root = JsonParser::new(text).parse()?; let object = match root { Json::Object(value) => value, _ => return Err(RegistryError::Json("index root must be an object".into())) };
    let protocol = string_field(&object, "protocol")?; let package = string_field(&object, "package")?;
    let versions = match field(&object, "versions")? { Json::Array(value) => value, _ => return Err(RegistryError::Json("field `versions` must be an array".into())) };
    let mut parsed = Vec::new();
    for version in versions { let object = match version { Json::Object(value) => value, _ => return Err(RegistryError::Json("version entry must be an object".into())) }; let dependencies = match object.iter().find(|(key, _)| key == "dependencies").map(|(_, value)| value) { None => Vec::new(), Some(Json::Array(items)) => items.iter().map(parse_dependency).collect::<Result<Vec<_>, _>>()?, Some(_) => return Err(RegistryError::Json("field `dependencies` must be an array".into())) }; parsed.push(IndexVersion { version: Version::parse(&string_field(object, "version")?).map_err(RegistryError::Json)?, metadata_url: string_field(object, "metadata_url")?, archive_url: string_field(object, "archive_url")?, checksum: string_field(object, "checksum")?, dependencies }); }
    Ok(IndexDocument { protocol, package, versions: parsed })
}
fn parse_dependency(value: &Json) -> Result<IndexDependency, RegistryError> { let object = match value { Json::Object(value) => value, _ => return Err(RegistryError::Json("dependency entry must be an object".into())) }; Ok(IndexDependency { name: string_field(object, "name")?, constraint: Constraint::parse(&string_field(object, "version")?).map_err(RegistryError::Json)? }) }

impl IndexDocument {
    pub fn validate(&self) -> Result<(), String> { if self.protocol != PROTOCOL_VERSION { return Err(format!("unsupported registry protocol `{}`", self.protocol)); } if self.package.is_empty() { return Err("registry index package name is empty".into()); } if self.versions.is_empty() { return Err(format!("registry index for `{}` has no versions", self.package)); } for version in &self.versions { if version.metadata_url.is_empty() || version.archive_url.is_empty() { return Err(format!("registry index `{}` has incomplete version {}", self.package, version.version)); } if !version.checksum.starts_with("sha256:") { return Err(format!("registry index `{}` version {} has invalid checksum", self.package, version.version)); } } Ok(()) }
    pub fn highest_matching(&self, constraint: &Constraint) -> Result<&IndexVersion, String> { self.validate()?; self.versions.iter().filter(|item| constraint.matches(&item.version)).max_by(|a, b| a.version.cmp(&b.version)).ok_or_else(|| format!("no registry version of `{}` satisfies {constraint}", self.package)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn builds_versioned_endpoints() { let config = RegistryClientConfig::new("https://registry.example.test/", "/tmp/snp-cache"); assert_eq!(config.index_url("demo"), "https://registry.example.test/v1/index/demo.json"); assert_eq!(config.metadata_url("demo", &Version::parse("1.2.3").unwrap()), "https://registry.example.test/v1/packages/demo/1.2.3/snp.toml"); }
    #[test] fn parses_index_json_and_dependencies() { let text = r#"{"protocol":"snp-registry-v1","package":"demo","versions":[{"version":"1.2.0","metadata_url":"m","archive_url":"a","checksum":"sha256:abc","dependencies":[{"name":"core","version":"^1.0.0"}]}]}"#; let index = parse_index_json(text).unwrap(); assert_eq!(index.package, "demo"); assert_eq!(index.versions[0].dependencies[0].constraint, Constraint::Caret(Version::parse("1.0.0").unwrap())); assert_eq!(index.highest_matching(&Constraint::Any).unwrap().version, Version::parse("1.2.0").unwrap()); }
    #[test] fn rejects_malformed_index() { let error = parse_index_json(r#"{"protocol":"snp-registry-v1","package":"demo","versions":[{"version":"bad"}]}"#).unwrap_err(); assert!(matches!(error, RegistryError::Json(_))); }
    #[test] fn rejects_wrong_protocol() { let index = IndexDocument { protocol: "old".into(), package: "demo".into(), versions: vec![] }; assert!(index.validate().is_err()); }
    #[test] fn fetches_index_over_http() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || { let (mut stream, _) = listener.accept().unwrap(); let mut request = [0u8; 512]; let _ = stream.read(&mut request); let body = r#"{"protocol":"snp-registry-v1","package":"demo","versions":[{"version":"1.0.0","metadata_url":"m","archive_url":"a","checksum":"sha256:abc"}]}"#; write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", body.len(), body).unwrap(); });
        let client = HttpRegistryClient::new(RegistryClientConfig::new(format!("http://{}", address), "/tmp/snp-cache"));
        let index = client.fetch_index("demo").unwrap();
        worker.join().unwrap();
        assert_eq!(index.versions[0].version, Version::parse("1.0.0").unwrap());
    }
}
