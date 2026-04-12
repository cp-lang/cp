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
    #[error("Unexpected EOF")]
    UnexpectedEof,
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    input: &'a str,
    lexer: Lexer<'a>,
    current_token: Option<(Token, Span)>,
    next_token: Option<(Token, Span)>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next().map(|(t, r)| (t, Span::from(r)));
        let next_token = lexer.next().map(|(t, r)| (t, Span::from(r)));
        Self { input, lexer, current_token, next_token }
    }

    fn bump(&mut self) -> ParseResult<(Token, Span)> {
        let current = self.current_token.take().ok_or(ParseError::UnexpectedEof)?;
        self.current_token = self.next_token.take();
        self.next_token = self.lexer.next().map(|(t, r)| (t, Span::from(r)));
        Ok(current)
    }

    fn peek(&self) -> Option<&Token> { self.current_token.as_ref().map(|(t, _)| t) }
    fn peek_span(&self) -> Span { self.current_token.as_ref().map(|(_, s)| *s).unwrap_or(Span { start: 0, end: 0 }) }

    fn expect(&mut self, expected: Token) -> ParseResult<Span> {
        let (token, span) = self.bump()?;
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) { Ok(span) } 
        else { Err(ParseError::ExpectedToken(expected, token, span)) }
    }

    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let start = self.peek_span().start;
        let mut body = Vec::new();
        while self.peek().is_some() { body.push(self.parse_module_item()?); }
        let end = self.peek_span().end;
        Ok(Module { span: Span { start, end }, body })
    }

    fn parse_module_item(&mut self) -> ParseResult<ModuleItem> {
        match self.peek() {
            Some(Token::Fn) | Some(Token::Class) | Some(Token::Interface) => Ok(ModuleItem::Decl(self.parse_decl()?)),
            _ => Ok(ModuleItem::Stmt(self.parse_stmt()?)),
        }
    }

    fn parse_decl(&mut self) -> ParseResult<Decl> {
        match self.peek() {
            Some(Token::Fn) => Ok(Decl::Func(self.parse_function()?)),
            Some(Token::Class) => Ok(Decl::Class(self.parse_class()?)),
            Some(Token::Interface) => Ok(Decl::Interface(self.parse_interface()?)),
            _ => Err(ParseError::UnexpectedToken(self.peek().unwrap().clone(), self.peek_span())),
        }
    }

    fn parse_class(&mut self) -> ParseResult<Class> {
        let start = self.expect(Token::Class)?.start;
        let ident = self.parse_ident()?;
        let mut implements = Vec::new();
        if self.peek() == Some(&Token::Implements) {
            self.bump()?;
            loop {
                implements.push(self.parse_ident()?);
                if self.peek() == Some(&Token::Comma) { self.bump()?; } else { break; }
            }
        }
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            if self.peek() == Some(&Token::Fn) { methods.push(self.parse_function()?); }
            else {
                let f_ident = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                self.expect(Token::Semicolon)?;
                fields.push(Field { span: f_ident.span, ident: f_ident, ty });
            }
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(Class { span: Span { start, end }, ident, implements, fields, methods })
    }

    fn parse_interface(&mut self) -> ParseResult<Interface> {
        let start = self.expect(Token::Interface)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            self.expect(Token::Fn)?;
            let m_ident = self.parse_ident()?;
            self.expect(Token::LParen)?;
            let params = self.parse_params()?;
            self.expect(Token::RParen)?;
            let mut return_type = None;
            if self.peek() == Some(&Token::Colon) { self.bump()?; return_type = Some(self.parse_type()?); }
            self.expect(Token::Semicolon)?;
            methods.push(MethodSig { span: m_ident.span, ident: m_ident, params, return_type });
        }
        let end = self.expect(Token::RBrace)?.end;
        Ok(Interface { span: Span { start, end }, ident, methods })
    }

    fn parse_function(&mut self) -> ParseResult<Function> {
        let start = self.expect(Token::Fn)?.start;
        let ident = self.parse_ident()?;
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;
        let mut return_type = None;
        if self.peek() == Some(&Token::Colon) { self.bump()?; return_type = Some(self.parse_type()?); }
        let body = self.parse_block_stmt()?;
        let end = body.span.end;
        Ok(Function { span: Span { start, end }, ident, params, return_type, body })
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Some(Token::Let) | Some(Token::Const) => Ok(Stmt::Var(self.parse_var_decl()?)),
            Some(Token::Return) => Ok(Stmt::Return(self.parse_return_stmt()?)),
            Some(Token::LBrace) => Ok(Stmt::Block(self.parse_block_stmt()?)),
            Some(Token::While) => Ok(Stmt::While(self.parse_while_stmt()?)),
            Some(Token::ZigEscape) => {
                let start = self.bump()?.1.start;
                self.expect(Token::LBrace)?;
                let code_start = self.peek_span().start;
                let mut brace_count = 1;
                while brace_count > 0 && self.peek().is_some() {
                    let (t, _) = self.bump()?;
                    match t { Token::LBrace => brace_count += 1, Token::RBrace => brace_count -= 1, _ => {} }
                }
                let code_end = self.peek_span().start; 
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

    fn parse_while_stmt(&mut self) -> ParseResult<WhileStmt> {
        let start = self.expect(Token::While)?.start;
        self.expect(Token::LParen)?;
        let test = self.parse_expr()?;
        self.expect(Token::RParen)?;
        let body = self.parse_stmt()?;
        let end = self.peek_span().end;
        Ok(WhileStmt { span: Span { start, end }, test, body: Box::new(body) })
    }

    fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        let (token, span) = self.bump()?;
        let kind = match token { Token::Let => VarDeclKind::Let, Token::Const => VarDeclKind::Const, _ => unreachable!() };
        let start = span.start;
        let pat = self.parse_pattern()?;
        let mut ty = None;
        if self.peek() == Some(&Token::Colon) { self.bump()?; ty = Some(self.parse_type()?); }
        let mut init = None;
        if self.peek() == Some(&Token::Assign) { self.bump()?; init = Some(self.parse_expr()?); }
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
                Ok(Pattern::Array(elements))
            },
            _ => Ok(Pattern::Ident(self.parse_ident()?)),
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

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        // High-level Arrow check could go here, but let's stick to HOF call chains
        self.parse_bin_expr(0)
    }

    fn parse_bin_expr(&mut self, min_prec: u8) -> ParseResult<Expr> {
        let mut left = self.parse_primary_expr()?;
        loop {
            match self.peek() {
                Some(Token::LParen) => {
                    self.bump()?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) { self.bump()?; }
                    }
                    let end = self.expect(Token::RParen)?.end;
                    left = Expr::Call(CallExpr { span: Span { start: left_span(&left).start, end }, callee: Box::new(left), args });
                },
                Some(Token::Dot) => {
                    self.bump()?;
                    let prop = self.parse_ident()?;
                    left = Expr::Member(MemberExpr { span: Span { start: left_span(&left).start, end: prop.span.end }, obj: Box::new(left), prop });
                },
                Some(Token::LBracket) => {
                    // --- SLICE & ACCESS SUPPORT ---
                    let start_span = self.bump()?.1;
                    let mut start_idx = None;
                    let mut end_idx = None;
                    let mut is_slice = false;

                    if self.peek() == Some(&Token::DotDot) {
                        self.bump()?;
                        is_slice = true;
                        if self.peek() != Some(&Token::RBracket) { end_idx = Some(Box::new(self.parse_expr()?)); }
                    } else {
                        let idx = self.parse_expr()?;
                        if self.peek() == Some(&Token::DotDot) {
                            self.bump()?;
                            is_slice = true;
                            start_idx = Some(Box::new(idx));
                            if self.peek() != Some(&Token::RBracket) { end_idx = Some(Box::new(self.parse_expr()?)); }
                        } else {
                            start_idx = Some(Box::new(idx));
                        }
                    }
                    let end_span = self.expect(Token::RBracket)?;
                    if is_slice {
                        left = Expr::Slice(SliceExpr { span: Span { start: left_span(&left).start, end: end_span.end }, obj: Box::new(left), start: start_idx, end: end_idx });
                    } else {
                        // Regular member access via index (simplified as MemberExpr for now)
                        left = Expr::Member(MemberExpr { span: Span { start: left_span(&left).start, end: end_span.end }, obj: Box::new(left), prop: Ident { span: start_span, sym: "at".to_string() } });
                    }
                },
                _ => break,
            }
        }
        while let Some(op) = self.peek_op() {
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
            Token::Ident(sym) => {
                // Check if it's an arrow function: x => ...
                if self.peek() == Some(&Token::FatArrow) {
                    self.bump()?;
                    let body = if self.peek() == Some(&Token::LBrace) { ArrowBody::Block(self.parse_block_stmt()?) } 
                               else { ArrowBody::Expr(self.parse_expr()?) };
                    Ok(Expr::Arrow(Box::new(ArrowExpr { span: Span { start: span.start, end: self.peek_span().end }, params: vec![Pattern::Ident(Ident { span, sym })], body: Box::new(body) })))
                } else { Ok(Expr::Ident(Ident { span, sym })) }
            },
            Token::LParen => {
                // Could be (a, b) => ... or (expr)
                // Simplified: for now only (expr)
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
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
                    Ok(Expr::New(CallExpr { span: Span { start: span.start, end }, callee: Box::new(Expr::Ident(ident)), args }))
                } else { Err(ParseError::UnexpectedToken(Token::New, span)) }
            },
            Token::Spawn => {
                let callee_expr = self.parse_primary_expr()?;
                if let Expr::Ident(ident) = callee_expr {
                    self.expect(Token::LParen)?;
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) { self.bump()?; }
                    }
                    let end = self.expect(Token::RParen)?.end;
                    Ok(Expr::Spawn(Box::new(SpawnExpr { span: Span { start: span.start, end }, callee: Box::new(Expr::Ident(ident)), args })))
                } else { Err(ParseError::UnexpectedToken(Token::Spawn, span)) }
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
                let end = self.expect(Token::RBracket)?.end;
                Ok(Expr::Array(elements))
            },
            _ => Err(ParseError::UnexpectedToken(token, span)),
        }
    }

    fn peek_op(&self) -> Option<BinaryOp> {
        match self.peek() {
            Some(Token::Plus) => Some(BinaryOp::Add), Some(Token::Minus) => Some(BinaryOp::Sub),
            Some(Token::Star) => Some(BinaryOp::Mul), Some(Token::Slash) => Some(BinaryOp::Div),
            Some(Token::Assign) => Some(BinaryOp::Eq), Some(Token::Less) => Some(BinaryOp::Lt),
            _ => None,
        }
    }

    fn op_precedence(&self, op: &BinaryOp) -> u8 {
        match op { BinaryOp::Eq => 1, BinaryOp::Lt => 2, BinaryOp::Add | BinaryOp::Sub => 3, BinaryOp::Mul | BinaryOp::Div => 4, _ => 0 }
    }

    fn parse_ident(&mut self) -> ParseResult<Ident> {
        let (token, span) = self.bump()?;
        if let Token::Ident(sym) = token { Ok(Ident { span, sym }) } 
        else { Err(ParseError::ExpectedToken(Token::Ident("Ident".to_string()), token, span)) }
    }

    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();
        while self.peek() != Some(&Token::RParen) {
            let ident = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { span: ident.span, ident, ty });
            if self.peek() == Some(&Token::Comma) { self.bump()?; }
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        let (token, span) = self.bump()?;
        match token {
            Token::I32 => Ok(Type::I32), Token::U64 => Ok(Type::U64), Token::F32 => Ok(Type::F32), Token::USize => Ok(Type::USize), Token::String => Ok(Type::String),
            Token::Ident(sym) => { if sym == "pid" { Ok(Type::PID) } else { Ok(Type::Ref(Ident { span, sym })) } },
            Token::LBracket => { self.expect(Token::RBracket)?; let elem_ty = self.parse_type()?; Ok(Type::Array(Box::new(elem_ty))) }
            _ => Err(ParseError::UnexpectedToken(token, span)),
        }
    }
}

fn left_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident(i) => i.span, Expr::Lit(_) => Span { start: 0, end: 0 }, Expr::Bin(b) => b.span,
        Expr::Call(c) => c.span, Expr::Member(m) => m.span, Expr::Slice(s) => s.span,
        _ => Span { start: 0, end: 0 },
    }
}

fn right_span(expr: &Expr) -> Span { left_span(expr) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_slice() {
        let input = "let s = arr[0..5];";
        let mut parser = Parser::new(input);
        let module = parser.parse_module().unwrap();
        assert_eq!(module.body.len(), 1);
    }
    #[test]
    fn test_parse_arrow_shorthand() {
        let input = "let f = x => x * 2;";
        let mut parser = Parser::new(input);
        let module = parser.parse_module().unwrap();
        assert_eq!(module.body.len(), 1);
    }
}
