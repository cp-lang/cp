use crate::ast::*;
use crate::lexer::{Token, Lexer};
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
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    input: &'a str,
    peeked: Option<(Token, Span)>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            input,
            peeked: None,
        }
    }

    fn bump(&mut self) -> ParseResult<(Token, Span)> {
        if let Some(p) = self.peeked.take() { return Ok(p); }
        let (token, range) = self.lexer.next().ok_or(ParseError::EndOfInput)?;
        Ok((token, Span { start: range.start, end: range.end }))
    }

    fn peek(&mut self) -> Option<&Token> {
        if self.peeked.is_none() { 
            let (token, range) = self.lexer.next()?;
            self.peeked = Some((token, Span { start: range.start, end: range.end }));
        }
        self.peeked.as_ref().map(|(t, _)| t)
    }

    fn peek_next(&mut self) -> Option<&Token> {
        None
    }

    fn peek_span(&mut self) -> Span {
        if self.peeked.is_none() { 
            if let Some((token, range)) = self.lexer.next() {
                self.peeked = Some((token, Span { start: range.start, end: range.end }));
            } else {
                return Span { start: 0, end: 0 };
            }
        }
        self.peeked.as_ref().map(|(_, s)| *s).unwrap_or(Span { start: 0, end: 0 })
    }

    fn expect(&mut self, expected: Token) -> ParseResult<Span> {
        let (token, span) = self.bump()?;
        if token == expected { Ok(span) } 
        else { Err(ParseError::ExpectedToken(expected, token, span)) }
    }

    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let mut body = Vec::new();
        let start = self.peek_span().start;
        while self.peek().is_some() {
            body.push(self.parse_module_item()?);
        }
        let end = self.peek_span().end;
        Ok(Module { span: Span { start, end }, body })
    }

    fn parse_module_item(&mut self) -> ParseResult<ModuleItem> {
        if self.peek() == Some(&Token::Export) { self.bump()?; }
        match self.peek() {
            Some(Token::Import) => Ok(ModuleItem::Import(self.parse_import_stmt()?)),
            Some(Token::Fn) | Some(Token::Class) | Some(Token::Interface) | Some(Token::ErrorKw) |
            Some(Token::Trait) | Some(Token::Impl) | Some(Token::Enum) | Some(Token::Async) => {
                Ok(ModuleItem::Decl(self.parse_decl()?))
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
            if self.peek() == Some(&Token::Comma) { self.bump()?; }
        }
        self.expect(Token::RBrace)?;
        self.expect(Token::From)?;
        let (token, span) = self.bump()?;
        let source = if let Token::StringLiteral(s) = token { 
            s
        } else { return Err(ParseError::ExpectedToken(Token::StringLiteral("source".to_string()), token, span)); };
        if self.peek() == Some(&Token::Semicolon) { self.bump()?; }
        let end = span.end;
        Ok(ImportStmt { span: Span { start, end }, specifiers, source })
    }

    fn parse_decl(&mut self) -> ParseResult<Decl> {
        match self.peek() {
            Some(Token::Fn) | Some(Token::Async) => Ok(Decl::Func(self.parse_function()?)),
            Some(Token::Class) => Ok(Decl::Class(self.parse_class()?)),
            Some(Token::Trait) => Ok(Decl::Trait(self.parse_trait()?)),
            Some(Token::Impl) => Ok(Decl::Impl(self.parse_impl()?)),
            Some(Token::Enum) => Ok(Decl::Enum(self.parse_enum()?)),
            Some(Token::ErrorKw) => Ok(Decl::ErrorSet(self.parse_error_set()?)),
            _ => Err(ParseError::UnexpectedToken(self.bump()?.0, self.peek_span())),
        }
    }

    fn parse_function(&mut self) -> ParseResult<Function> {
        let start = self.peek_span().start;
        let mut is_async = false;
        if self.peek() == Some(&Token::Async) { self.bump()?; is_async = true; }
        self.expect(Token::Fn)?;
        let ident = self.parse_ident()?;
        let params = self.parse_params()?;
        let mut return_type = None;
        if self.peek() == Some(&Token::Colon) { self.bump()?; return_type = Some(self.parse_type()?); }
        
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
        Ok(Function { span: Span { start, end }, ident, params, return_type, body, is_async })
    }

    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while self.peek() != Some(&Token::RParen) {
            let p_ident = self.parse_ident()?;
            let ty = if p_ident.sym == "this" && self.peek() != Some(&Token::Colon) {
                Type::Any
            } else {
                self.expect(Token::Colon)?;
                self.parse_type()?
            };
            params.push(Param { span: p_ident.span, ident: p_ident, ty, is_comptime: false });
            if self.peek() == Some(&Token::Comma) { self.bump()?; }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        if self.peek() == Some(&Token::LBracket) {
            self.bump()?;
            let mut elements = Vec::new();
            while self.peek() != Some(&Token::RBracket) {
                elements.push(self.parse_type()?);
                if self.peek() == Some(&Token::Comma) { self.bump()?; }
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
                fields.push(Field { span: f_ident.span, ident: f_ident, ty });
                if self.peek() == Some(&Token::Comma) { self.bump()?; }
            }
            self.expect(Token::RBrace)?;
            return Ok(Type::Object(fields));
        }
        let is_error_union = if self.peek() == Some(&Token::Question) { self.bump()?; true } else { false };
        let (token, span) = self.bump()?;
        let mut ty = match token {
            Token::I32 => Type::I32, Token::U32 => Type::U32, Token::U64 => Type::U64, Token::F32 => Type::F32, Token::USize => Type::USize, Token::String => Type::String, Token::Void => Type::Void, Token::Any => Type::Any, Token::Bool => Type::Bool,
            Token::Ident(sym) => { if sym == "pid" { Type::PID } else { Type::Ref(Ident { span, sym }) } },
            _ => return Err(ParseError::UnexpectedToken(token, span)),
        };
        if self.peek() == Some(&Token::Question) { self.bump()?; ty = Type::Optional(Box::new(ty)); }
        if is_error_union { ty = Type::ErrorUnion(Box::new(ty)); }
        Ok(ty)
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Some(Token::Let) | Some(Token::Const) => Ok(Stmt::Var(self.parse_var_decl()?)),
            Some(Token::While) => Ok(Stmt::While(self.parse_while_stmt()?)),
            Some(Token::If) => Ok(Stmt::If(self.parse_if_stmt()?)),
            Some(Token::Return) => Ok(Stmt::Return(self.parse_return_stmt()?)),
            Some(Token::Defer) => { self.bump()?; Ok(Stmt::Defer(Box::new(self.parse_stmt()?))) },
            Some(Token::ErrDefer) => { self.bump()?; Ok(Stmt::ErrDefer(Box::new(self.parse_stmt()?))) },
            Some(Token::Bang) => {
                if self.peek_next() == Some(&Token::LBrace) { self.bump()?; Ok(Stmt::ComptimeBlock(self.parse_block_stmt()?)) }
                else { Ok(Stmt::Expr(self.parse_expr()?)) }
            },
            Some(Token::LBrace) => Ok(Stmt::Block(self.parse_block_stmt()?)),
            Some(Token::ZigEscape) => {
                let start = self.bump()?.1.start;
                self.expect(Token::LBrace)?;
                let code_start = self.peek_span().start;
                let mut brace_count = 1;
                let mut code_end = code_start;
                while brace_count > 0 && self.peek().is_some() {
                    let (t, span) = self.bump()?;
                    match t { Token::LBrace => brace_count += 1, Token::RBrace => brace_count -= 1, _ => {} }
                    if brace_count == 0 { code_end = span.start; }
                }
                let end = self.peek_span().end;
                let raw_code = &self.input[code_start..code_end]; 
                Ok(Stmt::Expr(Expr::Zig(ZigEscapeExpr { span: Span { start, end }, code: raw_code.trim().to_string() })))
            },
            _ => {
                let expr = self.parse_expr()?;
                if self.peek() == Some(&Token::Semicolon) { self.bump()?; }
                Ok(Stmt::Expr(expr))
            },
        }
    }

    fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        let start = self.peek_span().start;
        let kind = match self.bump()?.0 { Token::Let => VarDeclKind::Let, Token::Const => VarDeclKind::Const, _ => unreachable!() };
        let pat = self.parse_pattern()?;
        let mut ty = None;
        if self.peek() == Some(&Token::Colon) { self.bump()?; ty = Some(self.parse_type()?); }
        self.expect(Token::Assign)?;
        let init = Some(self.parse_expr()?);
        self.expect(Token::Semicolon)?;
        let end = self.peek_span().end;
        Ok(VarDecl { span: Span { start, end }, kind, pat, ty, init })
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.peek() {
            Some(Token::LBracket) => {
                let mut elements = Vec::new();
                self.bump()?;
                while self.peek() != Some(&Token::RBracket) {
                    elements.push(self.parse_pattern()?);
                    if self.peek() == Some(&Token::Comma) { self.bump()?; }
                }
                self.expect(Token::RBracket)?;
                Ok(Pattern::Tuple(elements))
            },
            Some(Token::LBrace) => {
                let _start = self.bump()?.1.start;
                let mut props = Vec::new();
                while self.peek() != Some(&Token::RBrace) {
                    let key = self.parse_ident()?;
                    let mut val = None;
                    if self.peek() == Some(&Token::Colon) { self.bump()?; val = Some(self.parse_pattern()?); }
                    props.push(ObjectPatProp { key, val });
                    if self.peek() == Some(&Token::Comma) { self.bump()?; }
                }
                let _end = self.expect(Token::RBrace)?.end;
                Ok(Pattern::Object(props))
            },
            Some(Token::Dot) => {
                self.bump()?;
                let variant_name = self.parse_ident()?;
                let mut fields = None;
                if self.peek() == Some(&Token::FatArrow) { // No payload
                } else if self.peek() == Some(&Token::Pipe) {
                    self.bump()?;
                    let key = self.parse_ident()?;
                    fields = Some(vec![ObjectPatProp { key, val: None }]);
                    self.expect(Token::Pipe)?;
                }
                Ok(Pattern::Enum(EnumPattern { span: Span { start: 0, end: 0 }, enum_name: Ident { span: Span { start: 0, end: 0 }, sym: "".to_string() }, variant_name, fields }))
            },
            _ => {
                if self.peek() == Some(&Token::Bang) { self.bump()?; return Ok(Pattern::Wildcard); } // Hack for _
                Ok(Pattern::Ident(self.parse_ident()?))
            }
        }
    }

    fn parse_block_stmt(&mut self) -> ParseResult<BlockStmt> {
        let start = self.expect(Token::LBrace)?.start;
        let mut body = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() { body.push(self.parse_stmt()?); }
        let end = self.expect(Token::RBrace)?.end;
        Ok(BlockStmt { span: Span { start, end }, body })
    }

    fn parse_return_stmt(&mut self) -> ParseResult<ReturnStmt> {
        let start = self.expect(Token::Return)?.start;
        let mut arg = None;
        if self.peek() != Some(&Token::Semicolon) { arg = Some(self.parse_expr()?); }
        let end = self.expect(Token::Semicolon)?.end;
        Ok(ReturnStmt { span: Span { start, end }, arg })
    }

    fn parse_match_expr(&mut self) -> ParseResult<MatchExpr> {
        let start = self.peek_span().start;
        self.expect(Token::LParen)?;
        let disc = self.parse_expr()?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let mut arms = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            let pat = self.parse_pattern()?;
            self.expect(Token::FatArrow)?;
            let body = self.parse_stmt()?;
            if self.peek() == Some(&Token::Comma) { self.bump()?; }
            arms.push(MatchArm { span: Span { start: 0, end: 0 }, pat, body: Box::new(body) });
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(MatchExpr { span: Span { start, end }, disc: Box::new(disc), arms })
    }

    fn parse_while_stmt(&mut self) -> ParseResult<WhileStmt> {
        let start = self.expect(Token::While)?.start;
        self.expect(Token::LParen)?;
        let test = self.parse_expr()?;
        self.expect(Token::RParen)?;
        let body = self.parse_stmt()?;
        Ok(WhileStmt { span: Span { start, end: 0 }, test, body: Box::new(body) })
    }

    fn parse_if_stmt(&mut self) -> ParseResult<IfStmt> {
        let start = self.expect(Token::If)?.start;
        self.expect(Token::LParen)?;
        let test = self.parse_expr()?;
        self.expect(Token::RParen)?;
        let cons = self.parse_stmt()?;
        let mut alt = None;
        if self.peek() == Some(&Token::Else) { self.bump()?; alt = Some(Box::new(self.parse_stmt()?)); }
        Ok(IfStmt { span: Span { start, end: 0 }, test, cons: Box::new(cons), alt })
    }

    fn parse_class(&mut self) -> ParseResult<Class> {
        let start = self.expect(Token::Class)?.start;
        let ident = self.parse_ident()?;
        
        let mut implements = Vec::new();
        if self.peek() == Some(&Token::Extends) {
            self.bump()?;
            implements.push(self.parse_ident()?);
        }
        
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        self.expect(Token::LBrace)?;
        while self.peek() != Some(&Token::RBrace) {
            if self.peek() == Some(&Token::Fn) || self.peek() == Some(&Token::Async) {
                methods.push(self.parse_function()?);
            } else {
                let f_ident = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                if self.peek() == Some(&Token::Comma) || self.peek() == Some(&Token::Semicolon) { self.bump()?; }
                fields.push(Field { span: f_ident.span, ident: f_ident, ty });
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(Class { span: Span { start, end }, ident, fields, methods, implements })
    }

    fn parse_trait(&mut self) -> ParseResult<TraitDecl> {
        let start = self.expect(Token::Trait)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            let m_start = self.peek_span().start;
            self.expect(Token::Fn)?;
            let m_ident = self.parse_ident()?;
            let params = self.parse_params()?;
            let mut return_type = None;
            if self.peek() == Some(&Token::Colon) { self.bump()?; return_type = Some(self.parse_type()?); }
            self.expect(Token::Semicolon)?;
            methods.push(MethodSig { span: Span { start: m_start, end: self.peek_span().start }, ident: m_ident, params, return_type });
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(TraitDecl { span: Span { start, end }, ident, methods })
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
        Ok(ImplDecl { span: Span { start, end }, trait_name, target_name, methods })
    }

    fn parse_enum(&mut self) -> ParseResult<EnumDecl> {
        let start = self.expect(Token::Enum)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            let v_ident = self.parse_ident()?;
            let mut fields = None;
            if self.peek() == Some(&Token::LBrace) {
                self.bump()?;
                let mut f_list = Vec::new();
                while self.peek() != Some(&Token::RBrace) {
                    let f_ident = self.parse_ident()?;
                    self.expect(Token::Colon)?;
                    let ty = self.parse_type()?;
                    f_list.push(Field { span: f_ident.span, ident: f_ident, ty });
                    if self.peek() == Some(&Token::Comma) { self.bump()?; }
                }
                self.expect(Token::RBrace)?;
                fields = Some(f_list);
            }
            let mut value = None;
            if self.peek() == Some(&Token::Assign) {
                self.bump()?;
                value = Some(self.parse_expr()?);
            }
            variants.push(EnumVariant { span: v_ident.span, ident: v_ident, fields, value });
            if self.peek() == Some(&Token::Comma) { self.bump()?; }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(EnumDecl { span: Span { start, end }, ident, variants })
    }

    fn parse_error_set(&mut self) -> ParseResult<ErrorSetDecl> {
        let start = self.expect(Token::ErrorKw)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            variants.push(self.parse_ident()?);
            if self.peek() == Some(&Token::Comma) { self.bump()?; }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(ErrorSetDecl { span: Span { start, end }, ident, variants })
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
                        if self.peek_next() == Some(&Token::LParen) { self.bump()?; is_comptime = true; } 
                        else { break; }
                    }
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) { self.bump()?; }
                    }
                    let end = self.expect(Token::RParen)?.end;
                    left = Expr::Call(CallExpr { span: Span { start: left_span(&left).start, end }, callee: Box::new(left), args, is_comptime });
                },
                Some(Token::Dot) => {
                    self.bump()?;
                    let prop = self.parse_ident()?;
                    if self.peek() == Some(&Token::LBrace) {
                        self.bump()?;
                        let mut fields = Vec::new();
                        while self.peek() != Some(&Token::RBrace) {
                            let key = self.parse_ident()?;
                            self.expect(Token::Colon)?;
                            let val = self.parse_expr()?;
                            fields.push(EnumInitField { key, val });
                            if self.peek() == Some(&Token::Comma) { self.bump()?; }
                        }
                        let end = self.expect(Token::RBrace)?.end;
                        let enum_name = match &left {
                            Expr::Ident(id) => id.clone(),
                            _ => Ident { span: Span { start: 0, end: 0 }, sym: "Unknown".to_string() },
                        };
                        left = Expr::EnumInit(EnumInitExpr { span: Span { start: left_span(&left).start, end }, enum_name, variant_name: prop, fields: Some(fields) });
                    } else {
                        left = Expr::Member(MemberExpr { span: Span { start: left_span(&left).start, end: prop.span.end }, obj: Box::new(left), prop });
                    }
                },
                Some(Token::Question) => { self.bump()?; left = Expr::Question(Box::new(left)); },
                Some(Token::LBracket) => {
                    let start_span = self.bump()?.1;
                    let mut start_idx = None;
                    let mut end_idx = None;
                    let mut is_slice = false;
                    if self.peek() == Some(&Token::DotDot) {
                        self.bump()?; is_slice = true;
                        if self.peek() != Some(&Token::RBracket) { end_idx = Some(Box::new(self.parse_expr()?)); }
                    } else {
                        let idx = self.parse_expr()?;
                        if self.peek() == Some(&Token::DotDot) {
                            self.bump()?; is_slice = true;
                            start_idx = Some(Box::new(idx));
                            if self.peek() != Some(&Token::RBracket) { end_idx = Some(Box::new(self.parse_expr()?)); }
                        } else { start_idx = Some(Box::new(idx)); }
                    }
                    let end_span = self.expect(Token::RBracket)?;
                    if is_slice { left = Expr::Slice(SliceExpr { span: Span { start: left_span(&left).start, end: end_span.end }, obj: Box::new(left), start: start_idx, end: end_idx }); } 
                    else { left = Expr::Member(MemberExpr { span: Span { start: left_span(&left).start, end: end_span.end }, obj: Box::new(left), prop: Ident { span: start_span, sym: "at".to_string() } }); }
                },
                Some(Token::FatArrow) => {
                    self.bump()?;
                    let body = if self.peek() == Some(&Token::LBrace) { ArrowBody::Block(self.parse_block_stmt()?) } 
                               else { ArrowBody::Expr(self.parse_expr()?) };
                    let params = match left {
                        Expr::Ident(ref id) => vec![Pattern::Ident(id.clone())],
                        Expr::Array(ref elements) => {
                            let mut pats = Vec::new();
                            for e in elements {
                                if let Expr::Ident(id) = e { pats.push(Pattern::Ident(id.clone())); }
                                else { pats.push(Pattern::Wildcard); }
                            }
                            pats
                        },
                        _ => vec![],
                    };
                    left = Expr::Arrow(Box::new(ArrowExpr { span: Span { start: left_span(&left).start, end: self.peek_span().end }, params, body: Box::new(body) }));
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
            if prec < min_prec { break; }
            self.bump()?;
            let right = self.parse_bin_expr(prec + 1)?;
            let start = left_span(&left).start;
            let end = right_span(&right).end;
            left = Expr::Bin(BinExpr { span: Span { start, end }, op, left: Box::new(left), right: Box::new(right) });
        }
        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> ParseResult<Expr> {
        let (token, span) = self.bump()?;
        match token {
            Token::Spawn => {
                let callee_expr = self.parse_expr()?;
                Ok(Expr::Spawn(Box::new(SpawnExpr { span: Span { start: span.start, end: right_span(&callee_expr).end }, callee: Box::new(callee_expr), args: vec![] })))
            },
            Token::Ident(sym) => {
                if sym.starts_with('@') {
                    let name = sym.clone();
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                        args.push(self.parse_bin_expr(0)?);
                        if self.peek() == Some(&Token::Comma) { self.bump()?; }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expr::Builtin(BuiltinCall { span: Span { start: span.start, end: self.peek_span().end }, name, args }))
                } else {
                    Ok(Expr::Ident(Ident { span, sym }))
                }
            },
            Token::At => {
                let ident = self.parse_ident()?;
                let name = format!("@{}", ident.sym);
                self.expect(Token::LParen)?;
                let mut args = Vec::new();
                while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                    args.push(self.parse_bin_expr(0)?);
                    if self.peek() == Some(&Token::Comma) { self.bump()?; }
                }
                self.expect(Token::RParen)?;
                Ok(Expr::Builtin(BuiltinCall { span: Span { start: span.start, end: self.peek_span().end }, name, args }))
            },
            Token::Match => { let match_expr = self.parse_match_expr()?; Ok(Expr::Match(Box::new(match_expr))) },
            Token::LParen => { 
                let checkpoint = self.lexer.clone();
                let peeked_checkpoint = self.peeked.clone();
                
                // Try parsing as arrow function parameters
                if let Ok(params) = self.parse_arrow_params() {
                    if self.peek() == Some(&Token::FatArrow) {
                        self.bump()?;
                        let body = if self.peek() == Some(&Token::LBrace) { ArrowBody::Block(self.parse_block_stmt()?) } 
                                   else { ArrowBody::Expr(self.parse_expr()?) };
                        return Ok(Expr::Arrow(Box::new(ArrowExpr { span: Span { start: span.start, end: self.peek_span().end }, params, body: Box::new(body) })));
                    }
                }
                
                // Fallback to grouped expression
                self.lexer = checkpoint;
                self.peeked = peeked_checkpoint;
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            },
            Token::LBrace => {
                let mut fields = Vec::new();
                while self.peek() != Some(&Token::RBrace) {
                    let key = self.parse_ident()?;
                    self.expect(Token::Colon)?;
                    let val = self.parse_expr()?;
                    fields.push(ObjectField { key, val });
                    if self.peek() == Some(&Token::Comma) { self.bump()?; }
                }
                self.expect(Token::RBrace)?;
                Ok(Expr::Object(fields))
            },
            Token::Await => {
                let expr = self.parse_expr()?;
                Ok(Expr::Await(Box::new(expr)))
            },
            Token::IntLiteral(v) => Ok(Expr::Lit(Lit::Int(v))),
            Token::StringLiteral(s) => Ok(Expr::Lit(Lit::Str(s))),
            Token::New => {
                let callee_expr = self.parse_primary_expr()?;
                if let Expr::Ident(ident) = callee_expr {
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) { self.bump()?; }
                    }
                    let end = self.expect(Token::RParen)?.end;
                    Ok(Expr::New(CallExpr { span: Span { start: span.start, end }, callee: Box::new(Expr::Ident(ident)), args, is_comptime: false }))
                } else { Err(ParseError::UnexpectedToken(Token::New, span)) }
            },
            Token::Receive => {
                let mut ty = None;
                if self.peek() == Some(&Token::Less) { self.bump()?; ty = Some(Box::new(self.parse_type()?)); self.expect(Token::Greater)?; }
                self.expect(Token::LParen)?; self.expect(Token::RParen)?;
                Ok(Expr::Receive(ty))
            },
            Token::LBracket => {
                let mut elements = Vec::new();
                while self.peek() != Some(&Token::RBracket) {
                    elements.push(self.parse_expr()?);
                    if self.peek() == Some(&Token::Comma) { self.bump()?; }
                }
                let _end = self.expect(Token::RBracket)?.end;
                Ok(Expr::Array(elements))
            },
            _ => Err(ParseError::UnexpectedToken(token, span)),
        }
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
            if self.peek() == Some(&Token::Comma) { self.bump()?; }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    fn parse_ident(&mut self) -> ParseResult<Ident> {
        let (token, span) = self.bump()?;
        let sym = match token {
            Token::Ident(s) => s,
            Token::Spawn => "spawn".to_string(),
            Token::Receive => "receive".to_string(),
            Token::Await => "await".to_string(),
            Token::Async => "async".to_string(),
            Token::If => "if".to_string(),
            Token::While => "while".to_string(),
            Token::For => "for".to_string(),
            Token::Return => "return".to_string(),
            Token::Class => "class".to_string(),
            Token::Trait => "trait".to_string(),
            Token::Enum => "enum".to_string(),
            Token::Impl => "impl".to_string(),
            Token::Interface => "interface".to_string(),
            Token::Extends => "extends".to_string(),
            Token::Any => "any".to_string(),
            Token::Void => "void".to_string(),
            _ => return Err(ParseError::ExpectedToken(Token::Ident("Ident".to_string()), token, span)),
        };
        Ok(Ident { span, sym })
    }

    fn peek_op(&mut self) -> Option<BinaryOp> {
        match self.peek() {
            Some(Token::Plus) => Some(BinaryOp::Add),
            Some(Token::Minus) => Some(BinaryOp::Sub),
            Some(Token::Star) => Some(BinaryOp::Mul),
            Some(Token::Slash) => Some(BinaryOp::Div),
            Some(Token::Assign) => Some(BinaryOp::Eq),
            Some(Token::Less) => Some(BinaryOp::Lt),
            _ => None,
        }
    }

    fn op_precedence(&self, op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::Eq => 1,
            BinaryOp::Lt => 2,
            BinaryOp::Add | BinaryOp::Sub => 3,
            BinaryOp::Mul | BinaryOp::Div => 4,
            _ => 0,
        }
    }
}

