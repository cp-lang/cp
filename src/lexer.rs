use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+|//[^\n]*")] // Skip whitespace and comments
pub enum Token {
    // --- Keywords ---
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("match")]
    Match,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("spawn")]
    Spawn,
    #[token("receive")]
    Receive,
    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("as")]
    As,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("fn")]
    Fn,
    #[token("class")]
    Class,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("for")]
    For,
    #[token("enum")]
    Enum,
    #[token("interface")]
    Interface,
    #[token("implements")]
    Implements,
    #[token("new")]
    New,
    #[token("error")]
    ErrorKw,
    #[token("defer")]
    Defer,
    #[token("errdefer")]
    ErrDefer,
    #[token("void")]
    Void,

    // --- Types (RFC-001 4.1) ---
    #[token("i32")]
    I32,
    #[token("u64")]
    U64,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("usize")]
    USize,
    #[token("bool")]
    Bool,
    #[token("string")]
    String,

    // --- Literals ---
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLiteral(i64),
    #[regex(r"[0-9]*\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    StringLiteral(String),

    // --- Operators & Punctuation ---
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("==")]
    Eq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessEq,
    #[token(">=")]
    GreaterEq,
    #[token("?")]
    Question, // Error bubbling
    #[token("!")]
    Bang,
    #[token("..")]
    DotDot,   // Slices
    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("@")]
    At,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,

    // --- Zig Escape Hatch ---
    #[token("__zig__")]
    ZigEscape,

    // Error handling
    Error,
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Token::lexer(input),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, std::ops::Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let token_result = self.inner.next()?;
        let token = match token_result {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Lexer Error: Unknown slice '{s}' at {span:?}", s = self.inner.slice(), span = self.inner.span());
                Token::Error
            },
        };
        Some((token, self.inner.span()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lexing() {
        let input = "let x: i32 = 42;";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next().unwrap().0, Token::Let);
        assert_eq!(lexer.next().unwrap().0, Token::Ident("x".to_string()));
        assert_eq!(lexer.next().unwrap().0, Token::Colon);
        assert_eq!(lexer.next().unwrap().0, Token::I32);
        assert_eq!(lexer.next().unwrap().0, Token::Assign);
        assert_eq!(lexer.next().unwrap().0, Token::IntLiteral(42));
        assert_eq!(lexer.next().unwrap().0, Token::Semicolon);
    }

    #[test]
    fn test_rfc_features() {
        let input = "spawn aiWorker(); match result { Ok(v) => reply(v), Err(_) => ? }";
        let lexer = Lexer::new(input);
        
        // Simplified check
        let tokens: Vec<Token> = lexer.map(|(t, _)| t).collect();
        assert!(tokens.contains(&Token::Spawn));
        assert!(tokens.contains(&Token::Match));
        assert!(tokens.contains(&Token::FatArrow));
        assert!(tokens.contains(&Token::Question));
    }
}
