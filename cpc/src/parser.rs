use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::{Lexer, Token};
use crate::Span;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Diagnostic: {0:?}")]
    Diagnostic(Diagnostic),
    #[error("Unexpected token: {0:?} at {1:?}")]
    UnexpectedToken(Token, Span),
    #[error("Expected token: {0:?}, found {1:?} at {2:?}")]
    ExpectedToken(Token, Token, Span),
    #[error("End of input reached unexpectedly")]
    EndOfInput,
    #[error("Parse error: {0} at {1:?}")]
    Generic(String, Span),
    #[error("Multiple parse errors")]
    Multiple(Vec<Diagnostic>),
}

impl ParseError {
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            ParseError::Multiple(diags) => {
                diags.first().cloned().unwrap_or_else(|| Diagnostic::new("E0100", "syntax error"))
            }
            ParseError::Diagnostic(d) => d.clone(),
            ParseError::UnexpectedToken(tok, span) => match tok {
                Token::Interface => Diagnostic::new("E0102", "CP does not support 'interface' keyword")
                    .with_span(*span)
                    .with_cause("'interface' has been abolished in CP to decouple data layout from behavior.")
                    .with_help("in CP, use 'trait' for behavior contracts and 'class' / 'struct' for data containers.")
                    .with_suggestion("replace 'interface' with 'trait': trait <Name> { ... }"),
                Token::Try => Diagnostic::new("E0103", "traditional try/catch exception block is not supported in CP")
                    .with_span(*span)
                    .with_cause("CP has completely abolished try/catch in favor of zero-cost error unions and '?' operator.")
                    .with_help("use '?' to propagate errors upward, or 'expr catch err => { ... }' / '?? fallback' to handle them.")
                    .with_suggestion("use 'let val = expr ?;' or 'expr catch err => { ... }'"),
                Token::Mut => Diagnostic::new("E0104", "'mut' keyword is not used for variable declarations")
                    .with_span(*span)
                    .with_cause("in CP, variables declared with 'let' are already mutable by default, and 'const' is used for immutable bindings.")
                    .with_help("CP has abolished 'let mut'. Use 'let' directly for mutable variables. For mutable borrows in function parameters, prefix with '*': '*param: Type'.")
                    .with_suggestion("remove 'mut': use 'let ...' instead"),
                Token::Any => Diagnostic::new("E0105", "'any' type is not supported in CP")
                    .with_span(*span)
                    .with_cause("untyped 'any' is prohibited to ensure strict type safety and zero-cost runtime performance.")
                    .with_help("use a concrete strong type (e.g. i32, string), a generic type parameter '<T>', or '*anyopaque' for C pointers.")
                    .with_suggestion("replace 'any' with a concrete type or generic '<T>'"),
                _ => Diagnostic::new("E0100", format!("unexpected token '{:?}'", tok))
                    .with_span(*span)
                    .with_cause(format!("parser encountered unexpected token '{:?}'", tok))
                    .with_help("check the syntax around this token."),
            },
            ParseError::ExpectedToken(expected, found, span) => {
                if *expected == Token::LParen {
                    Diagnostic::new("E0101", "control flow statement must be enclosed in parentheses")
                        .with_span(*span)
                        .with_cause("in CP, control flow statements ('if', 'while', 'for') must be enclosed in parentheses to eliminate ambiguity with struct literal syntax and maintain syntax consistency.")
                        .with_help("headers in 'if', 'while' and 'for' must be enclosed in parentheses: if (condition) { ... }, while (condition) { ... }, for (item of collection) { ... }")
                        .with_suggestion("wrap the control flow expression in parentheses")
                } else {
                    Diagnostic::new("E0111", format!("expected token '{:?}', found '{:?}'", expected, found))
                        .with_span(*span)
                        .with_cause(format!("syntax requires '{:?}' at this position, but found '{:?}'", expected, found))
                        .with_help(format!("ensure that '{:?}' is provided.", expected))
                        .with_suggestion(format!("insert '{:?}' before '{:?}'", expected, found))
                }
            }
            ParseError::EndOfInput => Diagnostic::new("E0199", "end of input reached unexpectedly")
                .with_cause("source code terminated prematurely, likely missing a closing bracket or delimiter.")
                .with_help("check for unclosed braces '{', parentheses '(', or quotes."),
            ParseError::Generic(msg, span) => {
                if msg.contains("= cause:") || msg.contains("= suggestion:") || msg.contains("= help:") {
                    let first_line = msg.lines().next().unwrap_or(msg).to_string();
                    let mut diag = Diagnostic::new("E0101", first_line).with_span(*span);
                    for line in msg.lines() {
                        let trimmed = line.trim();
                        if let Some(rest) = trimmed.strip_prefix("= cause:") {
                            diag.cause = Some(rest.trim().to_string());
                        } else if let Some(rest) = trimmed.strip_prefix("= help:") {
                            diag.help = Some(rest.trim().to_string());
                        } else if let Some(rest) = trimmed.strip_prefix("= suggestion:") {
                            diag.suggestion = Some(rest.trim().to_string());
                        }
                    }
                    diag
                } else if msg.contains("Expected 'of' in for loop") {
                    Diagnostic::new("E0106", "invalid for loop syntax")
                        .with_span(*span)
                        .with_cause("CP does not use C-style three-clause 'for (;;)' loops.")
                        .with_help("CP uses 'for elem of collection' or 'while (cond)' for indexed loops.")
                        .with_suggestion("use 'for elem of list' or 'while (i < len)'")
                } else {
                    Diagnostic::new("E0190", format!("syntax error: {}", msg))
                        .with_span(*span)
                        .with_cause(msg.clone())
                        .with_help("check syntax matching CP language grammar rules.")
                }
            }
        }
    }

    pub fn to_diagnostics(&self) -> Vec<Diagnostic> {
        match self {
            ParseError::Multiple(diags) => diags.clone(),
            other => vec![other.to_diagnostic()],
        }
    }

    pub fn render_all(&self, source: Option<&str>, path: Option<&Path>) -> String {
        let diags = self.to_diagnostics();
        let mut out = String::new();
        for (i, d) in diags.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&d.render(source, path));
        }
        out
    }

    pub fn to_json_report(&self, source: Option<&str>, path: Option<&Path>) -> String {
        let diags = self.to_diagnostics();
        let json_diags = diags
            .iter()
            .map(|d| d.to_diagnostic_json(source, path))
            .collect();
        let report = crate::diagnostic::DiagnosticReport {
            status: "error".to_string(),
            diagnostics: json_diags,
        };
        serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    input: &'a str,
    peeked: Option<(Token, Span)>,
    peeked2: Option<(Token, Span)>,
    prev_token_end: usize,
    in_match_guard: bool,
    in_pipeline_rhs: usize,
    in_switch_arm: usize,
    in_paren_pattern: usize,
    pub errors: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            input,
            peeked: None,
            peeked2: None,
            prev_token_end: 0,
            in_match_guard: false,
            in_pipeline_rhs: 0,
            in_switch_arm: 0,
            in_paren_pattern: 0,
            errors: Vec::new(),
        }
    }

    fn bump(&mut self) -> ParseResult<(Token, Span)> {
        let (token, span) = if let Some(p) = self.peeked.take() {
            self.peeked = self.peeked2.take();
            p
        } else {
            let (token, range) = self.lexer.next().ok_or(ParseError::EndOfInput)?;
            (
                token,
                Span {
                    start: range.start,
                    end: range.end,
                },
            )
        };
        self.prev_token_end = span.end;
        Ok((token, span))
    }

    fn peek(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            let (token, range) = self.lexer.next()?;
            self.peeked = Some((
                token,
                Span {
                    start: range.start,
                    end: range.end,
                },
            ));
        }
        self.peeked.as_ref().map(|(t, _)| t)
    }

    fn peek_next(&mut self) -> Option<&Token> {
        let _ = self.peek();
        if self.peeked2.is_none() {
            if let Some((token, range)) = self.lexer.next() {
                self.peeked2 = Some((
                    token,
                    Span {
                        start: range.start,
                        end: range.end,
                    },
                ));
            }
        }
        self.peeked2.as_ref().map(|(t, _)| t)
    }

    fn peek_span(&mut self) -> Span {
        if self.peeked.is_none() {
            if let Some((token, range)) = self.lexer.next() {
                self.peeked = Some((
                    token,
                    Span {
                        start: range.start,
                        end: range.end,
                    },
                ));
            } else {
                return Span { start: 0, end: 0 };
            }
        }
        self.peeked
            .as_ref()
            .map(|(_, s)| *s)
            .unwrap_or(Span { start: 0, end: 0 })
    }

    fn expect(&mut self, expected: Token) -> ParseResult<Span> {
        let (token, span) = self.bump()?;
        if token == expected {
            Ok(span)
        } else {
            Err(ParseError::ExpectedToken(expected, token, span))
        }
    }

    pub fn push_error(&mut self, diag: Diagnostic) {
        if let Some(last) = self.errors.last() {
            if last.span == diag.span && last.code == diag.code {
                return;
            }
        }
        if self.errors.len() < 50 {
            self.errors.push(diag);
        }
    }

    pub fn recover_stmt(&mut self) {
        let mut brace_depth: usize = 0;
        let mut iterations = 0;
        while let Some(tok) = self.peek() {
            iterations += 1;
            if iterations > 1000 {
                let _ = self.bump();
                return;
            }
            match tok {
                Token::LBrace => {
                    brace_depth += 1;
                    let _ = self.bump();
                }
                Token::RBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                        let _ = self.bump();
                        if brace_depth == 0 {
                            return;
                        }
                    } else {
                        // Reached enclosing block's right brace
                        return;
                    }
                }
                Token::Semicolon => {
                    if brace_depth == 0 {
                        let _ = self.bump();
                        return;
                    } else {
                        let _ = self.bump();
                    }
                }
                Token::Let
                | Token::Const
                | Token::If
                | Token::While
                | Token::For
                | Token::Return
                | Token::Switch
                | Token::Break
                | Token::Continue
                | Token::Defer
                | Token::ErrDefer
                | Token::Unsafe
                    if brace_depth == 0 =>
                {
                    return;
                }
                _ => {
                    let _ = self.bump();
                }
            }
        }
    }

    pub fn recover_module_item(&mut self) {
        let mut brace_depth: usize = 0;
        let mut iterations = 0;
        while let Some(tok) = self.peek() {
            iterations += 1;
            if iterations > 1000 {
                let _ = self.bump();
                return;
            }
            match tok {
                Token::LBrace => {
                    brace_depth += 1;
                    let _ = self.bump();
                }
                Token::RBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                        let _ = self.bump();
                        if brace_depth == 0 {
                            return;
                        }
                    } else {
                        let _ = self.bump();
                        return;
                    }
                }
                Token::Semicolon if brace_depth == 0 => {
                    let _ = self.bump();
                    return;
                }
                Token::Export
                | Token::Private
                | Token::Import
                | Token::Class
                | Token::Struct
                | Token::Union
                | Token::Enum
                | Token::Fn
                | Token::Async
                | Token::TypeKw
                | Token::Const
                | Token::Let
                | Token::Var
                | Token::Trait
                | Token::Impl
                | Token::Interface
                | Token::At
                    if brace_depth == 0 =>
                {
                    return;
                }
                _ => {
                    let _ = self.bump();
                }
            }
        }
    }

    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let mut body = Vec::new();
        let start = self.peek_span().start;
        while self.peek().is_some() {
            let start_pos = self.peek_span().start;
            match self.parse_module_item() {
                Ok(items) => body.extend(items),
                Err(err) => {
                    self.push_error(err.to_diagnostic());
                    self.recover_module_item();
                    if let Some(_) = self.peek() {
                        if self.peek_span().start == start_pos {
                            let _ = self.bump();
                        }
                    }
                }
            }
        }
        let end = self.peek_span().end;
        if !self.errors.is_empty() {
            return Err(ParseError::Multiple(self.errors.clone()));
        }
        Ok(Module {
            span: Span { start, end },
            body,
        })
    }

    fn clone_lookahead(&self) -> Parser<'a> {
        Parser {
            lexer: self.lexer.clone(),
            input: self.input,
            peeked: self.peeked.clone(),
            peeked2: self.peeked2.clone(),
            prev_token_end: self.prev_token_end,
            in_match_guard: self.in_match_guard,
            in_pipeline_rhs: self.in_pipeline_rhs,
            in_switch_arm: self.in_switch_arm,
            in_paren_pattern: self.in_paren_pattern,
            errors: Vec::new(),
        }
    }

    fn is_decl_keyword(t: &Token) -> bool {
        matches!(
            t,
            Token::Export
                | Token::Const
                | Token::Let
                | Token::Var
                | Token::ThreadLocal
                | Token::Fn
                | Token::Async
                | Token::Static
                | Token::Unsafe
                | Token::Class
                | Token::Struct
                | Token::Union
                | Token::Ref
                | Token::Meta
                | Token::Dec
                | Token::Macro
                | Token::Trait
                | Token::Impl
                | Token::Enum
                | Token::Interface
                | Token::ErrorKw
                | Token::TypeKw
        ) || matches!(t, Token::Ident(id) if id == "extern" || id == "inline" || id == "noinline")
    }

    fn parse_meta_ident(&mut self) -> ParseResult<Ident> {
        let (tok, span) = self.bump()?;
        let sym = match tok {
            Token::Ident(s) => s,
            Token::Class => "class".to_string(),
            Token::Struct => "struct".to_string(),
            Token::Fn => "fn".to_string(),
            Token::Auto => "auto".to_string(),
            Token::Ref => "ref".to_string(),
            Token::Let => "let".to_string(),
            Token::Const => "const".to_string(),
            Token::Meta => "meta".to_string(),
            Token::Dec => "dec".to_string(),
            Token::Macro => "macro".to_string(),
            Token::Async => "async".to_string(),
            Token::Static => "static".to_string(),
            Token::Mut => "mut".to_string(),
            Token::Move => "move".to_string(),
            Token::Own => "own".to_string(),
            Token::Import => "import".to_string(),
            Token::Export => "export".to_string(),
            Token::TypeKw => "type".to_string(),
            Token::CStr => "cstr".to_string(),
            Token::As => "as".to_string(),
            Token::From => "from".to_string(),
            Token::Unsafe => "unsafe".to_string(),
            _ => {
                return Err(ParseError::ExpectedToken(
                    Token::Ident("identifier".to_string()),
                    tok,
                    span,
                ))
            }
        };
        Ok(Ident { span, sym })
    }

    fn parse_meta_arg(&mut self) -> ParseResult<MetaArg> {
        let is_ident_like = matches!(
            self.peek(),
            Some(Token::Ident(_))
                | Some(Token::Class)
                | Some(Token::Struct)
                | Some(Token::Fn)
                | Some(Token::Ref)
                | Some(Token::Let)
                | Some(Token::Const)
                | Some(Token::Meta)
                | Some(Token::Dec)
                | Some(Token::Macro)
                | Some(Token::Async)
                | Some(Token::Static)
                | Some(Token::Mut)
                | Some(Token::Move)
                | Some(Token::Own)
                | Some(Token::Import)
                | Some(Token::Export)
                | Some(Token::TypeKw)
                | Some(Token::CStr)
                | Some(Token::As)
                | Some(Token::From)
                | Some(Token::Unsafe)
        );
        if is_ident_like {
            let ident = self.parse_meta_ident()?;
            if self.peek() == Some(&Token::Colon) {
                self.bump()?; // consume Colon
                let expr = self.parse_primary_expr()?;
                return Ok(MetaArg::Named(ident, expr));
            } else if self.peek() == Some(&Token::Dot) {
                self.bump()?; // consume Dot
                let prop = self.parse_ident()?;
                return Ok(MetaArg::Positional(Expr::Member(MemberExpr {
                    span: Span {
                        start: ident.span.start,
                        end: prop.span.end,
                    },
                    obj: Box::new(Expr::Ident(ident)),
                    prop,
                })));
            } else {
                return Ok(MetaArg::Positional(Expr::Ident(ident)));
            }
        }
        let expr = self.parse_primary_expr()?;
        Ok(MetaArg::Positional(expr))
    }

    fn parse_single_meta_inner(&mut self, start: usize) -> ParseResult<MetaItem> {
        let name = self.parse_meta_ident()?;
        let mut args = Vec::new();
        if self.peek() == Some(&Token::LParen) {
            self.bump()?;
            while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                args.push(self.parse_meta_arg()?);
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
            }
            self.expect(Token::RParen)?;
        }
        let end = self.peek_span().start;
        Ok(MetaItem {
            span: Span { start, end },
            name,
            args,
        })
    }

    #[allow(dead_code)]
    fn parse_meta_item(&mut self) -> ParseResult<MetaItem> {
        let start = self.expect(Token::At)?.start;
        self.parse_single_meta_inner(start)
    }

    fn parse_meta_list(&mut self) -> ParseResult<Vec<MetaItem>> {
        let mut list = Vec::new();
        while self.peek() == Some(&Token::At) {
            if self.peek_next() == Some(&Token::LBracket) {
                let _ = self.bump()?; // consume '@'
                self.expect(Token::LBracket)?; // consume '['
                while self.peek() != Some(&Token::RBracket) && self.peek().is_some() {
                    let start = if self.peek() == Some(&Token::At) {
                        self.bump()?.1.start
                    } else {
                        self.peek_span().start
                    };
                    let item = self.parse_single_meta_inner(start)?;
                    list.push(item);
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                self.expect(Token::RBracket)?;
            } else {
                let start = self.expect(Token::At)?.start;
                let item = self.parse_single_meta_inner(start)?;
                list.push(item);
            }
        }
        Ok(list)
    }

    fn parse_meta_def(&mut self, meta_list: Vec<MetaItem>) -> ParseResult<MetaDef> {
        let start = self.expect(Token::Meta)?.start;
        let ident = self.parse_ident()?;
        let mut params = Vec::new();
        if self.peek() == Some(&Token::LParen) {
            self.bump()?;
            while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                let p_start = self.peek_span().start;
                let p_ident = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                let mut default = None;
                if self.peek() == Some(&Token::Assign) {
                    self.bump()?;
                    default = Some(self.parse_primary_expr()?);
                }
                params.push(MetaParam {
                    span: Span {
                        start: p_start,
                        end: self.peek_span().start,
                    },
                    ident: p_ident,
                    ty,
                    default,
                });
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
            }
            self.expect(Token::RParen)?;
        }
        if self.peek() == Some(&Token::Semicolon) {
            self.bump()?;
        }
        let end = self.peek_span().start;
        Ok(MetaDef {
            span: Span { start, end },
            ident,
            params,
            meta: meta_list,
        })
    }

    fn parse_dec_def(&mut self, meta_list: Vec<MetaItem>) -> ParseResult<DecDef> {
        let start = self.expect(Token::Dec)?.start;
        let ident = self.parse_ident()?;
        let params = self.parse_params()?;
        let mut return_type = None;
        if self.peek() == Some(&Token::Bang) {
            self.bump()?;
            if self.is_error_type_rhs_start() {
                let err = self.parse_error_type_rhs()?;
                let ok_ty = if self.peek().map(Self::is_type_start).unwrap_or(false)
                    && self.peek() != Some(&Token::LBrace)
                    && self.peek() != Some(&Token::Semicolon)
                {
                    self.parse_type()?
                } else {
                    Type::Void
                };
                return_type = Some(Type::ResultType {
                    ok: Box::new(ok_ty),
                    err,
                });
            } else {
                let ok_ty = if self.peek().map(Self::is_type_start).unwrap_or(false)
                    && self.peek() != Some(&Token::LBrace)
                    && self.peek() != Some(&Token::Semicolon)
                {
                    self.parse_type()?
                } else {
                    Type::Void
                };
                return_type = Some(Type::ErrorUnion(Box::new(ok_ty)));
            }
        } else if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
            self.bump()?;
            return_type = Some(self.parse_type()?);
        } else if self.peek().map(Self::is_type_start).unwrap_or(false)
            && self.peek() != Some(&Token::LBrace)
            && self.peek() != Some(&Token::Semicolon)
        {
            return_type = Some(self.parse_type()?);
        }
        let body = self.parse_block_stmt()?;
        let end = body.span.end;
        Ok(DecDef {
            span: Span { start, end },
            ident,
            params,
            return_type,
            body,
            meta: meta_list,
        })
    }

    fn parse_macro_def(&mut self, meta_list: Vec<MetaItem>) -> ParseResult<MacroDef> {
        let start = self.expect(Token::Macro)?.start;
        let ident = self.parse_ident()?;
        let params = self.parse_params()?;
        let mut return_type = None;
        if self.peek() == Some(&Token::Bang) {
            self.bump()?;
            if self.is_error_type_rhs_start() {
                let err = self.parse_error_type_rhs()?;
                let ok_ty = if self.peek().map(Self::is_type_start).unwrap_or(false)
                    && self.peek() != Some(&Token::LBrace)
                    && self.peek() != Some(&Token::Semicolon)
                {
                    self.parse_type()?
                } else {
                    Type::Void
                };
                return_type = Some(Type::ResultType {
                    ok: Box::new(ok_ty),
                    err,
                });
            } else {
                let ok_ty = if self.peek().map(Self::is_type_start).unwrap_or(false)
                    && self.peek() != Some(&Token::LBrace)
                    && self.peek() != Some(&Token::Semicolon)
                {
                    self.parse_type()?
                } else {
                    Type::Void
                };
                return_type = Some(Type::ErrorUnion(Box::new(ok_ty)));
            }
        } else if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
            self.bump()?;
            return_type = Some(self.parse_type()?);
        } else if self.peek().map(Self::is_type_start).unwrap_or(false)
            && self.peek() != Some(&Token::LBrace)
            && self.peek() != Some(&Token::Semicolon)
        {
            return_type = Some(self.parse_type()?);
        }
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        let mut body_stmts = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            let mut is_pub = true;
            if self.peek() == Some(&Token::Private) {
                self.bump()?;
                is_pub = false;
            } else if self.peek() == Some(&Token::Export) {
                self.bump()?;
                is_pub = true;
            }
            if self.peek() == Some(&Token::Fn) || self.peek() == Some(&Token::Async) {
                let mut func = self.parse_function()?;
                func.is_pub = is_pub;
                methods.push(func);
            } else {
                body_stmts.push(self.parse_stmt()?);
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(MacroDef {
            span: Span { start, end },
            ident,
            params,
            return_type,
            body: BlockStmt {
                span: Span { start, end },
                body: body_stmts,
            },
            methods,
            meta: meta_list,
        })
    }

    fn parse_module_item(&mut self) -> ParseResult<Vec<ModuleItem>> {
        let mut is_pub = false;
        let mut has_pub_or_priv = false;
        while self.peek() == Some(&Token::Export) || self.peek() == Some(&Token::Private) {
            has_pub_or_priv = true;
            if self.peek() == Some(&Token::Export) {
                is_pub = true;
            }
            self.bump()?;
        }
        if is_pub {
            if self.peek() == Some(&Token::LBrace) {
                return Ok(vec![ModuleItem::Export(self.parse_export_stmt()?)]);
            }
            if self.peek() == Some(&Token::Star) {
                return Ok(vec![ModuleItem::Export(self.parse_export_star_stmt()?)]);
            }
        }
        if self.peek() == Some(&Token::At) {
            // Check for @c { ... } or @c("lib") { ... } block
            let mut clone = self.clone_lookahead();
            let _ = clone.bump(); // consume '@'
            let is_c_block = if matches!(clone.peek(), Some(Token::Ident(s)) if s == "c") {
                let _ = clone.bump(); // consume 'c'
                if clone.peek() == Some(&Token::LParen) {
                    let _ = clone.bump();
                    let mut p_depth = 1;
                    while p_depth > 0 && clone.peek().is_some() {
                        match clone.bump() {
                            Ok((Token::LParen, _)) => p_depth += 1,
                            Ok((Token::RParen, _)) => p_depth -= 1,
                            _ => {}
                        }
                    }
                }
                clone.peek() == Some(&Token::LBrace)
            } else {
                false
            };
            if is_c_block {
                let _ = self.bump(); // consume '@'
                let c_ident = self.parse_ident()?; // consume 'c'
                let mut c_args = Vec::new();
                if self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                        c_args.push(self.parse_meta_arg()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RParen)?;
                }
                self.expect(Token::LBrace)?;
                let mut items = Vec::new();
                while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                    if matches!(self.peek(), Some(Token::StringLiteral(_))) {
                        let (tok, span) = self.bump()?;
                        if let Token::StringLiteral(header) = tok {
                            items.push(ModuleItem::Decl(Decl::CInclude(CIncludeDecl {
                                span,
                                header,
                            })));
                            if self.peek() == Some(&Token::Comma)
                                || self.peek() == Some(&Token::Semicolon)
                            {
                                self.bump()?;
                            }
                        }
                    } else if matches!(
                        self.peek(),
                        Some(Token::Const) | Some(Token::Var) | Some(Token::Let)
                    ) {
                        let mut v = self.parse_var_decl()?;
                        v.is_c = true;
                        v.is_extern = true;
                        if is_pub {
                            v.is_pub = true;
                        }
                        items.push(ModuleItem::Stmt(Stmt::Var(v)));
                    } else if matches!(self.peek(), Some(Token::Ident(s)) if s == "extern")
                        && matches!(
                            self.peek_next(),
                            Some(Token::Const)
                                | Some(Token::Var)
                                | Some(Token::Let)
                                | Some(Token::ThreadLocal)
                        )
                    {
                        self.bump()?; // consume 'extern'
                        let mut v = self.parse_var_decl()?;
                        v.is_c = true;
                        v.is_extern = true;
                        if is_pub {
                            v.is_pub = true;
                        }
                        items.push(ModuleItem::Stmt(Stmt::Var(v)));
                    } else {
                        let mut f = self.parse_function()?;
                        if is_pub {
                            f.is_pub = true;
                        }
                        if !f.meta.iter().any(|m| m.name.sym == "c") {
                            f.meta.push(MetaItem {
                                span: c_ident.span,
                                name: Ident {
                                    span: c_ident.span,
                                    sym: "c".to_string(),
                                },
                                args: c_args.clone(),
                            });
                        }
                        items.push(ModuleItem::Decl(Decl::Func(f)));
                    }
                }
                self.expect(Token::RBrace)?;
                return Ok(items);
            }

            let mut clone = self.clone_lookahead();
            let mut has_decl = false;
            while clone.peek() == Some(&Token::At) {
                if clone.bump().is_err() {
                    break;
                }
                if clone.peek() == Some(&Token::LBracket) {
                    if clone.bump().is_err() {
                        break;
                    }
                    let mut depth = 1;
                    while depth > 0 && clone.peek().is_some() {
                        if clone.peek() == Some(&Token::LBracket) {
                            depth += 1;
                        } else if clone.peek() == Some(&Token::RBracket) {
                            depth -= 1;
                        }
                        if clone.bump().is_err() {
                            break;
                        }
                    }
                } else {
                    if clone.bump().is_err() {
                        break;
                    }
                    if clone.peek() == Some(&Token::LParen) {
                        if clone.bump().is_err() {
                            break;
                        }
                        let mut depth = 1;
                        while depth > 0 && clone.peek().is_some() {
                            if clone.peek() == Some(&Token::LParen) {
                                depth += 1;
                            } else if clone.peek() == Some(&Token::RParen) {
                                depth -= 1;
                            }
                            if clone.bump().is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            let mut is_extern = false;
            if matches!(clone.peek(), Some(Token::Ident(id)) if id == "extern") {
                let _ = clone.bump();
                is_extern = true;
            }
            if let Some(tok) = clone.peek() {
                if matches!(
                    tok,
                    Token::Let | Token::Const | Token::Var | Token::ThreadLocal
                ) {
                    let meta_list = self.parse_meta_list()?;
                    if is_extern {
                        self.bump()?; // consume 'extern'
                    }
                    let is_tl = meta_list
                        .iter()
                        .any(|m| m.name.sym == "tls" || m.name.sym == "threadlocal");
                    let is_c = meta_list.iter().any(|m| m.name.sym == "c");
                    let mut v = self.parse_var_decl()?;
                    if has_pub_or_priv {
                        v.is_pub = is_pub;
                    }
                    if is_tl {
                        v.is_threadlocal = true;
                    }
                    if is_extern {
                        v.is_extern = true;
                        v.is_c = is_c || true;
                    } else if is_c {
                        v.is_c = true;
                    }
                    return Ok(vec![ModuleItem::Stmt(Stmt::Var(v))]);
                }
                if Self::is_decl_keyword(tok) || is_extern {
                    has_decl = true;
                }
            }
            if has_decl {
                let meta_list = self.parse_meta_list()?;
                return Ok(vec![ModuleItem::Decl(self.parse_decl(meta_list, is_pub)?)]);
            }
        }
        match self.peek() {
            Some(Token::Import) => Ok(vec![ModuleItem::Import(self.parse_import_stmt()?)]),
            Some(Token::ThreadLocal) => {
                let mut v = self.parse_var_decl()?;
                if has_pub_or_priv {
                    v.is_pub = is_pub;
                }
                v.is_threadlocal = true;
                Ok(vec![ModuleItem::Stmt(Stmt::Var(v))])
            }
            Some(Token::Let) | Some(Token::Const) | Some(Token::Var) => {
                let mut v = self.parse_var_decl()?;
                if has_pub_or_priv {
                    v.is_pub = is_pub;
                }
                if v.kind == VarDeclKind::Const && v.ty.is_none() {
                    if let Pattern::Ident(ident) = &v.pat {
                        if let Some(Expr::Zig(z)) = &v.init {
                            let trimmed = z.code.trim();
                            if trimmed.starts_with("fn ") || trimmed.starts_with("fn(") {
                                let mut sub_parser = Parser::new(trimmed);
                                if let Ok(target_ty) = sub_parser.parse_fn_type_sig(false) {
                                    return Ok(vec![ModuleItem::Decl(Decl::TypeAlias(
                                        TypeAliasDecl {
                                            span: Span {
                                                start: v.span.start,
                                                end: v.span.end,
                                            },
                                            ident: ident.clone(),
                                            target_ty,
                                            is_pub: v.is_pub,
                                        },
                                    ))]);
                                }
                            }
                        }
                    }
                }
                Ok(vec![ModuleItem::Stmt(Stmt::Var(v))])
            }
            Some(Token::Meta)
            | Some(Token::Dec)
            | Some(Token::Macro)
            | Some(Token::Fn)
            | Some(Token::Class)
            | Some(Token::Struct)
            | Some(Token::Union)
            | Some(Token::Ref)
            | Some(Token::Interface)
            | Some(Token::ErrorKw)
            | Some(Token::Trait)
            | Some(Token::Impl)
            | Some(Token::Enum)
            | Some(Token::Async)
            | Some(Token::Static)
            | Some(Token::TypeKw) => Ok(vec![ModuleItem::Decl(self.parse_decl(vec![], is_pub)?)]),
            Some(Token::Ident(ref id)) if id == "extern" || id == "inline" || id == "noinline" => {
                if id == "extern"
                    && matches!(
                        self.peek_next(),
                        Some(Token::Const)
                            | Some(Token::Var)
                            | Some(Token::Let)
                            | Some(Token::ThreadLocal)
                    )
                {
                    self.bump()?; // consume 'extern'
                    let mut v = self.parse_var_decl()?;
                    v.is_extern = true;
                    v.is_c = true;
                    if has_pub_or_priv {
                        v.is_pub = is_pub;
                    }
                    return Ok(vec![ModuleItem::Stmt(Stmt::Var(v))]);
                }
                Ok(vec![ModuleItem::Decl(self.parse_decl(vec![], is_pub)?)])
            }
            Some(Token::Unsafe) => {
                if self.peek_next() == Some(&Token::LBrace) {
                    Ok(vec![ModuleItem::Stmt(self.parse_stmt()?)])
                } else {
                    Ok(vec![ModuleItem::Decl(self.parse_decl(vec![], is_pub)?)])
                }
            }
            _ => Ok(vec![ModuleItem::Stmt(self.parse_stmt()?)]),
        }
    }

    fn parse_import_stmt(&mut self) -> ParseResult<ImportStmt> {
        let start = self.expect(Token::Import)?.start;
        let mut specifiers = Vec::new();
        let mut default_specifier = None;
        if self.peek() == Some(&Token::LBrace) {
            self.bump()?;
            while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                specifiers.push(self.parse_ident()?);
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
            }
            self.expect(Token::RBrace)?;
        } else if self.peek() == Some(&Token::Star) {
            self.bump()?;
            self.expect(Token::As)?;
            default_specifier = Some(self.parse_ident()?);
        } else if matches!(self.peek(), Some(Token::Ident(_))) {
            default_specifier = Some(self.parse_ident()?);
        }
        self.expect(Token::From)?;
        let (token, span) = self.bump()?;
        let source = if let Token::StringLiteral(s) = token {
            s
        } else {
            return Err(ParseError::ExpectedToken(
                Token::StringLiteral("source".to_string()),
                token,
                span,
            ));
        };
        if self.peek() == Some(&Token::Semicolon) {
            self.bump()?;
        }
        let end = span.end;
        Ok(ImportStmt {
            span: Span { start, end },
            specifiers,
            default_specifier,
            source,
        })
    }

    fn parse_export_stmt(&mut self) -> ParseResult<ExportStmt> {
        let start = self.expect(Token::LBrace)?.start;
        let mut specifiers = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            specifiers.push(self.parse_ident()?);
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        let mut end = self.expect(Token::RBrace)?.end;
        let mut source = None;
        if self.peek() == Some(&Token::From) {
            self.bump()?;
            let (token, span) = self.bump()?;
            let s = if let Token::StringLiteral(str_val) = token {
                str_val
            } else {
                return Err(ParseError::ExpectedToken(
                    Token::StringLiteral("source".to_string()),
                    token,
                    span,
                ));
            };
            end = span.end;
            source = Some(s);
        }
        if self.peek() == Some(&Token::Semicolon) {
            end = self.bump()?.1.end;
        }
        Ok(ExportStmt {
            span: Span { start, end },
            specifiers,
            source,
        })
    }

    fn parse_export_star_stmt(&mut self) -> ParseResult<ExportStmt> {
        let start = self.expect(Token::Star)?.start;
        self.expect(Token::From)?;
        let (token, span) = self.bump()?;
        let source = if let Token::StringLiteral(s) = token {
            s
        } else {
            return Err(ParseError::ExpectedToken(
                Token::StringLiteral("source".to_string()),
                token,
                span,
            ));
        };
        let mut end = span.end;
        if self.peek() == Some(&Token::Semicolon) {
            end = self.bump()?.1.end;
        }
        Ok(ExportStmt {
            span: Span { start, end },
            specifiers: vec![],
            source: Some(source),
        })
    }

    fn parse_decl(&mut self, meta: Vec<MetaItem>, mut is_pub: bool) -> ParseResult<Decl> {
        while self.peek() == Some(&Token::Export) || self.peek() == Some(&Token::Private) {
            if self.peek() == Some(&Token::Export) {
                is_pub = true;
            } else if self.peek() == Some(&Token::Private) {
                is_pub = false;
            }
            self.bump()?;
        }
        match self.peek() {
            Some(Token::Meta) => Ok(Decl::Meta(self.parse_meta_def(meta)?)),
            Some(Token::Dec) => Ok(Decl::Dec(self.parse_dec_def(meta)?)),
            Some(Token::Macro) => Ok(Decl::Macro(self.parse_macro_def(meta)?)),
            Some(Token::Static) => {
                if matches!(
                    self.peek_next(),
                    Some(Token::Class) | Some(Token::Struct) | Some(Token::Ref)
                ) {
                    let mut c = self.parse_class()?;
                    c.meta.extend(meta);
                    if is_pub {
                        c.is_pub = true;
                    }
                    Ok(Decl::Class(c))
                } else {
                    let mut f = self.parse_function()?;
                    f.meta.extend(meta);
                    if is_pub {
                        f.is_pub = true;
                    }
                    Ok(Decl::Func(f))
                }
            }
            Some(Token::Fn) | Some(Token::Async) | Some(Token::Unsafe) => {
                let mut f = self.parse_function()?;
                f.meta.extend(meta);
                if is_pub {
                    f.is_pub = true;
                }
                Ok(Decl::Func(f))
            }
            Some(Token::Ident(ref id)) if id == "extern" || id == "inline" || id == "noinline" => {
                let mut f = self.parse_function()?;
                f.meta.extend(meta);
                if is_pub {
                    f.is_pub = true;
                }
                Ok(Decl::Func(f))
            }
            Some(Token::Class) | Some(Token::Struct) | Some(Token::Union) | Some(Token::Ref) => {
                let mut c = self.parse_class()?;
                c.meta.extend(meta);
                if is_pub {
                    c.is_pub = true;
                }
                Ok(Decl::Class(c))
            }
            Some(Token::Trait) => {
                let mut t = self.parse_trait()?;
                t.meta = meta;
                Ok(Decl::Trait(t))
            }
            Some(Token::Impl) => Ok(Decl::Impl(self.parse_impl()?)),
            Some(Token::Enum) => {
                let mut e = self.parse_enum()?;
                e.meta = meta;
                Ok(Decl::Enum(e))
            }
            Some(Token::ErrorKw) => Ok(Decl::ErrorSet(self.parse_error_set()?)),
            Some(Token::TypeKw) => {
                let start = self.bump()?.1.start;
                let ident = self.parse_ident()?;
                self.expect(Token::Assign)?;
                let target_ty = self.parse_type()?;
                if self.peek() == Some(&Token::Semicolon) {
                    self.bump()?;
                }
                let end = self.peek_span().start;
                Ok(Decl::TypeAlias(TypeAliasDecl {
                    span: Span { start, end },
                    ident,
                    target_ty,
                    is_pub,
                }))
            }
            _ => Err(ParseError::UnexpectedToken(
                self.bump()?.0,
                self.peek_span(),
            )),
        }
    }

    fn parse_function(&mut self) -> ParseResult<Function> {
        let start = self.peek_span().start;
        let mut is_static = false;
        let mut is_async = false;
        let mut is_unsafe = false;
        let mut is_pub = false;
        let mut meta = Vec::new();
        while matches!(
            self.peek(),
            Some(Token::Static)
                | Some(Token::Async)
                | Some(Token::Unsafe)
                | Some(Token::Export)
                | Some(Token::Private)
        ) || matches!(self.peek(), Some(Token::Ident(id)) if id == "inline" || id == "extern" || id == "noinline")
        {
            if matches!(self.peek(), Some(Token::Ident(id)) if id == "inline" || id == "extern" || id == "noinline")
            {
                let (tok, span) = self.bump()?;
                let sym = match tok {
                    Token::Ident(s) => s,
                    _ => "inline".to_string(),
                };
                meta.push(MetaItem {
                    span,
                    name: Ident { span, sym },
                    args: vec![],
                });
            } else if self.peek() == Some(&Token::Export) {
                let span = self.bump()?.1;
                is_pub = true;
                meta.push(MetaItem {
                    span,
                    name: Ident {
                        span,
                        sym: "export".to_string(),
                    },
                    args: vec![],
                });
            } else if self.peek() == Some(&Token::Private) {
                self.bump()?;
                is_pub = false;
            } else if self.peek() == Some(&Token::Static) {
                self.bump()?;
                is_static = true;
            } else if self.peek() == Some(&Token::Async) {
                self.bump()?;
                is_async = true;
            } else if self.peek() == Some(&Token::Unsafe) {
                self.bump()?;
                is_unsafe = true;
            }
        }
        self.expect(Token::Fn)?;
        let ident = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;
        let params = self.parse_params()?;
        let mut return_type = None;
        if self.peek() == Some(&Token::Bang) {
            self.bump()?;
            if self.is_error_type_rhs_start() {
                let err = self.parse_error_type_rhs()?;
                let ok_ty = if self.peek().map(Self::is_type_start).unwrap_or(false)
                    && self.peek() != Some(&Token::LBrace)
                    && self.peek() != Some(&Token::Semicolon)
                {
                    self.parse_type()?
                } else {
                    Type::Void
                };
                return_type = Some(Type::ResultType {
                    ok: Box::new(ok_ty),
                    err,
                });
            } else {
                let ok_ty = if self.peek().map(Self::is_type_start).unwrap_or(false)
                    && self.peek() != Some(&Token::LBrace)
                    && self.peek() != Some(&Token::Semicolon)
                {
                    self.parse_type()?
                } else {
                    Type::Void
                };
                return_type = Some(Type::ErrorUnion(Box::new(ok_ty)));
            }
        } else if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
            self.bump()?;
            return_type = Some(self.parse_type()?);
        } else if self.peek().map(Self::is_type_start).unwrap_or(false)
            && self.peek() != Some(&Token::LBrace)
            && self.peek() != Some(&Token::Semicolon)
        {
            return_type = Some(self.parse_type()?);
        }

        let body;
        let end;
        if self.peek() == Some(&Token::Semicolon) {
            let span = self.bump()?.1;
            body = BlockStmt { span, body: vec![] };
            end = span.end;
        } else {
            body = self.parse_block_stmt()?;
            end = body.span.end;
        }
        Ok(Function {
            span: Span { start, end },
            ident,
            generic_params,
            params,
            return_type,
            body,
            is_async,
            is_static,
            is_unsafe,
            meta,
            is_pub,
        })
    }

    pub fn parse_generic_params(&mut self) -> ParseResult<Vec<GenericParam>> {
        let mut generic_params = Vec::new();
        if self.peek() == Some(&Token::Less) {
            self.bump()?;
            while self.peek() != Some(&Token::Greater) && self.peek().is_some() {
                if self.peek() == Some(&Token::Const) {
                    self.bump()?;
                    let p_ident = self.parse_ident()?;
                    self.expect(Token::Colon)?;
                    let ty = self.parse_type()?;
                    generic_params.push(GenericParam::Const { ident: p_ident, ty });
                } else {
                    let p_ident = self.parse_ident()?;
                    let mut bounds = Vec::new();
                    if self.peek() == Some(&Token::Colon) {
                        self.bump()?;
                        loop {
                            let bound_ident = self.parse_ident()?;
                            bounds.push(bound_ident);
                            if self.peek() == Some(&Token::Plus) {
                                self.bump()?;
                            } else {
                                break;
                            }
                        }
                    }
                    generic_params.push(GenericParam::Type {
                        ident: p_ident,
                        bounds,
                    });
                }
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
            }
            self.expect(Token::Greater)?;
        }
        Ok(generic_params)
    }

    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while self.peek() != Some(&Token::RParen) {
            let meta = self.parse_meta_list()?;
            let mut is_mut = false;
            let mut is_move = false;
            let mut is_comptime = meta.iter().any(|m| m.name.sym == "comptime");
            if matches!(self.peek(), Some(Token::Ident(id)) if id == "comptime") {
                self.bump()?;
                is_comptime = true;
            }

            if self.peek() == Some(&Token::Mut) {
                self.bump()?;
                is_mut = true;
            } else if self.peek() == Some(&Token::Move) || self.peek() == Some(&Token::Own) {
                self.bump()?;
                is_move = true;
            } else if self.peek() == Some(&Token::Star) {
                self.bump()?;
                is_mut = true;
                let p_ident = self.parse_ident()?;
                let mut ty = if (p_ident.sym == "this"
                    || p_ident.sym == "_"
                    || p_ident.sym == "next"
                    || p_ident.sym == "target")
                    && self.peek() != Some(&Token::Colon)
                {
                    Type::Any
                } else if self.peek() == Some(&Token::Colon) {
                    self.bump()?;
                    self.parse_type()?
                } else {
                    Type::Any
                };
                if p_ident.sym != "this"
                    && p_ident.sym != "_"
                    && !matches!(
                        ty,
                        Type::BorrowType { .. } | Type::RawPtr { .. } | Type::Any
                    )
                {
                    ty = Type::BorrowType {
                        inner: Box::new(ty),
                        is_mut: true,
                        lifetimes: vec![],
                    };
                }
                let default = if self.peek() == Some(&Token::Assign) {
                    self.bump()?;
                    Some(self.parse_primary_expr()?)
                } else {
                    None
                };
                params.push(Param {
                    span: p_ident.span,
                    ident: p_ident,
                    ty,
                    is_comptime,
                    is_mut,
                    is_move,
                    default,
                    meta,
                });
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
                continue;
            } else if self.peek() == Some(&Token::Ampersand) {
                self.bump()?;
                if self.peek() == Some(&Token::Mut) {
                    self.bump()?;
                    is_mut = true;
                }
                let p_ident = self.parse_ident()?;
                let mut ty = if (p_ident.sym == "this"
                    || p_ident.sym == "_"
                    || p_ident.sym == "next"
                    || p_ident.sym == "target")
                    && self.peek() != Some(&Token::Colon)
                {
                    Type::Any
                } else if self.peek() == Some(&Token::Colon) {
                    self.bump()?;
                    self.parse_type()?
                } else {
                    Type::Any
                };
                if p_ident.sym != "this"
                    && p_ident.sym != "_"
                    && !matches!(
                        ty,
                        Type::BorrowType { .. } | Type::RawPtr { .. } | Type::Any
                    )
                {
                    ty = Type::BorrowType {
                        inner: Box::new(ty),
                        is_mut,
                        lifetimes: vec![],
                    };
                }
                let default = if self.peek() == Some(&Token::Assign) {
                    self.bump()?;
                    Some(self.parse_primary_expr()?)
                } else {
                    None
                };
                params.push(Param {
                    span: p_ident.span,
                    ident: p_ident,
                    ty,
                    is_comptime,
                    is_mut,
                    is_move,
                    default,
                    meta,
                });
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
                continue;
            }

            let p_ident = self.parse_ident()?;
            let ty = if (p_ident.sym == "this"
                || p_ident.sym == "_"
                || p_ident.sym == "next"
                || p_ident.sym == "target")
                && self.peek() != Some(&Token::Colon)
            {
                Type::Any
            } else if self.peek() == Some(&Token::Colon) {
                self.bump()?;
                if self.peek() == Some(&Token::Move) || self.peek() == Some(&Token::Own) {
                    self.bump()?;
                    is_move = true;
                }
                self.parse_type()?
            } else {
                Type::Any
            };
            if matches!(ty, Type::BorrowType { is_mut: true, .. }) {
                is_mut = true;
            }
            let default = if self.peek() == Some(&Token::Assign) {
                self.bump()?;
                Some(self.parse_primary_expr()?)
            } else {
                None
            };
            params.push(Param {
                span: p_ident.span,
                ident: p_ident,
                ty,
                is_comptime,
                is_mut,
                is_move,
                meta,
                default,
            });
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    #[allow(dead_code)]
    fn is_type_start(t: &Token) -> bool {
        matches!(
            t,
            Token::U8
                | Token::I8
                | Token::U16
                | Token::I16
                | Token::I32
                | Token::U32
                | Token::I64
                | Token::U64
                | Token::I128
                | Token::U128
                | Token::F32
                | Token::F64
                | Token::ISize
                | Token::USize
                | Token::String
                | Token::Void
                | Token::Any
                | Token::Bool
                | Token::CStr
                | Token::Ident(_)
                | Token::LBracket
                | Token::LBrace
                | Token::Ampersand
                | Token::Star
                | Token::Question
                | Token::Fn
                | Token::At
                | Token::ErrorKw
                | Token::Bang
        )
    }

    fn is_error_type_rhs_start(&mut self) -> bool {
        if matches!(self.peek(), Some(Token::Ident(_)) | Some(Token::ErrorKw)) {
            return true;
        }
        if self.peek() == Some(&Token::LBrace) {
            let checkpoint = self.lexer.clone();
            let p1 = self.peeked.clone();
            let p2 = self.peeked2.clone();
            let _ = self.bump();
            let mut is_anon = false;
            if matches!(self.peek(), Some(Token::Ident(_))) {
                let _ = self.bump();
                if matches!(self.peek(), Some(Token::Comma) | Some(Token::RBrace)) {
                    is_anon = true;
                }
            }
            self.lexer = checkpoint.clone();
            self.peeked = p1.clone();
            self.peeked2 = p2.clone();
            return is_anon;
        }
        false
    }

    fn parse_error_type_rhs(&mut self) -> ParseResult<ErrorType> {
        if self.peek() == Some(&Token::LBrace) {
            self.bump()?;
            let mut variants = Vec::new();
            while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                variants.push(self.parse_ident()?);
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
            }
            self.expect(Token::RBrace)?;
            return Ok(ErrorType::AnonSet(variants));
        }
        if matches!(self.peek(), Some(Token::Ident(_)) | Some(Token::ErrorKw)) {
            let mut first = self.parse_ident()?;
            if first.sym == "Box" && self.peek() == Some(&Token::Less) {
                self.bump()?;
                if self.peek() == Some(&Token::Dyn) {
                    self.bump()?;
                }
                let tr = self.parse_ident()?;
                self.expect(Token::Greater)?;
                return Ok(ErrorType::BoxDyn(tr));
            }
            while self.peek() == Some(&Token::Dot) {
                self.bump()?;
                let next = self.parse_ident()?;
                first.sym.push('.');
                first.sym.push_str(&next.sym);
            }
            if self.peek() == Some(&Token::Pipe) || self.peek() == Some(&Token::PipePipe) {
                let mut list = vec![first];
                while self.peek() == Some(&Token::Pipe) || self.peek() == Some(&Token::PipePipe) {
                    self.bump()?;
                    if self.peek() == Some(&Token::LBrace) {
                        self.bump()?;
                        let mut variants = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            variants.push(self.parse_ident()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RBrace)?;
                        let anon_name = format!(
                            "error{{{}}}",
                            variants
                                .iter()
                                .map(|v| v.sym.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        list.push(Ident {
                            sym: anon_name,
                            span: Span { start: 0, end: 0 },
                        });
                        continue;
                    }
                    let mut next = self.parse_ident()?;
                    if (next.sym == "err" || next.sym == "error")
                        && self.peek() == Some(&Token::LBrace)
                    {
                        self.bump()?;
                        let mut variants = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            variants.push(self.parse_ident()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RBrace)?;
                        let anon_name = format!(
                            "error{{{}}}",
                            variants
                                .iter()
                                .map(|v| v.sym.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        list.push(Ident {
                            sym: anon_name,
                            span: next.span,
                        });
                        continue;
                    }
                    while self.peek() == Some(&Token::Dot) {
                        self.bump()?;
                        let sub = self.parse_ident()?;
                        next.sym.push('.');
                        next.sym.push_str(&sub.sym);
                    }
                    list.push(next);
                }
                return Ok(ErrorType::Union(list));
            }
            if first.sym == "Error"
                || first.sym == "error"
                || first.sym == "err"
                || first.sym == "Err"
            {
                return Ok(ErrorType::DefaultError);
            }
            if first.sym == "AnyErr" || first.sym == "anyErr" || first.sym == "anyerror" {
                return Ok(ErrorType::AnyErr);
            }
            return Ok(ErrorType::Named(first));
        }
        Ok(ErrorType::DefaultError)
    }

    fn parse_postfix_type_modifiers(&mut self, mut ty: Type) -> ParseResult<Type> {
        loop {
            if self.peek() == Some(&Token::Question) {
                self.bump()?;
                ty = Type::Optional(Box::new(ty));
            } else if self.peek() == Some(&Token::Bang) {
                if self.peek_next() == Some(&Token::Question) {
                    if matches!(ty, Type::CStr) {
                        self.bump()?;
                        self.bump()?;
                        ty = Type::Optional(Box::new(ty));
                        continue;
                    }
                }
                break;
            } else if self.peek() == Some(&Token::LBracket) {
                self.bump()?;
                if self.peek() == Some(&Token::RBracket) {
                    self.bump()?;
                    ty = Type::Array(Box::new(ty));
                } else if let Some(Token::IntLiteral(n)) = self.peek() {
                    let size = ArraySize::Literal(*n as usize);
                    self.bump()?;
                    self.expect(Token::RBracket)?;
                    ty = Type::FixedArray(Box::new(ty), size);
                } else if let Some(Token::Ident(_)) = self.peek() {
                    let id = self.parse_ident()?;
                    self.expect(Token::RBracket)?;
                    ty = Type::FixedArray(Box::new(ty), ArraySize::Ident(id));
                } else {
                    self.expect(Token::RBracket)?;
                    ty = Type::Array(Box::new(ty));
                }
            } else {
                break;
            }
        }
        Ok(ty)
    }

    fn parse_fn_type_sig(&mut self, is_c: bool) -> ParseResult<Type> {
        self.expect(Token::Fn)?;
        self.expect(Token::LParen)?;
        let mut param_types = Vec::new();
        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
            let mut is_comptime = false;
            if matches!(self.peek(), Some(Token::Ident(id)) if id == "comptime") {
                self.bump()?;
                is_comptime = true;
            }
            if matches!(self.peek(), Some(Token::Ident(_)) | Some(Token::ErrorKw))
                && self.peek_next() == Some(&Token::Colon)
            {
                self.bump()?; // name
                self.bump()?; // colon
            }
            let mut ty = self.parse_type()?;
            if is_comptime {
                ty = Type::Ref(Ident {
                    span: self.peek_span(),
                    sym: format!("comptime {}", Self::type_to_str(&ty)),
                });
            }
            param_types.push(ty);
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        self.expect(Token::RParen)?;
        let ret_ty = if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
            self.bump()?;
            self.parse_type()?
        } else if self.peek().map(Self::is_type_start).unwrap_or(false) {
            self.parse_type()?
        } else {
            Type::Void
        };
        if is_c {
            Ok(Type::CFn(param_types, Box::new(ret_ty)))
        } else {
            Ok(Type::Fn(param_types, Box::new(ret_ty)))
        }
    }

    fn parse_base_type(&mut self) -> ParseResult<Type> {
        if self.peek() == Some(&Token::Switch) {
            let start = self.peek_span().start;
            let mut brace_depth: i32 = 0;
            let mut paren_depth: i32 = 0;
            let mut had_brace = false;
            let mut end = start;
            while let Some(tok) = self.peek() {
                match tok {
                    Token::LBrace => {
                        brace_depth += 1;
                        had_brace = true;
                    }
                    Token::RBrace => {
                        brace_depth -= 1;
                        if brace_depth == 0 && paren_depth == 0 && had_brace {
                            let span = self.bump()?.1;
                            end = span.end;
                            break;
                        }
                    }
                    Token::LParen => paren_depth += 1,
                    Token::RParen => {
                        if paren_depth > 0 {
                            paren_depth -= 1;
                        }
                    }
                    _ => {}
                }
                let span = self.bump()?.1;
                end = span.end;
            }
            let raw_str = self.input[start..end].trim().to_string();
            let ty = Type::Ref(Ident {
                span: Span { start, end },
                sym: raw_str,
            });
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::Question) {
            self.bump()?;
            let inner = self.parse_base_type()?;
            let ty = Type::Optional(Box::new(inner));
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::Const) {
            self.bump()?;
            let inner = self.parse_base_type()?;
            let ty = Type::BorrowType {
                inner: Box::new(inner),
                is_mut: false,
                lifetimes: vec![],
            };
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::Dyn) {
            self.bump()?;
            let tr = self.parse_ident()?;
            return Ok(Type::BareDyn(tr));
        }
        if self.peek() == Some(&Token::Fn) {
            let ty = self.parse_fn_type_sig(false)?;
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::At) {
            let mut clone = self.clone_lookahead();
            let _ = clone.bump();
            if matches!(clone.peek(), Some(Token::Ident(s)) if s == "c") {
                let _ = clone.bump();
                if clone.peek() == Some(&Token::Fn) {
                    self.bump()?; // '@'
                    self.bump()?; // 'c'
                    let ty = self.parse_fn_type_sig(true)?;
                    return self.parse_postfix_type_modifiers(ty);
                }
            }
            self.bump()?; // consume '@'
            let id = self.parse_ident()?;
            let mut sym = format!("@{}", id.sym);
            if self.peek() == Some(&Token::LParen) {
                self.bump()?;
                sym.push('(');
                let mut paren_depth = 1;
                while paren_depth > 0 && self.peek().is_some() {
                    let (tok, t_span) = self.bump()?;
                    match tok {
                        Token::LParen => paren_depth += 1,
                        Token::RParen => paren_depth -= 1,
                        _ => {}
                    }
                    if paren_depth == 0 {
                        sym.push(')');
                        break;
                    }
                    sym.push_str(&self.input[t_span.start..t_span.end]);
                }
            }
            while self.peek() == Some(&Token::Dot) {
                self.bump()?;
                let next = self.parse_ident()?;
                sym.push('.');
                sym.push_str(&next.sym);
            }
            let ty = Type::Ref(Ident { span: id.span, sym });
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::LParen) {
            let mut clone = self.clone_lookahead();
            let _ = clone.bump();
            let is_fn_ptr = match clone.peek() {
                Some(Token::Fn) => true,
                Some(Token::At) => {
                    let _ = clone.bump();
                    matches!(clone.peek(), Some(Token::Ident(s)) if s == "c")
                }
                _ => false,
            };
            if is_fn_ptr {
                self.bump()?; // consume '('
                let is_c = if self.peek() == Some(&Token::At) {
                    self.bump()?; // '@'
                    self.bump()?; // 'c'
                    true
                } else {
                    false
                };
                let ty = self.parse_fn_type_sig(is_c)?;
                self.expect(Token::RParen)?;
                return self.parse_postfix_type_modifiers(ty);
            } else {
                self.bump()?; // consume '('
                let inner = self.parse_type()?;
                self.expect(Token::RParen)?;
                return self.parse_postfix_type_modifiers(inner);
            }
        }
        if self.peek() == Some(&Token::Star) {
            let mut depth = 0;
            while self.peek() == Some(&Token::Star) {
                self.bump()?;
                depth += 1;
            }
            if self.peek() == Some(&Token::Bang) {
                self.bump()?; // consume '!'
                let ty = Type::RawPtr {
                    inner: Box::new(Type::Any),
                    depth,
                };
                return self.parse_postfix_type_modifiers(ty);
            }
            let mut is_mut = true;
            if self.peek() == Some(&Token::Const) {
                self.bump()?;
                is_mut = false;
            }
            let mut lifetimes = Vec::new();
            if depth == 1 {
                if self.peek() == Some(&Token::LParen) {
                    let checkpoint = self.lexer.clone();
                    let peeked_checkpoint = self.peeked.clone();
                    let peeked2_checkpoint = self.peeked2.clone();
                    self.bump()?;
                    let mut is_lifetime = true;
                    let mut candidate_lifetimes = Vec::new();
                    while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                        if let Some(Token::Ident(_)) = self.peek() {
                            if let Ok(id) = self.parse_ident() {
                                candidate_lifetimes.push(id.sym);
                                if self.peek() == Some(&Token::Pipe) {
                                    self.bump()?;
                                }
                                continue;
                            }
                        }
                        is_lifetime = false;
                        break;
                    }
                    if is_lifetime
                        && self.peek() == Some(&Token::RParen)
                        && !candidate_lifetimes.is_empty()
                    {
                        self.bump()?;
                        if matches!(
                            self.peek(),
                            Some(Token::Ident(_))
                                | Some(Token::Star)
                                | Some(Token::Ampersand)
                                | Some(Token::LBracket)
                                | Some(Token::Const)
                                | Some(Token::String)
                                | Some(Token::I32)
                                | Some(Token::U32)
                                | Some(Token::I64)
                                | Some(Token::U64)
                                | Some(Token::I128)
                                | Some(Token::U128)
                                | Some(Token::F32)
                                | Some(Token::F64)
                                | Some(Token::USize)
                                | Some(Token::Bool)
                        ) {
                            lifetimes = candidate_lifetimes;
                        } else {
                            self.lexer = checkpoint;
                            self.peeked = peeked_checkpoint;
                            self.peeked2 = peeked2_checkpoint;
                        }
                    } else {
                        self.lexer = checkpoint;
                        self.peeked = peeked_checkpoint;
                        self.peeked2 = peeked2_checkpoint;
                    }
                } else if let Some(Token::Ident(_)) = self.peek() {
                    let checkpoint = self.lexer.clone();
                    let peeked_checkpoint = self.peeked.clone();
                    let peeked2_checkpoint = self.peeked2.clone();
                    let first_ident = self.parse_ident()?;
                    if matches!(
                        self.peek(),
                        Some(Token::Ident(_))
                            | Some(Token::String)
                            | Some(Token::I32)
                            | Some(Token::U32)
                            | Some(Token::I64)
                            | Some(Token::U64)
                            | Some(Token::I128)
                            | Some(Token::U128)
                            | Some(Token::F32)
                            | Some(Token::F64)
                            | Some(Token::USize)
                            | Some(Token::Bool)
                    ) {
                        lifetimes.push(first_ident.sym);
                    } else {
                        self.lexer = checkpoint;
                        self.peeked = peeked_checkpoint;
                        self.peeked2 = peeked2_checkpoint;
                    }
                }
            }
            let inner = self.parse_base_type()?;
            if self.peek() == Some(&Token::Bang) {
                let is_raw_ptr_bang = match self.peek_next() {
                    Some(Token::LBrace)
                    | Some(Token::RBrace)
                    | Some(Token::Comma)
                    | Some(Token::RParen)
                    | Some(Token::RBracket)
                    | Some(Token::Semicolon)
                    | Some(Token::Assign)
                    | Some(Token::Bang)
                    | Some(Token::Pipe)
                    | Some(Token::Pipeline)
                    | Some(Token::Greater)
                    | Some(Token::Question)
                    | None => true,
                    _ => false,
                };
                if is_raw_ptr_bang {
                    self.bump()?; // consume '!'
                    let ty = Type::RawPtr {
                        inner: Box::new(inner),
                        depth,
                    };
                    return self.parse_postfix_type_modifiers(ty);
                }
            }
            let mut ty = inner;
            for idx in 0..depth {
                ty = Type::BorrowType {
                    inner: Box::new(ty),
                    is_mut,
                    lifetimes: if idx == 0 { lifetimes.clone() } else { vec![] },
                };
            }
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::Ampersand) {
            self.bump()?;
            let mut is_mut = false;
            let mut lifetimes = Vec::new();
            if self.peek() == Some(&Token::Mut) {
                self.bump()?;
                is_mut = true;
            }
            if self.peek() == Some(&Token::LParen) {
                self.bump()?;
                while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                    let id = self.parse_ident()?;
                    lifetimes.push(id.sym);
                    if self.peek() == Some(&Token::Pipe) {
                        self.bump()?;
                    }
                }
                self.expect(Token::RParen)?;
                if self.peek() == Some(&Token::Mut) {
                    self.bump()?;
                    is_mut = true;
                }
            } else if let Some(Token::Ident(_)) = self.peek() {
                let checkpoint = self.lexer.clone();
                let peeked_checkpoint = self.peeked.clone();
                let peeked2_checkpoint = self.peeked2.clone();
                let first_ident = self.parse_ident()?;
                if self.peek() == Some(&Token::Mut) {
                    self.bump()?;
                    is_mut = true;
                    lifetimes.push(first_ident.sym);
                } else if matches!(
                    self.peek(),
                    Some(Token::Ident(_))
                        | Some(Token::String)
                        | Some(Token::I32)
                        | Some(Token::U32)
                        | Some(Token::I64)
                        | Some(Token::U64)
                        | Some(Token::I128)
                        | Some(Token::U128)
                        | Some(Token::F32)
                        | Some(Token::F64)
                        | Some(Token::USize)
                        | Some(Token::Bool)
                ) {
                    lifetimes.push(first_ident.sym);
                } else {
                    self.lexer = checkpoint;
                    self.peeked = peeked_checkpoint;
                    self.peeked2 = peeked2_checkpoint;
                }
            }
            let inner = self.parse_base_type()?;
            let ty = Type::BorrowType {
                inner: Box::new(inner),
                is_mut,
                lifetimes,
            };
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::LBracket) {
            let checkpoint = self.lexer.clone();
            let p1 = self.peeked.clone();
            let p2 = self.peeked2.clone();
            self.bump()?;
            if self.peek() == Some(&Token::Star) {
                self.bump()?;
                if self.peek() == Some(&Token::RBracket) {
                    self.bump()?;
                    let elem_ty = if self.peek() == Some(&Token::Bang) {
                        Type::Any
                    } else {
                        self.parse_base_type()?
                    };
                    let mut is_optional = false;
                    if self.peek() == Some(&Token::Question)
                        && self.peek_next() == Some(&Token::Bang)
                    {
                        self.bump()?;
                        self.bump()?;
                        is_optional = true;
                    } else if self.peek() == Some(&Token::Bang) {
                        self.bump()?;
                        if self.peek() == Some(&Token::Question) {
                            self.bump()?;
                            is_optional = true;
                        }
                    }
                    let (elem_ty, has_elem_opt) = match elem_ty {
                        Type::Optional(inner) => (*inner, true),
                        other => (other, false),
                    };
                    let mut ty = Type::ManyRawPtr(Box::new(elem_ty));
                    if is_optional || has_elem_opt {
                        ty = Type::Optional(Box::new(ty));
                    }
                    return self.parse_postfix_type_modifiers(ty);
                } else if matches!(self.peek(), Some(Token::Ident(s)) if s == "c") {
                    self.bump()?;
                    if self.peek() == Some(&Token::Colon) {
                        self.bump()?;
                        let sentinel = match self.bump()?.0 {
                            Token::IntLiteral(n) => n.to_string(),
                            Token::Ident(s) => s,
                            Token::Null => "null".to_string(),
                            _ => "0".to_string(),
                        };
                        self.expect(Token::RBracket)?;
                        let elem_ty = self.parse_base_type()?;
                        let mut is_optional = false;
                        if self.peek() == Some(&Token::Question)
                            && self.peek_next() == Some(&Token::Bang)
                        {
                            self.bump()?;
                            self.bump()?;
                            is_optional = true;
                        } else if self.peek() == Some(&Token::Bang) {
                            self.bump()?;
                            if self.peek() == Some(&Token::Question) {
                                self.bump()?;
                                is_optional = true;
                            }
                        }
                        let (elem_ty, has_elem_opt) = match elem_ty {
                            Type::Optional(inner) => (*inner, true),
                            other => (other, false),
                        };
                        let mut ty = Type::SentinelPtr(Box::new(elem_ty), sentinel);
                        if is_optional || has_elem_opt {
                            ty = Type::Optional(Box::new(ty));
                        }
                        return self.parse_postfix_type_modifiers(ty);
                    } else if self.peek() == Some(&Token::RBracket) {
                        self.bump()?;
                        let elem_ty = self.parse_base_type()?;
                        let mut is_optional = false;
                        if self.peek() == Some(&Token::Question)
                            && self.peek_next() == Some(&Token::Bang)
                        {
                            self.bump()?;
                            self.bump()?;
                            is_optional = true;
                        } else if self.peek() == Some(&Token::Bang) {
                            self.bump()?;
                            if self.peek() == Some(&Token::Question) {
                                self.bump()?;
                                is_optional = true;
                            }
                        }
                        let (elem_ty, has_elem_opt) = match elem_ty {
                            Type::Optional(inner) => (*inner, true),
                            other => (other, false),
                        };
                        let mut ty = Type::CPtr(Box::new(elem_ty));
                        if is_optional || has_elem_opt {
                            ty = Type::Optional(Box::new(ty));
                        }
                        return self.parse_postfix_type_modifiers(ty);
                    }
                } else if self.peek() == Some(&Token::Colon) {
                    self.bump()?;
                    let sentinel = match self.bump()?.0 {
                        Token::IntLiteral(n) => n.to_string(),
                        Token::Ident(s) => s,
                        Token::Null => "null".to_string(),
                        _ => "0".to_string(),
                    };
                    self.expect(Token::RBracket)?;
                    if self.peek() == Some(&Token::Const) {
                        self.bump()?;
                    }
                    let elem_ty = self.parse_base_type()?;
                    let mut is_optional = false;
                    if self.peek() == Some(&Token::Question)
                        && self.peek_next() == Some(&Token::Bang)
                    {
                        self.bump()?;
                        self.bump()?;
                        is_optional = true;
                    } else if self.peek() == Some(&Token::Bang) {
                        self.bump()?;
                        if self.peek() == Some(&Token::Question) {
                            self.bump()?;
                            is_optional = true;
                        }
                    }
                    let (elem_ty, has_elem_opt) = match elem_ty {
                        Type::Optional(inner) => (*inner, true),
                        other => (other, false),
                    };
                    let mut ty = Type::SentinelPtr(Box::new(elem_ty), sentinel);
                    if is_optional || has_elem_opt {
                        ty = Type::Optional(Box::new(ty));
                    }
                    return self.parse_postfix_type_modifiers(ty);
                }
            }
            self.lexer = checkpoint.clone();
            self.peeked = p1.clone();
            self.peeked2 = p2.clone();
            let (_, span) = self.bump()?;
            if let Some(Token::IntLiteral(_)) | Some(Token::Ident(_)) = self.peek() {
                let size_tok = self.bump()?.0;
                let size_str = match size_tok {
                    Token::IntLiteral(n) => n.to_string(),
                    Token::Ident(s) => s,
                    _ => unreachable!(),
                };
                if self.peek() == Some(&Token::RBracket) {
                    self.bump()?;
                    let has_const = if self.peek() == Some(&Token::Const) {
                        self.bump()?;
                        true
                    } else {
                        false
                    };
                    let elem_ty = if self.peek() == Some(&Token::Bang) {
                        Type::Any
                    } else {
                        self.parse_base_type()?
                    };
                    let base_str = match &elem_ty {
                        Type::U8 => "u8".to_string(),
                        Type::I8 => "i8".to_string(),
                        Type::U16 => "u16".to_string(),
                        Type::I16 => "i16".to_string(),
                        Type::U32 => "u32".to_string(),
                        Type::I32 => "i32".to_string(),
                        Type::U64 => "u64".to_string(),
                        Type::I64 => "i64".to_string(),
                        Type::Bool => "bool".to_string(),
                        Type::Ref(id) => id.sym.clone(),
                        _ => Self::type_to_str(&elem_ty),
                    };
                    let elem_str = if has_const {
                        format!("const {}", base_str)
                    } else {
                        base_str
                    };
                    let ty = Type::Ref(Ident {
                        span,
                        sym: format!("[{}]{}", size_str, elem_str),
                    });
                    return self.parse_postfix_type_modifiers(ty);
                }
            }
            self.lexer = checkpoint.clone();
            self.peeked = p1.clone();
            self.peeked2 = p2.clone();
            self.bump()?;
            if self.peek() == Some(&Token::Colon) {
                self.bump()?;
                let sentinel = match self.bump()?.0 {
                    Token::IntLiteral(n) => n.to_string(),
                    Token::Ident(s) => s,
                    Token::Null => "null".to_string(),
                    _ => "0".to_string(),
                };
                self.expect(Token::RBracket)?;
                let has_const = if self.peek() == Some(&Token::Const) {
                    self.bump()?;
                    true
                } else {
                    false
                };
                let elem_ty = self.parse_base_type()?;
                let mut is_optional = false;
                if self.peek() == Some(&Token::Question) && self.peek_next() == Some(&Token::Bang) {
                    self.bump()?;
                    self.bump()?;
                    is_optional = true;
                } else if self.peek() == Some(&Token::Bang) {
                    self.bump()?;
                    if self.peek() == Some(&Token::Question) {
                        self.bump()?;
                        is_optional = true;
                    }
                }
                let (elem_ty, has_elem_opt) = match elem_ty {
                    Type::Optional(inner) => (*inner, true),
                    other => (other, false),
                };
                let s_val = if has_const {
                    format!("{}:const", sentinel)
                } else {
                    sentinel
                };
                let mut ty = Type::SentinelSlice(Box::new(elem_ty), s_val);
                if is_optional || has_elem_opt {
                    ty = Type::Optional(Box::new(ty));
                }
                return self.parse_postfix_type_modifiers(ty);
            }
            if self.peek() == Some(&Token::RBracket) {
                self.bump()?;
                let has_const = if self.peek() == Some(&Token::Const) {
                    self.bump()?;
                    true
                } else {
                    false
                };
                if has_const
                    || matches!(
                        self.peek(),
                        Some(
                            Token::U8
                                | Token::I8
                                | Token::U16
                                | Token::I16
                                | Token::I32
                                | Token::U32
                                | Token::I64
                                | Token::U64
                                | Token::I128
                                | Token::U128
                                | Token::ISize
                                | Token::USize
                                | Token::F32
                                | Token::F64
                                | Token::String
                                | Token::Void
                                | Token::Any
                                | Token::Bool
                                | Token::CStr
                                | Token::Ident(_)
                                | Token::LBracket
                                | Token::LBrace
                                | Token::Star
                                | Token::Ampersand
                                | Token::TypeKw
                        )
                    )
                {
                    let elem_ty = self.parse_base_type()?;
                    let ty = if has_const {
                        Type::BorrowType {
                            inner: Box::new(Type::Array(Box::new(elem_ty))),
                            is_mut: false,
                            lifetimes: vec![],
                        }
                    } else {
                        Type::Array(Box::new(elem_ty))
                    };
                    return self.parse_postfix_type_modifiers(ty);
                }
                let ty = Type::Tuple(Vec::new());
                return self.parse_postfix_type_modifiers(ty);
            }
            if let Some(Token::IntLiteral(n)) = self.peek() {
                let sz = ArraySize::Literal(*n as usize);
                self.bump()?;
                if self.peek() == Some(&Token::RBracket) {
                    self.bump()?;
                    let elem_ty = self.parse_base_type()?;
                    let ty = Type::FixedArray(Box::new(elem_ty), sz);
                    return self.parse_postfix_type_modifiers(ty);
                }
            }
            if let Some(Token::Ident(_)) = self.peek() {
                let mut probe = self.clone_lookahead();
                if probe.bump().is_ok() && probe.peek() == Some(&Token::RBracket) {
                    let _ = probe.bump();
                    if matches!(
                        probe.peek(),
                        Some(
                            Token::U8
                                | Token::U16
                                | Token::U32
                                | Token::U64
                                | Token::I8
                                | Token::I16
                                | Token::I32
                                | Token::I64
                                | Token::USize
                                | Token::ISize
                                | Token::F32
                                | Token::F64
                                | Token::Bool
                                | Token::String
                                | Token::CStr
                                | Token::Star
                                | Token::Ampersand
                                | Token::LBracket
                        )
                    ) || matches!(probe.peek(), Some(Token::Ident(tid)) if tid.chars().next().map_or(false, |c| c.is_uppercase()))
                    {
                        let id = self.parse_ident()?;
                        self.expect(Token::RBracket)?;
                        let elem_ty = self.parse_base_type()?;
                        let ty = Type::FixedArray(Box::new(elem_ty), ArraySize::Ident(id));
                        return self.parse_postfix_type_modifiers(ty);
                    }
                }
            }
            let first = self.parse_type()?;
            if self.peek() == Some(&Token::Semicolon) {
                self.bump()?;
                let size = if let Some(Token::IntLiteral(n)) = self.peek() {
                    let sz = ArraySize::Literal(*n as usize);
                    self.bump()?;
                    sz
                } else if let Some(Token::Ident(_)) = self.peek() {
                    let id = self.parse_ident()?;
                    if self.peek() == Some(&Token::RBracket) {
                        ArraySize::Ident(id)
                    } else {
                        let mut expr_str = id.sym.clone();
                        let mut bracket_depth = 0;
                        while self.peek().is_some() {
                            if self.peek() == Some(&Token::RBracket) && bracket_depth == 0 {
                                break;
                            }
                            let (tok, t_span) = self.bump()?;
                            match tok {
                                Token::LBracket => bracket_depth += 1,
                                Token::RBracket => bracket_depth -= 1,
                                _ => {}
                            }
                            expr_str.push(' ');
                            expr_str.push_str(&self.input[t_span.start..t_span.end]);
                        }
                        ArraySize::Expr(expr_str)
                    }
                } else {
                    let mut expr_str = String::new();
                    let mut bracket_depth = 0;
                    while self.peek().is_some() {
                        if self.peek() == Some(&Token::RBracket) && bracket_depth == 0 {
                            break;
                        }
                        let (tok, t_span) = self.bump()?;
                        match tok {
                            Token::LBracket => bracket_depth += 1,
                            Token::RBracket => bracket_depth -= 1,
                            _ => {}
                        }
                        expr_str.push_str(&self.input[t_span.start..t_span.end]);
                    }
                    ArraySize::Expr(expr_str)
                };
                self.expect(Token::RBracket)?;
                let ty = Type::FixedArray(Box::new(first), size);
                return self.parse_postfix_type_modifiers(ty);
            }
            let mut elements = vec![first];
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
                while self.peek() != Some(&Token::RBracket) {
                    elements.push(self.parse_type()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
            }
            self.expect(Token::RBracket)?;
            let ty = Type::Tuple(elements);
            return self.parse_postfix_type_modifiers(ty);
        }
        if self.peek() == Some(&Token::LBrace) {
            self.bump()?;
            let mut fields = Vec::new();
            while self.peek() != Some(&Token::RBrace) {
                let f_ident = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                fields.push(Field {
                    span: f_ident.span,
                    ident: f_ident,
                    ty,
                    meta: Vec::new(),
                    default: None,
                    is_pub: true,
                    is_static: false,
                });
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
            }
            self.expect(Token::RBrace)?;
            return Ok(Type::Object(fields));
        }
        let is_legacy_error_union = if self.peek() == Some(&Token::Bang) {
            self.bump()?;
            true
        } else {
            false
        };
        let (token, span) = self.bump()?;
        let mut ty = match token {
            Token::U8 => Type::U8,
            Token::I8 => Type::I8,
            Token::U16 => Type::U16,
            Token::I16 => Type::I16,
            Token::I32 => Type::I32,
            Token::U32 => Type::U32,
            Token::I64 => Type::I64,
            Token::U64 => Type::U64,
            Token::I128 => Type::I128,
            Token::U128 => Type::U128,
            Token::ISize => Type::ISize,
            Token::USize => Type::USize,
            Token::F32 => Type::F32,
            Token::F64 => Type::F64,
            Token::String => Type::String,
            Token::Void => Type::Void,
            Token::Any => Type::Any,
            Token::Auto => Type::Auto,
            Token::Bool => Type::Bool,
            Token::CStr => Type::CStr,
            Token::ErrorKw => Type::Ref(Ident {
                span,
                sym: "Error".to_string(),
            }),
            Token::TypeKw => Type::Ref(Ident {
                span,
                sym: "type".to_string(),
            }),
            Token::Meta => {
                let mut sym = "meta".to_string();
                loop {
                    if self.peek() == Some(&Token::Dot) {
                        self.bump()?;
                        let next = self.parse_ident()?;
                        sym.push('.');
                        sym.push_str(&next.sym);
                    } else if self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        sym.push('(');
                        let mut paren_depth = 1;
                        while paren_depth > 0 && self.peek().is_some() {
                            let (tok, t_span) = self.bump()?;
                            match tok {
                                Token::LParen => paren_depth += 1,
                                Token::RParen => paren_depth -= 1,
                                _ => {}
                            }
                            if paren_depth == 0 {
                                sym.push(')');
                                break;
                            }
                            sym.push_str(&self.input[t_span.start..t_span.end]);
                        }
                    } else if self.peek() == Some(&Token::Less) {
                        self.bump()?;
                        sym.push('<');
                        let mut angle_depth = 1;
                        while angle_depth > 0 && self.peek().is_some() {
                            let (tok, t_span) = self.bump()?;
                            match tok {
                                Token::Less => angle_depth += 1,
                                Token::Greater => angle_depth -= 1,
                                _ => {}
                            }
                            if angle_depth == 0 {
                                sym.push('>');
                                break;
                            }
                            sym.push_str(&self.input[t_span.start..t_span.end]);
                        }
                    } else {
                        break;
                    }
                }
                Type::Ref(Ident { span, sym })
            }
            Token::Ident(mut sym) => {
                if sym == "cstr" {
                    Type::CStr
                } else if sym == "pid" {
                    Type::PID
                } else if sym == "auto" {
                    Type::Auto
                } else if sym == "Shared" && self.peek() == Some(&Token::Less) {
                    self.bump()?;
                    let inner = self.parse_type()?;
                    self.expect(Token::Greater)?;
                    Type::Shared(Box::new(inner))
                } else if (sym == "Mutex"
                    || sym == "RwLock"
                    || sym == "Atomic"
                    || sym == "LockGuard"
                    || sym == "ReadGuard"
                    || sym == "WriteGuard")
                    && self.peek() == Some(&Token::Less)
                {
                    self.bump()?;
                    let inner = self.parse_type()?;
                    self.expect(Token::Greater)?;
                    let inner_str = Self::type_to_str(&inner);
                    Type::Ref(Ident {
                        span,
                        sym: format!("{}<{}>", sym, inner_str),
                    })
                } else if sym == "Box" && self.peek() == Some(&Token::Less) {
                    self.bump()?;
                    if self.peek() == Some(&Token::Dyn) {
                        self.bump()?;
                    }
                    let tr = self.parse_ident()?;
                    self.expect(Token::Greater)?;
                    Type::BoxDyn(tr)
                } else if (sym == "Map" || sym == "Dict" || sym == "HashMap" || sym == "BTreeMap")
                    && self.peek() == Some(&Token::Less)
                {
                    self.bump()?;
                    let k = self.parse_type()?;
                    self.expect(Token::Comma)?;
                    let v = self.parse_type()?;
                    self.expect(Token::Greater)?;
                    if sym == "Dict" {
                        Type::Dict(Box::new(k), Box::new(v))
                    } else {
                        Type::Map(Box::new(k), Box::new(v))
                    }
                } else if (sym == "Set" || sym == "HashSet" || sym == "BTreeSet")
                    && self.peek() == Some(&Token::Less)
                {
                    self.bump()?;
                    let elem = self.parse_type()?;
                    self.expect(Token::Greater)?;
                    Type::Set(Box::new(elem))
                } else if (sym == "Array" || sym == "Vec") && self.peek() == Some(&Token::Less) {
                    self.bump()?;
                    let elem = self.parse_type()?;
                    self.expect(Token::Greater)?;
                    Type::Array(Box::new(elem))
                } else {
                    let mut sym = sym;
                    loop {
                        if self.peek() == Some(&Token::Dot) {
                            self.bump()?;
                            let next = self.parse_ident()?;
                            sym.push('.');
                            sym.push_str(&next.sym);
                        } else if self.peek() == Some(&Token::LParen) {
                            self.bump()?;
                            sym.push('(');
                            let mut paren_depth = 1;
                            while paren_depth > 0 && self.peek().is_some() {
                                let (tok, t_span) = self.bump()?;
                                match tok {
                                    Token::LParen => paren_depth += 1,
                                    Token::RParen => paren_depth -= 1,
                                    _ => {}
                                }
                                if paren_depth == 0 {
                                    sym.push(')');
                                    break;
                                }
                                sym.push_str(&self.input[t_span.start..t_span.end]);
                            }
                        } else if self.peek() == Some(&Token::Less) {
                            self.bump()?;
                            sym.push('<');
                            let mut angle_depth = 1;
                            while angle_depth > 0 && self.peek().is_some() {
                                let (tok, t_span) = self.bump()?;
                                match tok {
                                    Token::Less => angle_depth += 1,
                                    Token::Greater => angle_depth -= 1,
                                    _ => {}
                                }
                                if angle_depth == 0 {
                                    sym.push('>');
                                    break;
                                }
                                sym.push_str(&self.input[t_span.start..t_span.end]);
                            }
                        } else {
                            break;
                        }
                    }
                    Type::Ref(Ident { span, sym })
                }
            }
            _ => return Err(ParseError::UnexpectedToken(token, span)),
        };
        ty = self.parse_postfix_type_modifiers(ty)?;
        if is_legacy_error_union {
            ty = Type::ErrorUnion(Box::new(ty));
        }
        Ok(ty)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        if self.peek() == Some(&Token::If) {
            self.bump()?; // consume 'if'
            self.expect(Token::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(Token::RParen)?;
            let cons = self.parse_type()?;
            self.expect(Token::Else)?;
            let alt = self.parse_type()?;
            return Ok(Type::Conditional(
                Box::new(cond),
                Box::new(cons),
                Box::new(alt),
            ));
        }
        if self.peek() == Some(&Token::At) {
            let mut clone = self.clone_lookahead();
            let _ = clone.bump();
            if clone.peek() == Some(&Token::If)
                || matches!(clone.peek(), Some(Token::Ident(s)) if s == "if")
            {
                self.bump()?; // consume '@'
                self.bump()?; // consume 'if'
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;
                let cons = self.parse_type()?;
                self.expect(Token::Else)?;
                let alt = self.parse_type()?;
                return Ok(Type::Conditional(
                    Box::new(cond),
                    Box::new(cons),
                    Box::new(alt),
                ));
            }
        }
        if self.peek() == Some(&Token::Bang) {
            self.bump()?;
            if matches!(
                self.peek(),
                Some(Token::LBrace)
                    | Some(Token::Semicolon)
                    | Some(Token::Comma)
                    | Some(Token::RParen)
                    | Some(Token::RBracket)
                    | Some(Token::Assign)
                    | None
            ) {
                return Ok(Type::ErrorUnion(Box::new(Type::Void)));
            }
            let inner = self.parse_type()?;
            return Ok(Type::ErrorUnion(Box::new(inner)));
        }
        let base = self.parse_base_type()?;
        if self.peek() == Some(&Token::Bang) {
            self.bump()?;
            // Check if LHS is an error set (Zig-style: anyerror!T or SomeSet.Error!T or error{...}!T)
            if let Type::Ref(id) = &base {
                if id.sym == "anyerror" || id.sym == "AnyErr" {
                    let ok_ty = self.parse_type()?;
                    return Ok(Type::ResultType {
                        ok: Box::new(ok_ty),
                        err: ErrorType::AnyErr,
                    });
                }
                if id.sym.ends_with("Error")
                    || id.sym.ends_with("Err")
                    || id.sym.starts_with("error{")
                {
                    let ok_ty = self.parse_type()?;
                    return Ok(Type::ResultType {
                        ok: Box::new(ok_ty),
                        err: ErrorType::Named(id.clone()),
                    });
                }
            }
            if self.is_error_type_rhs_start() {
                let err = self.parse_error_type_rhs()?;
                return Ok(Type::ResultType {
                    ok: Box::new(base),
                    err,
                });
            } else {
                return Ok(Type::ErrorUnion(Box::new(base)));
            }
        }
        Ok(base)
    }

    pub fn type_to_str(ty: &Type) -> String {
        match ty {
            Type::U8 => "u8".to_string(),
            Type::I8 => "i8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::U32 => "u32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            Type::I128 => "i128".to_string(),
            Type::U128 => "u128".to_string(),
            Type::ISize => "isize".to_string(),
            Type::USize => "usize".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Any => "any".to_string(),
            Type::Auto => "auto".to_string(),
            Type::PID => "PID".to_string(),
            Type::Array(inner) => format!("[{}]", Self::type_to_str(inner)),
            Type::FixedArray(inner, n) => format!("[{}; {}]", Self::type_to_str(inner), n),
            Type::ManyRawPtr(inner) => format!("[*]{}", Self::type_to_str(inner)),
            Type::CPtr(inner) => format!("[*c]{}", Self::type_to_str(inner)),
            Type::SentinelPtr(inner, n) => format!("[*:{n}]{}", Self::type_to_str(inner)),
            Type::SentinelSlice(inner, n) => format!("[:{n}]{}", Self::type_to_str(inner)),
            Type::Optional(inner) => format!("{}?", Self::type_to_str(inner)),
            Type::Shared(inner) => format!("Shared<{}>", Self::type_to_str(inner)),
            Type::Ref(id) => id.sym.clone(),
            Type::Conditional(_cond, cons, alt) => {
                format!(
                    "@if (...) {} else {}",
                    Self::type_to_str(cons),
                    Self::type_to_str(alt)
                )
            }
            _ => "any".to_string(),
        }
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        if matches!(self.peek(), Some(Token::Ident(id)) if id == "comptime" || id == "inline") {
            let is_inline = matches!(self.peek(), Some(Token::Ident(id)) if id == "inline");
            let mut clone = self.clone_lookahead();
            let _ = clone.bump();
            if clone.peek() == Some(&Token::LBrace) {
                self.bump()?; // consume 'comptime'
                return Ok(Stmt::ComptimeBlock(self.parse_block_stmt()?));
            } else if clone.peek() == Some(&Token::For)
                || matches!(clone.peek(), Some(Token::Ident(id)) if id == "for")
            {
                self.bump()?; // consume 'comptime' or 'inline'
                return Ok(Stmt::For(self.parse_for_stmt_with_comptime(true)?));
            } else if clone.peek() == Some(&Token::While) {
                self.bump()?; // consume 'comptime' or 'inline'
                let mut w = self.parse_while_stmt()?;
                w.is_inline = is_inline;
                return Ok(Stmt::While(w));
            } else if matches!(
                clone.peek(),
                Some(Token::Var) | Some(Token::Let) | Some(Token::Const)
            ) {
                return Ok(Stmt::Var(self.parse_var_decl()?));
            }
        }
        if matches!(self.peek(), Some(Token::Ident(id)) if id == "extern")
            && matches!(
                self.peek_next(),
                Some(Token::Var) | Some(Token::Let) | Some(Token::Const) | Some(Token::ThreadLocal)
            )
        {
            self.bump()?; // consume 'extern'
            let mut v = self.parse_var_decl()?;
            v.is_extern = true;
            v.is_c = true;
            return Ok(Stmt::Var(v));
        }
        if matches!(self.peek(), Some(Token::Ident(_))) && self.peek_next() == Some(&Token::Colon) {
            let mut clone = self.clone_lookahead();
            if let Ok(label) = clone.parse_ident() {
                if clone.peek() == Some(&Token::Colon) {
                    let _ = clone.bump(); // consume ':'
                    if clone.peek() == Some(&Token::While) {
                        self.bump()?; // consume ident
                        self.bump()?; // consume ':'
                        let mut w = self.parse_while_stmt()?;
                        w.label = Some(label);
                        return Ok(Stmt::While(w));
                    } else if clone.peek() == Some(&Token::For)
                        || matches!(clone.peek(), Some(Token::Ident(id)) if id == "for")
                    {
                        self.bump()?; // consume ident
                        self.bump()?; // consume ':'
                        let mut f = self.parse_for_stmt()?;
                        f.label = Some(label);
                        return Ok(Stmt::For(f));
                    } else if clone.peek() == Some(&Token::LBrace) {
                        self.bump()?; // consume ident
                        self.bump()?; // consume ':'
                        let block = self.parse_block_stmt()?;
                        return Ok(Stmt::Expr(Expr::LabeledBlock {
                            label: Some(label),
                            block,
                        }));
                    }
                }
            }
        }
        match self.peek() {
            Some(Token::Fn) => {
                let func = self.parse_function()?;
                Ok(Stmt::Func(Box::new(func)))
            }
            Some(Token::Let) | Some(Token::Const) | Some(Token::Var) | Some(Token::ThreadLocal) => {
                Ok(Stmt::Var(self.parse_var_decl()?))
            }
            Some(Token::While) => Ok(Stmt::While(self.parse_while_stmt()?)),
            Some(Token::For) => Ok(Stmt::For(self.parse_for_stmt()?)),
            Some(Token::If) => Ok(Stmt::If(self.parse_if_stmt()?)),
            Some(Token::Return) => Ok(Stmt::Return(self.parse_return_stmt()?)),
            Some(Token::Break) => {
                let start = self.bump()?.1.start;
                // Check if followed by colon for label: break :label [value]
                if self.peek() == Some(&Token::Colon) {
                    self.bump()?; // consume ':'
                    let label = self.parse_ident()?;
                    let val = if self.peek() != Some(&Token::Semicolon)
                        && self.peek() != Some(&Token::RBrace)
                        && self.peek().is_some()
                    {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    if self.peek() == Some(&Token::Semicolon) {
                        self.bump()?;
                    }
                    let end = self.peek_span().start;
                    return Ok(Stmt::Expr(Expr::LabeledBreak(
                        Span { start, end },
                        label,
                        val,
                    )));
                }
                // Check if break has a value (e.g. break value;)
                if self.peek() != Some(&Token::Semicolon)
                    && self.peek() != Some(&Token::RBrace)
                    && self.peek().is_some()
                {
                    let val = self.parse_expr()?;
                    if self.peek() == Some(&Token::Semicolon) {
                        self.bump()?;
                    }
                    let end = self.peek_span().start;
                    return Ok(Stmt::Expr(Expr::Break(
                        Span { start, end },
                        Some(Box::new(val)),
                    )));
                }
                if self.peek() == Some(&Token::Semicolon) {
                    self.bump()?;
                }
                Ok(Stmt::Break)
            }
            Some(Token::Continue) => {
                let start = self.bump()?.1.start;
                if self.peek() == Some(&Token::Colon) {
                    self.bump()?; // consume ':'
                    let label = self.parse_ident()?;
                    if self.peek() == Some(&Token::Semicolon) {
                        self.bump()?;
                    }
                    let end = self.peek_span().start;
                    return Ok(Stmt::Expr(Expr::LabeledContinue(
                        Span { start, end },
                        label,
                    )));
                }
                if self.peek() == Some(&Token::Semicolon) {
                    self.bump()?;
                }
                Ok(Stmt::Continue)
            }
            Some(Token::Try) => {
                if self.peek_next() != Some(&Token::LBrace) {
                    let expr = self.parse_expr()?;
                    if self.peek() == Some(&Token::Semicolon) {
                        self.bump()?;
                    }
                    return Ok(Stmt::Expr(expr));
                }
                let start = self.expect(Token::Try)?.start;
                let try_block = self.parse_block_stmt()?;
                let mut err_ident = None;
                let mut is_err_mut = false;
                let mut catch_body = None;
                if self.peek() == Some(&Token::Catch) {
                    self.bump()?;
                    if self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        if self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                        }
                        err_ident = Some(self.parse_ident()?);
                        self.expect(Token::RParen)?;
                    } else if self.peek() == Some(&Token::Pipe) {
                        self.bump()?;
                        if self.peek() == Some(&Token::Star) || self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                        }
                        err_ident = Some(self.parse_ident()?);
                        self.expect(Token::Pipe)?;
                    } else if self.peek() != Some(&Token::LBrace) {
                        let checkpoint = self.lexer.clone();
                        let p1 = self.peeked.clone();
                        let p2 = self.peeked2.clone();
                        let mut is_binding = false;
                        if self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                            is_binding = true;
                        }
                        if matches!(
                            self.peek(),
                            Some(Token::Ident(_)) | Some(Token::ErrorKw) | Some(Token::Auto)
                        ) {
                            let id = self.parse_ident()?;
                            if self.peek() == Some(&Token::FatArrow) {
                                self.bump()?;
                                err_ident = Some(id);
                                is_binding = true;
                            } else if is_err_mut {
                                err_ident = Some(id);
                            }
                        }
                        if !is_binding {
                            self.lexer = checkpoint;
                            self.peeked = p1;
                            self.peeked2 = p2;
                            is_err_mut = false;
                        }
                    }
                    let body = self.parse_stmt()?;
                    catch_body = Some(Box::new(body));
                }
                Ok(Stmt::TryCatch {
                    span: Span {
                        start,
                        end: self.peek_span().end,
                    },
                    try_block,
                    err_ident,
                    is_err_mut,
                    catch_body,
                })
            }
            Some(Token::Defer) => {
                self.bump()?;
                Ok(Stmt::Defer(Box::new(self.parse_stmt()?)))
            }
            Some(Token::ErrDefer) => {
                self.bump()?;
                Ok(Stmt::ErrDefer(Box::new(self.parse_stmt()?)))
            }
            Some(Token::Bang) => {
                if self.peek_next() == Some(&Token::LBrace) {
                    self.bump()?;
                    Ok(Stmt::ComptimeBlock(self.parse_block_stmt()?))
                } else {
                    let expr = self.parse_expr()?;
                    if self.peek() == Some(&Token::Semicolon) {
                        self.bump()?;
                    }
                    Ok(Stmt::Expr(expr))
                }
            }
            Some(Token::At) => {
                if matches!(self.peek_next(), Some(Token::Ident(id)) if id == "tls" || id == "threadlocal")
                {
                    return Ok(Stmt::Var(self.parse_var_decl()?));
                }
                if matches!(self.peek_next(), Some(Token::Ident(id)) if id == "comptime") {
                    let mut clone = self.clone_lookahead();
                    let _ = clone.bump();
                    let _ = clone.bump();
                    if clone.peek() == Some(&Token::LBrace) {
                        self.bump()?;
                        self.bump()?;
                        return Ok(Stmt::ComptimeBlock(self.parse_block_stmt()?));
                    }
                }
                if self.peek_next() == Some(&Token::LBrace) {
                    self.bump()?; // consume @
                    Ok(Stmt::ComptimeBlock(self.parse_block_stmt()?))
                } else if self.peek_next() == Some(&Token::If)
                    || matches!(self.peek_next(), Some(Token::Ident(id)) if id == "if")
                {
                    self.bump()?; // consume @
                    Ok(Stmt::If(self.parse_if_stmt_with_comptime(true)?))
                } else if self.peek_next() == Some(&Token::For)
                    || matches!(self.peek_next(), Some(Token::Ident(id)) if id == "for")
                {
                    self.bump()?; // consume @
                    Ok(Stmt::For(self.parse_for_stmt_with_comptime(true)?))
                } else {
                    let mut clone = self.clone_lookahead();
                    let mut is_local_fn = false;
                    while clone.peek() == Some(&Token::At) {
                        if clone.bump().is_err() {
                            break;
                        }
                        if clone.peek() == Some(&Token::LBracket) {
                            if clone.bump().is_err() {
                                break;
                            }
                            let mut depth = 1;
                            while depth > 0 && clone.peek().is_some() {
                                if clone.peek() == Some(&Token::LBracket) {
                                    depth += 1;
                                } else if clone.peek() == Some(&Token::RBracket) {
                                    depth -= 1;
                                }
                                if clone.bump().is_err() {
                                    break;
                                }
                            }
                        } else {
                            if clone.bump().is_err() {
                                break;
                            }
                            if clone.peek() == Some(&Token::LParen) {
                                if clone.bump().is_err() {
                                    break;
                                }
                                let mut depth = 1;
                                while depth > 0 && clone.peek().is_some() {
                                    if clone.peek() == Some(&Token::LParen) {
                                        depth += 1;
                                    } else if clone.peek() == Some(&Token::RParen) {
                                        depth -= 1;
                                    }
                                    if clone.bump().is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    let mut is_extern = false;
                    if matches!(clone.peek(), Some(Token::Ident(id)) if id == "extern") {
                        let _ = clone.bump();
                        is_extern = true;
                    }
                    if matches!(
                        clone.peek(),
                        Some(Token::Let)
                            | Some(Token::Const)
                            | Some(Token::Var)
                            | Some(Token::ThreadLocal)
                    ) {
                        let meta_list = self.parse_meta_list()?;
                        if is_extern {
                            self.bump()?;
                        }
                        let is_tl = meta_list
                            .iter()
                            .any(|m| m.name.sym == "tls" || m.name.sym == "threadlocal");
                        let is_c = meta_list.iter().any(|m| m.name.sym == "c");
                        let mut v = self.parse_var_decl()?;
                        if is_tl {
                            v.is_threadlocal = true;
                        }
                        if is_extern {
                            v.is_extern = true;
                            v.is_c = is_c || true;
                        } else if is_c {
                            v.is_c = true;
                        }
                        return Ok(Stmt::Var(v));
                    }
                    if matches!(clone.peek(), Some(Token::Fn) | Some(Token::Async)) {
                        is_local_fn = true;
                    }

                    if is_local_fn {
                        let meta_list = self.parse_meta_list()?;
                        let mut func = self.parse_function()?;
                        func.meta = meta_list;
                        return Ok(Stmt::Func(Box::new(func)));
                    }

                    let expr = self.parse_expr()?;
                    if self.peek() == Some(&Token::Semicolon) {
                        self.bump()?;
                    }
                    Ok(Stmt::Expr(expr))
                }
            }
            Some(Token::Unsafe) => {
                self.bump()?;
                Ok(Stmt::UnsafeBlock(self.parse_block_stmt()?))
            }
            Some(Token::Arena) => {
                self.bump()?;
                Ok(Stmt::ArenaBlock(self.parse_block_stmt()?))
            }
            Some(Token::LBrace) => Ok(Stmt::Block(self.parse_block_stmt()?)),
            Some(Token::ZigEscape) => {
                let (span, code) = self.parse_raw_zig_block()?;
                Ok(Stmt::Expr(Expr::Zig(ZigEscapeExpr { span, code })))
            }
            _ => {
                let expr = self.parse_expr()?;
                if self.peek() == Some(&Token::Semicolon) {
                    self.bump()?;
                }
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        let start = self.peek_span().start;
        let mut is_comptime = false;
        if matches!(self.peek(), Some(Token::Ident(id)) if id == "comptime") {
            let mut clone = self.clone_lookahead();
            let _ = clone.bump();
            if matches!(
                clone.peek(),
                Some(Token::Let) | Some(Token::Var) | Some(Token::Const)
            ) {
                self.bump()?; // consume 'comptime'
                is_comptime = true;
            }
        }
        let mut is_threadlocal = false;
        if self.peek() == Some(&Token::ThreadLocal) {
            self.bump()?;
            is_threadlocal = true;
        } else if self.peek() == Some(&Token::At) {
            let mut clone = self.clone_lookahead();
            let _ = clone.bump();
            if matches!(clone.peek(), Some(Token::Ident(id)) if id == "tls" || id == "threadlocal")
            {
                self.bump()?; // consume @
                self.bump()?; // consume tls / threadlocal
                is_threadlocal = true;
            }
        }
        let kind = match self.bump()?.0 {
            Token::Let | Token::Var => VarDeclKind::Let,
            Token::Const => VarDeclKind::Const,
            tok => return Err(ParseError::ExpectedToken(Token::Let, tok, self.peek_span())),
        };
        if self.peek() == Some(&Token::Mut) {
            let (_, span) = self.bump()?;
            return Err(ParseError::UnexpectedToken(Token::Mut, span));
        }

        let pat = self.parse_pattern()?;
        let mut ty = None;
        if self.peek() == Some(&Token::Colon) {
            self.bump()?;
            ty = Some(self.parse_type()?);
        }
        let init = if self.peek() == Some(&Token::Assign) {
            self.bump()?;
            Some(self.parse_expr()?)
        } else {
            Some(Expr::Lit(Lit::Undefined))
        };
        if self.peek() == Some(&Token::Semicolon) {
            self.bump()?;
        }
        let end = self.peek_span().end;
        Ok(VarDecl {
            span: Span { start, end },
            kind,
            pat,
            ty,
            init,
            is_threadlocal,
            is_pub: false,
            is_comptime,
            is_extern: false,
            is_c: false,
        })
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        let first = self.parse_single_pattern()?;
        if self.peek() == Some(&Token::Pipe)
            || (self.in_switch_arm > 0
                && self.in_paren_pattern == 0
                && self.peek() == Some(&Token::Comma))
        {
            let mut pats = vec![first];
            while self.peek() == Some(&Token::Pipe)
                || (self.in_switch_arm > 0
                    && self.in_paren_pattern == 0
                    && self.peek() == Some(&Token::Comma))
            {
                self.bump()?;
                pats.push(self.parse_single_pattern()?);
            }
            return Ok(Pattern::Or(pats));
        }
        Ok(first)
    }

    fn try_parse_range_bound(&mut self) -> Option<RangeBound> {
        let (tok, span) = self.bump().ok()?;
        match tok {
            Token::Minus => {
                let (next_tok, next_span) = self.bump().ok()?;
                match next_tok {
                    Token::IntLiteral(v) => Some(RangeBound::Int(-v)),
                    Token::Ident(s) => Some(RangeBound::NegIdent(Ident {
                        span: Span {
                            start: span.start,
                            end: next_span.end,
                        },
                        sym: s,
                    })),
                    _ => None,
                }
            }
            Token::IntLiteral(v) => Some(RangeBound::Int(v)),
            Token::Ident(s) => Some(RangeBound::Ident(Ident { span, sym: s })),
            _ => None,
        }
    }

    fn parse_single_pattern(&mut self) -> ParseResult<Pattern> {
        match self.peek() {
            Some(Token::At) => {
                let expr = self.parse_postfix_expr()?;
                Ok(Pattern::Expr(Box::new(expr)))
            }
            Some(Token::Mut) => {
                let (_, span) = self.bump()?;
                return Err(ParseError::UnexpectedToken(Token::Mut, span));
            }
            Some(Token::Null) => {
                self.bump()?;
                Ok(Pattern::Null)
            }
            Some(Token::DotDotDot) => {
                self.bump()?;
                if !matches!(
                    self.peek(),
                    Some(
                        Token::Comma
                            | Token::RBracket
                            | Token::RBrace
                            | Token::RParen
                            | Token::Pipe
                            | Token::FatArrow
                    ) | None
                ) {
                    let id = self.parse_ident()?;
                    Ok(Pattern::Rest(Box::new(Pattern::Ident(id))))
                } else {
                    Ok(Pattern::Rest(Box::new(Pattern::Wildcard)))
                }
            }
            Some(Token::LBracket) => {
                let mut elements = Vec::new();
                self.bump()?;
                self.in_paren_pattern += 1;
                while self.peek() != Some(&Token::RBracket) {
                    elements.push(self.parse_pattern()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                self.expect(Token::RBracket)?;
                self.in_paren_pattern -= 1;
                Ok(Pattern::Tuple(elements))
            }
            Some(Token::LBrace) => {
                let _start = self.bump()?.1.start;
                self.in_paren_pattern += 1;
                let mut props = Vec::new();
                while self.peek() != Some(&Token::RBrace) {
                    if self.peek() == Some(&Token::DotDotDot) {
                        let dot_span = self.bump()?.1;
                        let (key, val) =
                            if !matches!(self.peek(), Some(Token::Comma | Token::RBrace) | None) {
                                if self.peek() == Some(&Token::Star) {
                                    let (_, s) = self.bump()?;
                                    (
                                        Ident {
                                            span: s,
                                            sym: "*".to_string(),
                                        },
                                        Some(Pattern::Wildcard),
                                    )
                                } else {
                                    let rest_ident = self.parse_ident()?;
                                    (rest_ident, None)
                                }
                            } else {
                                (
                                    Ident {
                                        span: dot_span,
                                        sym: "*".to_string(),
                                    },
                                    Some(Pattern::Wildcard),
                                )
                            };
                        props.push(ObjectPatProp {
                            key,
                            val,
                            is_rest: true,
                        });
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                        continue;
                    }
                    let key = self.parse_ident()?;
                    if key.sym == "mut"
                        && self.peek() != Some(&Token::Colon)
                        && matches!(self.peek(), Some(Token::Ident(_)))
                    {
                        return Err(ParseError::UnexpectedToken(Token::Mut, key.span));
                    }
                    let mut val = None;
                    if self.peek() == Some(&Token::Colon) {
                        self.bump()?;
                        val = Some(self.parse_pattern()?);
                    }
                    props.push(ObjectPatProp {
                        key,
                        val,
                        is_rest: false,
                    });
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                let _end = self.expect(Token::RBrace)?.end;
                self.in_paren_pattern -= 1;
                Ok(Pattern::Object(props))
            }
            Some(Token::Dot) => {
                self.bump()?;
                let variant_name = self.parse_ident()?;
                let mut fields = None;
                let mut tuple_pats = None;
                if self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    self.in_paren_pattern += 1;
                    let mut pats = Vec::new();
                    while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                        pats.push(self.parse_pattern()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RParen)?;
                    self.in_paren_pattern -= 1;
                    tuple_pats = Some(pats);
                } else if self.peek() == Some(&Token::LBracket) {
                    self.bump()?;
                    self.in_paren_pattern += 1;
                    let mut pats = Vec::new();
                    while self.peek() != Some(&Token::RBracket) && self.peek().is_some() {
                        pats.push(self.parse_pattern()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RBracket)?;
                    self.in_paren_pattern -= 1;
                    tuple_pats = Some(pats);
                } else if self.peek() == Some(&Token::LBrace) {
                    self.bump()?;
                    self.in_paren_pattern += 1;
                    let mut props = Vec::new();
                    while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                        if self.peek() == Some(&Token::DotDotDot) {
                            let dot_span = self.bump()?.1;
                            let (key, val) = if !matches!(
                                self.peek(),
                                Some(Token::Comma | Token::RBrace) | None
                            ) {
                                let id = self.parse_ident()?;
                                (id, None)
                            } else {
                                (
                                    Ident {
                                        span: dot_span,
                                        sym: "_".to_string(),
                                    },
                                    None,
                                )
                            };
                            props.push(ObjectPatProp {
                                key,
                                val,
                                is_rest: true,
                            });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                            continue;
                        }
                        let key = self.parse_ident()?;
                        let mut val = None;
                        if self.peek() == Some(&Token::Colon) {
                            self.bump()?;
                            val = Some(self.parse_pattern()?);
                        }
                        props.push(ObjectPatProp {
                            key,
                            val,
                            is_rest: false,
                        });
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RBrace)?;
                    self.in_paren_pattern -= 1;
                    fields = Some(props);
                } else if let Some(Token::Ident(_)) = self.peek() {
                    let key = self.parse_ident()?;
                    tuple_pats = Some(vec![Pattern::Ident(key)]);
                }
                Ok(Pattern::Enum(EnumPattern {
                    span: Span { start: 0, end: 0 },
                    enum_name: Ident {
                        span: Span { start: 0, end: 0 },
                        sym: "".to_string(),
                    },
                    variant_name,
                    fields,
                    tuple_pats,
                }))
            }
            _ => {
                if self.peek() == Some(&Token::Star)
                    || self.peek() == Some(&Token::Bang)
                    || self.peek() == Some(&Token::Else)
                {
                    self.bump()?;
                    return Ok(Pattern::Wildcard);
                }
                // Try parsing range pattern e.g. 1...ARES_ENOSERVER or -ARES_ENOSERVER...-1 or 1..=10
                {
                    let mut probe = self.clone_lookahead();
                    if let Some(start_bound) = probe.try_parse_range_bound() {
                        if matches!(
                            probe.peek(),
                            Some(Token::DotDotDot) | Some(Token::DotDotEq) | Some(Token::DotDot)
                        ) {
                            let op = probe.bump().unwrap().0;
                            let inclusive = op == Token::DotDotDot || op == Token::DotDotEq;
                            if let Some(end_bound) = probe.try_parse_range_bound() {
                                *self = probe;
                                return Ok(Pattern::Range {
                                    start: start_bound,
                                    end: end_bound,
                                    inclusive,
                                });
                            }
                        }
                    }
                }
                if self.peek() == Some(&Token::Minus) {
                    let mut probe = self.clone_lookahead();
                    let _ = probe.bump();
                    if let Some(Token::IntLiteral(_)) = probe.peek() {
                        self.bump()?;
                        if let (Token::IntLiteral(v), _) = self.bump()? {
                            return Ok(Pattern::Lit(Lit::Int(-v)));
                        }
                    }
                }
                if let Some(Token::IntLiteral(_)) = self.peek() {
                    if let (Token::IntLiteral(v), _) = self.bump()? {
                        return Ok(Pattern::Lit(Lit::Int(v)));
                    }
                }
                if let Some(Token::StringLiteral(_)) = self.peek() {
                    if let (Token::StringLiteral(s), _) = self.bump()? {
                        return Ok(Pattern::Lit(Lit::Str(s)));
                    }
                }
                let first_id = self.parse_ident()?;
                if self.peek() == Some(&Token::ColonAssign) {
                    self.bump()?;
                    let subpat = self.parse_pattern()?;
                    return Ok(Pattern::Binding {
                        ident: first_id,
                        subpat: Box::new(subpat),
                    });
                }
                if self.in_switch_arm > 0 && self.peek() == Some(&Token::Colon) {
                    self.bump()?;
                    let ty = self.parse_type()?;
                    return Ok(Pattern::TypeMatch {
                        ident: first_id,
                        ty,
                    });
                }
                if first_id.sym == "true" {
                    return Ok(Pattern::Lit(Lit::Bool(true)));
                }
                if first_id.sym == "false" {
                    return Ok(Pattern::Lit(Lit::Bool(false)));
                }
                if (first_id.sym == "Err" || first_id.sym == "err" || first_id.sym == "error")
                    && self.peek() == Some(&Token::LParen)
                {
                    self.bump()?;
                    self.in_paren_pattern += 1;
                    let first_p = self.parse_pattern()?;
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                        let mut pats = vec![first_p];
                        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                            pats.push(self.parse_pattern()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RParen)?;
                        self.in_paren_pattern -= 1;
                        return Ok(Pattern::Err(Box::new(Pattern::Tuple(pats))));
                    } else {
                        self.expect(Token::RParen)?;
                        self.in_paren_pattern -= 1;
                        return Ok(Pattern::Err(Box::new(first_p)));
                    }
                }
                if self.peek() == Some(&Token::Dot) {
                    self.bump()?;
                    let variant_name = self.parse_ident()?;
                    let mut fields = None;
                    let mut tuple_pats = None;
                    if self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        self.in_paren_pattern += 1;
                        let mut pats = Vec::new();
                        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                            pats.push(self.parse_pattern()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RParen)?;
                        self.in_paren_pattern -= 1;
                        tuple_pats = Some(pats);
                    } else if self.peek() == Some(&Token::LBracket) {
                        self.bump()?;
                        self.in_paren_pattern += 1;
                        let mut pats = Vec::new();
                        while self.peek() != Some(&Token::RBracket) && self.peek().is_some() {
                            pats.push(self.parse_pattern()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RBracket)?;
                        self.in_paren_pattern -= 1;
                        tuple_pats = Some(pats);
                    } else if self.peek() == Some(&Token::LBrace) {
                        self.bump()?;
                        self.in_paren_pattern += 1;
                        let mut props = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            if self.peek() == Some(&Token::DotDotDot) {
                                let dot_span = self.bump()?.1;
                                let (key, val) = if !matches!(
                                    self.peek(),
                                    Some(Token::Comma | Token::RBrace) | None
                                ) {
                                    let id = self.parse_ident()?;
                                    (id, None)
                                } else {
                                    (
                                        Ident {
                                            span: dot_span,
                                            sym: "_".to_string(),
                                        },
                                        None,
                                    )
                                };
                                props.push(ObjectPatProp {
                                    key,
                                    val,
                                    is_rest: true,
                                });
                                if self.peek() == Some(&Token::Comma) {
                                    self.bump()?;
                                }
                                continue;
                            }
                            let key = self.parse_ident()?;
                            let mut val = None;
                            if self.peek() == Some(&Token::Colon) {
                                self.bump()?;
                                val = Some(self.parse_pattern()?);
                            }
                            props.push(ObjectPatProp {
                                key,
                                val,
                                is_rest: false,
                            });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RBrace)?;
                        self.in_paren_pattern -= 1;
                        fields = Some(props);
                    } else if let Some(Token::Ident(_)) = self.peek() {
                        let key = self.parse_ident()?;
                        tuple_pats = Some(vec![Pattern::Ident(key)]);
                    }
                    return Ok(Pattern::Enum(EnumPattern {
                        span: first_id.span,
                        enum_name: first_id,
                        variant_name,
                        fields,
                        tuple_pats,
                    }));
                }
                if first_id
                    .sym
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_uppercase())
                    && self.peek() == Some(&Token::LBrace)
                {
                    self.bump()?;
                    self.in_paren_pattern += 1;
                    let mut props = Vec::new();
                    while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                        if self.peek() == Some(&Token::DotDotDot) {
                            let dot_span = self.bump()?.1;
                            let (key, val) = if !matches!(
                                self.peek(),
                                Some(Token::Comma | Token::RBrace) | None
                            ) {
                                let id = self.parse_ident()?;
                                (id, None)
                            } else {
                                (
                                    Ident {
                                        span: dot_span,
                                        sym: "_".to_string(),
                                    },
                                    None,
                                )
                            };
                            props.push(ObjectPatProp {
                                key,
                                val,
                                is_rest: true,
                            });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                            continue;
                        }
                        let key = self.parse_ident()?;
                        let mut val = None;
                        if self.peek() == Some(&Token::Colon) {
                            self.bump()?;
                            val = Some(self.parse_pattern()?);
                        }
                        props.push(ObjectPatProp {
                            key,
                            val,
                            is_rest: false,
                        });
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RBrace)?;
                    self.in_paren_pattern -= 1;
                    return Ok(Pattern::Enum(EnumPattern {
                        span: first_id.span,
                        enum_name: first_id,
                        variant_name: Ident {
                            span: Span { start: 0, end: 0 },
                            sym: "".to_string(),
                        },
                        fields: Some(props),
                        tuple_pats: None,
                    }));
                }
                Ok(Pattern::Ident(first_id))
            }
        }
    }

    fn parse_block_stmt(&mut self) -> ParseResult<BlockStmt> {
        let start = self.expect(Token::LBrace)?.start;
        let mut body = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            let start_pos = self.peek_span().start;
            match self.parse_stmt() {
                Ok(stmt) => body.push(stmt),
                Err(err) => {
                    self.push_error(err.to_diagnostic());
                    self.recover_stmt();
                    if let Some(_) = self.peek() {
                        if self.peek_span().start == start_pos {
                            let _ = self.bump();
                        }
                    }
                }
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(BlockStmt {
            span: Span { start, end },
            body,
        })
    }

    fn parse_return_stmt(&mut self) -> ParseResult<ReturnStmt> {
        let span = self.expect(Token::Return)?;
        let mut arg = None;
        let has_arg = if matches!(
            self.peek(),
            Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
        ) {
            false
        } else {
            let next_start = self.peek_span().start;
            if next_start >= span.end && self.input[span.end..next_start].contains('\n') {
                false
            } else {
                true
            }
        };
        if has_arg {
            arg = Some(self.parse_expr()?);
        }
        if self.peek() == Some(&Token::Semicolon) {
            self.bump()?;
        }
        let end = self.peek_span().end;
        Ok(ReturnStmt {
            span: Span {
                start: span.start,
                end,
            },
            arg,
        })
    }

    fn parse_match_expr(&mut self) -> ParseResult<MatchExpr> {
        let start = self.peek_span().start;
        let disc = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let mut arms = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            let is_inline = if matches!(self.peek(), Some(Token::Ident(id)) if id == "inline") {
                self.bump()?;
                true
            } else {
                false
            };
            self.in_switch_arm += 1;
            let pat = self.parse_pattern()?;
            self.in_switch_arm -= 1;
            let guard = if self.peek() == Some(&Token::If) {
                self.bump()?;
                let prev = self.in_match_guard;
                self.in_match_guard = true;
                let g_res = self.parse_expr();
                self.in_match_guard = prev;
                Some(g_res?)
            } else {
                None
            };
            self.expect(Token::FatArrow)?;
            let (capture, is_capture_ptr) = if self.peek() == Some(&Token::Pipe) {
                self.bump()?; // consume '|'
                let is_ptr =
                    if self.peek() == Some(&Token::Star) || self.peek() == Some(&Token::Mut) {
                        self.bump()?;
                        true
                    } else {
                        false
                    };
                let cap_id = self.parse_ident()?;
                self.expect(Token::Pipe)?;
                (Some(cap_id), is_ptr)
            } else {
                (None, false)
            };
            let body = if self.peek() == Some(&Token::LBrace) {
                Stmt::Block(self.parse_block_stmt()?)
            } else if self.peek() == Some(&Token::Return) {
                Stmt::Return(self.parse_return_stmt()?)
            } else {
                let expr = self.parse_expr()?;
                Stmt::Expr(expr)
            };
            if self.peek() == Some(&Token::Comma) || self.peek() == Some(&Token::Semicolon) {
                self.bump()?;
            }
            arms.push(MatchArm {
                span: Span { start: 0, end: 0 },
                pat,
                guard,
                body: Box::new(body),
                is_inline,
                capture,
                is_capture_ptr,
            });
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(MatchExpr {
            span: Span { start, end },
            disc: Box::new(disc),
            arms,
        })
    }

    fn parse_while_stmt(&mut self) -> ParseResult<WhileStmt> {
        let start = self.expect(Token::While)?.start;
        if self.peek() != Some(&Token::LParen) {
            return Err(ParseError::Generic(
                "CP Syntax Error: 'while' condition requires parentheses.\n  = help: in CP, while loops require parentheses around the condition.\n  = suggestion: wrap the condition in parentheses: while (condition) { ... }".to_string(),
                self.peek_span(),
            ));
        }
        self.expect(Token::LParen)?;
        let mut capture = None;
        let mut is_ptr_capture = false;
        if (self.peek() == Some(&Token::Star) || self.peek() == Some(&Token::Mut))
            && matches!(
                self.peek_next(),
                Some(Token::Ident(_)) | Some(Token::ErrorKw)
            )
        {
            self.bump()?;
            is_ptr_capture = true;
            capture = Some(self.parse_ident()?);
            self.expect(Token::ColonAssign)?;
        } else if matches!(self.peek(), Some(Token::Ident(_)) | Some(Token::ErrorKw))
            && self.peek_next() == Some(&Token::ColonAssign)
        {
            capture = Some(self.parse_ident()?);
            self.expect(Token::ColonAssign)?;
        }
        let test = self.parse_expr()?;
        self.expect(Token::RParen)?;
        let mut continue_expr = None;
        if self.peek() == Some(&Token::Colon) {
            self.bump()?; // consume ':'
            if self.peek() == Some(&Token::LParen) {
                self.bump()?; // consume '('
                continue_expr = Some(self.parse_expr()?);
                self.expect(Token::RParen)?;
            } else {
                continue_expr = Some(self.parse_expr()?);
            }
        }
        if self.peek() == Some(&Token::Pipe) {
            self.bump()?; // consume '|'
            if self.peek() == Some(&Token::Star) {
                self.bump()?;
                is_ptr_capture = true;
            }
            capture = Some(self.parse_ident()?);
            self.expect(Token::Pipe)?;
        }
        if continue_expr.is_none() && self.peek() == Some(&Token::Colon) {
            self.bump()?; // consume ':'
            if self.peek() == Some(&Token::LParen) {
                self.bump()?; // consume '('
                continue_expr = Some(self.parse_expr()?);
                self.expect(Token::RParen)?;
            } else {
                continue_expr = Some(self.parse_expr()?);
            }
        }
        let body = self.parse_stmt()?;
        Ok(WhileStmt {
            span: Span { start, end: 0 },
            test,
            body: Box::new(body),
            capture,
            is_ptr_capture,
            label: None,
            continue_expr,
            is_inline: false,
        })
    }

    fn parse_for_stmt(&mut self) -> ParseResult<ForStmt> {
        self.parse_for_stmt_with_comptime(false)
    }

    fn parse_for_stmt_with_comptime(&mut self, is_comptime: bool) -> ParseResult<ForStmt> {
        let start = if self.peek() == Some(&Token::For) {
            self.bump()?.1.start
        } else if matches!(self.peek(), Some(Token::Ident(id)) if id == "for") {
            self.bump()?.1.start
        } else {
            self.expect(Token::For)?.start
        };
        if self.peek() != Some(&Token::LParen) {
            return Err(ParseError::Generic(
                "CP Syntax Error: 'for' loop statement requires parentheses.\n  = cause: 'for' loop header is missing enclosing parentheses: 'for (...)'.\n  = help: in CP, control flow statements ('if', 'while', 'for') must be enclosed in parentheses to eliminate ambiguity with struct literal syntax and maintain syntax consistency.\n  = suggestion: wrap the loop header in parentheses: for (item of collection) { ... }".to_string(),
                self.peek_span(),
            ));
        }
        self.expect(Token::LParen)?;

        // Check if this is Zig-style for loop: for (collection) |capture|
        let is_zig_style = {
            let mut clone = self.clone_lookahead();
            let mut depth = 0;
            let mut found_of_or_in = false;
            let mut has_pipe_after_rparen = false;
            while let Some(tok) = clone.peek() {
                match tok {
                    Token::LParen => depth += 1,
                    Token::RParen => {
                        if depth == 0 {
                            clone.bump().ok();
                            if clone.peek() == Some(&Token::Pipe) {
                                has_pipe_after_rparen = true;
                            }
                            break;
                        }
                        depth -= 1;
                    }
                    Token::Of | Token::In if depth == 0 => {
                        found_of_or_in = true;
                        break;
                    }
                    _ => {}
                }
                clone.bump().ok();
            }
            !found_of_or_in && has_pipe_after_rparen
        };

        if is_zig_style {
            let header_start = self.peek_span().start;
            let mut depth = 0;
            while let Some(tok) = self.peek() {
                if tok == &Token::LParen {
                    depth += 1;
                } else if tok == &Token::RParen {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                self.bump()?;
            }
            let header_end = self.peek_span().start;
            let raw_header = self.input[header_start..header_end].trim().to_string();
            self.expect(Token::RParen)?;
            self.expect(Token::Pipe)?;
            let pat_start = self.peek_span().start;
            let mut pipe_depth = 0;
            while let Some(tok) = self.peek() {
                if tok == &Token::Pipe && pipe_depth == 0 {
                    break;
                }
                self.bump()?;
            }
            let pat_end = self.peek_span().start;
            let raw_pat = self.input[pat_start..pat_end].trim().to_string();
            self.expect(Token::Pipe)?;
            let body = self.parse_stmt()?;
            return Ok(ForStmt {
                span: Span { start, end: 0 },
                mode: IterMode::Borrow,
                pat: Pattern::Ident(Ident {
                    span: Span {
                        start: pat_start,
                        end: pat_end,
                    },
                    sym: raw_pat,
                }),
                expr: Expr::Ident(Ident {
                    span: Span {
                        start: header_start,
                        end: header_end,
                    },
                    sym: raw_header.clone(),
                }),
                body: Box::new(body),
                is_comptime,
                label: None,
                raw_header: Some(raw_header),
            });
        }

        if self.peek() == Some(&Token::Let) || self.peek() == Some(&Token::Const) {
            self.bump()?;
        }
        let mode = if self.peek() == Some(&Token::Star) {
            self.bump()?;
            IterMode::MutBorrow
        } else if self.peek() == Some(&Token::Ampersand) {
            self.bump()?;
            if self.peek() == Some(&Token::Mut) {
                self.bump()?;
                IterMode::MutBorrow
            } else {
                IterMode::Borrow
            }
        } else if self.peek() == Some(&Token::Move) {
            self.bump()?;
            IterMode::Move
        } else {
            IterMode::Borrow
        };
        let pat = self.parse_pattern()?;
        if matches!(self.peek(), Some(Token::Of) | Some(Token::In)) {
            self.bump()?;
        } else {
            return Err(ParseError::Generic(
                "Expected 'of' in for loop, e.g. 'for (x of list)'".to_string(),
                self.peek_span(),
            ));
        }
        let mut expr = self.parse_expr()?;
        if self.peek() == Some(&Token::DotDot) || self.peek() == Some(&Token::DotDotEq) {
            let inclusive = self.bump()?.0 == Token::DotDotEq;
            let end_expr = self.parse_expr()?;
            let span = Span {
                start: left_span(&expr).start,
                end: right_span(&end_expr).end,
            };
            expr = Expr::Range {
                span,
                start: Box::new(expr),
                end: Box::new(end_expr),
                inclusive,
            };
        }
        self.expect(Token::RParen)?;
        let body = self.parse_stmt()?;
        Ok(ForStmt {
            span: Span { start, end: 0 },
            mode,
            pat,
            expr,
            body: Box::new(body),
            is_comptime,
            label: None,
            raw_header: None,
        })
    }

    fn parse_if_stmt(&mut self) -> ParseResult<IfStmt> {
        self.parse_if_stmt_with_comptime(false)
    }

    fn parse_if_stmt_with_comptime(&mut self, mut is_comptime: bool) -> ParseResult<IfStmt> {
        let start = if self.peek() == Some(&Token::If) {
            self.bump()?.1.start
        } else if matches!(self.peek(), Some(Token::Ident(id)) if id == "if") {
            self.bump()?.1.start
        } else {
            self.expect(Token::If)?.start
        };
        let mut capture = None;
        let mut is_ptr_capture = false;
        let test = if self.peek() == Some(&Token::LParen) {
            self.bump()?;
            if matches!(self.peek(), Some(Token::Ident(id)) if id == "comptime") {
                self.bump()?;
                is_comptime = true;
            }
            if (self.peek() == Some(&Token::Star) || self.peek() == Some(&Token::Mut))
                && matches!(
                    self.peek_next(),
                    Some(Token::Ident(_)) | Some(Token::ErrorKw)
                )
            {
                self.bump()?;
                is_ptr_capture = true;
                capture = Some(self.parse_ident()?);
                self.expect(Token::ColonAssign)?;
            } else if matches!(self.peek(), Some(Token::Ident(_)) | Some(Token::ErrorKw))
                && self.peek_next() == Some(&Token::ColonAssign)
            {
                capture = Some(self.parse_ident()?);
                self.expect(Token::ColonAssign)?;
            }
            let t = self.parse_expr()?;
            self.expect(Token::RParen)?;
            t
        } else {
            return Err(ParseError::Generic(
                "CP Syntax Error: 'if' condition requires parentheses.\n  = help: in CP, control flow conditions must be enclosed in parentheses to avoid ambiguity with struct literals.\n  = suggestion: wrap the condition in parentheses: if (condition) { ... }".to_string(),
                self.peek_span(),
            ));
        };
        if self.peek() == Some(&Token::Pipe) {
            self.bump()?; // consume '|'
            if self.peek() == Some(&Token::Star) {
                self.bump()?;
                is_ptr_capture = true;
            }
            capture = Some(self.parse_ident()?);
            self.expect(Token::Pipe)?;
        }
        let cons = self.parse_stmt()?;
        let mut alt = None;
        if self.peek() == Some(&Token::Else) {
            self.bump()?;
            alt = Some(Box::new(self.parse_stmt()?));
        } else if self.peek() == Some(&Token::At)
            && (self.peek_next() == Some(&Token::Else)
                || matches!(self.peek_next(), Some(Token::Ident(id)) if id == "else"))
        {
            self.bump()?; // consume @
            self.bump()?; // consume else
            alt = Some(Box::new(self.parse_stmt()?));
        }
        Ok(IfStmt {
            span: Span { start, end: 0 },
            test,
            cons: Box::new(cons),
            alt,
            is_comptime,
            capture,
            is_ptr_capture,
        })
    }

    fn parse_class(&mut self) -> ParseResult<Class> {
        let start = self.peek_span().start;
        let mut is_ref = false;
        let mut _is_struct = false;
        let mut is_static = false;
        let mut is_union = false;
        let mut class_meta = Vec::new();
        loop {
            if self.peek() == Some(&Token::Static) {
                self.bump()?;
                is_static = true;
            } else if self.peek() == Some(&Token::Ref) {
                self.bump()?;
                is_ref = true;
            } else {
                break;
            }
        }
        if self.peek() == Some(&Token::Union) {
            self.bump()?;
            is_union = true;
            if self.peek() == Some(&Token::LParen) {
                self.bump()?;
                let tag_expr = if self.peek() == Some(&Token::Enum) {
                    let span = self.bump()?.1;
                    Expr::Ident(Ident {
                        span,
                        sym: "enum".to_string(),
                    })
                } else {
                    self.parse_expr()?
                };
                self.expect(Token::RParen)?;
                class_meta.push(MetaItem {
                    span: left_span(&tag_expr),
                    name: Ident {
                        span: left_span(&tag_expr),
                        sym: "tag".to_string(),
                    },
                    args: vec![MetaArg::Positional(tag_expr)],
                });
            }
        } else if self.peek() == Some(&Token::Struct) {
            let span = self.bump()?.1;
            is_ref = false;
            _is_struct = true;
            class_meta.push(MetaItem {
                span,
                name: Ident {
                    span,
                    sym: "value".to_string(),
                },
                args: vec![],
            });
        } else {
            self.expect(Token::Class)?;
        }
        let ident = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;

        let mut extends = None;
        let mut implements = Vec::new();
        if self.peek() == Some(&Token::Extends) {
            self.bump()?;
            let parent = self.parse_ident()?;
            if is_static {
                return Err(ParseError::Generic(
                    format!(
                        "Static class or struct '{}' cannot extend another class",
                        ident.sym
                    ),
                    parent.span,
                ));
            }
            if parent.sym == "Agent" {
                implements.push(parent.clone());
            } else {
                extends = Some(parent);
            }
        }
        if self.peek() == Some(&Token::Implements) {
            self.bump()?;
            implements.push(self.parse_ident()?);
            while self.peek() == Some(&Token::Comma) {
                self.bump()?;
                implements.push(self.parse_ident()?);
            }
        }

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut extra_zig = Vec::new();
        self.expect(Token::LBrace)?;
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            if self.peek() == Some(&Token::ZigEscape) {
                let (_, code) = self.parse_raw_zig_block()?;
                extra_zig.push(code);
                continue;
            }
            let meta_list = self.parse_meta_list()?;
            let mut member_is_pub = true;
            let mut has_pub_or_priv = false;
            while self.peek() == Some(&Token::Export) || self.peek() == Some(&Token::Private) {
                has_pub_or_priv = true;
                if self.peek() == Some(&Token::Export) {
                    member_is_pub = true;
                } else if self.peek() == Some(&Token::Private) {
                    member_is_pub = false;
                }
                self.bump()?;
            }
            let mut is_field_static = is_static;
            if self.peek() == Some(&Token::Static) {
                if !matches!(
                    self.peek_next(),
                    Some(Token::Fn) | Some(Token::Async) | Some(Token::Colon)
                ) && !matches!(self.peek_next(), Some(Token::Ident(id)) if id == "inline" || id == "extern" || id == "noinline")
                {
                    self.bump()?;
                    is_field_static = true;
                }
            }
            if self.peek_next() != Some(&Token::Colon)
                && (self.peek() == Some(&Token::Const)
                    || self.peek() == Some(&Token::Var)
                    || self.peek() == Some(&Token::TypeKw)
                    || self.peek() == Some(&Token::Union)
                    || self.peek() == Some(&Token::Struct)
                    || self.peek() == Some(&Token::Enum)
                    || self.peek() == Some(&Token::Class)
                    || self.peek() == Some(&Token::Interface)
                    || self.peek() == Some(&Token::Trait))
            {
                let start_idx = self.peek_span().start;
                let mut brace_depth: i32 = 0;
                let mut paren_depth: i32 = 0;
                let mut bracket_depth: i32 = 0;
                let mut had_brace = false;
                let mut end_idx = start_idx;
                while let Some(tok) = self.peek() {
                    match tok {
                        Token::LBrace => {
                            brace_depth += 1;
                            had_brace = true;
                        }
                        Token::RBrace => {
                            if brace_depth == 0 {
                                break;
                            }
                            brace_depth -= 1;
                            if brace_depth == 0
                                && paren_depth == 0
                                && bracket_depth == 0
                                && had_brace
                            {
                                let span = self.bump()?.1;
                                end_idx = span.end;
                                while let Some(tok) = self.peek() {
                                    if *tok == Token::Semicolon {
                                        let s_span = self.bump()?.1;
                                        end_idx = s_span.end;
                                        break;
                                    }
                                    if matches!(tok, Token::RBrace) {
                                        break;
                                    }
                                    let s_span = self.bump()?.1;
                                    end_idx = s_span.end;
                                }
                                break;
                            }
                        }
                        Token::LParen => paren_depth += 1,
                        Token::RParen => {
                            if paren_depth > 0 {
                                paren_depth -= 1;
                            }
                        }
                        Token::LBracket => bracket_depth += 1,
                        Token::RBracket => {
                            if bracket_depth > 0 {
                                bracket_depth -= 1;
                            }
                        }
                        Token::Semicolon => {
                            if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                                let span = self.bump()?.1;
                                end_idx = span.end;
                                break;
                            }
                        }
                        _ => {}
                    }
                    let span = self.bump()?.1;
                    end_idx = span.end;
                }
                let raw_code = &self.input[start_idx..end_idx];
                let pub_prefix = if member_is_pub && has_pub_or_priv {
                    "pub "
                } else {
                    ""
                };
                extra_zig.push(format!("{}{}", pub_prefix, raw_code.trim()));
                continue;
            }
            if self.peek() == Some(&Token::Let) {
                self.bump()?;
            }
            let is_method = match self.peek() {
                Some(Token::Fn) | Some(Token::Async) => true,
                Some(Token::Static) => {
                    matches!(self.peek_next(), Some(Token::Fn) | Some(Token::Async))
                        || matches!(self.peek_next(), Some(Token::Ident(id)) if id == "inline" || id == "extern" || id == "noinline")
                }
                Some(Token::Ident(id)) if id == "inline" || id == "extern" || id == "noinline" => {
                    matches!(self.peek_next(), Some(Token::Fn) | Some(Token::Async))
                }
                _ => false,
            };
            if is_method {
                let mut method = self.parse_function()?;
                if is_static {
                    if let Some(first) = method.params.first() {
                        if first.ident.sym == "this" || first.ident.sym == "_" {
                            return Err(ParseError::Generic(
                                format!(
                                    "Static class or struct '{}' cannot declare instance method '{}' with receiver 'this' or '_'",
                                    ident.sym, method.ident.sym
                                ),
                                first.span,
                            ));
                        }
                    }
                    if method.ident.sym == "init" || method.ident.sym == "release" {
                        return Err(ParseError::Generic(
                            format!(
                                "Static class or struct '{}' cannot declare instance lifecycle method '{}'",
                                ident.sym, method.ident.sym
                            ),
                            method.span,
                        ));
                    }
                    method.is_static = true;
                }
                method.meta.extend(meta_list);
                method.is_pub = member_is_pub;
                methods.push(method);
            } else {
                let f_ident = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                if matches!(ty, Type::BorrowType { .. }) {
                    is_ref = true;
                }
                let default = if self.peek() == Some(&Token::Assign) {
                    self.bump()?;
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                if self.peek() == Some(&Token::Comma) || self.peek() == Some(&Token::Semicolon) {
                    self.bump()?;
                }
                fields.push(Field {
                    span: f_ident.span,
                    ident: f_ident,
                    ty,
                    meta: meta_list,
                    default,
                    is_pub: member_is_pub,
                    is_static: is_field_static,
                });
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(Class {
            span: Span { start, end },
            ident,
            fields,
            methods,
            implements,
            extends,
            is_ref,
            meta: class_meta,
            extra_zig,
            is_pub: false,
            is_static,
            is_union,
            generic_params,
        })
    }

    fn parse_trait(&mut self) -> ParseResult<TraitDecl> {
        let start = self.expect(Token::Trait)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            let m_start = self.peek_span().start;
            let mut is_unsafe = false;
            if self.peek() == Some(&Token::Unsafe) {
                self.bump()?;
                is_unsafe = true;
            }
            self.expect(Token::Fn)?;
            let m_ident = self.parse_ident()?;
            let generic_params = self.parse_generic_params()?;
            let params = self.parse_params()?;
            let mut return_type = None;
            if self.peek() == Some(&Token::Bang) {
                self.bump()?;
                if self.is_error_type_rhs_start() {
                    let err = self.parse_error_type_rhs()?;
                    return_type = Some(Type::ResultType {
                        ok: Box::new(Type::Void),
                        err,
                    });
                } else {
                    return_type = Some(Type::ErrorUnion(Box::new(Type::Void)));
                }
            } else if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
                self.bump()?;
                return_type = Some(self.parse_type()?);
            }
            let default_body = if self.peek() == Some(&Token::LBrace) {
                Some(self.parse_block_stmt()?)
            } else {
                self.expect(Token::Semicolon)?;
                None
            };
            methods.push(MethodSig {
                span: Span {
                    start: m_start,
                    end: self.peek_span().start,
                },
                ident: m_ident,
                generic_params,
                params,
                return_type,
                default_body,
                is_unsafe,
            });
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(TraitDecl {
            span: Span { start, end },
            ident,
            methods,
            meta: Vec::new(),
        })
    }

    fn parse_impl(&mut self) -> ParseResult<ImplDecl> {
        let start = self.expect(Token::Impl)?.start;
        let trait_name = self.parse_ident()?;
        self.expect(Token::For)?;
        let target_name = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            methods.push(self.parse_function()?);
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(ImplDecl {
            span: Span { start, end },
            trait_name,
            target_name,
            methods,
        })
    }

    fn parse_raw_zig_block(&mut self) -> ParseResult<(Span, String)> {
        let start = self.bump()?.1.start;
        self.expect(Token::LBrace)?;
        let code_start = self.peek_span().start;
        let mut brace_count = 1;
        let mut code_end = code_start;
        while brace_count > 0 && self.peek().is_some() {
            let (t, span) = self.bump()?;
            match t {
                Token::LBrace => brace_count += 1,
                Token::RBrace => brace_count -= 1,
                _ => {}
            }
            if brace_count == 0 {
                code_end = span.start;
            }
        }
        let end = self.peek_span().end;
        let raw_code = self.input[code_start..code_end].trim().to_string();
        Ok((Span { start, end }, raw_code))
    }

    fn parse_enum(&mut self) -> ParseResult<EnumDecl> {
        let start = self.expect(Token::Enum)?.start;
        let ident = self.parse_ident()?;
        let mut tag_type = None;
        if self.peek() == Some(&Token::Colon) {
            self.bump()?;
            tag_type = Some(self.parse_type()?);
        } else if self.peek() == Some(&Token::LParen) {
            self.bump()?;
            tag_type = Some(self.parse_type()?);
            self.expect(Token::RParen)?;
        }
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        let mut methods = Vec::new();
        let mut extra_zig = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            if self.peek() == Some(&Token::ZigEscape) {
                let (_, code) = self.parse_raw_zig_block()?;
                extra_zig.push(code);
                continue;
            }
            let meta_list = self.parse_meta_list()?;
            let mut method_is_pub = false;
            while self.peek() == Some(&Token::Export) || self.peek() == Some(&Token::Private) {
                if self.peek() == Some(&Token::Export) {
                    method_is_pub = true;
                } else {
                    method_is_pub = false;
                }
                self.bump()?;
            }
            if self.peek_next() != Some(&Token::Colon)
                && (self.peek() == Some(&Token::Const)
                    || self.peek() == Some(&Token::Var)
                    || self.peek() == Some(&Token::TypeKw)
                    || self.peek() == Some(&Token::Union)
                    || self.peek() == Some(&Token::Struct)
                    || self.peek() == Some(&Token::Enum)
                    || self.peek() == Some(&Token::Class)
                    || self.peek() == Some(&Token::Interface)
                    || self.peek() == Some(&Token::Trait))
            {
                let start_idx = self.peek_span().start;
                let mut brace_depth: i32 = 0;
                let mut paren_depth: i32 = 0;
                let mut bracket_depth: i32 = 0;
                let mut had_brace = false;
                let mut end_idx = start_idx;
                while let Some(tok) = self.peek() {
                    match tok {
                        Token::LBrace => {
                            brace_depth += 1;
                            had_brace = true;
                        }
                        Token::RBrace => {
                            if brace_depth == 0 {
                                break;
                            }
                            brace_depth -= 1;
                            if brace_depth == 0
                                && paren_depth == 0
                                && bracket_depth == 0
                                && had_brace
                            {
                                let span = self.bump()?.1;
                                end_idx = span.end;
                                while let Some(tok) = self.peek() {
                                    if *tok == Token::Semicolon {
                                        let s_span = self.bump()?.1;
                                        end_idx = s_span.end;
                                        break;
                                    }
                                    if matches!(tok, Token::RBrace) {
                                        break;
                                    }
                                    let s_span = self.bump()?.1;
                                    end_idx = s_span.end;
                                }
                                break;
                            }
                        }
                        Token::LParen => paren_depth += 1,
                        Token::RParen => {
                            if paren_depth > 0 {
                                paren_depth -= 1;
                            }
                        }
                        Token::LBracket => bracket_depth += 1,
                        Token::RBracket => {
                            if bracket_depth > 0 {
                                bracket_depth -= 1;
                            }
                        }
                        Token::Semicolon => {
                            if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                                let span = self.bump()?.1;
                                end_idx = span.end;
                                break;
                            }
                        }
                        _ => {}
                    }
                    let span = self.bump()?.1;
                    end_idx = span.end;
                }
                let raw_code = &self.input[start_idx..end_idx];
                let pub_prefix = if method_is_pub { "pub " } else { "" };
                extra_zig.push(format!("{}{}", pub_prefix, raw_code.trim()));
                continue;
            }
            if matches!(
                self.peek(),
                Some(Token::Fn) | Some(Token::Async) | Some(Token::Static)
            ) {
                let mut method = self.parse_function()?;
                method.meta = meta_list;
                if method_is_pub {
                    method.is_pub = true;
                }
                methods.push(method);
                continue;
            }
            let v_ident = self.parse_ident()?;
            let mut fields = None;
            let mut tuple_type = None;
            let mut from_err = None;
            if self.peek() == Some(&Token::Colon) {
                self.bump()?;
                if self.peek() == Some(&Token::LBrace) {
                    self.bump()?;
                    let mut f_list = Vec::new();
                    while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                        let f_ident = self.parse_ident()?;
                        self.expect(Token::Colon)?;
                        let ty = self.parse_type()?;
                        f_list.push(Field {
                            span: f_ident.span,
                            ident: f_ident,
                            ty,
                            meta: Vec::new(),
                            default: None,
                            is_pub: true,
                            is_static: false,
                        });
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RBrace)?;
                    fields = Some(f_list);
                } else {
                    tuple_type = Some(self.parse_type()?);
                }
            } else if self.peek() == Some(&Token::LParen) {
                self.bump()?;
                tuple_type = Some(self.parse_type()?);
                self.expect(Token::RParen)?;
            } else if self.peek() == Some(&Token::LBrace) {
                self.bump()?;
                let mut f_list = Vec::new();
                while self.peek() != Some(&Token::RBrace) {
                    let f_ident = self.parse_ident()?;
                    self.expect(Token::Colon)?;
                    let ty = self.parse_type()?;
                    f_list.push(Field {
                        span: f_ident.span,
                        ident: f_ident,
                        ty,
                        meta: Vec::new(),
                        default: None,
                        is_pub: true,
                        is_static: false,
                    });
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                self.expect(Token::RBrace)?;
                fields = Some(f_list);
            }
            if self.peek() == Some(&Token::From) {
                self.bump()?;
                from_err = Some(self.parse_ident()?);
            }
            let mut value = None;
            if self.peek() == Some(&Token::Assign) {
                self.bump()?;
                value = Some(self.parse_expr()?);
            }
            variants.push(EnumVariant {
                span: v_ident.span,
                ident: v_ident,
                fields,
                value,
                tuple_type,
                from_err,
            });
            if self.peek() == Some(&Token::Comma) || self.peek() == Some(&Token::Semicolon) {
                self.bump()?;
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(EnumDecl {
            span: Span { start, end },
            ident,
            variants,
            meta: Vec::new(),
            methods,
            tag_type,
            extra_zig,
        })
    }

    fn parse_error_set(&mut self) -> ParseResult<ErrorSetDecl> {
        let start = self.expect(Token::ErrorKw)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            variants.push(self.parse_ident()?);
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        let mut unions = Vec::new();
        while self.peek() == Some(&Token::PipePipe) || self.peek() == Some(&Token::Pipe) {
            self.bump()?;
            let mut u_name = self.parse_ident()?.sym;
            while self.peek() == Some(&Token::Dot) {
                self.bump()?;
                let next = self.parse_ident()?;
                u_name.push('.');
                u_name.push_str(&next.sym);
            }
            unions.push(u_name);
        }
        if self.peek() == Some(&Token::Semicolon) {
            self.bump()?;
        }
        Ok(ErrorSetDecl {
            span: Span { start, end },
            ident,
            variants,
            unions,
        })
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_bin_expr(0)
    }

    fn parse_bin_expr(&mut self, min_prec: u8) -> ParseResult<Expr> {
        let mut left = self.parse_primary_expr()?;
        loop {
            match self.peek() {
                Some(Token::LParen) | Some(Token::Bang) => {
                    let mut is_comptime = false;
                    if self.peek() == Some(&Token::Bang) {
                        if self.peek_next() == Some(&Token::LParen) {
                            self.bump()?;
                            is_comptime = true;
                        } else {
                            break;
                        }
                    }
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    let end = self.expect(Token::RParen)?.end;
                    left = Expr::Call(CallExpr {
                        span: Span {
                            start: left_span(&left).start,
                            end,
                        },
                        callee: Box::new(left),
                        args,
                        is_comptime,
                    });
                }
                Some(Token::QuestionDot) => {
                    self.bump()?;
                    let prop = self.parse_ident()?;
                    if self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        let mut args = Vec::new();
                        while self.peek() != Some(&Token::RParen) {
                            args.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RParen)?.end;
                        let opt_mem = Expr::OptMember(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                        left = Expr::OptCall(CallExpr {
                            span: Span {
                                start: left_span(&opt_mem).start,
                                end,
                            },
                            callee: Box::new(opt_mem),
                            args,
                            is_comptime: false,
                        });
                    } else {
                        left = Expr::OptMember(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                    }
                }
                Some(Token::DotQuestion) => {
                    self.bump()?;
                    left = Expr::Unwrap(Box::new(left));
                    continue;
                }
                Some(Token::Dot) => {
                    if self.peek_next() == Some(&Token::Star) {
                        self.bump()?; // consume .
                        self.bump()?; // consume *
                        left = Expr::Deref(Box::new(left));
                        continue;
                    }
                    if self.peek_next() == Some(&Token::Question) {
                        self.bump()?; // consume .
                        self.bump()?; // consume ?
                        left = Expr::Unwrap(Box::new(left));
                        continue;
                    }
                    if self.peek_next() == Some(&Token::QuestionDot) {
                        self.bump()?; // consume .
                        self.bump()?; // consume ?.
                        left = Expr::Unwrap(Box::new(left));
                        let prop = self.parse_ident()?;
                        left = Expr::Member(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                        continue;
                    }
                    if let Expr::Deref(_) = left {
                        let span = self.peek_span();
                        return Err(ParseError::Generic(
                            "Redundant dereference '.*.' before field access. In CP, pointer and reference fields are accessed directly using '.' (e.g., write 'obj.field' instead of 'obj.*.field').".to_string(),
                            span,
                        ));
                    }
                    self.bump()?;
                    let prop = self.parse_ident()?;
                    if let Expr::Ident(ref obj_id) = left {
                        if obj_id.sym == "Box"
                            && prop.sym == "new"
                            && self.peek() == Some(&Token::LParen)
                        {
                            self.bump()?;
                            let inner = self.parse_expr()?;
                            self.expect(Token::RParen)?;
                            left = Expr::BoxNew(Box::new(inner));
                            continue;
                        }
                        if obj_id.sym == "reflect" {
                            if prop.sym == "type" && self.peek() == Some(&Token::Less) {
                                self.bump()?; // consume '<'
                                let target_ty = self.parse_type()?;
                                self.expect(Token::Greater)?;
                                self.expect(Token::LParen)?;
                                let end_tok = self.expect(Token::RParen)?;
                                left = Expr::ReflectType {
                                    span: Span {
                                        start: obj_id.span.start,
                                        end: end_tok.end,
                                    },
                                    target_ty,
                                };
                                continue;
                            } else if prop.sym == "of" && self.peek() == Some(&Token::LParen) {
                                self.bump()?; // consume '('
                                let inner = self.parse_expr()?;
                                let end_tok = self.expect(Token::RParen)?;
                                left = Expr::ReflectOf {
                                    span: Span {
                                        start: obj_id.span.start,
                                        end: end_tok.end,
                                    },
                                    expr: Box::new(inner),
                                };
                                continue;
                            }
                        }
                    }
                    if (prop.sym == "downcast" || prop.sym == "downcast_mut")
                        && self.peek() == Some(&Token::Less)
                    {
                        self.bump()?;
                        let target_ty = self.parse_ident()?;
                        self.expect(Token::Greater)?;
                        self.expect(Token::LParen)?;
                        self.expect(Token::RParen)?;
                        left = Expr::Downcast {
                            obj: Box::new(left),
                            target_ty,
                            is_mut: prop.sym == "downcast_mut",
                        };
                        continue;
                    }
                    let is_prop_type_like =
                        prop.sym.chars().next().map_or(false, |c| c.is_uppercase());
                    let is_enum_variant_init = if let Expr::Ident(ref obj_id) = left {
                        obj_id
                            .sym
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_uppercase())
                            && is_prop_type_like
                    } else {
                        false
                    };

                    if (is_enum_variant_init || is_prop_type_like)
                        && self.peek() == Some(&Token::LBrace)
                    {
                        self.bump()?;
                        let mut fields = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            if self.peek() == Some(&Token::Dot) {
                                self.bump()?;
                            }
                            let key = self.parse_ident()?;
                            if self.peek() == Some(&Token::Colon)
                                || self.peek() == Some(&Token::Assign)
                            {
                                self.bump()?;
                            } else {
                                self.expect(Token::Colon)?;
                            }
                            let val = self.parse_expr()?;
                            fields.push(EnumInitField { key, val });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RBrace)?.end;
                        let (enum_name, variant_name) = if is_enum_variant_init {
                            let enum_name = match &left {
                                Expr::Ident(id) => id.clone(),
                                _ => Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Unknown".to_string(),
                                },
                            };
                            (enum_name, prop)
                        } else {
                            let left_str =
                                expr_to_member_str(&left).unwrap_or_else(|| "Unknown".to_string());
                            let full_ty_sym = format!("{}.{}", left_str, prop.sym);
                            let full_ty_ident = Ident {
                                span: Span {
                                    start: left_span(&left).start,
                                    end: prop.span.end,
                                },
                                sym: full_ty_sym,
                            };
                            let empty_variant = Ident {
                                span: prop.span,
                                sym: String::new(),
                            };
                            (full_ty_ident, empty_variant)
                        };
                        left = Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end,
                            },
                            enum_name,
                            variant_name,
                            fields: Some(fields),
                            tuple_args: None,
                        });
                        continue;
                    } else if is_enum_variant_init && self.peek() == Some(&Token::LBracket) {
                        self.bump()?;
                        let mut args = Vec::new();
                        while self.peek() != Some(&Token::RBracket) && self.peek().is_some() {
                            args.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RBracket)?.end;
                        let enum_name = match &left {
                            Expr::Ident(id) => id.clone(),
                            _ => Ident {
                                span: Span { start: 0, end: 0 },
                                sym: "Unknown".to_string(),
                            },
                        };
                        left = Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end,
                            },
                            enum_name,
                            variant_name: prop,
                            fields: None,
                            tuple_args: Some(args),
                        });
                        continue;
                    } else if is_enum_variant_init && self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        let mut args = Vec::new();
                        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                            args.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RParen)?.end;
                        let enum_name = match &left {
                            Expr::Ident(id) => id.clone(),
                            _ => Ident {
                                span: Span { start: 0, end: 0 },
                                sym: "Unknown".to_string(),
                            },
                        };
                        left = Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end,
                            },
                            enum_name,
                            variant_name: prop,
                            fields: None,
                            tuple_args: Some(args),
                        });
                        continue;
                    } else {
                        left = Expr::Member(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                    }
                }
                Some(Token::Question) => {
                    self.bump()?;
                    if self.peek() == Some(&Token::Continue) {
                        let span = self.bump()?.1;
                        left = Expr::QuestionFallback {
                            expr: Box::new(left),
                            fallback: Box::new(Expr::Continue(span)),
                            is_return_fallback: false,
                        };
                    } else if self.peek() == Some(&Token::Break) {
                        let span = self.bump()?.1;
                        left = Expr::QuestionFallback {
                            expr: Box::new(left),
                            fallback: Box::new(Expr::Break(span, None)),
                            is_return_fallback: false,
                        };
                    } else if self.peek() == Some(&Token::Return) {
                        let span = self.bump()?.1;
                        let has_arg = if matches!(
                            self.peek(),
                            Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                        ) {
                            false
                        } else {
                            let next_start = self.peek_span().start;
                            if next_start >= span.end
                                && self.input[span.end..next_start].contains('\n')
                            {
                                false
                            } else {
                                true
                            }
                        };
                        let val = if has_arg {
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        left = Expr::QuestionFallback {
                            expr: Box::new(left),
                            fallback: Box::new(Expr::Return(span, val)),
                            is_return_fallback: true,
                        };
                    } else if matches!(
                        self.peek(),
                        Some(Token::IntLiteral(_))
                            | Some(Token::FloatLiteral(_))
                            | Some(Token::StringLiteral(_))
                            | Some(Token::TemplateStringRaw(_))
                            | Some(Token::Ident(_))
                            | Some(Token::Null)
                            | Some(Token::New)
                            | Some(Token::LParen)
                            | Some(Token::LBracket)
                            | Some(Token::Minus)
                    ) {
                        let fallback = self.parse_expr()?;
                        left = Expr::QuestionFallback {
                            expr: Box::new(left),
                            fallback: Box::new(fallback),
                            is_return_fallback: false,
                        };
                    } else {
                        left = Expr::Question(Box::new(left));
                    }
                }
                Some(Token::QuestionQuestion) => {
                    self.bump()?;
                    let right = if self.peek() == Some(&Token::Continue) {
                        let span = self.bump()?.1;
                        Expr::Continue(span)
                    } else if self.peek() == Some(&Token::Break) {
                        let span = self.bump()?.1;
                        Expr::Break(span, None)
                    } else if self.peek() == Some(&Token::Return) {
                        let span = self.bump()?.1;
                        let has_arg = if matches!(
                            self.peek(),
                            Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                        ) {
                            false
                        } else {
                            let next_start = self.peek_span().start;
                            if next_start >= span.end
                                && self.input[span.end..next_start].contains('\n')
                            {
                                false
                            } else {
                                true
                            }
                        };
                        let val = if has_arg {
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        Expr::Return(span, val)
                    } else {
                        self.parse_expr()?
                    };
                    left = Expr::NullCoalesce {
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Catch) => {
                    self.bump()?;
                    let mut is_err_mut = false;
                    let mut err_ident = None;
                    if self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        if self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                        }
                        err_ident = Some(self.parse_ident()?);
                        self.expect(Token::RParen)?;
                    } else if self.peek() == Some(&Token::Pipe) {
                        self.bump()?;
                        if self.peek() == Some(&Token::Star) || self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                        }
                        err_ident = Some(self.parse_ident()?);
                        self.expect(Token::Pipe)?;
                    } else if self.peek() != Some(&Token::LBrace) {
                        let checkpoint = self.lexer.clone();
                        let p1 = self.peeked.clone();
                        let p2 = self.peeked2.clone();
                        let mut is_binding = false;
                        if self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                            is_binding = true;
                        }
                        if matches!(
                            self.peek(),
                            Some(Token::Ident(_)) | Some(Token::ErrorKw) | Some(Token::Auto)
                        ) {
                            let id = self.parse_ident()?;
                            if self.peek() == Some(&Token::FatArrow) {
                                self.bump()?;
                                err_ident = Some(id);
                                is_binding = true;
                            } else if is_err_mut {
                                err_ident = Some(id);
                            }
                        }
                        if !is_binding {
                            self.lexer = checkpoint;
                            self.peeked = p1;
                            self.peeked2 = p2;
                            is_err_mut = false;
                        }
                    }
                    let body = if self.peek() == Some(&Token::LBrace) {
                        Stmt::Block(self.parse_block_stmt()?)
                    } else if self.peek() == Some(&Token::Return) {
                        Stmt::Return(self.parse_return_stmt()?)
                    } else {
                        Stmt::Expr(self.parse_expr()?)
                    };
                    left = Expr::Catch {
                        expr: Box::new(left),
                        err_ident,
                        is_err_mut,
                        body: Box::new(body),
                    };
                }
                Some(Token::LBracket) => {
                    let _start_span = self.bump()?.1;
                    let mut start_idx = None;
                    let mut end_idx = None;
                    let mut sentinel = None;
                    let mut is_slice = false;
                    if self.peek() == Some(&Token::DotDot) {
                        self.bump()?;
                        is_slice = true;
                        if self.peek() != Some(&Token::RBracket)
                            && self.peek() != Some(&Token::Colon)
                        {
                            end_idx = Some(Box::new(self.parse_expr()?));
                        }
                    } else {
                        let idx = self.parse_expr()?;
                        if self.peek() == Some(&Token::DotDot) {
                            self.bump()?;
                            is_slice = true;
                            start_idx = Some(Box::new(idx));
                            if self.peek() != Some(&Token::RBracket)
                                && self.peek() != Some(&Token::Colon)
                            {
                                end_idx = Some(Box::new(self.parse_expr()?));
                            }
                        } else {
                            start_idx = Some(Box::new(idx));
                        }
                    }
                    if is_slice && self.peek() == Some(&Token::Colon) {
                        self.bump()?;
                        sentinel = Some(Box::new(self.parse_expr()?));
                    }
                    let end_span = self.expect(Token::RBracket)?;
                    if is_slice {
                        left = Expr::Slice(SliceExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: end_span.end,
                            },
                            obj: Box::new(left),
                            start: start_idx,
                            end: end_idx,
                            sentinel,
                        });
                    } else {
                        left = Expr::Index {
                            obj: Box::new(left),
                            index: start_idx.unwrap(),
                        };
                    }
                }
                Some(Token::FatArrow) => {
                    if self.in_match_guard || !matches!(left, Expr::Ident(_) | Expr::Array(_)) {
                        break;
                    }
                    self.bump()?;
                    let body = if self.peek() == Some(&Token::LBrace) {
                        ArrowBody::Block(self.parse_block_stmt()?)
                    } else {
                        ArrowBody::Expr(self.parse_expr()?)
                    };
                    let params = match left {
                        Expr::Ident(ref id) => vec![Pattern::Ident(id.clone())],
                        Expr::Array(ref elements) => {
                            let mut pats = Vec::new();
                            for e in elements {
                                if let Expr::Ident(id) = e {
                                    pats.push(Pattern::Ident(id.clone()));
                                } else {
                                    pats.push(Pattern::Wildcard);
                                }
                            }
                            pats
                        }
                        _ => vec![],
                    };
                    left = Expr::Arrow(Box::new(ArrowExpr {
                        span: Span {
                            start: left_span(&left).start,
                            end: self.peek_span().end,
                        },
                        is_async: false,
                        params,
                        body: Box::new(body),
                    }));
                }
                Some(Token::As) => {
                    self.bump()?;
                    let target_ty = self.parse_type()?;
                    left = Expr::Cast {
                        span: Span {
                            start: left_span(&left).start,
                            end: self.peek_span().end,
                        },
                        expr: Box::new(left),
                        target_ty,
                    };
                }
                _ => {
                    if matches!(self.peek(), Some(Token::Ident(id)) if id == "orelse") {
                        self.bump()?;
                        let right = if self.peek() == Some(&Token::LBrace) {
                            let block = self.parse_block_stmt()?;
                            Expr::LabeledBlock { label: None, block }
                        } else if self.peek() == Some(&Token::Continue) {
                            let span = self.bump()?.1;
                            Expr::Continue(span)
                        } else if self.peek() == Some(&Token::Break) {
                            let span = self.bump()?.1;
                            if self.peek() == Some(&Token::Colon) {
                                self.bump()?; // consume ':'
                                let label = self.parse_ident()?;
                                let val = if !matches!(
                                    self.peek(),
                                    Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                                ) {
                                    Some(Box::new(self.parse_expr()?))
                                } else {
                                    None
                                };
                                Expr::LabeledBreak(span, label, val)
                            } else {
                                let has_val = !matches!(
                                    self.peek(),
                                    Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                                );
                                let val = if has_val {
                                    Some(Box::new(self.parse_expr()?))
                                } else {
                                    None
                                };
                                Expr::Break(span, val)
                            }
                        } else if self.peek() == Some(&Token::Return) {
                            let span = self.bump()?.1;
                            let has_arg = if matches!(
                                self.peek(),
                                Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                            ) {
                                false
                            } else {
                                let next_start = self.peek_span().start;
                                !(next_start >= span.end
                                    && self.input[span.end..next_start].contains('\n'))
                            };
                            let val = if has_arg {
                                Some(Box::new(self.parse_expr()?))
                            } else {
                                None
                            };
                            Expr::Return(span, val)
                        } else {
                            self.parse_expr()?
                        };
                        left = Expr::NullCoalesce {
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                        continue;
                    }
                    break;
                }
            }
        }
        while self.peek().is_some() {
            let compound_op = match self.peek() {
                Some(Token::PlusAssign) => Some(BinaryOp::Add),
                Some(Token::MinusAssign) => Some(BinaryOp::Sub),
                Some(Token::StarAssign) => Some(BinaryOp::Mul),
                Some(Token::SlashAssign) => Some(BinaryOp::Div),
                Some(Token::PercentAssign) => Some(BinaryOp::Mod),
                Some(Token::AmpAssign) => Some(BinaryOp::BitAnd),
                Some(Token::PipeAssign) => Some(BinaryOp::BitOr),
                Some(Token::CaretAssign) => Some(BinaryOp::BitXor),
                Some(Token::ShlAssign) => Some(BinaryOp::Shl),
                Some(Token::ShrAssign) => Some(BinaryOp::Shr),
                _ => None,
            };
            if let Some(inner_op) = compound_op {
                if 1 < min_prec {
                    break;
                }
                self.bump()?;
                let right = self.parse_bin_expr(1)?;
                let start = left_span(&left).start;
                let end = right_span(&right).end;
                let desugared_rhs = Expr::Bin(BinExpr {
                    span: Span { start, end },
                    op: inner_op,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                });
                left = Expr::Bin(BinExpr {
                    span: Span { start, end },
                    op: BinaryOp::Assign,
                    left: Box::new(left),
                    right: Box::new(desugared_rhs),
                });
                continue;
            }
            let op = match self.peek_op() {
                Some(op) => op,
                None => break,
            };
            let prec = self.op_precedence(&op);
            if prec < min_prec {
                break;
            }
            self.bump()?;
            let is_pipe = op == BinaryOp::Pipe;
            if is_pipe {
                self.in_pipeline_rhs += 1;
            }
            let right = if op == BinaryOp::LogicalOr && self.peek() == Some(&Token::Continue) {
                let span = self.bump()?.1;
                Expr::Continue(span)
            } else if op == BinaryOp::LogicalOr && self.peek() == Some(&Token::Break) {
                let span = self.bump()?.1;
                Expr::Break(span, None)
            } else if op == BinaryOp::LogicalOr && self.peek() == Some(&Token::Return) {
                let span = self.bump()?.1;
                let has_arg = if matches!(
                    self.peek(),
                    Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                ) {
                    false
                } else {
                    let next_start = self.peek_span().start;
                    if next_start >= span.end && self.input[span.end..next_start].contains('\n') {
                        false
                    } else {
                        true
                    }
                };
                let val = if has_arg {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Expr::Return(span, val)
            } else {
                self.parse_bin_expr(prec + 1)?
            };
            if is_pipe {
                self.in_pipeline_rhs -= 1;
            }
            let start = left_span(&left).start;
            let end = right_span(&right).end;
            left = Expr::Bin(BinExpr {
                span: Span { start, end },
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_postfix_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_primary_expr()?;
        loop {
            match self.peek() {
                Some(Token::LParen) | Some(Token::Bang) => {
                    let next_start = self.peek_span().start;
                    if next_start >= self.prev_token_end
                        && self.input[self.prev_token_end..next_start].contains('\n')
                    {
                        break;
                    }
                    let mut is_comptime = false;
                    if self.peek() == Some(&Token::Bang) {
                        if self.peek_next() == Some(&Token::LParen) {
                            self.bump()?;
                            is_comptime = true;
                        } else {
                            break;
                        }
                    }
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    let end = self.expect(Token::RParen)?.end;
                    left = Expr::Call(CallExpr {
                        span: Span {
                            start: left_span(&left).start,
                            end,
                        },
                        callee: Box::new(left),
                        args,
                        is_comptime,
                    });
                }
                Some(Token::QuestionDot) => {
                    self.bump()?;
                    let prop = self.parse_ident()?;
                    if self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        let mut args = Vec::new();
                        while self.peek() != Some(&Token::RParen) {
                            args.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RParen)?.end;
                        let opt_mem = Expr::OptMember(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                        left = Expr::OptCall(CallExpr {
                            span: Span {
                                start: left_span(&opt_mem).start,
                                end,
                            },
                            callee: Box::new(opt_mem),
                            args,
                            is_comptime: false,
                        });
                    } else {
                        left = Expr::OptMember(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                    }
                }
                Some(Token::DotQuestion) => {
                    self.bump()?;
                    left = Expr::Unwrap(Box::new(left));
                    continue;
                }
                Some(Token::Dot) => {
                    if self.peek_next() == Some(&Token::Star) {
                        self.bump()?; // consume .
                        self.bump()?; // consume *
                        left = Expr::Deref(Box::new(left));
                        continue;
                    }
                    if self.peek_next() == Some(&Token::Question) {
                        self.bump()?; // consume .
                        self.bump()?; // consume ?
                        left = Expr::Unwrap(Box::new(left));
                        continue;
                    }
                    if self.peek_next() == Some(&Token::QuestionDot) {
                        self.bump()?; // consume .
                        self.bump()?; // consume ?.
                        left = Expr::Unwrap(Box::new(left));
                        let prop = self.parse_ident()?;
                        left = Expr::Member(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                        continue;
                    }
                    if let Expr::Deref(_) = left {
                        let span = self.peek_span();
                        return Err(ParseError::Generic(
                            "Redundant dereference '.*.' before field access. In CP, pointer and reference fields are accessed directly using '.' (e.g., write 'obj.field' instead of 'obj.*.field').".to_string(),
                            span,
                        ));
                    }
                    self.bump()?;
                    let prop = self.parse_ident()?;
                    if let Expr::Ident(ref obj_id) = left {
                        if obj_id.sym == "Box"
                            && prop.sym == "new"
                            && self.peek() == Some(&Token::LParen)
                        {
                            self.bump()?;
                            let inner = self.parse_expr()?;
                            self.expect(Token::RParen)?;
                            left = Expr::BoxNew(Box::new(inner));
                            continue;
                        }
                        if obj_id.sym == "reflect" {
                            if prop.sym == "type" && self.peek() == Some(&Token::Less) {
                                self.bump()?; // consume '<'
                                let target_ty = self.parse_type()?;
                                self.expect(Token::Greater)?;
                                self.expect(Token::LParen)?;
                                let end_tok = self.expect(Token::RParen)?;
                                left = Expr::ReflectType {
                                    span: Span {
                                        start: obj_id.span.start,
                                        end: end_tok.end,
                                    },
                                    target_ty,
                                };
                                continue;
                            } else if prop.sym == "of" && self.peek() == Some(&Token::LParen) {
                                self.bump()?; // consume '('
                                let inner = self.parse_expr()?;
                                let end_tok = self.expect(Token::RParen)?;
                                left = Expr::ReflectOf {
                                    span: Span {
                                        start: obj_id.span.start,
                                        end: end_tok.end,
                                    },
                                    expr: Box::new(inner),
                                };
                                continue;
                            }
                        }
                    }
                    if (prop.sym == "downcast" || prop.sym == "downcast_mut")
                        && self.peek() == Some(&Token::Less)
                    {
                        self.bump()?;
                        let target_ty = self.parse_ident()?;
                        self.expect(Token::Greater)?;
                        self.expect(Token::LParen)?;
                        self.expect(Token::RParen)?;
                        left = Expr::Downcast {
                            obj: Box::new(left),
                            target_ty,
                            is_mut: prop.sym == "downcast_mut",
                        };
                        continue;
                    }
                    let is_prop_type_like =
                        prop.sym.chars().next().map_or(false, |c| c.is_uppercase());
                    let is_enum_variant_init = if let Expr::Ident(ref obj_id) = left {
                        obj_id
                            .sym
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_uppercase())
                            && is_prop_type_like
                    } else {
                        false
                    };

                    if (is_enum_variant_init || is_prop_type_like)
                        && self.peek() == Some(&Token::LBrace)
                    {
                        self.bump()?;
                        let mut fields = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            if self.peek() == Some(&Token::Dot) {
                                self.bump()?;
                            }
                            let key = self.parse_ident()?;
                            if self.peek() == Some(&Token::Colon)
                                || self.peek() == Some(&Token::Assign)
                            {
                                self.bump()?;
                            } else {
                                self.expect(Token::Colon)?;
                            }
                            let val = self.parse_expr()?;
                            fields.push(EnumInitField { key, val });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RBrace)?.end;
                        let (enum_name, variant_name) = if is_enum_variant_init {
                            let enum_name = match &left {
                                Expr::Ident(id) => id.clone(),
                                _ => Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Unknown".to_string(),
                                },
                            };
                            (enum_name, prop)
                        } else {
                            let left_str =
                                expr_to_member_str(&left).unwrap_or_else(|| "Unknown".to_string());
                            let full_ty_sym = format!("{}.{}", left_str, prop.sym);
                            let full_ty_ident = Ident {
                                span: Span {
                                    start: left_span(&left).start,
                                    end: prop.span.end,
                                },
                                sym: full_ty_sym,
                            };
                            let empty_variant = Ident {
                                span: prop.span,
                                sym: String::new(),
                            };
                            (full_ty_ident, empty_variant)
                        };
                        left = Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end,
                            },
                            enum_name,
                            variant_name,
                            fields: Some(fields),
                            tuple_args: None,
                        });
                        continue;
                    } else if is_enum_variant_init && self.peek() == Some(&Token::LBracket) {
                        self.bump()?;
                        let mut args = Vec::new();
                        while self.peek() != Some(&Token::RBracket) && self.peek().is_some() {
                            args.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RBracket)?.end;
                        let enum_name = match &left {
                            Expr::Ident(id) => id.clone(),
                            _ => Ident {
                                span: Span { start: 0, end: 0 },
                                sym: "Unknown".to_string(),
                            },
                        };
                        left = Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end,
                            },
                            enum_name,
                            variant_name: prop,
                            fields: None,
                            tuple_args: Some(args),
                        });
                        continue;
                    } else if is_enum_variant_init && self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        let mut args = Vec::new();
                        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                            args.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RParen)?.end;
                        let enum_name = match &left {
                            Expr::Ident(id) => id.clone(),
                            _ => Ident {
                                span: Span { start: 0, end: 0 },
                                sym: "Unknown".to_string(),
                            },
                        };
                        left = Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end,
                            },
                            enum_name,
                            variant_name: prop,
                            fields: None,
                            tuple_args: Some(args),
                        });
                        continue;
                    } else {
                        left = Expr::Member(MemberExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: prop.span.end,
                            },
                            obj: Box::new(left),
                            prop,
                        });
                    }
                }
                Some(Token::LBracket) => {
                    let _start_span = self.bump()?.1;
                    let mut start_idx = None;
                    let mut end_idx = None;
                    let mut sentinel = None;
                    let mut is_slice = false;
                    if self.peek() == Some(&Token::DotDot) {
                        self.bump()?;
                        is_slice = true;
                        if self.peek() != Some(&Token::RBracket)
                            && self.peek() != Some(&Token::Colon)
                        {
                            end_idx = Some(Box::new(self.parse_expr()?));
                        }
                    } else {
                        let idx = self.parse_expr()?;
                        if self.peek() == Some(&Token::DotDot) {
                            self.bump()?;
                            is_slice = true;
                            start_idx = Some(Box::new(idx));
                            if self.peek() != Some(&Token::RBracket)
                                && self.peek() != Some(&Token::Colon)
                            {
                                end_idx = Some(Box::new(self.parse_expr()?));
                            }
                        } else {
                            start_idx = Some(Box::new(idx));
                        }
                    }
                    if is_slice && self.peek() == Some(&Token::Colon) {
                        self.bump()?;
                        sentinel = Some(Box::new(self.parse_expr()?));
                    }
                    let end_span = self.expect(Token::RBracket)?;
                    if is_slice {
                        left = Expr::Slice(SliceExpr {
                            span: Span {
                                start: left_span(&left).start,
                                end: end_span.end,
                            },
                            obj: Box::new(left),
                            start: start_idx,
                            end: end_idx,
                            sentinel,
                        });
                    } else {
                        left = Expr::Index {
                            obj: Box::new(left),
                            index: start_idx.unwrap(),
                        };
                    }
                }
                _ => {
                    if matches!(self.peek(), Some(Token::Ident(id)) if id == "orelse") {
                        self.bump()?;
                        let right = if self.peek() == Some(&Token::LBrace) {
                            let block = self.parse_block_stmt()?;
                            Expr::LabeledBlock { label: None, block }
                        } else if self.peek() == Some(&Token::Continue) {
                            let span = self.bump()?.1;
                            Expr::Continue(span)
                        } else if self.peek() == Some(&Token::Break) {
                            let span = self.bump()?.1;
                            if self.peek() == Some(&Token::Colon) {
                                self.bump()?; // consume ':'
                                let label = self.parse_ident()?;
                                let val = if !matches!(
                                    self.peek(),
                                    Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                                ) {
                                    Some(Box::new(self.parse_expr()?))
                                } else {
                                    None
                                };
                                Expr::LabeledBreak(span, label, val)
                            } else {
                                let has_val = !matches!(
                                    self.peek(),
                                    Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                                );
                                let val = if has_val {
                                    Some(Box::new(self.parse_expr()?))
                                } else {
                                    None
                                };
                                Expr::Break(span, val)
                            }
                        } else if self.peek() == Some(&Token::Return) {
                            let span = self.bump()?.1;
                            let has_arg = if matches!(
                                self.peek(),
                                Some(Token::Semicolon | Token::RBrace | Token::RParen) | None
                            ) {
                                false
                            } else {
                                let next_start = self.peek_span().start;
                                !(next_start >= span.end
                                    && self.input[span.end..next_start].contains('\n'))
                            };
                            let val = if has_arg {
                                Some(Box::new(self.parse_expr()?))
                            } else {
                                None
                            };
                            Expr::Return(span, val)
                        } else {
                            self.parse_expr()?
                        };
                        left = Expr::NullCoalesce {
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                        continue;
                    }
                    break;
                }
            }
        }
        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> ParseResult<Expr> {
        let (token, span) = self.bump()?;
        match token {
            Token::Try => {
                let inner = self.parse_postfix_expr()?;
                Ok(Expr::Question(Box::new(inner)))
            }
            Token::Undefined => Ok(Expr::Lit(Lit::Undefined)),
            Token::Null => Ok(Expr::Lit(Lit::Null)),
            Token::Minus => {
                if let Some(Token::IntLiteral(_)) = self.peek() {
                    if let (Token::IntLiteral(v), _) = self.bump()? {
                        return Ok(Expr::Lit(Lit::Int(-v)));
                    }
                }
                if let Some(Token::FloatLiteral(_)) = self.peek() {
                    if let (Token::FloatLiteral(f), _) = self.bump()? {
                        return Ok(Expr::Lit(Lit::Float(-f)));
                    }
                }
                let inner = self.parse_primary_expr()?;
                Ok(Expr::Bin(BinExpr {
                    span,
                    op: BinaryOp::Sub,
                    left: Box::new(Expr::Lit(Lit::Int(0))),
                    right: Box::new(inner),
                }))
            }
            Token::Dot => {
                if self.peek() == Some(&Token::LBrace) {
                    self.bump()?;
                    let fields = self.parse_object_fields(true)?;
                    return Ok(Expr::Object(fields));
                }
                if self.in_pipeline_rhs > 0 {
                    if matches!(
                        self.peek(),
                        Some(Token::Ident(_)) | Some(Token::ErrorKw) | Some(Token::Auto)
                    ) {
                        let prop = self.parse_ident()?;
                        if prop.sym.chars().next().map_or(false, |c| c.is_uppercase())
                            && self.peek() == Some(&Token::LBrace)
                        {
                            self.bump()?;
                            let mut fields = Vec::new();
                            while self.peek() != Some(&Token::RBrace) {
                                let key = self.parse_ident()?;
                                self.expect(Token::Colon)?;
                                let val = self.parse_expr()?;
                                fields.push(EnumInitField { key, val });
                                if self.peek() == Some(&Token::Comma) {
                                    self.bump()?;
                                }
                            }
                            let end = self.expect(Token::RBrace)?.end;
                            Ok(Expr::EnumInit(EnumInitExpr {
                                span: Span {
                                    start: span.start,
                                    end,
                                },
                                enum_name: Ident {
                                    span,
                                    sym: String::new(),
                                },
                                variant_name: prop,
                                fields: Some(fields),
                                tuple_args: None,
                            }))
                        } else {
                            Ok(Expr::Member(MemberExpr {
                                span: Span {
                                    start: span.start,
                                    end: prop.span.end,
                                },
                                obj: Box::new(Expr::PipePlaceholder(span)),
                                prop,
                            }))
                        }
                    } else {
                        Ok(Expr::PipePlaceholder(span))
                    }
                } else {
                    if matches!(
                        self.peek(),
                        Some(Token::Ident(_)) | Some(Token::ErrorKw) | Some(Token::Auto)
                    ) {
                        let variant_name = self.parse_ident()?;
                        let mut fields = None;
                        let mut tuple_args = None;
                        let mut end = variant_name.span.end;
                        if self.peek() == Some(&Token::LBrace) {
                            self.bump()?;
                            let mut f_list = Vec::new();
                            while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                                if self.peek() == Some(&Token::Dot) {
                                    self.bump()?;
                                }
                                let key = self.parse_ident()?;
                                if self.peek() == Some(&Token::Colon)
                                    || self.peek() == Some(&Token::Assign)
                                {
                                    self.bump()?;
                                } else {
                                    self.expect(Token::Colon)?;
                                }
                                let val = self.parse_expr()?;
                                f_list.push(EnumInitField { key, val });
                                if self.peek() == Some(&Token::Comma) {
                                    self.bump()?;
                                }
                            }
                            end = self.expect(Token::RBrace)?.end;
                            fields = Some(f_list);
                        } else if self.peek() == Some(&Token::LBracket) {
                            self.bump()?;
                            let mut args = Vec::new();
                            while self.peek() != Some(&Token::RBracket) && self.peek().is_some() {
                                args.push(self.parse_expr()?);
                                if self.peek() == Some(&Token::Comma) {
                                    self.bump()?;
                                }
                            }
                            end = self.expect(Token::RBracket)?.end;
                            tuple_args = Some(args);
                        } else if self.peek() == Some(&Token::LParen) {
                            self.bump()?;
                            let mut args = Vec::new();
                            while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                                args.push(self.parse_expr()?);
                                if self.peek() == Some(&Token::Comma) {
                                    self.bump()?;
                                }
                            }
                            end = self.expect(Token::RParen)?.end;
                            tuple_args = Some(args);
                        }
                        Ok(Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: span.start,
                                end,
                            },
                            enum_name: Ident {
                                span,
                                sym: String::new(),
                            },
                            variant_name,
                            fields,
                            tuple_args,
                        }))
                    } else {
                        return Err(ParseError::Generic(
                            "Pipeline placeholder '.' can only be used on the right-hand side of a pipeline ('|>') operator".to_string(),
                            span,
                        ));
                    }
                }
            }
            Token::Meta => Ok(Expr::Ident(Ident {
                span,
                sym: "meta".to_string(),
            })),
            Token::Super => Ok(Expr::Ident(Ident {
                span,
                sym: "super".to_string(),
            })),
            Token::String => Ok(Expr::Ident(Ident {
                span,
                sym: "string".to_string(),
            })),
            Token::ErrorKw => {
                let text = &self.input[span.start..span.end];
                if self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    let first = self.parse_expr()?;
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                        let second = self.parse_expr()?;
                        self.expect(Token::RParen)?;
                        return Ok(Expr::ErrWrap(Box::new(Expr::Call(CallExpr {
                            span: Span {
                                start: span.start,
                                end: self.peek_span().start,
                            },
                            callee: Box::new(Expr::Member(MemberExpr {
                                span,
                                obj: Box::new(Expr::Ident(Ident {
                                    span,
                                    sym: "Err".to_string(),
                                })),
                                prop: Ident {
                                    span,
                                    sym: "init".to_string(),
                                },
                            })),
                            args: vec![first, second],
                            is_comptime: false,
                        }))));
                    }
                    self.expect(Token::RParen)?;
                    return Ok(Expr::ErrWrap(Box::new(first)));
                }
                if (text == "packed" || text == "extern")
                    && (self.peek() == Some(&Token::Struct)
                        || self.peek() == Some(&Token::Union)
                        || self.peek() == Some(&Token::Enum))
                {
                    let start_idx = span.start;
                    let mut brace_depth: i32 = 0;
                    let mut paren_depth: i32 = 0;
                    let mut bracket_depth: i32 = 0;
                    let mut had_brace = false;
                    let mut end_idx = span.end;
                    while let Some(tok) = self.peek() {
                        match tok {
                            Token::LBrace => {
                                brace_depth += 1;
                                had_brace = true;
                            }
                            Token::RBrace => {
                                if brace_depth == 0 {
                                    break;
                                }
                                brace_depth -= 1;
                                if brace_depth == 0
                                    && paren_depth == 0
                                    && bracket_depth == 0
                                    && had_brace
                                {
                                    let s = self.bump()?.1;
                                    end_idx = s.end;
                                    break;
                                }
                            }
                            Token::LParen => paren_depth += 1,
                            Token::RParen => {
                                if paren_depth > 0 {
                                    paren_depth -= 1;
                                }
                            }
                            Token::LBracket => bracket_depth += 1,
                            Token::RBracket => {
                                if bracket_depth > 0 {
                                    bracket_depth -= 1;
                                }
                            }
                            _ => {}
                        }
                        let s = self.bump()?.1;
                        end_idx = s.end;
                    }
                    let raw_code = self.input[start_idx..end_idx].to_string();
                    return Ok(Expr::Zig(ZigEscapeExpr {
                        span: Span {
                            start: start_idx,
                            end: end_idx,
                        },
                        code: raw_code,
                    }));
                }
                let is_error_literal = (text == "err" || text == "error")
                    && self.peek() == Some(&Token::Dot)
                    && matches!(self.peek_next(), Some(Token::Ident(id)) if id.chars().next().map_or(false, |c| c.is_ascii_uppercase()));
                if is_error_literal {
                    Ok(Expr::Ident(Ident {
                        span,
                        sym: "error".to_string(),
                    }))
                } else {
                    let sym = text.to_string();
                    if sym.chars().next().map_or(false, |c| c.is_uppercase())
                        && self.peek() == Some(&Token::LBrace)
                    {
                        self.bump()?;
                        let mut fields = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            if self.peek() == Some(&Token::Dot) {
                                self.bump()?;
                            }
                            let key = self.parse_ident()?;
                            if self.peek() == Some(&Token::Colon)
                                || self.peek() == Some(&Token::Assign)
                            {
                                self.bump()?;
                            } else {
                                self.expect(Token::Colon)?;
                            }
                            let val = self.parse_expr()?;
                            fields.push(EnumInitField { key, val });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RBrace)?.end;
                        return Ok(Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: span.start,
                                end,
                            },
                            enum_name: Ident { span, sym },
                            variant_name: Ident {
                                span: Span { start: 0, end: 0 },
                                sym: String::new(),
                            },
                            fields: Some(fields),
                            tuple_args: None,
                        }));
                    }
                    Ok(Expr::Ident(Ident { span, sym }))
                }
            }
            Token::Unsafe => {
                let block = self.parse_block_stmt()?;
                Ok(Expr::Unsafe(block))
            }
            Token::Arena => {
                let block = self.parse_block_stmt()?;
                Ok(Expr::Arena(block))
            }
            Token::Question => {
                let start_idx = span.start;
                let mut brace_depth: i32 = 0;
                let mut paren_depth: i32 = 0;
                let mut bracket_depth: i32 = 0;
                let mut end_idx = span.end;
                while let Some(tok) = self.peek() {
                    match tok {
                        Token::Semicolon | Token::Comma => {
                            if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                                break;
                            }
                        }
                        Token::LBrace => brace_depth += 1,
                        Token::RBrace => {
                            if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                                break;
                            }
                            brace_depth -= 1;
                        }
                        Token::LParen => paren_depth += 1,
                        Token::RParen => {
                            if paren_depth > 0 {
                                paren_depth -= 1;
                            }
                        }
                        Token::LBracket => bracket_depth += 1,
                        Token::RBracket => {
                            if bracket_depth > 0 {
                                bracket_depth -= 1;
                            }
                        }
                        _ => {}
                    }
                    let s = self.bump()?.1;
                    end_idx = s.end;
                }
                let raw_code = self.input[start_idx..end_idx].to_string();
                Ok(Expr::Zig(ZigEscapeExpr {
                    span: Span {
                        start: start_idx,
                        end: end_idx,
                    },
                    code: raw_code,
                }))
            }
            Token::Struct | Token::Union | Token::Enum => {
                let start_idx = span.start;
                let mut brace_depth: i32 = 0;
                let mut paren_depth: i32 = 0;
                let mut bracket_depth: i32 = 0;
                let mut had_brace = false;
                let mut end_idx = span.end;
                while let Some(tok) = self.peek() {
                    match tok {
                        Token::LBrace => {
                            brace_depth += 1;
                            had_brace = true;
                        }
                        Token::RBrace => {
                            if brace_depth == 0 {
                                break;
                            }
                            brace_depth -= 1;
                            if brace_depth == 0
                                && paren_depth == 0
                                && bracket_depth == 0
                                && had_brace
                            {
                                let s = self.bump()?.1;
                                end_idx = s.end;
                                break;
                            }
                        }
                        Token::LParen => paren_depth += 1,
                        Token::RParen => {
                            if paren_depth > 0 {
                                paren_depth -= 1;
                            }
                        }
                        Token::LBracket => bracket_depth += 1,
                        Token::RBracket => {
                            if bracket_depth > 0 {
                                bracket_depth -= 1;
                            }
                        }
                        _ => {}
                    }
                    let s = self.bump()?.1;
                    end_idx = s.end;
                }
                let raw_code = self.input[start_idx..end_idx].to_string();
                Ok(Expr::Zig(ZigEscapeExpr {
                    span: Span {
                        start: start_idx,
                        end: end_idx,
                    },
                    code: raw_code,
                }))
            }
            Token::Star => {
                if self.in_pipeline_rhs > 0
                    && matches!(
                        self.peek(),
                        Some(
                            Token::Comma
                                | Token::RParen
                                | Token::RBracket
                                | Token::RBrace
                                | Token::Semicolon
                                | Token::Pipe
                        ) | None
                    )
                {
                    return Ok(Expr::PipePlaceholder(span));
                }
                if self.peek() == Some(&Token::Const) || self.peek() == Some(&Token::Fn) {
                    let start_idx = span.start;
                    let mut brace_depth: i32 = 0;
                    let mut paren_depth: i32 = 0;
                    let mut bracket_depth: i32 = 0;
                    let mut end_idx = span.end;
                    while let Some(tok) = self.peek() {
                        match tok {
                            Token::Semicolon | Token::Comma => {
                                if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                                    break;
                                }
                            }
                            Token::LBrace => brace_depth += 1,
                            Token::RBrace => {
                                if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                                    break;
                                }
                                brace_depth -= 1;
                            }
                            Token::LParen => paren_depth += 1,
                            Token::RParen => {
                                if paren_depth > 0 {
                                    paren_depth -= 1;
                                }
                            }
                            Token::LBracket => bracket_depth += 1,
                            Token::RBracket => {
                                if bracket_depth > 0 {
                                    bracket_depth -= 1;
                                }
                            }
                            _ => {}
                        }
                        let s = self.bump()?.1;
                        end_idx = s.end;
                    }
                    let raw_code = self.input[start_idx..end_idx].to_string();
                    return Ok(Expr::Zig(ZigEscapeExpr {
                        span: Span {
                            start: start_idx,
                            end: end_idx,
                        },
                        code: raw_code,
                    }));
                }
                let mut depth = 1;
                while self.peek() == Some(&Token::Star) {
                    self.bump()?;
                    depth += 1;
                }
                let inner = self.parse_postfix_expr()?;
                if self.peek() == Some(&Token::Bang) {
                    let mut end = self.bump()?.1.end;
                    let mut is_optional = false;
                    if self.peek() == Some(&Token::Question) {
                        end = self.bump()?.1.end;
                        is_optional = true;
                    }
                    let mut target_ty = Type::RawPtr {
                        inner: Box::new(Type::Any),
                        depth,
                    };
                    if is_optional {
                        target_ty = Type::Optional(Box::new(target_ty));
                    }
                    return Ok(Expr::Cast {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        expr: Box::new(Expr::Borrow(Box::new(inner), true)),
                        target_ty,
                    });
                } else if self.peek() == Some(&Token::Question)
                    && self.peek_next() == Some(&Token::Bang)
                {
                    self.bump()?;
                    let end = self.bump()?.1.end;
                    let target_ty = Type::Optional(Box::new(Type::RawPtr {
                        inner: Box::new(Type::Any),
                        depth,
                    }));
                    return Ok(Expr::Cast {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        expr: Box::new(Expr::Borrow(Box::new(inner), true)),
                        target_ty,
                    });
                }
                Ok(Expr::Borrow(Box::new(inner), true))
            }
            Token::Bang => {
                let inner = self.parse_postfix_expr()?;
                Ok(Expr::Builtin(BuiltinCall {
                    span,
                    name: "@not".to_string(),
                    args: vec![inner],
                }))
            }
            Token::Tilde => {
                let inner = self.parse_postfix_expr()?;
                Ok(Expr::BitNot(Box::new(inner)))
            }
            Token::U8
            | Token::I8
            | Token::U16
            | Token::I16
            | Token::U32
            | Token::I32
            | Token::U64
            | Token::I64
            | Token::U128
            | Token::I128
            | Token::USize
            | Token::ISize
            | Token::F32
            | Token::F64
            | Token::Bool
            | Token::Void
            | Token::CStr
            | Token::Any => {
                let sym = match token {
                    Token::U8 => "u8",
                    Token::I8 => "i8",
                    Token::U16 => "u16",
                    Token::I16 => "i16",
                    Token::U32 => "u32",
                    Token::I32 => "i32",
                    Token::U64 => "u64",
                    Token::I64 => "i64",
                    Token::U128 => "u128",
                    Token::I128 => "i128",
                    Token::USize => "usize",
                    Token::ISize => "isize",
                    Token::F32 => "f32",
                    Token::F64 => "f64",
                    Token::Bool => "bool",
                    Token::Void => "void",
                    Token::CStr => "cstr",
                    Token::Any => "anytype",
                    _ => unreachable!(),
                }
                .to_string();
                Ok(Expr::Ident(Ident { span, sym }))
            }
            Token::Ident(mut sym) => {
                if sym == "asm" {
                    let mut is_volatile = true;
                    if matches!(self.peek(), Some(Token::Ident(id)) if id == "volatile") {
                        self.bump()?;
                        is_volatile = true;
                    }
                    if self.peek() == Some(&Token::LParen) {
                        self.bump()?; // consume '('
                        let inner_start = self.peek_span().start;
                        let mut paren_depth = 1;
                        let mut end = inner_start;
                        while paren_depth > 0 && self.peek().is_some() {
                            let (tok, t_span) = self.bump()?;
                            match tok {
                                Token::LParen => paren_depth += 1,
                                Token::RParen => {
                                    paren_depth -= 1;
                                    if paren_depth == 0 {
                                        end = t_span.start;
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        let content = self.input[inner_start..end].trim().to_string();
                        let asm_end = self.peek_span().start;
                        return Ok(Expr::InlineAsm {
                            span: Span {
                                start: span.start,
                                end: asm_end,
                            },
                            is_volatile,
                            content,
                        });
                    }
                }
                if ((sym == "packed" || sym == "extern")
                    && (self.peek() == Some(&Token::Struct)
                        || self.peek() == Some(&Token::Union)
                        || self.peek() == Some(&Token::Enum)
                        || self.peek() == Some(&Token::LParen)))
                    || (sym == "opaque"
                        && (self.peek() == Some(&Token::LBrace)
                            || self.peek() == Some(&Token::Struct)
                            || self.peek() == Some(&Token::Union)))
                {
                    let start_idx = span.start;
                    let mut brace_depth: i32 = 0;
                    let mut paren_depth: i32 = 0;
                    let mut bracket_depth: i32 = 0;
                    let mut had_brace = false;
                    let mut end_idx = span.end;
                    while let Some(tok) = self.peek() {
                        match tok {
                            Token::LBrace => {
                                brace_depth += 1;
                                had_brace = true;
                            }
                            Token::RBrace => {
                                if brace_depth == 0 {
                                    break;
                                }
                                brace_depth -= 1;
                                if brace_depth == 0
                                    && paren_depth == 0
                                    && bracket_depth == 0
                                    && had_brace
                                {
                                    let s = self.bump()?.1;
                                    end_idx = s.end;
                                    break;
                                }
                            }
                            Token::LParen => paren_depth += 1,
                            Token::RParen => {
                                if paren_depth > 0 {
                                    paren_depth -= 1;
                                }
                            }
                            Token::LBracket => bracket_depth += 1,
                            Token::RBracket => {
                                if bracket_depth > 0 {
                                    bracket_depth -= 1;
                                }
                            }
                            _ => {}
                        }
                        let s = self.bump()?.1;
                        end_idx = s.end;
                    }
                    let raw_code = self.input[start_idx..end_idx].to_string();
                    return Ok(Expr::Zig(ZigEscapeExpr {
                        span: Span {
                            start: start_idx,
                            end: end_idx,
                        },
                        code: raw_code,
                    }));
                }
                if self.peek() == Some(&Token::Colon) && self.peek_next() == Some(&Token::LBrace) {
                    self.bump()?; // consume ':'
                    let block = self.parse_block_stmt()?;
                    return Ok(Expr::LabeledBlock {
                        label: Some(Ident { span, sym }),
                        block,
                    });
                }
                if (sym == "Err" || sym == "err") && self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    let inner = self.parse_expr()?;
                    self.expect(Token::RParen)?;
                    return Ok(Expr::ErrWrap(Box::new(inner)));
                }
                if sym.chars().next().map_or(false, |c| c.is_uppercase()) {
                    if self.peek() == Some(&Token::Less) {
                        let mut probe = self.clone_lookahead();
                        if probe.bump().is_ok() {
                            let mut depth = 1;
                            let mut found_greater = false;
                            while let Ok((tok, _)) = probe.bump() {
                                if tok == Token::Less {
                                    depth += 1;
                                } else if tok == Token::Greater {
                                    depth -= 1;
                                    if depth == 0 {
                                        found_greater = true;
                                        break;
                                    }
                                } else if tok == Token::Semicolon || tok == Token::RBrace {
                                    break;
                                }
                            }
                            if found_greater
                                && (probe.peek() == Some(&Token::LBrace)
                                    || probe.peek() == Some(&Token::Dot))
                            {
                                self.bump()?;
                                sym.push('<');
                                let mut angle_depth = 1;
                                while angle_depth > 0 && self.peek().is_some() {
                                    let (tok, t_span) = self.bump()?;
                                    match tok {
                                        Token::Less => angle_depth += 1,
                                        Token::Greater => angle_depth -= 1,
                                        _ => {}
                                    }
                                    if angle_depth == 0 {
                                        sym.push('>');
                                        break;
                                    }
                                    sym.push_str(&self.input[t_span.start..t_span.end]);
                                }
                            }
                        }
                    }
                    if self.peek() == Some(&Token::LBrace) {
                        self.bump()?;
                        let mut fields = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            if self.peek() == Some(&Token::Dot) {
                                self.bump()?;
                            }
                            let key = self.parse_ident()?;
                            if self.peek() == Some(&Token::Colon)
                                || self.peek() == Some(&Token::Assign)
                            {
                                self.bump()?;
                            } else {
                                self.expect(Token::Colon)?;
                            }
                            let val = self.parse_expr()?;
                            fields.push(EnumInitField { key, val });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        let end = self.expect(Token::RBrace)?.end;
                        return Ok(Expr::EnumInit(EnumInitExpr {
                            span: Span {
                                start: span.start,
                                end,
                            },
                            enum_name: Ident { span, sym },
                            variant_name: Ident {
                                span,
                                sym: String::new(),
                            },
                            fields: Some(fields),
                            tuple_args: None,
                        }));
                    }
                }
                if sym.starts_with('@') {
                    if sym == "@is_beam" {
                        return Ok(Expr::Ident(Ident { span, sym }));
                    }
                    let name = sym.clone();
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                        args.push(self.parse_bin_expr(0)?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expr::Builtin(BuiltinCall {
                        span: Span {
                            start: span.start,
                            end: self.peek_span().end,
                        },
                        name,
                        args,
                    }))
                } else {
                    Ok(Expr::Ident(Ident { span, sym }))
                }
            }
            Token::At => {
                if self.peek() == Some(&Token::LBrace) {
                    let block = self.parse_block_stmt()?;
                    return Ok(Expr::Comptime(block));
                }
                if self.peek() == Some(&Token::If) {
                    self.bump()?; // consume 'if'
                    self.expect(Token::LParen)?;
                    let test = self.parse_expr()?;
                    self.expect(Token::RParen)?;
                    if self.peek() == Some(&Token::FatArrow) {
                        self.bump()?;
                    }
                    let cons = self.parse_expr()?;
                    self.expect(Token::Else)?;
                    let alt = self.parse_expr()?;
                    let end = right_span(&alt).end;
                    return Ok(Expr::If {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        test: Box::new(test),
                        cons: Box::new(cons),
                        alt: Some(Box::new(alt)),
                        is_comptime: true,
                    });
                }
                let ident = self.parse_ident()?;
                let name = format!("@{}", ident.sym);
                if name == "@is_beam" {
                    return Ok(Expr::Ident(Ident {
                        span: Span {
                            start: span.start,
                            end: ident.span.end,
                        },
                        sym: name,
                    }));
                }
                if name == "@as" {
                    self.expect(Token::LParen)?;
                    let target_ty = self.parse_type()?;
                    self.expect(Token::Comma)?;
                    let expr = self.parse_expr()?;
                    let end = self.expect(Token::RParen)?.end;
                    return Ok(Expr::Cast {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        expr: Box::new(expr),
                        target_ty,
                    });
                }
                if name == "@asm" {
                    let mut is_volatile = true;
                    if matches!(self.peek(), Some(Token::Ident(id)) if id == "volatile") {
                        self.bump()?;
                        is_volatile = true;
                    }
                    self.expect(Token::LParen)?;
                    let inner_start = self.peek_span().start;
                    let mut paren_depth = 1;
                    let mut end = inner_start;
                    while paren_depth > 0 && self.peek().is_some() {
                        let (tok, t_span) = self.bump()?;
                        match tok {
                            Token::LParen => paren_depth += 1,
                            Token::RParen => {
                                paren_depth -= 1;
                                if paren_depth == 0 {
                                    end = t_span.start;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    let content = self.input[inner_start..end].trim().to_string();
                    let asm_end = self.peek_span().start;
                    return Ok(Expr::InlineAsm {
                        span: Span {
                            start: span.start,
                            end: asm_end,
                        },
                        is_volatile,
                        content,
                    });
                }
                if name == "@map" || name == "@comptimeMap" || name == "@comptime_map" {
                    self.expect(Token::LParen)?;
                    let (target_ty, entries) = if self.peek() == Some(&Token::LBrace) {
                        self.bump()?; // consume '{'
                        let fields = self.parse_object_fields(false)?;
                        (None, fields)
                    } else {
                        let mut ty = self.parse_type()?;
                        self.expect(Token::Comma)?;
                        if self.peek() != Some(&Token::LBrace) {
                            ty = self.parse_type()?;
                            self.expect(Token::Comma)?;
                        }
                        self.expect(Token::LBrace)?;
                        let fields = self.parse_object_fields(false)?;
                        (Some(ty), fields)
                    };
                    let end = self.expect(Token::RParen)?.end;
                    return Ok(Expr::ComptimeMap {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        target_ty,
                        entries,
                    });
                }
                self.expect(Token::LParen)?;
                let mut args = Vec::new();
                while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                    args.push(self.parse_bin_expr(0)?);
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                let end = self.expect(Token::RParen)?.end;
                let _is_known_builtin = matches!(
                    name.as_str(),
                    "@print"
                        | "@print_err"
                        | "@not"
                        | "@spawn"
                        | "@worker_spawn"
                        | "@net_accept_async"
                        | "@net_read_async"
                        | "@net_write_async"
                        | "@net_listen"
                        | "@net_close"
                        | "@net_connect"
                        | "@net_send"
                        | "@yield_to_beam"
                        | "@io_write_stdout"
                        | "@io_write_stderr"
                        | "@io_read_stdin_line"
                        | "@http_fetch"
                        | "@http_serve"
                        | "@fs_read_async"
                        | "@fs_write_file"
                        | "@fs_mkdir"
                        | "@fs_stat_size"
                        | "@fs_is_dir"
                        | "@fs_remove"
                        | "@fs_rmdir"
                        | "@fs_rename"
                        | "@fs_copy_file"
                        | "@fs_exists"
                        | "@fs_read_file"
                        | "@crypto_uuid"
                        | "@crypto_hash"
                        | "@hash"
                        | "@self"
                        | "@sender"
                        | "@reply"
                        | "@send"
                        | "@receive"
                        | "@task_join"
                        | "@create_pending_task"
                        | "@complete_task"
                        | "@alloc"
                        | "@free"
                        | "@get_arena"
                        | "@shared_tensor_init"
                        | "@release"
                        | "@json_parse"
                        | "@json_stringify"
                        | "@compileError"
                        | "@compileLog"
                        | "@typeOf"
                        | "@typeName"
                        | "@fieldsOf"
                        | "@field"
                        | "@hasField"
                        | "@sizeOf"
                        | "@alignOf"
                        | "@as"
                        | "@test"
                        | "@expect_eq"
                );
                Ok(Expr::Builtin(BuiltinCall {
                    span: Span {
                        start: span.start,
                        end,
                    },
                    name,
                    args,
                }))
            }
            Token::Match | Token::Switch => {
                let match_expr = self.parse_match_expr()?;
                Ok(Expr::Match(Box::new(match_expr)))
            }
            Token::Fn => {
                let params = self.parse_params()?;
                let mut _return_type = None;
                if self.peek() == Some(&Token::Bang) {
                    self.bump()?;
                    if self.is_error_type_rhs_start() {
                        let err = self.parse_error_type_rhs()?;
                        _return_type = Some(Type::ResultType {
                            ok: Box::new(Type::Void),
                            err,
                        });
                    } else {
                        _return_type = Some(Type::ErrorUnion(Box::new(Type::Void)));
                    }
                } else if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
                    self.bump()?;
                    _return_type = Some(self.parse_type()?);
                } else if self.peek().map(Self::is_type_start).unwrap_or(false)
                    && self.peek() != Some(&Token::LBrace)
                    && self.peek() != Some(&Token::Semicolon)
                {
                    _return_type = Some(self.parse_type()?);
                }

                if self.peek() != Some(&Token::LBrace) {
                    let end_idx = self.peek_span().start;
                    let raw_code = self.input[span.start..end_idx].trim().to_string();
                    let clean_code = raw_code.replace("): ", ") ").replace(") : ", ") ");
                    return Ok(Expr::Zig(ZigEscapeExpr {
                        span: Span {
                            start: span.start,
                            end: end_idx,
                        },
                        code: clean_code,
                    }));
                }
                let body = ArrowBody::Block(self.parse_block_stmt()?);
                let pat_params = params
                    .into_iter()
                    .map(|p| Pattern::Ident(p.ident))
                    .collect();
                Ok(Expr::Arrow(Box::new(ArrowExpr {
                    span: Span {
                        start: span.start,
                        end: self.peek_span().end,
                    },
                    is_async: false,
                    params: pat_params,
                    body: Box::new(body),
                })))
            }
            Token::LParen => {
                if self.peek() == Some(&Token::LBrace) {
                    let block = self.parse_block_stmt()?;
                    self.expect(Token::RParen)?;
                    return Ok(Expr::LabeledBlock { label: None, block });
                }
                let checkpoint = self.lexer.clone();
                let peeked_checkpoint = self.peeked.clone();
                let peeked2_checkpoint = self.peeked2.clone();

                // Try parsing as arrow function parameters
                if let Ok(params) = self.parse_arrow_params() {
                    if self.peek() == Some(&Token::FatArrow) {
                        self.bump()?;
                        let body = if self.peek() == Some(&Token::LBrace) {
                            ArrowBody::Block(self.parse_block_stmt()?)
                        } else {
                            ArrowBody::Expr(self.parse_expr()?)
                        };
                        return Ok(Expr::Arrow(Box::new(ArrowExpr {
                            span: Span {
                                start: span.start,
                                end: self.peek_span().end,
                            },
                            is_async: false,
                            params,
                            body: Box::new(body),
                        })));
                    }
                }

                // Fallback to grouped expression
                self.lexer = checkpoint;
                self.peeked = peeked_checkpoint;
                self.peeked2 = peeked2_checkpoint;
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::Async => {
                if self.peek() == Some(&Token::Fn) {
                    self.bump()?;
                    let params = self.parse_params()?;
                    let mut _return_type = None;
                    if self.peek() == Some(&Token::Bang) {
                        self.bump()?;
                        if self.is_error_type_rhs_start() {
                            let err = self.parse_error_type_rhs()?;
                            _return_type = Some(Type::ResultType {
                                ok: Box::new(Type::Void),
                                err,
                            });
                        } else {
                            _return_type = Some(Type::ErrorUnion(Box::new(Type::Void)));
                        }
                    } else if self.peek() == Some(&Token::Colon)
                        || self.peek() == Some(&Token::Arrow)
                    {
                        self.bump()?;
                        _return_type = Some(self.parse_type()?);
                    }
                    let body = ArrowBody::Block(self.parse_block_stmt()?);
                    let pat_params = params
                        .into_iter()
                        .map(|p| Pattern::Ident(p.ident))
                        .collect();
                    Ok(Expr::Arrow(Box::new(ArrowExpr {
                        span: Span {
                            start: span.start,
                            end: self.peek_span().end,
                        },
                        is_async: true,
                        params: pat_params,
                        body: Box::new(body),
                    })))
                } else if self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    let params = self.parse_arrow_params()?;
                    self.expect(Token::FatArrow)?;
                    let body = if self.peek() == Some(&Token::LBrace) {
                        ArrowBody::Block(self.parse_block_stmt()?)
                    } else {
                        ArrowBody::Expr(self.parse_expr()?)
                    };
                    Ok(Expr::Arrow(Box::new(ArrowExpr {
                        span: Span {
                            start: span.start,
                            end: self.peek_span().end,
                        },
                        is_async: true,
                        params,
                        body: Box::new(body),
                    })))
                } else {
                    let ident = self.parse_ident()?;
                    self.expect(Token::FatArrow)?;
                    let body = if self.peek() == Some(&Token::LBrace) {
                        ArrowBody::Block(self.parse_block_stmt()?)
                    } else {
                        ArrowBody::Expr(self.parse_expr()?)
                    };
                    Ok(Expr::Arrow(Box::new(ArrowExpr {
                        span: Span {
                            start: span.start,
                            end: self.peek_span().end,
                        },
                        is_async: true,
                        params: vec![Pattern::Ident(ident)],
                        body: Box::new(body),
                    })))
                }
            }
            Token::HashLBrace => {
                let fields = self.parse_object_fields(false)?;
                Ok(Expr::Dict(fields))
            }
            Token::LBrace => {
                let fields = self.parse_object_fields(false)?;
                Ok(Expr::Object(fields))
            }
            Token::Await => {
                let expr = self.parse_expr()?;
                Ok(Expr::Await(Box::new(expr)))
            }
            Token::Move | Token::Own => {
                let expr = self.parse_postfix_expr()?;
                Ok(Expr::Move(Box::new(expr)))
            }
            Token::Ampersand => {
                let mut is_mut = false;
                if self.peek() == Some(&Token::Mut) {
                    self.bump()?;
                    is_mut = true;
                }
                let expr = self.parse_postfix_expr()?;
                if self.peek() == Some(&Token::Bang) {
                    let mut end = self.bump()?.1.end;
                    let mut is_optional = false;
                    if self.peek() == Some(&Token::Question) {
                        end = self.bump()?.1.end;
                        is_optional = true;
                    }
                    let mut target_ty = Type::RawPtr {
                        inner: Box::new(Type::Any),
                        depth: 1,
                    };
                    if is_optional {
                        target_ty = Type::Optional(Box::new(target_ty));
                    }
                    return Ok(Expr::Cast {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        expr: Box::new(Expr::Borrow(Box::new(expr), is_mut)),
                        target_ty,
                    });
                } else if self.peek() == Some(&Token::Question)
                    && self.peek_next() == Some(&Token::Bang)
                {
                    self.bump()?;
                    let end = self.bump()?.1.end;
                    let target_ty = Type::Optional(Box::new(Type::RawPtr {
                        inner: Box::new(Type::Any),
                        depth: 1,
                    }));
                    return Ok(Expr::Cast {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        expr: Box::new(Expr::Borrow(Box::new(expr), is_mut)),
                        target_ty,
                    });
                }
                Ok(Expr::Borrow(Box::new(expr), is_mut))
            }
            Token::IntLiteral(v) => Ok(Expr::Lit(Lit::Int(v))),
            Token::FloatLiteral(v) => Ok(Expr::Lit(Lit::Float(v))),
            Token::StringLiteral(s) => Ok(Expr::Lit(Lit::Str(s))),
            Token::TemplateStringRaw(s) => {
                let start = span.start;
                let mut quasis = Vec::new();
                let mut expressions = Vec::new();
                let mut current_quasi = String::new();
                let chars: Vec<char> = s.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        let next_c = chars[i + 1];
                        if next_c == '$' {
                            current_quasi.push('$');
                            i += 2;
                            continue;
                        } else if next_c == '`' {
                            current_quasi.push('`');
                            i += 2;
                            continue;
                        } else if next_c == '\\' {
                            current_quasi.push('\\');
                            i += 2;
                            continue;
                        }
                    }

                    if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
                        quasis.push(current_quasi.clone());
                        current_quasi.clear();
                        i += 2;
                        let mut brace_depth = 1;
                        let mut expr_str = String::new();
                        let mut in_single_quote = false;
                        let mut in_double_quote = false;
                        let mut escape = false;

                        while i < chars.len() {
                            let c = chars[i];
                            if escape {
                                escape = false;
                            } else if c == '\\' {
                                escape = true;
                            } else if c == '"' && !in_single_quote {
                                in_double_quote = !in_double_quote;
                            } else if c == '\'' && !in_double_quote {
                                in_single_quote = !in_single_quote;
                            } else if !in_single_quote && !in_double_quote {
                                if c == '{' {
                                    brace_depth += 1;
                                } else if c == '}' {
                                    brace_depth -= 1;
                                    if brace_depth == 0 {
                                        break;
                                    }
                                }
                            }

                            expr_str.push(c);
                            i += 1;
                        }
                        if brace_depth != 0 {
                            return Err(ParseError::UnexpectedToken(
                                Token::Error,
                                Span {
                                    start,
                                    end: span.end,
                                },
                            ));
                        }
                        i += 1;

                        let mut sub_parser = Parser::new(&expr_str);
                        let expr = sub_parser.parse_expr()?;
                        expressions.push(expr);
                    } else {
                        current_quasi.push(chars[i]);
                        i += 1;
                    }
                }
                quasis.push(current_quasi);
                Ok(Expr::Template(TemplateLit {
                    span: Span {
                        start,
                        end: span.end,
                    },
                    quasis,
                    expressions,
                }))
            }
            Token::New => {
                let callee_expr = self.parse_primary_expr()?;
                if let Expr::Ident(mut ident) = callee_expr {
                    if self.peek() == Some(&Token::Less) {
                        self.bump()?;
                        ident.sym.push('<');
                        let mut depth = 1;
                        while depth > 0 && self.peek().is_some() {
                            let (tok, t_span) = self.bump()?;
                            match tok {
                                Token::Less => depth += 1,
                                Token::Greater => depth -= 1,
                                _ => {}
                            }
                            if depth == 0 {
                                ident.sym.push('>');
                                break;
                            }
                            ident.sym.push_str(&self.input[t_span.start..t_span.end]);
                        }
                    }
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    let end = self.expect(Token::RParen)?.end;
                    Ok(Expr::New(CallExpr {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        callee: Box::new(Expr::Ident(ident)),
                        args,
                        is_comptime: false,
                    }))
                } else {
                    Err(ParseError::UnexpectedToken(Token::New, span))
                }
            }

            Token::LBracket => {
                if self.peek() == Some(&Token::RBracket) {
                    self.bump()?;
                    return Ok(Expr::Array(Vec::new()));
                }
                if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Star) {
                    let is_sentinel_slice = self.peek() == Some(&Token::Colon);
                    self.bump()?; // consume ':' or '*'
                    let mut is_c_ptr = false;
                    let mut sentinel = String::new();
                    if is_sentinel_slice {
                        sentinel = match self.bump()?.0 {
                            Token::IntLiteral(n) => n.to_string(),
                            Token::Ident(s) => s,
                            Token::Null => "null".to_string(),
                            _ => "0".to_string(),
                        };
                    } else {
                        if matches!(self.peek(), Some(Token::Ident(id)) if id == "c") {
                            self.bump()?;
                            is_c_ptr = true;
                        } else if self.peek() == Some(&Token::Colon) {
                            self.bump()?;
                            sentinel = match self.bump()?.0 {
                                Token::IntLiteral(n) => n.to_string(),
                                Token::Ident(s) => s,
                                Token::Null => "null".to_string(),
                                _ => "0".to_string(),
                            };
                        }
                    }
                    self.expect(Token::RBracket)?;
                    let has_const = if self.peek() == Some(&Token::Const) {
                        self.bump()?;
                        true
                    } else {
                        false
                    };
                    let elem_ty = self.parse_base_type()?;
                    let elem_str = Self::type_to_str(&elem_ty);
                    let const_prefix = if has_const { "const " } else { "" };
                    let ty_sym = if is_sentinel_slice {
                        format!("[:{}]{}{}", sentinel, const_prefix, elem_str)
                    } else if is_c_ptr {
                        format!("[*c]{}{}", const_prefix, elem_str)
                    } else if !sentinel.is_empty() {
                        format!("[*:{sentinel}]{}{}", const_prefix, elem_str)
                    } else {
                        format!("[*]{}{}", const_prefix, elem_str)
                    };
                    return Ok(Expr::Ident(Ident { span, sym: ty_sym }));
                }
                let first = self.parse_expr()?;
                if self.peek() == Some(&Token::Semicolon) {
                    self.bump()?;
                    let count = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    return Ok(Expr::ArrayRepeat(Box::new(first), Box::new(count)));
                }
                let mut elements = vec![first];
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                    while self.peek() != Some(&Token::RBracket) {
                        elements.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                }
                let _end = self.expect(Token::RBracket)?.end;
                Ok(Expr::Array(elements))
            }
            Token::Continue => Ok(Expr::Continue(span)),
            Token::Break => {
                if self.peek() == Some(&Token::Colon) {
                    self.bump()?;
                    let label = self.parse_ident()?;
                    let val = if !matches!(
                        self.peek(),
                        Some(Token::Semicolon | Token::RBrace | Token::RParen | Token::Comma)
                            | None
                    ) {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    Ok(Expr::LabeledBreak(span, label, val))
                } else {
                    let has_val = !matches!(
                        self.peek(),
                        Some(Token::Semicolon | Token::RBrace | Token::RParen | Token::Comma)
                            | None
                    );
                    let val = if has_val {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    Ok(Expr::Break(span, val))
                }
            }
            Token::If => {
                let start = span.start;
                self.expect(Token::LParen)?;
                let mut is_comptime = false;
                if matches!(self.peek(), Some(Token::Ident(id)) if id == "comptime") {
                    self.bump()?;
                    is_comptime = true;
                }
                let test = self.parse_expr()?;
                self.expect(Token::RParen)?;
                let cons = if self.peek() == Some(&Token::LBrace) {
                    let block = self.parse_block_stmt()?;
                    Expr::LabeledBlock { label: None, block }
                } else {
                    self.parse_expr()?
                };
                let mut alt = None;
                if self.peek() == Some(&Token::Else) {
                    self.bump()?;
                    let alt_expr = if self.peek() == Some(&Token::LBrace) {
                        let block = self.parse_block_stmt()?;
                        Expr::LabeledBlock { label: None, block }
                    } else {
                        self.parse_expr()?
                    };
                    alt = Some(Box::new(alt_expr));
                }
                let end = self.peek_span().end;
                Ok(Expr::If {
                    span: Span { start, end },
                    test: Box::new(test),
                    cons: Box::new(cons),
                    alt,
                    is_comptime,
                })
            }
            _ => Err(ParseError::UnexpectedToken(token, span)),
        }
    }

    fn parse_object_fields(&mut self, is_dot_brace: bool) -> ParseResult<Vec<ObjectField>> {
        let mut fields = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            if self.peek() == Some(&Token::DotDotDot) {
                self.bump()?;
                let val = self.parse_expr()?;
                fields.push(ObjectField {
                    key: None,
                    val,
                    is_spread: true,
                });
            } else if is_dot_brace {
                let is_keyed = if self.peek() == Some(&Token::Dot)
                    && self.peek_next() != Some(&Token::LBrace)
                {
                    let checkpoint = self.lexer.clone();
                    let peeked_checkpoint = self.peeked.clone();
                    let peeked2_checkpoint = self.peeked2.clone();
                    let _ = self.bump();
                    let has_key = if self.parse_ident().is_ok() {
                        matches!(self.peek(), Some(Token::Colon) | Some(Token::Assign))
                    } else {
                        false
                    };
                    self.lexer = checkpoint;
                    self.peeked = peeked_checkpoint;
                    self.peeked2 = peeked2_checkpoint;
                    has_key
                } else {
                    false
                };

                if is_keyed {
                    self.bump()?;
                    let key = self.parse_ident()?;
                    self.bump()?;
                    let val = self.parse_expr()?;
                    fields.push(ObjectField {
                        key: Some(key),
                        val,
                        is_spread: false,
                    });
                } else {
                    let val = self.parse_expr()?;
                    fields.push(ObjectField {
                        key: None,
                        val,
                        is_spread: false,
                    });
                }
            } else {
                let is_str = matches!(self.peek(), Some(Token::StringLiteral(_)));
                let key = if is_str {
                    let (tok, span) = self.bump()?;
                    if let Token::StringLiteral(s) = tok {
                        Ident { span, sym: s }
                    } else {
                        unreachable!()
                    }
                } else {
                    self.parse_ident()?
                };
                if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Assign) {
                    self.bump()?;
                    let val = self.parse_expr()?;
                    fields.push(ObjectField {
                        key: Some(key),
                        val,
                        is_spread: false,
                    });
                } else {
                    // Shorthand { key } -> { key: key }
                    fields.push(ObjectField {
                        key: Some(key.clone()),
                        val: Expr::Ident(key),
                        is_spread: false,
                    });
                }
            }
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        self.expect(Token::RBrace)?;
        Ok(fields)
    }

    fn parse_arrow_params(&mut self) -> ParseResult<Vec<Pattern>> {
        let mut params = Vec::new();
        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
            if self.peek() == Some(&Token::Star) || self.peek() == Some(&Token::Mut) {
                self.bump()?;
            }
            let ident = self.parse_ident()?;
            params.push(Pattern::Ident(ident));
            if self.peek() == Some(&Token::Colon) {
                self.bump()?;
                self.parse_type()?;
            }
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    fn parse_ident(&mut self) -> ParseResult<Ident> {
        let (token, span) = self.bump()?;
        let sym = match token {
            Token::Ident(s) => s,

            Token::Match => "match".to_string(),
            Token::ZigEscape => "zig".to_string(),
            Token::Switch => "switch".to_string(),
            Token::Try => "try".to_string(),
            Token::Catch => "catch".to_string(),
            Token::Null => "null".to_string(),
            Token::Dyn => "dyn".to_string(),
            Token::Super => "super".to_string(),
            Token::ErrorKw => {
                let text = &self.input[span.start..span.end];
                text.to_string()
            }
            Token::Await => "await".to_string(),
            Token::Async => "async".to_string(),
            Token::If => "if".to_string(),
            Token::While => "while".to_string(),
            Token::For => "for".to_string(),
            Token::Return => "return".to_string(),
            Token::Class => "class".to_string(),
            Token::Struct => "struct".to_string(),
            Token::Ref => "ref".to_string(),
            Token::Trait => "trait".to_string(),
            Token::Enum => "enum".to_string(),
            Token::Impl => "impl".to_string(),
            Token::Interface => "interface".to_string(),
            Token::Extends => "extends".to_string(),
            Token::Any => "any".to_string(),
            Token::Void => "void".to_string(),
            Token::String => "string".to_string(),
            Token::Bool => "bool".to_string(),
            Token::U8 => "u8".to_string(),
            Token::I8 => "i8".to_string(),
            Token::U16 => "u16".to_string(),
            Token::I16 => "i16".to_string(),
            Token::I32 => "i32".to_string(),
            Token::U32 => "u32".to_string(),
            Token::I64 => "i64".to_string(),
            Token::U64 => "u64".to_string(),
            Token::I128 => "i128".to_string(),
            Token::U128 => "u128".to_string(),
            Token::F32 => "f32".to_string(),
            Token::F64 => "f64".to_string(),
            Token::ISize => "isize".to_string(),
            Token::USize => "usize".to_string(),
            Token::New => "new".to_string(),
            Token::From => "from".to_string(),
            Token::As => "as".to_string(),
            Token::Meta => "meta".to_string(),
            Token::Dec => "dec".to_string(),
            Token::Macro => "macro".to_string(),
            Token::Unsafe => "unsafe".to_string(),
            Token::Of => "of".to_string(),
            Token::In => "in".to_string(),
            Token::TypeKw => "type".to_string(),
            Token::Import => "import".to_string(),
            Token::Export => "export".to_string(),
            Token::Const => "const".to_string(),
            Token::Let => "let".to_string(),
            Token::Var => "var".to_string(),
            Token::ThreadLocal => "threadlocal".to_string(),
            Token::Mut => "mut".to_string(),
            Token::Static => "static".to_string(),
            Token::CStr => "cstr".to_string(),
            Token::Else => "else".to_string(),
            Token::Break => "break".to_string(),
            Token::Continue => "continue".to_string(),
            Token::Move => "move".to_string(),
            Token::Own => "own".to_string(),
            Token::Fn => "fn".to_string(),
            Token::Auto => "auto".to_string(),
            _ => {
                return Err(ParseError::ExpectedToken(
                    Token::Ident("Ident".to_string()),
                    token,
                    span,
                ))
            }
        };
        Ok(Ident { span, sym })
    }

    fn peek_op(&mut self) -> Option<BinaryOp> {
        match self.peek() {
            Some(Token::Plus) => Some(BinaryOp::Add),
            Some(Token::Minus) => Some(BinaryOp::Sub),
            Some(Token::Star) => Some(BinaryOp::Mul),
            Some(Token::WrappingAdd) => Some(BinaryOp::WrappingAdd),
            Some(Token::WrappingSub) => Some(BinaryOp::WrappingSub),
            Some(Token::WrappingMul) => Some(BinaryOp::WrappingMul),
            Some(Token::SaturatingAdd) => Some(BinaryOp::SaturatingAdd),
            Some(Token::SaturatingSub) => Some(BinaryOp::SaturatingSub),
            Some(Token::Slash) => Some(BinaryOp::Div),
            Some(Token::Percent) => Some(BinaryOp::Mod),
            Some(Token::Ampersand) => Some(BinaryOp::BitAnd),
            Some(Token::Pipe) => Some(BinaryOp::BitOr),
            Some(Token::Caret) => Some(BinaryOp::BitXor),
            Some(Token::Shl) => Some(BinaryOp::Shl),
            Some(Token::Shr) => Some(BinaryOp::Shr),
            Some(Token::AmpAmp) => Some(BinaryOp::LogicalAnd),
            Some(Token::PipePipe) => Some(BinaryOp::LogicalOr),
            Some(Token::Ident(ref id)) if id == "and" => Some(BinaryOp::LogicalAnd),
            Some(Token::Ident(ref id)) if id == "or" => Some(BinaryOp::LogicalOr),
            Some(Token::Assign) => Some(BinaryOp::Assign),
            Some(Token::Eq) => Some(BinaryOp::Eq),
            Some(Token::NotEq) => Some(BinaryOp::NotEq),
            Some(Token::Less) => Some(BinaryOp::Lt),
            Some(Token::LessEq) => Some(BinaryOp::LtEq),
            Some(Token::Greater) => Some(BinaryOp::Gt),
            Some(Token::GreaterEq) => Some(BinaryOp::GtEq),
            Some(Token::Pipeline) => Some(BinaryOp::Pipe),
            _ => None,
        }
    }

    fn op_precedence(&self, op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::Assign => 1,
            BinaryOp::LogicalOr => 2,
            BinaryOp::LogicalAnd => 3,
            BinaryOp::BitOr => 4,
            BinaryOp::BitXor => 5,
            BinaryOp::BitAnd => 6,
            BinaryOp::Eq | BinaryOp::NotEq => 7,
            BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => 8,
            BinaryOp::Pipe => 9,
            BinaryOp::Shl | BinaryOp::Shr => 10,
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::WrappingAdd
            | BinaryOp::WrappingSub
            | BinaryOp::SaturatingAdd
            | BinaryOp::SaturatingSub => 11,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod | BinaryOp::WrappingMul => 12,
        }
    }
}

