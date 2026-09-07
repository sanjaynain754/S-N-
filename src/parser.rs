//! S+N++ recursive-descent parser.

use crate::ast::{Expr, Function, Program, Stmt, Type};
use crate::lexer::Token;

pub fn parse(source: &str) -> Result<Vec<Function>, String> {
    Ok(parse_program(source)?.functions)
}

pub fn parse_program(source: &str) -> Result<Program, String> {
    let tokens = crate::lexer::lex(source)?;
    let mut parser = Parser::new(tokens);
    parser.program()
}

struct Parser { tokens: Vec<Token>, position: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, position: 0 } }
    fn current(&self) -> &Token { &self.tokens[self.position] }
    fn bump(&mut self) -> Token { let token = self.tokens[self.position].clone(); self.position += 1; token }
    fn eat(&mut self, wanted: Token) -> Result<(), String> {
        if *self.current() == wanted { self.bump(); Ok(()) }
        else { Err(format!("expected {:?}, got {:?}", wanted, self.current())) }
    }
    fn program(&mut self) -> Result<Program, String> {
        let mut imports = Vec::new();
        while *self.current() == Token::Import { imports.push(self.import_path()?); }
        let mut functions = Vec::new();
        while *self.current() != Token::Eof { functions.push(self.function()?); }
        Ok(Program { imports, functions })
    }
    fn import_path(&mut self) -> Result<String, String> {
        self.eat(Token::Import)?;
        let mut parts = Vec::new();
        match self.bump() { Token::Ident(name) => parts.push(name), token => return Err(format!("expected module name after import, got {:?}", token)) }
        while *self.current() == Token::Dot {
            self.bump();
            match self.bump() { Token::Ident(name) => parts.push(name), token => return Err(format!("expected module name after `.`, got {:?}", token)) }
        }
        if *self.current() == Token::Semi { self.bump(); }
        Ok(parts.join("."))
    }
    fn parse_type(&mut self) -> Result<Type, String> {
        if *self.current() == Token::Amp {
            self.bump();
            if *self.current() == Token::Mut { self.bump(); return Ok(Type::MutRef(Box::new(self.parse_type()?))); }
            return Ok(Type::Ref(Box::new(self.parse_type()?)));
        }
        match self.bump() {
            Token::Ident(name) => match name.as_str() {
                "Int" => Ok(Type::Int), "String" => Ok(Type::Str), "Bool" => Ok(Type::Bool),
                "Unit" => Ok(Type::Unit), "Channel" => Ok(Type::Channel), "Thread" => Ok(Type::Thread),
                _ => Err(format!("unknown type `{name}`")),
            },
            token => Err(format!("expected type, got {:?}", token)),
        }
    }
    fn function(&mut self) -> Result<Function, String> {
        self.eat(Token::Fn)?;
        let name = match self.bump() { Token::Ident(name) => name, _ => return Err("expected function name".into()) };
        self.eat(Token::LParen)?;
        let mut params = Vec::new(); let mut param_types = Vec::new();
        if *self.current() != Token::RParen {
            loop {
                match self.bump() { Token::Ident(name) => params.push(name), _ => return Err("expected parameter".into()) }
                self.eat(Token::Colon)?; param_types.push(self.parse_type()?);
                if *self.current() == Token::Comma { self.bump(); } else { break; }
            }
        }
        self.eat(Token::RParen)?;
        let return_type = if *self.current() == Token::Arrow { self.bump(); self.parse_type()? } else { Type::Unit };
        self.eat(Token::LBrace)?;
        let body = self.block()?;
        Ok(Function { name, params, param_types, return_type, body })
    }
    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while *self.current() != Token::RBrace && *self.current() != Token::Eof {
            statements.push(self.statement()?);
            if *self.current() == Token::Semi { self.bump(); }
        }
        self.eat(Token::RBrace)?; Ok(statements)
    }
    fn statement(&mut self) -> Result<Stmt, String> {
        match self.current() {
            Token::Let => { self.bump(); self.let_statement() }
            Token::If => self.if_statement(),
            Token::While => self.while_statement(),
            Token::Return => { self.bump(); Ok(Stmt::Return(self.expression()?)) }
            Token::Ident(name) => {
                if self.tokens.get(self.position + 1) == Some(&Token::Eq) {
                    let name = name.clone(); self.bump(); self.bump(); return Ok(Stmt::Assign(name, self.expression()?));
                }
                Ok(Stmt::Expr(self.expression()?))
            }
            _ => Ok(Stmt::Expr(self.expression()?)),
        }
    }
    fn let_statement(&mut self) -> Result<Stmt, String> {
        let name = match self.bump() { Token::Ident(name) => name, _ => return Err("expected variable name".into()) };
        let declared = if *self.current() == Token::Colon { self.bump(); Some(self.parse_type()?) } else { None };
        if *self.current() != Token::Eq {
            if let Some(ty) = declared { return Ok(Stmt::Decl(name, ty)); }
            return Err("let requires an initializer or explicit type".into());
        }
        self.bump(); Ok(Stmt::Let(name, self.expression()?))
    }
    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.bump(); let condition = self.expression()?; self.eat(Token::LBrace)?; let yes = self.block()?;
        let no = if *self.current() == Token::Else { self.bump(); self.eat(Token::LBrace)?; self.block()? } else { Vec::new() };
        Ok(Stmt::If(condition, yes, no))
    }
    fn while_statement(&mut self) -> Result<Stmt, String> { self.bump(); let condition = self.expression()?; self.eat(Token::LBrace)?; Ok(Stmt::While(condition, self.block()?)) }
    fn expression(&mut self) -> Result<Expr, String> { self.compare() }
    fn compare(&mut self) -> Result<Expr, String> { let mut expression = self.add()?; while matches!(self.current(), Token::EqEq|Token::BangEq|Token::Lt|Token::Lte|Token::Gt|Token::Gte) { let op = self.bump(); expression = Expr::Binary(Box::new(expression), op, Box::new(self.add()?)); } Ok(expression) }
    fn add(&mut self) -> Result<Expr, String> { let mut expression = self.multiply()?; while matches!(self.current(), Token::Plus|Token::Minus) { let op = self.bump(); expression = Expr::Binary(Box::new(expression), op, Box::new(self.multiply()?)); } Ok(expression) }
    fn multiply(&mut self) -> Result<Expr, String> { let mut expression = self.atom()?; while matches!(self.current(), Token::Star|Token::Slash) { let op = self.bump(); expression = Expr::Binary(Box::new(expression), op, Box::new(self.atom()?)); } Ok(expression) }
    fn atom(&mut self) -> Result<Expr, String> {
        match self.bump() {
            Token::Int(value) => Ok(Expr::Int(value)), Token::Str(value) => Ok(Expr::Str(value)),
            Token::True => Ok(Expr::Bool(true)), Token::False => Ok(Expr::Bool(false)),
            Token::Ident(name) => {
                if *self.current() == Token::LParen {
                    self.bump(); let mut args = Vec::new();
                    if *self.current() != Token::RParen { loop { args.push(self.expression()?); if *self.current() == Token::Comma { self.bump(); } else { break; } } }
                    self.eat(Token::RParen)?; Ok(Expr::Call(name, args))
                } else { Ok(Expr::Var(name)) }
            }
            Token::LParen => { let expression = self.expression()?; self.eat(Token::RParen)?; Ok(expression) }
            token => Err(format!("unexpected token {:?}", token)),
        }
    }
}
