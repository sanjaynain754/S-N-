//! S+N++ static type, ownership and definite-assignment checker.

use std::collections::HashMap;
use crate::ast::{Expr, Function, Stmt, Type};
use crate::lexer::Token;

fn type_name(t: &Type) -> &'static str { match t { Type::Int => "Int", Type::Str => "String", Type::Bool => "Bool", Type::Unit => "Unit", Type::Channel => "Channel", Type::Thread => "Thread", Type::List => "List", Type::Map => "Map", Type::Ref(_) => "&reference", Type::MutRef(_) => "&mut reference", Type::Unknown => "Unknown" } }

fn expr_type(x: &Expr, env: &HashMap<String, Type>, sigs: &HashMap<String, (Vec<Type>, Type)>) -> Result<Type, String> {
    match x {
        Expr::Int(_) => Ok(Type::Int), Expr::Str(_) => Ok(Type::Str), Expr::Bool(_) => Ok(Type::Bool),
        Expr::Var(n) => env.get(n).cloned().ok_or_else(|| format!("undefined variable `{n}`")),
        Expr::Call(n, args) => {
            if matches!(n.as_str(), "print"|"spawn"|"join"|"channel"|"send"|"receive"|"str_len"|"str_contains"|"str_upper"|"str_lower"|"str_trim"|"str_concat"|"list"|"list_push"|"list_len"|"list_get"|"list_set"|"list_concat"|"map"|"map_set"|"map_get"|"map_has") {
                for arg in args { expr_type(arg, env, sigs)?; }
                return Ok(match n.as_str() { "spawn" => Type::Thread, "channel" => Type::Channel, "receive"|"list_get"|"map_get" => Type::Unknown, "str_len"|"list_len" => Type::Int, "str_contains"|"map_has" => Type::Bool, "str_upper"|"str_lower"|"str_trim"|"str_concat" => Type::Str, "list"|"list_push"|"list_set"|"list_concat" => Type::List, "map"|"map_set" => Type::Map, _ => Type::Unit });
            }
            let (params, ret) = sigs.get(n).ok_or_else(|| format!("unknown function `{n}`"))?;
            if args.len() != params.len() { return Err(format!("function `{n}` expects {} argument(s), got {}", params.len(), args.len())); }
            for (i, arg) in args.iter().enumerate() {
                let got = expr_type(arg, env, sigs)?;
                let compatible = match &params[i] { Type::Ref(inner) | Type::MutRef(inner) => got == **inner, _ => got == params[i] };
                if !compatible { return Err(format!("function `{n}` argument {} expects {}, got {}", i + 1, type_name(&params[i]), type_name(&got))); }
            }
            Ok(ret.clone())
        }
        Expr::Binary(a, op, b) => {
            let left = expr_type(a, env, sigs)?; let right = expr_type(b, env, sigs)?;
            match op {
                Token::Plus => if left == right && (left == Type::Int || left == Type::Str) { Ok(left) } else { Err(format!("operator + expects two Int values or two String values, got {} and {}", type_name(&left), type_name(&right))) },
                Token::Minus|Token::Star|Token::Slash => if left == Type::Int && right == Type::Int { Ok(Type::Int) } else { Err(format!("arithmetic operator expects Int, got {} and {}", type_name(&left), type_name(&right))) },
                Token::EqEq|Token::BangEq => if left == right { Ok(Type::Bool) } else { Err(format!("comparison requires matching types, got {} and {}", type_name(&left), type_name(&right))) },
                Token::Lt|Token::Lte|Token::Gt|Token::Gte => if left == Type::Int && right == Type::Int { Ok(Type::Bool) } else { Err("ordering comparison expects Int values".into()) },
                _ => Err("invalid operator".into()),
            }
        }
    }
}