fn left_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident(i) => i.span,
        Expr::Lit(_) => Span { start: 0, end: 0 },
        Expr::Bin(b) => b.span,
        Expr::Call(c) | Expr::OptCall(c) | Expr::New(c) => c.span,
        Expr::Member(m) | Expr::OptMember(m) => m.span,
        Expr::Slice(s) => s.span,
        Expr::Match(m) => m.span,
        Expr::Spawn(s) => s.span,
        Expr::Receive(_) => Span { start: 0, end: 0 },
        Expr::Zig(z) => z.span,
        Expr::Question(e) | Expr::ErrWrap(e) | Expr::BoxNew(e) | Expr::Unwrap(e) => left_span(e),
        Expr::QuestionFallback { expr, .. } | Expr::Catch { expr, .. } => left_span(expr),
        Expr::NullCoalesce { left, .. } => left_span(left),
        Expr::Downcast { obj, .. } => left_span(obj),
        Expr::Array(_) => Span { start: 0, end: 0 },
        Expr::ArrayRepeat(val, _) => left_span(val),
        Expr::Object(_) | Expr::Dict(_) => Span { start: 0, end: 0 },
        Expr::Index { obj, .. } => left_span(obj),
        Expr::Arrow(a) => a.span,
        Expr::EnumInit(e) => e.span,
        Expr::Template(t) => t.span,
        Expr::Await(e) => left_span(e),
        Expr::Move(e) => left_span(e),
        Expr::Borrow(e, _) => left_span(e),
        Expr::Builtin(b) => b.span,
        Expr::ReflectType { span, .. } | Expr::ReflectOf { span, .. } => *span,
        Expr::PipePlaceholder(span) => *span,
        Expr::Range { span, .. } => *span,
        Expr::Cast { span, .. } => *span,
        Expr::Deref(d) => left_span(d),
        Expr::BitNot(d) => left_span(d),
        Expr::Unsafe(b) | Expr::Comptime(b) | Expr::Arena(b) => b.span,
        Expr::If { span, .. } => *span,
        Expr::Continue(span) | Expr::LabeledContinue(span, _) => *span,
        Expr::Break(span, _) | Expr::LabeledBreak(span, _, _) => *span,
        Expr::LabeledBlock { block, .. } => block.span,
        Expr::Return(span, _) => *span,
        Expr::InlineAsm { span, .. } | Expr::ComptimeMap { span, .. } => *span,
    }
}

