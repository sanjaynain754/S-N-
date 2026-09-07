//! S+N++ abstract syntax tree and language data models.

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Str,
    Bool,
    Unit,
    Channel,
    Thread,
    Ref(Box<Type>),
    MutRef(Box<Type>),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Str(String),
    Bool(bool),
    Var(String),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Decl(String, Type),
    Assign(String, Expr),
    Expr(Expr),
    Return(Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub param_types: Vec<Type>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    Unit,
    Thread(usize),
    Channel(usize),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<String>,
    pub functions: Vec<Function>,
}
