use crate::ast::*;
use crate::Span;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemaError {
    #[error("Undefined symbol: {0} at {1:?}")] UndefinedSymbol(String, Span),
    #[error("Duplicate symbol: {0} at {1:?}")] DuplicateSymbol(String, Span),
    #[error("Type mismatch: expected {0:?}, found {1:?} at {2:?}")] TypeMismatch(Type, Type, Span),
    #[error("Missing type or initializer for variable at {0:?}")] MissingType(Span),
    #[error("Cannot assign to const variable {0} at {1:?}")] ConstAssignment(String, Span),
    #[error("Not a function at {0:?}")] NotAFunction(Span),
    #[error("Class {0} does not implement interface {1} correctly: missing method {2} at {3:?}")] InterfaceNotSatisfied(String, String, String, Span),
}

pub type SemaResult<T> = Result<T, SemaError>;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Var { ty: Type, kind: VarDeclKind },
    Func { params: Vec<Type>, ret: Option<Type> },
    Class(Class),
    Interface(Interface),
}

pub struct Scope { symbols: HashMap<String, SymbolKind> }

pub struct Analyzer {
    scopes: Vec<Scope>,
    classes: HashMap<String, Class>,
    interfaces: HashMap<String, Interface>,
}

impl Analyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            scopes: vec![Scope { symbols: HashMap::new() }],
            classes: HashMap::new(),
            interfaces: HashMap::new(),
        };
        analyzer.define("reply".to_string(), SymbolKind::Func { params: vec![Type::PID, Type::I32], ret: Some(Type::I32) }, Span { start: 0, end: 0 }).unwrap();
        analyzer
    }

    fn enter_scope(&mut self) { self.scopes.push(Scope { symbols: HashMap::new() }); }
    fn exit_scope(&mut self) { self.scopes.pop(); }

    fn define(&mut self, name: String, kind: SymbolKind, span: Span) -> SemaResult<()> {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.symbols.contains_key(&name) { return Err(SemaError::DuplicateSymbol(name, span)); }
        current_scope.symbols.insert(name, kind);
        Ok(())
    }

    fn resolve(&self, name: &str) -> Option<&SymbolKind> {
        for scope in self.scopes.iter().rev() { if let Some(kind) = scope.symbols.get(name) { return Some(kind); } }
        None
    }

    pub fn analyze_module(&mut self, module: &Module) -> SemaResult<()> {
        for item in &module.body {
            if let ModuleItem::Decl(decl) = item {
                match decl {
                    Decl::Class(c) => {
                        self.classes.insert(c.ident.sym.clone(), c.clone());
                        self.define(c.ident.sym.clone(), SymbolKind::Class(c.clone()), c.ident.span)?;
                    },
                    Decl::Interface(i) => {
                        self.interfaces.insert(i.ident.sym.clone(), i.clone());
                        self.define(i.ident.sym.clone(), SymbolKind::Interface(i.clone()), i.ident.span)?;
                    },
                    _ => {}
                }
            }
        }
        for item in &module.body {
            match item {
                ModuleItem::Decl(decl) => self.analyze_decl(decl)?,
                ModuleItem::Stmt(stmt) => self.analyze_stmt(stmt)?,
            }
        }
        Ok(())
    }

    fn analyze_decl(&mut self, decl: &Decl) -> SemaResult<()> {
        match decl {
            Decl::Func(f) => self.analyze_function(f),
            Decl::Class(c) => self.analyze_class(c),
            Decl::Interface(_) => Ok(()), 
        }
    }

    fn analyze_class(&mut self, class: &Class) -> SemaResult<()> {
        for iface_ident in &class.implements {
            let iface = self.interfaces.get(&iface_ident.sym)
                .ok_or_else(|| SemaError::UndefinedSymbol(iface_ident.sym.clone(), iface_ident.span))?;
            for m_sig in &iface.methods {
                let has_method = class.methods.iter().any(|m| m.ident.sym == m_sig.ident.sym);
                if !has_method {
                    return Err(SemaError::InterfaceNotSatisfied(class.ident.sym.clone(), iface.ident.sym.clone(), m_sig.ident.sym.clone(), class.span));
                }
            }
        }
        Ok(())
    }

    fn analyze_function(&mut self, func: &Function) -> SemaResult<()> {
        let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
        self.define(func.ident.sym.clone(), SymbolKind::Func { params: param_types.clone(), ret: func.return_type.clone() }, func.ident.span)?;
        self.enter_scope();
        for param in &func.params { self.define(param.ident.sym.clone(), SymbolKind::Var { ty: param.ty.clone(), kind: VarDeclKind::Let }, param.ident.span)?; }
        self.analyze_block(&func.body)?;
        self.exit_scope();
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> SemaResult<()> {
        match stmt {
            Stmt::Var(var) => self.analyze_var_decl(var),
            Stmt::Block(block) => self.analyze_block(block),
            Stmt::Return(ret) => { if let Some(expr) = &ret.arg { self.infer_type(expr)?; } Ok(()) },
            Stmt::Expr(expr) => { self.infer_type(expr)?; Ok(()) },
            Stmt::While(w) => {
                let test_ty = self.infer_type(&w.test)?;
                if test_ty != Type::Bool { return Err(SemaError::TypeMismatch(Type::Bool, test_ty, w.span)); }
                self.analyze_stmt(&w.body)
            },
            _ => Ok(()),
        }
    }

    fn analyze_var_decl(&mut self, var: &VarDecl) -> SemaResult<()> {
        let inferred_ty = if let Some(init) = &var.init {
            let ty = self.infer_type(init)?;
            if let Some(explicit_ty) = &var.ty {
                if *explicit_ty != ty {
                    if let (Type::Ref(target), Type::Ref(source)) = (explicit_ty, &ty) {
                        if let (Some(iface), Some(cls)) = (self.interfaces.get(&target.sym), self.classes.get(&source.sym)) {
                             if !cls.implements.iter().any(|i| i.sym == iface.ident.sym) { return Err(SemaError::TypeMismatch(explicit_ty.clone(), ty, var.span)); }
                        } else { return Err(SemaError::TypeMismatch(explicit_ty.clone(), ty, var.span)); }
                    } else { return Err(SemaError::TypeMismatch(explicit_ty.clone(), ty, var.span)); }
                }
            }
            ty
        } else { var.ty.clone().ok_or_else(|| SemaError::MissingType(var.span))? };
        self.analyze_pattern(&var.pat, inferred_ty, &var.kind)
    }

    fn analyze_pattern(&mut self, pat: &Pattern, ty: Type, kind: &VarDeclKind) -> SemaResult<()> {
        match pat {
            Pattern::Ident(ident) => self.define(ident.sym.clone(), SymbolKind::Var { ty, kind: kind.clone() }, ident.span),
            Pattern::Array(elements) => {
                let elem_ty = match ty { Type::Array(inner) => *inner, _ => return Err(SemaError::TypeMismatch(Type::Array(Box::new(Type::Unknown)), ty, Span { start: 0, end: 0 })) };
                for el in elements { self.analyze_pattern(el, elem_ty.clone(), kind)?; }
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
                match self.resolve(&ident.sym) {
                    Some(SymbolKind::Var { ty, .. }) => Ok(ty.clone()),
                    Some(SymbolKind::Func { params, ret }) => Ok(Type::Fn(params.clone(), Box::new(ret.clone().unwrap_or(Type::I32)))),
                    Some(SymbolKind::Class(c)) => Ok(Type::Ref(c.ident.clone())),
                    _ => Err(SemaError::UndefinedSymbol(ident.sym.clone(), ident.span)),
                }
            },
            Expr::Bin(bin) => {
                let left = self.infer_type(&bin.left)?;
                let right = self.infer_type(&bin.right)?;
                if bin.op == BinaryOp::Eq { Ok(left) } else if bin.op == BinaryOp::Lt { Ok(Type::Bool) } else { Ok(left) }
            },
            Expr::Call(call) => {
                let callee_ty = self.infer_type(&call.callee)?;
                match callee_ty { Type::Fn(_, ret) => Ok(*ret), _ => Err(SemaError::NotAFunction(call.span)) }
            },
            Expr::New(call) => {
                if let Expr::Ident(ident) = &*call.callee { if self.classes.contains_key(&ident.sym) { return Ok(Type::Ref(ident.clone())); } }
                Err(SemaError::UndefinedSymbol("Class".to_string(), call.span))
            },
            Expr::Spawn(_) => Ok(Type::PID),
            Expr::Receive(ty) => Ok(ty.as_ref().map(|t| (**t).clone()).unwrap_or(Type::I32)),
            Expr::Slice(slice) => {
                let obj_ty = self.infer_type(&slice.obj)?;
                Ok(obj_ty) // Slicing an Array results in an Array, String results in String
            },
            Expr::Member(member) => {
                let obj_ty = self.infer_type(&member.obj)?;
                match obj_ty {
                    Type::String => {
                        match member.prop.sym.as_str() {
                            "length" => Ok(Type::USize),
                            "at" => Ok(Type::Fn(vec![Type::I32], Box::new(Type::String))),
                            _ => Err(SemaError::UndefinedSymbol(member.prop.sym.clone(), member.prop.span)),
                        }
                    },
                    Type::Array(inner) => {
                        match member.prop.sym.as_str() {
                            "length" => Ok(Type::USize),
                            "at" => Ok(Type::Fn(vec![Type::I32], inner)),
                            "map" => {
                                let callback_ty = Type::Fn(vec![(*inner).clone()], Box::new(Type::Unknown)); // TBD later
                                Ok(Type::Fn(vec![callback_ty], Box::new(Type::Array(Box::new(Type::Unknown)))))
                            },
                            _ => Err(SemaError::UndefinedSymbol(member.prop.sym.clone(), member.prop.span)),
                        }
                    },
                    Type::Ref(ident) => {
                        if let Some(cls) = self.classes.get(&ident.sym) {
                            if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == member.prop.sym) { return Ok(f.ty.clone()); }
                            if let Some(m) = cls.methods.iter().find(|m| m.ident.sym == member.prop.sym) {
                                let param_types: Vec<Type> = m.params.iter().map(|p| p.ty.clone()).collect();
                                return Ok(Type::Fn(param_types, Box::new(m.return_type.clone().unwrap_or(Type::I32))));
                            }
                        }
                        if let Some(iface) = self.interfaces.get(&ident.sym) {
                            if let Some(m) = iface.methods.iter().find(|m| m.ident.sym == member.prop.sym) {
                                let param_types: Vec<Type> = m.params.iter().map(|p| p.ty.clone()).collect();
                                return Ok(Type::Fn(param_types, Box::new(m.return_type.clone().unwrap_or(Type::I32))));
                            }
                        }
                        Err(SemaError::UndefinedSymbol(member.prop.sym.clone(), member.prop.span))
                    },
                    _ => Ok(Type::I32),
                }
            },
            Expr::Array(elements) => {
                if elements.is_empty() { return Ok(Type::Array(Box::new(Type::Unknown))); }
                let first_ty = self.infer_type(&elements[0])?;
                Ok(Type::Array(Box::new(first_ty)))
            },
            Expr::Arrow(arrow) => {
                let mut param_tys = Vec::new();
                for p in &arrow.params { if let Pattern::Ident(_) = p { param_tys.push(Type::I32); } }
                let ret_ty = match &*arrow.body { ArrowBody::Expr(e) => self.infer_type(e)?, ArrowBody::Block(_) => Type::I32 };
                Ok(Type::Fn(param_tys, Box::new(ret_ty)))
            },
            _ => Ok(Type::I32),
        }
    }
}
