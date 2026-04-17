use crate::ast::*;
use crate::Span;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemaError {
    #[error("Undefined symbol: {0} at {1:?}")] UndefinedSymbol(String, Span),
    #[error("Type mismatch: expected {0:?}, found {1:?}")] TypeMismatch(Type, Type, Span),
    #[error("Not a function at {0:?}")] NotAFunction(Span),
    #[error("Const assignment to {0} at {1:?}")] ConstAssignment(String, Span),
    #[error("Invalid await at {0:?}")] InvalidAwait(Span),
    #[error("Await can only be used in async functions at {0:?}")] AwaitOutsideAsync(Span),
}

pub type SemaResult<T> = Result<T, SemaError>;

#[derive(Clone)]
pub enum SymbolKind {
    Var { ty: Type, kind: VarDeclKind },
    Func { params: Vec<Type>, ret: Option<Type>, is_async: bool },
    Class(Class),
    Trait(TraitDecl),
    Enum(EnumDecl),
}

pub struct Analyzer {
    scopes: Vec<HashMap<String, SymbolKind>>,
    classes: HashMap<String, Class>,
    current_fn_is_async: bool,
    pub symbol_map: HashMap<Span, SymbolKind>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()], classes: HashMap::new(), current_fn_is_async: false, symbol_map: HashMap::new() }
    }

    pub fn analyze_module(&mut self, module: &Module) -> SemaResult<()> {
        for item in &module.body {
            match item {
                ModuleItem::Decl(Decl::Class(c)) => {
                    self.classes.insert(c.ident.sym.clone(), c.clone());
                    self.define(c.ident.sym.clone(), SymbolKind::Class(c.clone()), c.ident.span)?;
                    self.symbol_map.insert(c.ident.span, SymbolKind::Class(c.clone()));
                }
                ModuleItem::Decl(Decl::Trait(t)) => {
                    self.define(t.ident.sym.clone(), SymbolKind::Trait(t.clone()), t.ident.span)?;
                    self.symbol_map.insert(t.ident.span, SymbolKind::Trait(t.clone()));
                }
                ModuleItem::Decl(Decl::Enum(e)) => {
                    self.define(e.ident.sym.clone(), SymbolKind::Enum(e.clone()), e.ident.span)?;
                    self.symbol_map.insert(e.ident.span, SymbolKind::Enum(e.clone()));
                }
                _ => {}
            }
        }
        for item in &module.body {
            match item {
                ModuleItem::Decl(decl) => self.analyze_decl(decl)?,
                ModuleItem::Stmt(stmt) => self.analyze_stmt(stmt)?,
                ModuleItem::Import(_) => {}
            }
        }
        Ok(())
    }

    fn analyze_decl(&mut self, decl: &Decl) -> SemaResult<()> {
        match decl {
            Decl::Func(f) => {
                let params = f.params.iter().map(|p| p.ty.clone()).collect();
                let kind = SymbolKind::Func { params, ret: f.return_type.clone(), is_async: f.is_async };
                self.define(f.ident.sym.clone(), kind.clone(), f.ident.span)?;
                self.symbol_map.insert(f.ident.span, kind);

                let old_async = self.current_fn_is_async;
                self.current_fn_is_async = f.is_async;
                self.enter_scope();
                for p in &f.params {
                    let p_kind = SymbolKind::Var { ty: p.ty.clone(), kind: VarDeclKind::Let };
                    self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span)?;
                    self.symbol_map.insert(p.ident.span, p_kind);
                }
                for stmt in &f.body.body { self.analyze_stmt(stmt)?; }
                self.exit_scope();
                self.current_fn_is_async = old_async;
            },
            Decl::Class(c) => {
                for m in &c.methods {
                    let old_async = self.current_fn_is_async;
                    self.current_fn_is_async = m.is_async;
                    self.analyze_block(&m.body)?;
                    self.current_fn_is_async = old_async;
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> SemaResult<()> {
        match stmt {
            Stmt::Var(var) => {
                let mut inferred_ty = Type::Unknown;
                if let Some(init) = &var.init { 
                    inferred_ty = self.infer_type(init)?; 
                    self.analyze_expr(init)?;
                }
                let ty = var.ty.clone().unwrap_or(inferred_ty);
                self.analyze_pattern(&var.pat, ty, &var.kind)?;
            },
            Stmt::Expr(expr) => { 
                self.infer_type(expr)?; 
                self.analyze_expr(expr)?;
            },
            Stmt::Block(block) => self.analyze_block(block)?,
            Stmt::If(i) => { 
                self.infer_type(&i.test)?; 
                self.analyze_expr(&i.test)?;
                self.analyze_stmt(&i.cons)?; 
                if let Some(alt) = &i.alt { self.analyze_stmt(alt)?; } 
            },
            Stmt::While(w) => { 
                self.infer_type(&w.test)?; 
                self.analyze_expr(&w.test)?;
                self.analyze_stmt(&w.body)?; 
            },
            Stmt::Return(r) => { 
                if let Some(arg) = &r.arg { 
                    self.infer_type(arg)?; 
                    self.analyze_expr(arg)?;
                } 
            },
            _ => {}
        }
        Ok(())
    }

    fn analyze_expr(&mut self, expr: &Expr) -> SemaResult<()> {
        match expr {
            Expr::Ident(ident) => {
                if let Some(kind) = self.resolve(&ident.sym) {
                    self.symbol_map.insert(ident.span, kind.clone());
                }
            }
            Expr::Bin(bin) => {
                self.analyze_expr(&bin.left)?;
                self.analyze_expr(&bin.right)?;
            }
            Expr::Call(call) => {
                self.analyze_expr(&call.callee)?;
                for arg in &call.args { self.analyze_expr(arg)?; }
            }
            Expr::New(call) => {
                self.analyze_expr(&call.callee)?;
                for arg in &call.args { self.analyze_expr(arg)?; }
            }
            Expr::Member(member) => {
                self.analyze_expr(&member.obj)?;
            }
            Expr::Slice(slice) => {
                self.analyze_expr(&slice.obj)?;
                if let Some(s) = &slice.start { self.analyze_expr(s)?; }
                if let Some(e) = &slice.end { self.analyze_expr(e)?; }
            }
            Expr::Match(m) => {
                self.analyze_expr(&m.disc)?;
                for arm in &m.arms {
                    self.analyze_stmt(&arm.body)?;
                }
            }
            Expr::Array(elements) => {
                for el in elements { self.analyze_expr(el)?; }
            }
            Expr::Object(fields) => {
                for f in fields { self.analyze_expr(&f.val)?; }
            }
            Expr::Await(inner) => {
                self.analyze_expr(inner)?;
            }
            Expr::Builtin(call) => {
                for arg in &call.args { self.analyze_expr(arg)?; }
            }
            _ => {}
        }
        Ok(())
    }

    fn enter_scope(&mut self) { self.scopes.push(HashMap::new()); }
    fn exit_scope(&mut self) { self.scopes.pop(); }
    fn define(&mut self, name: String, kind: SymbolKind, _span: Span) -> SemaResult<()> {
        if self.scopes.last().unwrap().contains_key(&name) {
            // Already defined in current scope
        }
        self.scopes.last_mut().unwrap().insert(name, kind);
        Ok(())
    }
    fn resolve(&self, name: &str) -> Option<&SymbolKind> {
        for scope in self.scopes.iter().rev() { if let Some(kind) = scope.get(name) { return Some(kind); } }
        None
    }

    fn types_eq(&self, a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::Ref(t1), Type::Ref(t2)) => t1.sym == t2.sym,
            (Type::Optional(t1), Type::Optional(t2)) => self.types_eq(t1, t2),
            (Type::ErrorUnion(t1), Type::ErrorUnion(t2)) => self.types_eq(t1, t2),
            (Type::Array(t1), Type::Array(t2)) => self.types_eq(t1, t2),
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                if t1.len() != t2.len() { return false; }
                for (i, p) in t1.iter().enumerate() { if !self.types_eq(p, &t2[i]) { return false; } }
                true
            },
            (Type::Object(f1), Type::Object(f2)) => {
                if f1.len() != f2.len() { return false; }
                for (i, f) in f1.iter().enumerate() {
                    if f.ident.sym != f2[i].ident.sym || !self.types_eq(&f.ty, &f2[i].ty) { return false; }
                }
                true
            },
            (Type::Fn(p1, r1), Type::Fn(p2, r2)) => {
                if p1.len() != p2.len() { return false; }
                for (i, p) in p1.iter().enumerate() { if !self.types_eq(p, &p2[i]) { return false; } }
                self.types_eq(r1, r2)
            },
            (Type::Void, Type::Void) => true,
            (Type::Any, _) | (_, Type::Any) => true,
            _ => a == b,
        }
    }

    fn analyze_pattern(&mut self, pat: &Pattern, ty: Type, kind: &VarDeclKind) -> SemaResult<()> {
        match pat {
            Pattern::Ident(ident) => self.define(ident.sym.clone(), SymbolKind::Var { ty, kind: kind.clone() }, ident.span),
            Pattern::Tuple(elements) => {
                let element_types = match ty {
                    Type::Tuple(tys) => tys,
                    _ => return Err(SemaError::TypeMismatch(Type::Tuple(vec![]), ty, Span { start: 0, end: 0 })),
                };
                if elements.len() != element_types.len() {
                    return Err(SemaError::TypeMismatch(Type::Tuple(vec![]), Type::Tuple(element_types), Span { start: 0, end: 0 }));
                }
                for (i, el) in elements.iter().enumerate() {
                    self.analyze_pattern(el, element_types[i].clone(), kind)?;
                }
                Ok(())
            },
            Pattern::Object(props) => {
                let fields = match ty {
                    Type::Object(fs) => fs,
                    _ => return Err(SemaError::TypeMismatch(Type::Object(vec![]), ty, Span { start: 0, end: 0 })),
                };
                for prop in props {
                    let field = fields.iter().find(|f| f.ident.sym == prop.key.sym)
                        .ok_or_else(|| SemaError::UndefinedSymbol(prop.key.sym.clone(), prop.key.span))?;
                    if let Some(inner_pat) = &prop.val {
                        self.analyze_pattern(inner_pat, field.ty.clone(), kind)?;
                    } else {
                        self.analyze_pattern(&Pattern::Ident(prop.key.clone()), field.ty.clone(), kind)?;
                    }
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }

    fn analyze_block(&mut self, block: &BlockStmt) -> SemaResult<()> {
        self.enter_scope();
        for stmt in &block.body { self.analyze_stmt(stmt)?; }
        self.exit_scope();
        Ok(())
    }

    fn infer_type(&self, expr: &Expr) -> SemaResult<Type> {
        match expr {
            Expr::Lit(lit) => match lit {
                Lit::Int(_) => Ok(Type::I32), Lit::Float(_) => Ok(Type::F32), Lit::Bool(_) => Ok(Type::Bool), Lit::Str(_) => Ok(Type::String), Lit::Null => Ok(Type::Unknown),
            },
            Expr::Ident(ident) => {
                if ident.sym == "true" || ident.sym == "false" { return Ok(Type::Bool); }
                if ident.sym == "null" { return Ok(Type::Optional(Box::new(Type::Unknown))); }
                if ident.sym.starts_with('@') { return Ok(Type::Fn(vec![], Box::new(Type::Unknown))); }
                match self.resolve(&ident.sym) {
                    Some(SymbolKind::Var { ty, .. }) => Ok(ty.clone()),
                    Some(SymbolKind::Func { params, ret, is_async: _ }) => Ok(Type::Fn(params.clone(), Box::new(ret.clone().unwrap_or(Type::I32)))),
                    Some(SymbolKind::Class(c)) => Ok(Type::Ref(c.ident.clone())),
                    _ => Err(SemaError::UndefinedSymbol(ident.sym.clone(), ident.span)),
                }
            },
            Expr::Bin(bin) => {
                let left = self.infer_type(&bin.left)?;
                let right = self.infer_type(&bin.right)?;
                if bin.op == BinaryOp::Eq {
                    if let Expr::Ident(ident) = &*bin.left {
                        if let Some(SymbolKind::Var { kind: VarDeclKind::Const, .. }) = self.resolve(&ident.sym) { return Err(SemaError::ConstAssignment(ident.sym.clone(), bin.span)); }
                    }
                    if !self.types_eq(&left, &right) { return Err(SemaError::TypeMismatch(left, right, bin.span)); }
                    Ok(left)
                } else if bin.op == BinaryOp::Lt { Ok(Type::Bool) } 
                else { if !self.types_eq(&left, &right) { return Err(SemaError::TypeMismatch(left, right, bin.span)); } Ok(left) }
            },
            Expr::Call(call) => {
                let callee_ty = self.infer_type(&call.callee)?;
                match callee_ty { Type::Fn(_, ret) => Ok(*ret), _ => Err(SemaError::NotAFunction(call.span)) }
            },
            Expr::New(call) => {
                if let Expr::Ident(ident) = &*call.callee { if self.classes.contains_key(&ident.sym) { return Ok(Type::Ref(ident.clone())); } }
                Err(SemaError::UndefinedSymbol("Class".to_string(), call.span))
            },
            Expr::EnumInit(init) => Ok(Type::Ref(init.enum_name.clone())),
            Expr::Spawn(_) => Ok(Type::PID),
            Expr::Receive(ty) => Ok(ty.as_ref().map(|t| (**t).clone()).unwrap_or(Type::I32)),
            Expr::Slice(slice) => self.infer_type(&slice.obj),
            Expr::Question(inner) => {
                let inner_ty = self.infer_type(inner)?;
                match inner_ty { Type::ErrorUnion(t) | Type::Optional(t) => Ok(*t), _ => Ok(inner_ty) }
            },
            Expr::Object(fields) => {
                let mut fs = Vec::new();
                for f in fields {
                    let ty = self.infer_type(&f.val)?;
                    fs.push(Field { span: f.key.span, ident: f.key.clone(), ty });
                }
                Ok(Type::Object(fs))
            },
            Expr::Array(elements) => {
                let mut tys = Vec::new();
                for el in elements { tys.push(self.infer_type(el)?); }
                Ok(Type::Tuple(tys))
            },
            Expr::Await(inner) => {
                if !self.current_fn_is_async { return Err(SemaError::AwaitOutsideAsync(inner_span(inner))); }
                self.infer_type(inner) 
            },
            Expr::Builtin(call) => {
                // Builtins could have specific return types. 
                // For now, @map returns an array, @print returns void.
                match call.name.as_str() {
                    "@print" | "@fs_write_file" | "@fs_mkdir" | "@fs_remove" | "@fs_copy" | "@net_send" | "@net_close" | "@net_listen" => Ok(Type::Void),
                    "@fs_read_file" => Ok(Type::String),
                    "@fs_exists" => Ok(Type::Bool),
                    "@net_connect" => Ok(Type::I32),
                    "@self" => Ok(Type::PID),
                    "@shared_tensor_init" => Ok(Type::Ref(Ident { span: Span { start: 0, end: 0 }, sym: "SharedTensor".to_string() })),
                    "@map" => {
                        if call.args.len() > 0 {
                            let inner = self.infer_type(&call.args[0])?;
                            Ok(inner)
                        } else { Ok(Type::Unknown) }
                    },
                    _ => Ok(Type::Unknown),
                }
            },
            _ => Ok(Type::I32),
        }
    }
}

fn inner_span(expr: &Expr) -> Span {
    match expr { Expr::Ident(i) => i.span, Expr::Lit(_) => Span { start: 0, end: 0 }, Expr::Bin(b) => b.span, _ => Span { start: 0, end: 0 } }
}