fn check_typed_stmts(xs: &[Stmt], env: &mut HashMap<String, Type>, sigs: &HashMap<String, (Vec<Type>, Type)>, ret: &Type) -> Result<(), String> {
    for statement in xs { match statement {
        Stmt::Let(name, expr) => { let ty = expr_type(expr, env, sigs)?; env.insert(name.clone(), ty); }
        Stmt::Decl(name, ty) => { env.insert(name.clone(), ty.clone()); }
        Stmt::Assign(name, expr) => { let got = expr_type(expr, env, sigs)?; let old = env.get(name).ok_or_else(|| format!("assignment to undefined variable `{name}`"))?; if *old != got { return Err(format!("cannot assign {} to `{name}` of type {}", type_name(&got), type_name(old))); } }
        Stmt::Expr(expr) => { expr_type(expr, env, sigs)?; }
        Stmt::Return(expr) => { let got = expr_type(expr, env, sigs)?; if *ret != Type::Unit && got != *ret { return Err(format!("return type mismatch: expected {}, got {}", type_name(ret), type_name(&got))); } }
        Stmt::If(condition, yes, no) => { if expr_type(condition, env, sigs)? != Type::Bool { return Err("if condition must be Bool".into()); } check_typed_stmts(yes, &mut env.clone(), sigs, ret)?; check_typed_stmts(no, &mut env.clone(), sigs, ret)?; }
        Stmt::While(condition, body) => { if expr_type(condition, env, sigs)? != Type::Bool { return Err("while condition must be Bool".into()); } check_typed_stmts(body, &mut env.clone(), sigs, ret)?; }
    }} Ok(())
}

fn copyable(ty: &Type) -> bool { matches!(ty, Type::Int|Type::Bool|Type::Unit|Type::Channel|Type::Thread|Type::Ref(_)|Type::MutRef(_)) }
fn moved_var(expr: &Expr) -> Option<String> { if let Expr::Var(name) = expr { Some(name.clone()) } else { None } }
fn ownership_expr(x: &Expr, env: &HashMap<String, Type>, moved: &mut HashMap<String, bool>, move_value: bool, sigs: &HashMap<String, (Vec<Type>, Type)>) -> Result<(), String> { match x {
    Expr::Var(name) => { if *moved.get(name).unwrap_or(&false) { return Err(format!("use of moved value `{name}`; use `.clone()` before moving it again")); } if move_value && !copyable(env.get(name).unwrap_or(&Type::Unknown)) { moved.insert(name.clone(), true); } }
    Expr::Binary(a, _, b) => { ownership_expr(a, env, moved, false, sigs)?; ownership_expr(b, env, moved, false, sigs)?; }
    Expr::Call(name, args) => { for arg in args { let borrow = if matches!(name.as_str(), "print"|"spawn"|"str_len"|"str_contains"|"str_upper"|"str_lower"|"str_trim"|"str_concat"|"list"|"list_push"|"list_len"|"list_get"|"list_set"|"list_concat"|"map"|"map_set"|"map_get"|"map_has") { true } else { sigs.get(name).and_then(|(params, _)| params.first()).map(|ty| matches!(ty, Type::Ref(_) | Type::MutRef(_))).unwrap_or(false) }; ownership_expr(arg, env, moved, !borrow, sigs)?; } }
    _ => {}
} Ok(()) }
fn ownership_stmts(xs: &[Stmt], env: &mut HashMap<String, Type>, moved: &mut HashMap<String, bool>, sigs: &HashMap<String, (Vec<Type>, Type)>) -> Result<(), String> { for statement in xs { match statement {
    Stmt::Let(name, expr) => { ownership_expr(expr, env, moved, moved_var(expr).is_some(), sigs)?; let ty = expr_type(expr, env, &HashMap::new()).unwrap_or(Type::Unknown); env.insert(name.clone(), ty); moved.insert(name.clone(), false); }
    Stmt::Assign(name, expr) => { ownership_expr(expr, env, moved, moved_var(expr).is_some(), sigs)?; if !env.contains_key(name) { return Err(format!("assignment to undefined variable `{name}`")); } moved.insert(name.clone(), false); }
    Stmt::Decl(name, ty) => { env.insert(name.clone(), ty.clone()); moved.insert(name.clone(), false); }
    Stmt::Expr(expr)|Stmt::Return(expr) => ownership_expr(expr, env, moved, false, sigs)?,
    Stmt::If(condition, yes, no) => { ownership_expr(condition, env, moved, false, sigs)?; let before = moved.clone(); let mut a = before.clone(); let mut b = before; ownership_stmts(yes, env, &mut a, sigs)?; ownership_stmts(no, env, &mut b, sigs)?; for (name, value) in a.iter().chain(b.iter()) { if *value { moved.insert(name.clone(), true); } } }
    Stmt::While(condition, body) => { ownership_expr(condition, env, moved, false, sigs)?; let mut loop_state = moved.clone(); ownership_stmts(body, env, &mut loop_state, sigs)?; }
}} Ok(()) }