fn right_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident(i) => i.span,
        Expr::Lit(_) => Span { start: 0, end: 0 },
        Expr::Bin(b) => right_span(&b.right),
        Expr::Call(c) | Expr::OptCall(c) | Expr::New(c) => Span {
            start: 0,
            end: c.span.end,
        },
        Expr::Member(m) | Expr::OptMember(m) => m.span,
        Expr::Slice(s) => s.span,
        Expr::Match(m) => m.span,
        Expr::Spawn(s) => s.span,
        Expr::Receive(_) => Span { start: 0, end: 0 },
        Expr::Zig(z) => z.span,
        Expr::Question(e) | Expr::ErrWrap(e) | Expr::BoxNew(e) | Expr::Unwrap(e) => right_span(e),
        Expr::QuestionFallback { fallback, .. } => right_span(fallback),
        Expr::NullCoalesce { right, .. } => right_span(right),
        Expr::Catch { expr, .. } => right_span(expr),
        Expr::Downcast { target_ty, .. } => target_ty.span,
        Expr::Array(_) => Span { start: 0, end: 0 },
        Expr::ArrayRepeat(_, count) => right_span(count),
        Expr::Object(_) | Expr::Dict(_) => Span { start: 0, end: 0 },
        Expr::Index { index, .. } => right_span(index),
        Expr::Arrow(a) => a.span,
        Expr::EnumInit(e) => e.span,
        Expr::Template(t) => t.span,
        Expr::Await(e) => right_span(e),
        Expr::Move(e) => right_span(e),
        Expr::Borrow(e, _) => right_span(e),
        Expr::Builtin(b) => b.span,
        Expr::ReflectType { span, .. } | Expr::ReflectOf { span, .. } => *span,
        Expr::PipePlaceholder(span) => *span,
        Expr::Range { span, .. } => *span,
        Expr::Cast { span, .. } => *span,
        Expr::Deref(d) => right_span(d),
        Expr::BitNot(d) => right_span(d),
        Expr::Unsafe(b) | Expr::Comptime(b) | Expr::Arena(b) => b.span,
        Expr::If { span, .. } => *span,
        Expr::Continue(span) | Expr::LabeledContinue(span, _) => *span,
        Expr::Break(span, val) | Expr::LabeledBreak(span, _, val) => {
            val.as_ref().map(|v| right_span(v)).unwrap_or(*span)
        }
        Expr::LabeledBlock { block, .. } => block.span,
        Expr::Return(span, val) => val.as_ref().map(|v| right_span(v)).unwrap_or(*span),
        Expr::InlineAsm { span, .. } | Expr::ComptimeMap { span, .. } => *span,
    }
}

pub fn expr_to_member_str(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(id) => Some(id.sym.clone()),
        Expr::Member(m) => {
            let base = expr_to_member_str(&m.obj)?;
            Some(format!("{}.{}", base, m.prop.sym))
        }
        _ => None,
    }
}
