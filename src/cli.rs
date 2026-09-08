//! S+N++ command-line interface.

use std::env;
use std::fs;
use std::io::{self, Write};

pub fn run() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("S+N++ — usage: snp init <dir> [name] | snp package-check [snp.toml] | snp run <file.snp> | snp check <file.snp> | snp build <file.snp> | snp repl");
        return;
    }
    if args[1] == "repl" {
        repl();
        return;
    }
    if args[1] == "init" {
        let root = args.get(2).cloned().unwrap_or_else(|| ".".into());
        let name = args.get(3).cloned().unwrap_or_else(|| std::path::Path::new(&root).file_name().and_then(|x| x.to_str()).unwrap_or("splusnpp-app").into());
        match crate::package::init_project(&root, &name) {
            Ok(()) => println!("initialized package `{name}` at {root}"),
            Err(error) => eprintln!("error: {error}"),
        }
        return;
    }
    if args[1] == "package-check" {
        let manifest_path = args.get(2).cloned().unwrap_or_else(|| "snp.toml".into());
        let path = std::path::Path::new(&manifest_path);
        match crate::package::Manifest::load(path).and_then(|manifest| {
            let root = path.parent().unwrap_or(std::path::Path::new("."));
            let deps = crate::package::resolve_local(root, &manifest)?;
            let entry = root.join(&manifest.entry);
            let source = fs::read_to_string(&entry).map_err(|e| format!("cannot read package entry {}: {e}", entry.display()))?;
            let program = crate::parser::parse_program(&source)?;
            crate::package::validate_imports(&program.imports, &manifest)?;
            Ok((manifest, deps))
        }) {
            Ok((manifest, deps)) => println!("ok: package `{}` v{} with {} local dependenc{}", manifest.name, manifest.version, deps.len(), if deps.len() == 1 { "y" } else { "ies" }),
            Err(error) => eprintln!("error: {error}"),
        }
        return;
    }
    if args.len() < 3 { eprintln!("missing source file"); return; }
    let source_path = &args[2];
    let source = match fs::read_to_string(source_path) {
        Ok(source) => source,
        Err(error) => { eprintln!("cannot read {source_path}: {error}"); return; }
    };
    match args[1].as_str() {
        "check" => match crate::parser::parse_program(&source).and_then(|program| { crate::stdlib::resolve(&program.imports)?; crate::typecheck::check(&program.functions) }) {
            Ok(()) => println!("ok: valid S+N++ program"),
            Err(error) => eprintln!("{}", diagnostic(&source, &error)),
        },
        "build" => match crate::build_artifact(&source) {
            Ok(bytes) => { let artifact = format!("{source_path}.snpbc"); match fs::write(&artifact, bytes) { Ok(()) => println!("built: {artifact}"), Err(error) => eprintln!("cannot write {artifact}: {error}") } }
            Err(error) => eprintln!("{}", diagnostic(&source, &error)),
        },
        "run" => match crate::execute(&source) {
            Ok(value) if value != crate::Value::Unit => println!("program returned {}", show(&value)),
            Ok(_) => {},
            Err(error) => eprintln!("{}", diagnostic(&source, &error)),
        },
        command => eprintln!("unknown command `{command}`; use init, package-check, run, check, build or repl"),
    }
}

fn repl() {
    let mut line = String::new();
    loop {
        print!("snp> "); let _ = io::stdout().flush(); line.clear();
        if io::stdin().read_line(&mut line).unwrap_or(0) == 0 { break; }
        let source = format!("fn main() {{{line}}}");
        match crate::execute(&source) {
            Ok(value) if value != crate::Value::Unit => println!("{}", show(&value)),
            Ok(_) => {},
            Err(error) => eprintln!("{}", diagnostic(&source, &error)),
        }
    }
}

fn show(value: &crate::Value) -> String {
    match value {
        crate::Value::Int(value) => value.to_string(),
        crate::Value::Str(value) => value.clone(),
        crate::Value::Bool(value) => value.to_string(),
        crate::Value::Unit => "unit".into(),
        crate::Value::Thread(id) => format!("thread({id})"),
        crate::Value::Channel(id) => format!("channel({id})"),
        crate::Value::List(items) => format!("list(len={})", items.len()),
        crate::Value::Map(items) => format!("map(len={})", items.len()),
    }
}

fn diagnostic(source: &str, error: &str) -> String {
    let needle = error.split('`').nth(1);
    let mut line_no = 1;
    let mut line_text = "";
    for (index, line) in source.lines().enumerate() {
        if needle.map(|name| line.contains(name)).unwrap_or(false) {
            line_no = index + 1;
            line_text = line;
            break;
        }
        if line_text.is_empty() { line_text = line; }
    }
    format!("error: {error}\n  --> source.snp:{line_no}:1\n   |\n{:>3} | {line_text}\n   | ^", line_no)
}