fn definite_expr(x: &Expr, types: &HashMap<String, Type>, assigned: &HashMap<String, ()>) -> Result<(), String> { match x {
    Expr::Var(name) => { if !types.contains_key(name) { return Err(format!("undefined variable `{name}`")); } if !assigned.contains_key(name) { return Err(format!("use of uninitialized variable `{name}`")); } }
    Expr::Binary(a, _, b) => { definite_expr(a, types, assigned)?; definite_expr(b, types, assigned)?; }
    Expr::Call(_, args) => for arg in args { definite_expr(arg, types, assigned)?; }
    _ => {}
} Ok(()) }
fn intersect(a: &HashMap<String, ()>, b: &HashMap<String, ()>) -> HashMap<String, ()> { a.iter().filter(|(name, _)| b.contains_key(*name)).map(|(name, value)| (name.clone(), *value)).collect() }
fn definite_stmts(xs: &[Stmt], types: &mut HashMap<String, Type>, assigned: &mut HashMap<String, ()>) -> Result<(), String> { for statement in xs { match statement {
    Stmt::Decl(name, ty) => { types.insert(name.clone(), ty.clone()); assigned.remove(name); }
    Stmt::Let(name, expr) => { definite_expr(expr, types, assigned)?; types.insert(name.clone(), expr_type(expr, types, &HashMap::new()).unwrap_or(Type::Unknown)); assigned.insert(name.clone(), ()); }
    Stmt::Assign(name, expr) => { if !types.contains_key(name) { return Err(format!("assignment to undefined variable `{name}`")); } definite_expr(expr, types, assigned)?; assigned.insert(name.clone(), ()); }
    Stmt::Expr(expr)|Stmt::Return(expr) => definite_expr(expr, types, assigned)?,
    Stmt::If(condition, yes, no) => { definite_expr(condition, types, assigned)?; let mut a = assigned.clone(); let mut b = assigned.clone(); definite_stmts(yes, types, &mut a)?; definite_stmts(no, types, &mut b)?; *assigned = intersect(&a, &b); }
    Stmt::While(condition, body) => { definite_expr(condition, types, assigned)?; let mut body_state = assigned.clone(); definite_stmts(body, types, &mut body_state)?; }
}} Ok(()) }

pub fn check(functions: &[Function]) -> Result<(), String> {
    let mut names = HashMap::new(); let mut signatures = HashMap::new();
    for function in functions { if names.insert(function.name.clone(), ()).is_some() { return Err(format!("duplicate function `{}`", function.name)); } signatures.insert(function.name.clone(), (function.param_types.clone(), function.return_type.clone())); }
    if !names.contains_key("main") { return Err("program must define fn main()".into()); }
    for function in functions {
        let mut env = HashMap::new(); for (i, name) in function.params.iter().enumerate() { env.insert(name.clone(), function.param_types.get(i).cloned().unwrap_or(Type::Unknown)); }
        check_typed_stmts(&function.body, &mut env, &signatures, &function.return_type)?;
        let mut definite = HashMap::new(); for (i, name) in function.params.iter().enumerate() { definite.insert(name.clone(), function.param_types.get(i).cloned().unwrap_or(Type::Unknown)); }
        let mut assigned = HashMap::new(); for name in &function.params { assigned.insert(name.clone(), ()); }
        definite_stmts(&function.body, &mut definite, &mut assigned)?;
        let mut moved = HashMap::new(); ownership_stmts(&function.body, &mut env, &mut moved, &signatures)?;
    }
    Ok(())
}
