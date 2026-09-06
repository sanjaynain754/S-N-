//! S+N++ recursive-descent parser facade.

pub fn parse(source: &str) -> Result<Vec<crate::Function>, String> {
    let tokens = crate::lexer::lex(source)?;
    let mut parser = crate::Parser::new(tokens);
    parser.program()
}
