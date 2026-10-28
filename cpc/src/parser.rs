use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: {0:?} at {1:?}")]
    UnexpectedToken(Token, Span),
    #[error("Expected token: {0:?}, found {1:?} at {2:?}")]
    ExpectedToken(Token, Token, Span),
    #[error("End of input reached unexpectedly")]
    EndOfInput,
    #[error("Parse error: {0} at {1:?}")]
    Generic(String, Span),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    input: &'a str,
    peeked: Option<(Token, Span)>,
    peeked2: Option<(Token, Span)>,
    in_match_guard: bool,
    in_pipeline_rhs: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            input,
            peeked: None,
            peeked2: None,
            in_match_guard: false,
            in_pipeline_rhs: 0,
        }
    }

    fn bump(&mut self) -> ParseResult<(Token, Span)> {
        if let Some(p) = self.peeked.take() {
            self.peeked = self.peeked2.take();
            return Ok(p);
        }
        let (token, range) = self.lexer.next().ok_or(ParseError::EndOfInput)?;
        Ok((
            token,
            Span {
                start: range.start,
                end: range.end,
            },
        ))
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

    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let mut body = Vec::new();
        let start = self.peek_span().start;
        while self.peek().is_some() {
            body.push(self.parse_module_item()?);
        }
        let end = self.peek_span().end;
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
            in_match_guard: self.in_match_guard,
            in_pipeline_rhs: self.in_pipeline_rhs,
        }
    }

    fn is_decl_keyword(t: &Token) -> bool {
        matches!(
            t,
            Token::Fn
                | Token::Async
                | Token::Static
                | Token::Unsafe
                | Token::Class
                | Token::Struct
                | Token::Ref
                | Token::Meta
                | Token::Dec
                | Token::Macro
                | Token::Trait
                | Token::Impl
                | Token::Enum
                | Token::Interface
                | Token::ErrorKw
        )
    }

    fn parse_meta_ident(&mut self) -> ParseResult<Ident> {
        let (tok, span) = self.bump()?;
        let sym = match tok {
            Token::Ident(s) => s,
            Token::Class => "class".to_string(),
            Token::Struct => "struct".to_string(),
            Token::Fn => "fn".to_string(),
            Token::Ref => "ref".to_string(),
            Token::Let => "let".to_string(),
            Token::Const => "const".to_string(),
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
        );
        if is_ident_like {
            let ident = self.parse_meta_ident()?;
            if self.peek() == Some(&Token::Colon) {
                self.bump()?; // consume Colon
                let expr = self.parse_primary_expr()?;
                return Ok(MetaArg::Named(ident, expr));
            } else {
                return Ok(MetaArg::Positional(Expr::Ident(ident)));
            }
        }
        let expr = self.parse_primary_expr()?;
        Ok(MetaArg::Positional(expr))
    }

    fn parse_meta_item(&mut self) -> ParseResult<MetaItem> {
        let start = self.expect(Token::At)?.start;
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

    fn parse_meta_list(&mut self) -> ParseResult<Vec<MetaItem>> {
        let mut list = Vec::new();
        while self.peek() == Some(&Token::At) {
            list.push(self.parse_meta_item()?);
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
        if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
            self.bump()?;
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
        if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
            self.bump()?;
            return_type = Some(self.parse_type()?);
        }
        let body = self.parse_block_stmt()?;
        let end = body.span.end;
        Ok(MacroDef {
            span: Span { start, end },
            ident,
            params,
            return_type,
            body,
            meta: meta_list,
        })
    }

    fn parse_module_item(&mut self) -> ParseResult<ModuleItem> {
        if self.peek() == Some(&Token::Export) {
            self.bump()?;
        }
        if self.peek() == Some(&Token::At) {
            let mut clone = self.clone_lookahead();
            let mut has_decl = false;
            while clone.peek() == Some(&Token::At) {
                if clone.bump().is_err() {
                    break;
                }
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
            if let Some(tok) = clone.peek() {
                if Self::is_decl_keyword(tok) {
                    has_decl = true;
                }
            }
            if has_decl {
                let meta_list = self.parse_meta_list()?;
                return Ok(ModuleItem::Decl(self.parse_decl(meta_list)?));
            }
        }
        match self.peek() {
            Some(Token::Import) => Ok(ModuleItem::Import(self.parse_import_stmt()?)),
            Some(Token::Meta)
            | Some(Token::Dec)
            | Some(Token::Macro)
            | Some(Token::Fn)
            | Some(Token::Class)
            | Some(Token::Struct)
            | Some(Token::Ref)
            | Some(Token::Interface)
            | Some(Token::ErrorKw)
            | Some(Token::Trait)
            | Some(Token::Impl)
            | Some(Token::Enum)
            | Some(Token::Async)
            | Some(Token::Static) => Ok(ModuleItem::Decl(self.parse_decl(vec![])?)),
            Some(Token::Unsafe) => {
                if self.peek_next() == Some(&Token::LBrace) {
                    Ok(ModuleItem::Stmt(self.parse_stmt()?))
                } else {
                    Ok(ModuleItem::Decl(self.parse_decl(vec![])?))
                }
            }
            _ => Ok(ModuleItem::Stmt(self.parse_stmt()?)),
        }
    }

    fn parse_import_stmt(&mut self) -> ParseResult<ImportStmt> {
        let start = self.expect(Token::Import)?.start;
        self.expect(Token::LBrace)?;
        let mut specifiers = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            specifiers.push(self.parse_ident()?);
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        self.expect(Token::RBrace)?;
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
            source,
        })
    }

    fn parse_decl(&mut self, meta: Vec<MetaItem>) -> ParseResult<Decl> {
        match self.peek() {
            Some(Token::Meta) => Ok(Decl::Meta(self.parse_meta_def(meta)?)),
            Some(Token::Dec) => Ok(Decl::Dec(self.parse_dec_def(meta)?)),
            Some(Token::Macro) => Ok(Decl::Macro(self.parse_macro_def(meta)?)),
            Some(Token::Fn) | Some(Token::Async) | Some(Token::Static) | Some(Token::Unsafe) => {
                let mut f = self.parse_function()?;
                f.meta = meta;
                Ok(Decl::Func(f))
            }
            Some(Token::Class) | Some(Token::Struct) | Some(Token::Ref) => {
                let mut c = self.parse_class()?;
                c.meta = meta;
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
        while matches!(
            self.peek(),
            Some(Token::Static) | Some(Token::Async) | Some(Token::Unsafe)
        ) {
            if self.peek() == Some(&Token::Static) {
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
        let params = self.parse_params()?;
        let mut return_type = None;
        if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
            self.bump()?;
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
            params,
            return_type,
            body,
            is_async,
            is_static,
            is_unsafe,
            meta: Vec::new(),
        })
    }

    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while self.peek() != Some(&Token::RParen) {
            let meta = self.parse_meta_list()?;
            let mut is_mut = false;
            let mut is_move = false;

            if self.peek() == Some(&Token::Mut) {
                self.bump()?;
                is_mut = true;
            } else if self.peek() == Some(&Token::Move) {
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
                    is_comptime: false,
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
                    is_comptime: false,
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
                is_comptime: false,
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
            Token::I32
                | Token::U32
                | Token::U64
                | Token::F32
                | Token::F64
                | Token::USize
                | Token::String
                | Token::Void
                | Token::Any
                | Token::Bool
                | Token::Ident(_)
                | Token::LBracket
                | Token::LBrace
                | Token::Ampersand
        )
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
            let first = self.parse_ident()?;
            if first.sym == "Box" && self.peek() == Some(&Token::Less) {
                self.bump()?;
                if self.peek() == Some(&Token::Dyn) {
                    self.bump()?;
                }
                let tr = self.parse_ident()?;
                self.expect(Token::Greater)?;
                return Ok(ErrorType::BoxDyn(tr));
            }
            if self.peek() == Some(&Token::Pipe) {
                let mut list = vec![first];
                while self.peek() == Some(&Token::Pipe) {
                    self.bump()?;
                    list.push(self.parse_ident()?);
                }
                return Ok(ErrorType::Union(list));
            }
            if first.sym == "Error" || first.sym == "error" {
                return Ok(ErrorType::DefaultError);
            }
            return Ok(ErrorType::Named(first));
        }
        Ok(ErrorType::DefaultError)
    }

    fn parse_base_type(&mut self) -> ParseResult<Type> {
        if self.peek() == Some(&Token::Question) {
            self.bump()?;
            let inner = self.parse_base_type()?;
            return Ok(Type::Optional(Box::new(inner)));
        }
        if self.peek() == Some(&Token::Dyn) {
            self.bump()?;
            let tr = self.parse_ident()?;
            return Ok(Type::BareDyn(tr));
        }
        if self.peek() == Some(&Token::Star) {
            let mut depth = 0;
            while self.peek() == Some(&Token::Star) {
                self.bump()?;
                depth += 1;
            }
            let mut lifetimes = Vec::new();
            if depth == 1 {
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
                            | Some(Token::U64)
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
                    | None => true,
                    _ => false,
                };
                if is_raw_ptr_bang {
                    self.bump()?; // consume '!'
                    return Ok(Type::RawPtr {
                        inner: Box::new(inner),
                        depth,
                    });
                }
            }
            let mut ty = inner;
            for idx in 0..depth {
                ty = Type::BorrowType {
                    inner: Box::new(ty),
                    is_mut: true,
                    lifetimes: if idx == 0 { lifetimes.clone() } else { vec![] },
                };
            }
            return Ok(ty);
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
                        | Some(Token::U64)
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
            return Ok(Type::BorrowType {
                inner: Box::new(inner),
                is_mut,
                lifetimes,
            });
        }
        if self.peek() == Some(&Token::LBracket) {
            self.bump()?;
            let mut elements = Vec::new();
            while self.peek() != Some(&Token::RBracket) {
                elements.push(self.parse_type()?);
                if self.peek() == Some(&Token::Comma) {
                    self.bump()?;
                }
            }
            self.expect(Token::RBracket)?;
            return Ok(Type::Tuple(elements));
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
            Token::I32 => Type::I32,
            Token::U32 => Type::U32,
            Token::U64 => Type::U64,
            Token::F32 => Type::F32,
            Token::F64 => Type::F64,
            Token::USize => Type::USize,
            Token::String => Type::String,
            Token::Void => Type::Void,
            Token::Any => Type::Any,
            Token::Bool => Type::Bool,
            Token::ErrorKw => Type::Ref(Ident {
                span,
                sym: "Error".to_string(),
            }),
            Token::Ident(sym) => {
                if sym == "pid" {
                    Type::PID
                } else if sym == "Shared" && self.peek() == Some(&Token::Less) {
                    self.bump()?;
                    let inner = self.parse_type()?;
                    self.expect(Token::Greater)?;
                    Type::Shared(Box::new(inner))
                } else if sym == "Box" && self.peek() == Some(&Token::Less) {
                    self.bump()?;
                    if self.peek() == Some(&Token::Dyn) {
                        self.bump()?;
                    }
                    let tr = self.parse_ident()?;
                    self.expect(Token::Greater)?;
                    Type::BoxDyn(tr)
                } else if (sym == "Map" || sym == "Dict") && self.peek() == Some(&Token::Less) {
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
                } else if sym == "Set" && self.peek() == Some(&Token::Less) {
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
                    Type::Ref(Ident { span, sym })
                }
            }
            _ => return Err(ParseError::UnexpectedToken(token, span)),
        };
        loop {
            if self.peek() == Some(&Token::Question) {
                self.bump()?;
                ty = Type::Optional(Box::new(ty));
            } else if self.peek() == Some(&Token::LBracket) {
                self.bump()?;
                self.expect(Token::RBracket)?;
                ty = Type::Array(Box::new(ty));
            } else {
                break;
            }
        }
        if is_legacy_error_union {
            ty = Type::ErrorUnion(Box::new(ty));
        }
        Ok(ty)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        let base = self.parse_base_type()?;
        if self.peek() == Some(&Token::Bang) {
            self.bump()?;
            let err = self.parse_error_type_rhs()?;
            return Ok(Type::ResultType {
                ok: Box::new(base),
                err,
            });
        }
        Ok(base)
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Some(Token::Let) | Some(Token::Const) => Ok(Stmt::Var(self.parse_var_decl()?)),
            Some(Token::While) => Ok(Stmt::While(self.parse_while_stmt()?)),
            Some(Token::For) => Ok(Stmt::For(self.parse_for_stmt()?)),
            Some(Token::If) => Ok(Stmt::If(self.parse_if_stmt()?)),
            Some(Token::Return) => Ok(Stmt::Return(self.parse_return_stmt()?)),
            Some(Token::Try) => {
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
                    } else if self.peek() != Some(&Token::LBrace) {
                        if self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                        }
                        if matches!(self.peek(), Some(Token::Ident(_))) {
                            err_ident = Some(self.parse_ident()?);
                        }
                        if self.peek() == Some(&Token::FatArrow) {
                            self.bump()?;
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
                    Ok(Stmt::Expr(self.parse_expr()?))
                }
            }
            Some(Token::Unsafe) => {
                self.bump()?;
                Ok(Stmt::UnsafeBlock(self.parse_block_stmt()?))
            }
            Some(Token::LBrace) => Ok(Stmt::Block(self.parse_block_stmt()?)),
            Some(Token::ZigEscape) => {
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
                let raw_code = &self.input[code_start..code_end];
                Ok(Stmt::Expr(Expr::Zig(ZigEscapeExpr {
                    span: Span { start, end },
                    code: raw_code.trim().to_string(),
                })))
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
        let kind = match self.bump()?.0 {
            Token::Let => VarDeclKind::Let,
            Token::Const => VarDeclKind::Const,
            _ => unreachable!(),
        };
        if self.peek() == Some(&Token::Mut) {
            self.bump()?;
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
        })
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        let first = self.parse_single_pattern()?;
        if self.peek() == Some(&Token::Pipe) {
            let mut pats = vec![first];
            while self.peek() == Some(&Token::Pipe) {
                self.bump()?;
                pats.push(self.parse_single_pattern()?);
            }
            return Ok(Pattern::Or(pats));
        }
        Ok(first)
    }

    fn parse_single_pattern(&mut self) -> ParseResult<Pattern> {
        match self.peek() {
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
                while self.peek() != Some(&Token::RBracket) {
                    elements.push(self.parse_pattern()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Pattern::Tuple(elements))
            }
            Some(Token::LBrace) => {
                let _start = self.bump()?.1.start;
                let mut props = Vec::new();
                while self.peek() != Some(&Token::RBrace) {
                    if self.peek() == Some(&Token::DotDotDot) {
                        self.bump()?;
                        if !matches!(self.peek(), Some(Token::Comma | Token::RBrace) | None) {
                            let _ = self.parse_ident()?;
                        }
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
                    props.push(ObjectPatProp { key, val });
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                let _end = self.expect(Token::RBrace)?.end;
                Ok(Pattern::Object(props))
            }
            Some(Token::Dot) => {
                self.bump()?;
                let variant_name = self.parse_ident()?;
                let mut fields = None;
                let mut tuple_pats = None;
                if self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    let mut pats = Vec::new();
                    while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                        pats.push(self.parse_pattern()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.bump()?;
                        }
                    }
                    self.expect(Token::RParen)?;
                    tuple_pats = Some(pats);
                } else if self.peek() == Some(&Token::Pipe) {
                    self.bump()?;
                    let key = self.parse_ident()?;
                    fields = Some(vec![ObjectPatProp { key, val: None }]);
                    self.expect(Token::Pipe)?;
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
                if self.peek() == Some(&Token::Star) || self.peek() == Some(&Token::Bang) {
                    self.bump()?;
                    return Ok(Pattern::Wildcard);
                }
                if self.peek() == Some(&Token::Minus) {
                    self.bump()?;
                    if let (Token::IntLiteral(v), _) = self.bump()? {
                        return Ok(Pattern::Lit(Lit::Int(-v)));
                    }
                }
                if let Some(Token::IntLiteral(_)) = self.peek() {
                    if let (Token::IntLiteral(v), _) = self.bump()? {
                        if self.peek() == Some(&Token::DotDotEq)
                            || self.peek() == Some(&Token::DotDot)
                        {
                            let inclusive = self.bump()?.0 == Token::DotDotEq;
                            let end_v = if let (Token::IntLiteral(ev), _) = self.bump()? {
                                ev
                            } else {
                                v
                            };
                            return Ok(Pattern::Range {
                                start: v,
                                end: end_v,
                                inclusive,
                            });
                        }
                        return Ok(Pattern::Lit(Lit::Int(v)));
                    }
                }
                if let Some(Token::StringLiteral(_)) = self.peek() {
                    if let (Token::StringLiteral(s), _) = self.bump()? {
                        return Ok(Pattern::Lit(Lit::Str(s)));
                    }
                }
                let first_id = self.parse_ident()?;
                if first_id.sym == "true" {
                    return Ok(Pattern::Lit(Lit::Bool(true)));
                }
                if first_id.sym == "false" {
                    return Ok(Pattern::Lit(Lit::Bool(false)));
                }
                if first_id.sym == "Err" && self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    let inner = self.parse_pattern()?;
                    self.expect(Token::RParen)?;
                    return Ok(Pattern::Err(Box::new(inner)));
                }
                if self.peek() == Some(&Token::Dot) {
                    self.bump()?;
                    let variant_name = self.parse_ident()?;
                    let mut fields = None;
                    let mut tuple_pats = None;
                    if self.peek() == Some(&Token::LParen) {
                        self.bump()?;
                        let mut pats = Vec::new();
                        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                            pats.push(self.parse_pattern()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RParen)?;
                        tuple_pats = Some(pats);
                    } else if self.peek() == Some(&Token::LBrace) {
                        self.bump()?;
                        let mut props = Vec::new();
                        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                            let key = self.parse_ident()?;
                            let mut val = None;
                            if self.peek() == Some(&Token::Colon) {
                                self.bump()?;
                                val = Some(self.parse_pattern()?);
                            }
                            props.push(ObjectPatProp { key, val });
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::RBrace)?;
                        fields = Some(props);
                    }
                    return Ok(Pattern::Enum(EnumPattern {
                        span: first_id.span,
                        enum_name: first_id,
                        variant_name,
                        fields,
                        tuple_pats,
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
            body.push(self.parse_stmt()?);
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(BlockStmt {
            span: Span { start, end },
            body,
        })
    }

    fn parse_return_stmt(&mut self) -> ParseResult<ReturnStmt> {
        let start = self.expect(Token::Return)?.start;
        let mut arg = None;
        if self.peek() != Some(&Token::Semicolon) && self.peek() != Some(&Token::RBrace) {
            arg = Some(self.parse_expr()?);
        }
        if self.peek() == Some(&Token::Semicolon) {
            self.bump()?;
        }
        let end = self.peek_span().end;
        Ok(ReturnStmt {
            span: Span { start, end },
            arg,
        })
    }

    fn parse_match_expr(&mut self) -> ParseResult<MatchExpr> {
        let start = self.peek_span().start;
        let disc = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let mut arms = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            let pat = self.parse_pattern()?;
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
        self.expect(Token::LParen)?;
        let test = self.parse_expr()?;
        self.expect(Token::RParen)?;
        let body = self.parse_stmt()?;
        Ok(WhileStmt {
            span: Span { start, end: 0 },
            test,
            body: Box::new(body),
        })
    }

    fn parse_for_stmt(&mut self) -> ParseResult<ForStmt> {
        let start = self.expect(Token::For)?.start;
        let has_paren = if self.peek() == Some(&Token::LParen) {
            self.bump()?;
            true
        } else {
            false
        };
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
                "Expected 'of' in for loop, e.g. 'for x of list'".to_string(),
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
        if has_paren {
            self.expect(Token::RParen)?;
        }
        let body = self.parse_stmt()?;
        Ok(ForStmt {
            span: Span { start, end: 0 },
            mode,
            pat,
            expr,
            body: Box::new(body),
        })
    }

    fn parse_if_stmt(&mut self) -> ParseResult<IfStmt> {
        let start = self.expect(Token::If)?.start;
        self.expect(Token::LParen)?;
        let test = self.parse_expr()?;
        self.expect(Token::RParen)?;
        let cons = self.parse_stmt()?;
        let mut alt = None;
        if self.peek() == Some(&Token::Else) {
            self.bump()?;
            alt = Some(Box::new(self.parse_stmt()?));
        }
        Ok(IfStmt {
            span: Span { start, end: 0 },
            test,
            cons: Box::new(cons),
            alt,
        })
    }

    fn parse_class(&mut self) -> ParseResult<Class> {
        let start = self.peek_span().start;
        let mut is_ref = false;
        if self.peek() == Some(&Token::Ref) {
            self.bump()?;
            is_ref = true;
        }
        if self.peek() == Some(&Token::Struct) {
            self.bump()?;
            is_ref = true;
        } else {
            self.expect(Token::Class)?;
        }
        let ident = self.parse_ident()?;

        let mut extends = None;
        let mut implements = Vec::new();
        if self.peek() == Some(&Token::Extends) {
            self.bump()?;
            let parent = self.parse_ident()?;
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
        self.expect(Token::LBrace)?;
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            let meta_list = self.parse_meta_list()?;
            if matches!(
                self.peek(),
                Some(Token::Fn) | Some(Token::Async) | Some(Token::Static)
            ) {
                let mut method = self.parse_function()?;
                method.meta = meta_list;
                methods.push(method);
            } else {
                let f_ident = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                if matches!(ty, Type::BorrowType { .. }) {
                    is_ref = true;
                }
                if self.peek() == Some(&Token::Comma) || self.peek() == Some(&Token::Semicolon) {
                    self.bump()?;
                }
                fields.push(Field {
                    span: f_ident.span,
                    ident: f_ident,
                    ty,
                    meta: meta_list,
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
            meta: Vec::new(),
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
            let params = self.parse_params()?;
            let mut return_type = None;
            if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
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

    fn parse_enum(&mut self) -> ParseResult<EnumDecl> {
        let start = self.expect(Token::Enum)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            let v_ident = self.parse_ident()?;
            let mut fields = None;
            let mut tuple_type = None;
            let mut from_err = None;
            if self.peek() == Some(&Token::LParen) {
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
            if self.peek() == Some(&Token::Comma) {
                self.bump()?;
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(EnumDecl {
            span: Span { start, end },
            ident,
            variants,
            meta: Vec::new(),
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
        Ok(ErrorSetDecl {
            span: Span { start, end },
            ident,
            variants,
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
                Some(Token::Dot) => {
                    if self.peek_next() == Some(&Token::Star) {
                        self.bump()?; // consume .
                        self.bump()?; // consume *
                        left = Expr::Deref(Box::new(left));
                        continue;
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
                    let is_enum_init = if let Expr::Ident(ref obj_id) = left {
                        obj_id
                            .sym
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_uppercase())
                            && prop.sym.chars().next().map_or(false, |c| c.is_uppercase())
                    } else {
                        false
                    };
                    if is_enum_init && self.peek() == Some(&Token::LBrace) {
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
                            fields: Some(fields),
                        });
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
                    if self.peek() == Some(&Token::Return) {
                        self.bump()?;
                        let fallback = self.parse_expr()?;
                        left = Expr::QuestionFallback {
                            expr: Box::new(left),
                            fallback: Box::new(fallback),
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
                    let right = self.parse_expr()?;
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
                    } else if self.peek() != Some(&Token::LBrace) {
                        if self.peek() == Some(&Token::Mut) {
                            self.bump()?;
                            is_err_mut = true;
                        }
                        if matches!(self.peek(), Some(Token::Ident(_))) {
                            err_ident = Some(self.parse_ident()?);
                        }
                        if self.peek() == Some(&Token::FatArrow) {
                            self.bump()?;
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
                    let mut is_slice = false;
                    if self.peek() == Some(&Token::DotDot) {
                        self.bump()?;
                        is_slice = true;
                        if self.peek() != Some(&Token::RBracket) {
                            end_idx = Some(Box::new(self.parse_expr()?));
                        }
                    } else {
                        let idx = self.parse_expr()?;
                        if self.peek() == Some(&Token::DotDot) {
                            self.bump()?;
                            is_slice = true;
                            start_idx = Some(Box::new(idx));
                            if self.peek() != Some(&Token::RBracket) {
                                end_idx = Some(Box::new(self.parse_expr()?));
                            }
                        } else {
                            start_idx = Some(Box::new(idx));
                        }
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
                _ => break,
            }
        }
        while self.peek().is_some() {
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
            let right = self.parse_bin_expr(prec + 1)?;
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
                Some(Token::Dot) => {
                    if self.peek_next() == Some(&Token::Star) {
                        self.bump()?; // consume .
                        self.bump()?; // consume *
                        left = Expr::Deref(Box::new(left));
                        continue;
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
                    let is_enum_init = if let Expr::Ident(ref obj_id) = left {
                        obj_id
                            .sym
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_uppercase())
                            && prop.sym.chars().next().map_or(false, |c| c.is_uppercase())
                    } else {
                        false
                    };
                    if is_enum_init && self.peek() == Some(&Token::LBrace) {
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
                            fields: Some(fields),
                        });
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
                    let mut is_slice = false;
                    if self.peek() == Some(&Token::DotDot) {
                        self.bump()?;
                        is_slice = true;
                        if self.peek() != Some(&Token::RBracket) {
                            end_idx = Some(Box::new(self.parse_expr()?));
                        }
                    } else {
                        let idx = self.parse_expr()?;
                        if self.peek() == Some(&Token::DotDot) {
                            self.bump()?;
                            is_slice = true;
                            start_idx = Some(Box::new(idx));
                            if self.peek() != Some(&Token::RBracket) {
                                end_idx = Some(Box::new(self.parse_expr()?));
                            }
                        } else {
                            start_idx = Some(Box::new(idx));
                        }
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
                        });
                    } else {
                        left = Expr::Index {
                            obj: Box::new(left),
                            index: start_idx.unwrap(),
                        };
                    }
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> ParseResult<Expr> {
        let (token, span) = self.bump()?;
        match token {
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
                if self.in_pipeline_rhs > 0 {
                    if let Some(Token::Ident(_)) = self.peek() {
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
                    if let Some(Token::Ident(_)) = self.peek() {
                        let variant_name = self.parse_ident()?;
                        Ok(Expr::EnumInit(EnumInitExpr {
                            span,
                            enum_name: Ident {
                                span,
                                sym: String::new(),
                            },
                            variant_name,
                            fields: None,
                        }))
                    } else {
                        return Err(ParseError::Generic(
                            "Pipeline placeholder '.' can only be used on the right-hand side of a pipeline ('|>') operator".to_string(),
                            span,
                        ));
                    }
                }
            }
            Token::Super => Ok(Expr::Ident(Ident {
                span,
                sym: "super".to_string(),
            })),
            Token::ErrorKw => Ok(Expr::Ident(Ident {
                span,
                sym: "error".to_string(),
            })),
            Token::Unsafe => {
                let block = self.parse_block_stmt()?;
                Ok(Expr::Unsafe(block))
            }
            Token::Star => {
                let mut depth = 1;
                while self.peek() == Some(&Token::Star) {
                    self.bump()?;
                    depth += 1;
                }
                let inner = self.parse_postfix_expr()?;
                if self.peek() == Some(&Token::Bang) {
                    let end = self.bump()?.1.end;
                    return Ok(Expr::Cast {
                        span: Span {
                            start: span.start,
                            end,
                        },
                        expr: Box::new(Expr::Borrow(Box::new(inner), true)),
                        target_ty: Type::RawPtr {
                            inner: Box::new(Type::Any),
                            depth,
                        },
                    });
                }
                Ok(Expr::Borrow(Box::new(inner), true))
            }
            Token::Bang => {
                let inner = self.parse_primary_expr()?;
                Ok(Expr::Builtin(BuiltinCall {
                    span,
                    name: "@not".to_string(),
                    args: vec![inner],
                }))
            }
            Token::Ident(sym) => {
                if sym == "Err" && self.peek() == Some(&Token::LParen) {
                    self.bump()?;
                    let inner = self.parse_expr()?;
                    self.expect(Token::RParen)?;
                    return Ok(Expr::ErrWrap(Box::new(inner)));
                }
                if sym.chars().next().map_or(false, |c| c.is_uppercase())
                    && self.peek() == Some(&Token::LBrace)
                {
                    self.bump()?;
                    let mut fields = Vec::new();
                    while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                        let key = self.parse_ident()?;
                        self.expect(Token::Colon)?;
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
                    }));
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
            }
            Token::Match | Token::Switch => {
                let match_expr = self.parse_match_expr()?;
                Ok(Expr::Match(Box::new(match_expr)))
            }
            Token::Fn => {
                let params = self.parse_params()?;
                let mut _return_type = None;
                if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
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
                    is_async: false,
                    params: pat_params,
                    body: Box::new(body),
                })))
            }
            Token::LParen => {
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
                    if self.peek() == Some(&Token::Colon) || self.peek() == Some(&Token::Arrow) {
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
                let fields = self.parse_object_fields()?;
                Ok(Expr::Dict(fields))
            }
            Token::LBrace => {
                let fields = self.parse_object_fields()?;
                Ok(Expr::Object(fields))
            }
            Token::Await => {
                let expr = self.parse_expr()?;
                Ok(Expr::Await(Box::new(expr)))
            }
            Token::Move => {
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
                if let Expr::Ident(ident) = callee_expr {
                    if self.peek() == Some(&Token::Less) {
                        self.bump()?;
                        while self.peek() != Some(&Token::Greater) && self.peek().is_some() {
                            self.parse_type()?;
                            if self.peek() == Some(&Token::Comma) {
                                self.bump()?;
                            }
                        }
                        self.expect(Token::Greater)?;
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
                let mut elements = Vec::new();
                while self.peek() != Some(&Token::RBracket) {
                    elements.push(self.parse_expr()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.bump()?;
                    }
                }
                let _end = self.expect(Token::RBracket)?.end;
                Ok(Expr::Array(elements))
            }
            _ => Err(ParseError::UnexpectedToken(token, span)),
        }
    }

    fn parse_object_fields(&mut self) -> ParseResult<Vec<ObjectField>> {
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
                if self.peek() == Some(&Token::Colon) {
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
            Token::Switch => "switch".to_string(),
            Token::Try => "try".to_string(),
            Token::Catch => "catch".to_string(),
            Token::Null => "null".to_string(),
            Token::Dyn => "dyn".to_string(),
            Token::Super => "super".to_string(),
            Token::ErrorKw => "Error".to_string(),
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
            Token::I32 => "i32".to_string(),
            Token::U32 => "u32".to_string(),
            Token::U64 => "u64".to_string(),
            Token::F32 => "f32".to_string(),
            Token::F64 => "f64".to_string(),
            Token::USize => "usize".to_string(),
            Token::New => "new".to_string(),
            Token::From => "from".to_string(),
            Token::As => "as".to_string(),
            Token::Meta => "meta".to_string(),
            Token::Unsafe => "unsafe".to_string(),
            Token::Of => "of".to_string(),
            Token::In => "in".to_string(),
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
            Some(Token::Slash) => Some(BinaryOp::Div),
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
            BinaryOp::Eq | BinaryOp::NotEq => 2,
            BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => 2,
            BinaryOp::Pipe => 3,
            BinaryOp::Add | BinaryOp::Sub => 4,
            BinaryOp::Mul | BinaryOp::Div => 5,
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
        Expr::Question(e) | Expr::ErrWrap(e) | Expr::BoxNew(e) => left_span(e),
        Expr::QuestionFallback { expr, .. } | Expr::Catch { expr, .. } => left_span(expr),
        Expr::NullCoalesce { left, .. } => left_span(left),
        Expr::Downcast { obj, .. } => left_span(obj),
        Expr::Array(_) => Span { start: 0, end: 0 },
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
        Expr::Unsafe(b) => b.span,
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
        Expr::Question(e) | Expr::ErrWrap(e) | Expr::BoxNew(e) => right_span(e),
        Expr::QuestionFallback { fallback, .. } => right_span(fallback),
        Expr::NullCoalesce { right, .. } => right_span(right),
        Expr::Catch { expr, .. } => right_span(expr),
        Expr::Downcast { target_ty, .. } => target_ty.span,
        Expr::Array(_) => Span { start: 0, end: 0 },
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
        Expr::Unsafe(b) => b.span,
    }
}