fn left_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident(i) => i.span,
        Expr::Lit(_) => Span { start: 0, end: 0 },
        Expr::Bin(b) => b.span,
        Expr::Call(c) => c.span,
        Expr::New(c) => c.span,
        Expr::Member(m) => m.span,
        Expr::Slice(s) => s.span,
        Expr::Match(m) => m.span,
        Expr::Spawn(s) => s.span,
        Expr::Receive(_) => Span { start: 0, end: 0 },
        Expr::Zig(z) => z.span,
        Expr::Question(e) => left_span(e),
        Expr::Array(_) => Span { start: 0, end: 0 },
        Expr::Object(_) => Span { start: 0, end: 0 },
        Expr::Arrow(a) => a.span,
        Expr::EnumInit(e) => e.span,
        Expr::Template(t) => t.span,
        Expr::Await(e) => left_span(e),
        Expr::Builtin(b) => b.span,
    }
}

fn right_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident(i) => i.span,
        Expr::Lit(_) => Span { start: 0, end: 0 },
        Expr::Bin(b) => right_span(&b.right),
        Expr::Call(c) => Span { start: 0, end: c.span.end },
        Expr::New(c) => Span { start: 0, end: c.span.end },
        Expr::Member(m) => m.span,
        Expr::Slice(s) => s.span,
        Expr::Match(m) => m.span,
        Expr::Spawn(s) => s.span,
        Expr::Receive(_) => Span { start: 0, end: 0 },
        Expr::Zig(z) => z.span,
        Expr::Question(e) => right_span(e),
        Expr::Array(_) => Span { start: 0, end: 0 },
        Expr::Object(_) => Span { start: 0, end: 0 },
        Expr::Arrow(a) => a.span,
        Expr::EnumInit(e) => e.span,
        Expr::Template(t) => t.span,
        Expr::Await(e) => right_span(e),
        Expr::Builtin(b) => b.span,
    }
}
