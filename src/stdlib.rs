//! Built-in S+N++ standard-library module registry.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleSpec {
    pub name: &'static str,
    pub functions: &'static [&'static str],
}

pub fn registry() -> BTreeMap<&'static str, ModuleSpec> {
    BTreeMap::from([
        ("std.io", ModuleSpec { name: "std.io", functions: &["print"] }),
        ("std.string", ModuleSpec { name: "std.string", functions: &["str_len", "str_contains", "str_upper", "str_lower", "str_trim", "str_concat"] }),
        ("std.collections", ModuleSpec { name: "std.collections", functions: &["list", "list_push", "list_len", "list_get"] }),
        ("std.concurrency", ModuleSpec { name: "std.concurrency", functions: &["channel", "send", "receive", "spawn", "join"] }),
    ])
}

pub fn resolve(imports: &[String]) -> Result<(), String> {
    let modules = registry();
    for import in imports {
        if !modules.contains_key(import.as_str()) {
            return Err(format!("unknown standard-library module `{import}`; available modules: std.io, std.string, std.collections, std.concurrency"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_core_modules() {
        assert!(resolve(&["std.io".into(), "std.concurrency".into()]).is_ok());
    }

    #[test]
    fn rejects_unknown_modules() {
        let error = resolve(&["std.network".into()]).unwrap_err();
        assert!(error.contains("unknown standard-library module"));
    }
}
