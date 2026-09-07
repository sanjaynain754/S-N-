//! S+N++ lexer and token definitions.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fn, Let, If, Else, While, Return, True, False, Mut, Import,
    Ident(String), Int(i64), Str(String), Amp,
    Plus, Minus, Star, Slash, Eq, EqEq, BangEq,
    Lt, Lte, Gt, Gte, LParen, RParen, LBrace, RBrace,
    Comma, Colon, Dot, Arrow, Semi, Eof,
}

pub fn lex(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if c == '#' { while i < chars.len() && chars[i] != '\n' { i += 1; } continue; }
        if c.is_ascii_digit() {
            let s = i;
            while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
            let text: String = chars[s..i].iter().collect();
            out.push(Token::Int(text.parse().map_err(|_| "invalid integer".to_string())?));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let s = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
            let word: String = chars[s..i].iter().collect();
            out.push(match word.as_str() {
                "fn" => Token::Fn, "let" => Token::Let, "if" => Token::If,
                "else" => Token::Else, "while" => Token::While, "return" => Token::Return,
                "true" => Token::True, "false" => Token::False, "mut" => Token::Mut, "import" => Token::Import,
                _ => Token::Ident(word),
            });
            continue;
        }
        if c == '"' {
            i += 1;
            let s = i;
            while i < chars.len() && chars[i] != '"' { i += 1; }
            if i == chars.len() { return Err("unterminated string".into()); }
            out.push(Token::Str(chars[s..i].iter().collect()));
            i += 1;
            continue;
        }
        let token = match c {
            '&' => Token::Amp, '+' => Token::Plus, '*' => Token::Star, '/' => Token::Slash,
            '(' => Token::LParen, ')' => Token::RParen, '{' => Token::LBrace, '}' => Token::RBrace,
            ',' => Token::Comma, ':' => Token::Colon, '.' => Token::Dot, ';' => Token::Semi,
            '-' => { if i + 1 < chars.len() && chars[i + 1] == '>' { i += 1; Token::Arrow } else { Token::Minus } }
            '=' => { if i + 1 < chars.len() && chars[i + 1] == '=' { i += 1; Token::EqEq } else { Token::Eq } }
            '!' => { if i + 1 < chars.len() && chars[i + 1] == '=' { i += 1; Token::BangEq } else { return Err("unexpected !".into()); } }
            '<' => { if i + 1 < chars.len() && chars[i + 1] == '=' { i += 1; Token::Lte } else { Token::Lt } }
            '>' => { if i + 1 < chars.len() && chars[i + 1] == '=' { i += 1; Token::Gte } else { Token::Gt } }
            _ => return Err(format!("unexpected character: {c}")),
        };
        out.push(token);
        i += 1;
    }
    out.push(Token::Eof);
    Ok(out)
}
