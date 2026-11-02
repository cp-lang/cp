use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+|//[^\n]*|/\*([^*]|\*[^/])*\*/")] // Skip whitespace and comments
pub enum Token {
    // --- Keywords ---
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("match")]
    Match,
    #[token("switch")]
    Switch,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("null")]
    Null,
    #[token("dyn")]
    Dyn,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("mut")]
    Mut,
    #[token("move")]
    Move,
    #[token("ref")]
    Ref,
    #[token("struct")]
    Struct,
    #[token("static")]
    Static,
    #[token("unsafe")]
    Unsafe,
    #[token("super")]
    Super,
    #[token("meta")]
    Meta,
    #[token("dec")]
    Dec,
    #[token("macro")]
    Macro,

    #[token("export")]
    #[token("pub")]
    Export,
    #[token("undefined")]
    Undefined,
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
    #[token("of")]
    Of,
    #[token("in")]
    In,
    #[token("enum")]
    Enum,
    #[token("interface")]
    Interface,
    #[token("extends")]
    Extends,
    #[token("implements")]
    Implements,
    #[token("new")]
    New,
    #[token("error")]
    #[token("err")]
    ErrorKw,
    #[token("defer")]
    Defer,
    #[token("errdefer")]
    ErrDefer,
    #[token("void")]
    Void,
    #[token("any")]
    Any,
    #[token("auto")]
    #[token("anytype")]
    Auto,

    // --- Types (RFC-001 4.1) ---
    #[token("u8")]
    U8,
    #[token("i8")]
    I8,
    #[token("u16")]
    U16,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("u32")]
    U32,
    #[token("i64")]
    I64,
    #[token("u64")]
    U64,
    #[token("i128")]
    I128,
    #[token("u128")]
    U128,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("isize")]
    ISize,
    #[token("usize")]
    USize,
    #[token("bool")]
    Bool,
    #[token("string")]
    String,

    // --- Literals ---
    #[regex(r"(\$\{?[a-zA-Z0-9_]+\}?|[a-zA-Z_])(\$\{?[a-zA-Z0-9_]+\}?|[a-zA-Z0-9_])*", |lex| lex.slice().to_string())]
    Ident(String),
    #[regex(r"0x[0-9a-fA-F]+", |lex| u64::from_str_radix(&lex.slice()[2..], 16).map(|v| v as i64).ok())]
    #[regex(r"0b[01]+", |lex| u64::from_str_radix(&lex.slice()[2..], 2).map(|v| v as i64).ok())]
    #[regex(r"0o[0-7]+", |lex| u64::from_str_radix(&lex.slice()[2..], 8).map(|v| v as i64).ok())]
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().or_else(|_| lex.slice().parse::<u64>().map(|v| v as i64)).ok())]
    #[regex(r"'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        let content = &s[1..s.len()-1];
        if content.starts_with('\\') {
            match content {
                "\\n" => Some(b'\n' as i64),
                "\\r" => Some(b'\r' as i64),
                "\\t" => Some(b'\t' as i64),
                "\\0" => Some(0),
                "\\\\" => Some(b'\\' as i64),
                "\\'" => Some(b'\'' as i64),
                _ if content.len() == 2 => Some(content.as_bytes()[1] as i64),
                _ => None,
            }
        } else {
            content.chars().next().map(|c| c as i64)
        }
    })]
    IntLiteral(i64),
    #[regex(r"[0-9]*\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLiteral(String),
    #[regex(r#"`([^`\\]|\\.)*`"#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    TemplateStringRaw(String),

    // --- Operators & Punctuation ---
    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("%=")]
    PercentAssign,
    #[token("&=")]
    AmpAssign,
    #[token("|=")]
    PipeAssign,
    #[token("^=")]
    CaretAssign,
    #[token("<<=")]
    ShlAssign,
    #[token(">>=")]
    ShrAssign,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("&&")]
    AmpAmp,
    #[token("||")]
    PipePipe,
    #[token("%")]
    Percent,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
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
    #[token("??")]
    QuestionQuestion,
    #[token("?.")]
    QuestionDot,
    #[token("?")]
    Question, // Error bubbling / fallback
    #[token("!")]
    Bang,
    #[token("...")]
    DotDotDot,
    #[token("..=")]
    DotDotEq,
    #[token("..")]
    DotDot, // Slices / Ranges
    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("#{")]
    HashLBrace,
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
    #[token(":=")]
    ColonAssign,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("@")]
    At,
    #[token("&")]
    Ampersand,
    #[token("$")]
    Dollar,
    #[token("|>")]
    Pipeline,
    #[token("|")]
    Pipe,

    // --- Zig Escape Hatch ---
    #[token("__zig__")]
    #[token("zig")]
    ZigEscape,

    // Error handling
    Error,
}

#[derive(Clone)]
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
                eprintln!(
                    "Lexer Error: Unknown slice '{s}' at {span:?}",
                    s = self.inner.slice(),
                    span = self.inner.span()
                );
                Token::Error
            }
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

        assert!(tokens.contains(&Token::Match));
        assert!(tokens.contains(&Token::FatArrow));
        assert!(tokens.contains(&Token::Question));
    }
}
