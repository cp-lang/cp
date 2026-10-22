use crate::ast::*;
use crate::Span;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemaError {
    #[error("Undefined symbol: {0} at {1:?}")]
    UndefinedSymbol(String, Span),
    #[error("Type mismatch: expected {0:?}, found {1:?} at {2:?}")]
    TypeMismatch(Type, Type, Span),
    #[error("Not a function at {0:?}")]
    NotAFunction(Span),
    #[error("Const assignment to {0} at {1:?}")]
    ConstAssignment(String, Span),
    #[error("Invalid await at {0:?}")]
    InvalidAwait(Span),
    #[error("Await can only be used in async functions at {0:?}")]
    AwaitOutsideAsync(Span),
    #[error("Use of moved value '{0}' at {1:?}")]
    UseOfMovedValue(String, Span),
    #[error("Cannot mutate a read-only borrow '{0}' at {1:?}")]
    CannotMutateReadOnlyBorrow(String, Span),
    #[error("Cannot borrow '{0}' as mutable because it is also borrowed as immutable in the current scope at {1:?}")]
    MutableBorrowConflict(String, Span),
    #[error(
        "Compile Error: Meta '@{0}' can only be applied to {1}, but was applied to {2} at {3:?}"
    )]
    MetaInvalidTarget(String, String, String, Span),
    #[error("Compile Error: Meta '@{0}' is marked with '@once' and cannot be applied multiple times at {1:?}")]
    MetaDuplicateOnce(String, Span),
    #[error("Compile Error: Argument to meta '@{0}' must be a compile-time constant at {1:?}")]
    MetaArgNotConstant(String, Span),
    #[error("Compile Error: Type '{0}' cannot be reflected at runtime. Add '@reflect' to the class definition to enable runtime reflection at {1:?}")]
    ReflectNotEnabled(String, Span),
}

pub type SemaResult<T> = Result<T, SemaError>;

#[derive(Clone, Debug)]
pub struct MetaDefInfo {
    pub allowed_targets: std::collections::HashSet<String>,
    pub is_once: bool,
}

#[derive(Clone)]
pub enum SymbolKind {
    Var {
        ty: Type,
        kind: VarDeclKind,
    },
    Func {
        params: Vec<Type>,
        ret: Option<Type>,
        is_async: bool,
    },
    Class(Class),
    Trait(TraitDecl),
    Enum(EnumDecl),
}

pub struct Analyzer {
    scopes: Vec<HashMap<String, SymbolKind>>,
    classes: HashMap<String, Class>,
    current_fn_is_async: bool,
    pub symbol_map: HashMap<Span, SymbolKind>,
    pub meta_defs: HashMap<String, MetaDefInfo>,
}

impl Analyzer {
    pub fn new() -> Self {
        let mut meta_defs = HashMap::new();
        let mut target_targets = std::collections::HashSet::new();
        target_targets.insert("meta".to_string());
        meta_defs.insert(
            "target".to_string(),
            MetaDefInfo {
                allowed_targets: target_targets,
                is_once: true,
            },
        );
        let mut once_targets = std::collections::HashSet::new();
        once_targets.insert("meta".to_string());
        meta_defs.insert(
            "once".to_string(),
            MetaDefInfo {
                allowed_targets: once_targets,
                is_once: true,
            },
        );
        let mut reflect_targets = std::collections::HashSet::new();
        reflect_targets.insert("class".to_string());
        reflect_targets.insert("struct".to_string());
        meta_defs.insert(
            "reflect".to_string(),
            MetaDefInfo {
                allowed_targets: reflect_targets,
                is_once: true,
            },
        );

        Self {
            scopes: vec![HashMap::new()],
            classes: HashMap::new(),
            current_fn_is_async: false,
            symbol_map: HashMap::new(),
            meta_defs,
        }
    }

    pub fn analyze_module(&mut self, module: &Module) -> SemaResult<()> {
        let mock_span = Span { start: 0, end: 0 };
        let regex_class = crate::ast::Class {
            span: mock_span.clone(),
            ident: crate::ast::Ident {
                span: mock_span.clone(),
                sym: "RegExp".to_string(),
            },
            fields: vec![],
            methods: vec![],
            implements: vec![],
            extends: None,
            is_ref: false,
            meta: Vec::new(),
        };
        self.classes
            .insert("RegExp".to_string(), regex_class.clone());
        self.scopes[0].insert("RegExp".to_string(), SymbolKind::Class(regex_class));
        let set_class = crate::ast::Class {
            span: mock_span.clone(),
            ident: crate::ast::Ident {
                span: mock_span.clone(),
                sym: "Set".to_string(),
            },
            fields: vec![],
            methods: vec![],
            implements: vec![],
            extends: None,
            is_ref: false,
            meta: Vec::new(),
        };
        self.classes.insert("Set".to_string(), set_class.clone());
        self.scopes[0].insert("Set".to_string(), SymbolKind::Class(set_class));

        let map_class = crate::ast::Class {
            span: mock_span.clone(),
            ident: crate::ast::Ident {
                span: mock_span.clone(),
                sym: "Map".to_string(),
            },
            fields: vec![],
            methods: vec![],
            implements: vec![],
            extends: None,
            is_ref: false,
            meta: Vec::new(),
        };
        self.classes.insert("Map".to_string(), map_class.clone());
        self.scopes[0].insert("Map".to_string(), SymbolKind::Class(map_class));

        let error_class = crate::ast::Class {
            span: mock_span.clone(),
            ident: crate::ast::Ident {
                span: mock_span.clone(),
                sym: "Error".to_string(),
            },
            fields: vec![
                crate::ast::Field {
                    span: mock_span.clone(),
                    ident: crate::ast::Ident {
                        span: mock_span.clone(),
                        sym: "code".to_string(),
                    },
                    ty: Type::I32,
                    meta: Vec::new(),
                },
                crate::ast::Field {
                    span: mock_span.clone(),
                    ident: crate::ast::Ident {
                        span: mock_span.clone(),
                        sym: "msg".to_string(),
                    },
                    ty: Type::String,
                    meta: Vec::new(),
                },
            ],
            methods: vec![],
            implements: vec![],
            extends: None,
            is_ref: false,
            meta: Vec::new(),
        };
        self.classes
            .insert("Error".to_string(), error_class.clone());
        self.scopes[0].insert("Error".to_string(), SymbolKind::Class(error_class));

        let box_class = crate::ast::Class {
            span: mock_span.clone(),
            ident: crate::ast::Ident {
                span: mock_span.clone(),
                sym: "Box".to_string(),
            },
            fields: vec![],
            methods: vec![],
            implements: vec![],
            extends: None,
            is_ref: false,
            meta: Vec::new(),
        };
        self.classes.insert("Box".to_string(), box_class.clone());
        self.scopes[0].insert("Box".to_string(), SymbolKind::Class(box_class));

        for reflect_name in &["TypeInfo", "FieldInfo", "MethodInfo"] {
            let cls = crate::ast::Class {
                span: mock_span.clone(),
                ident: crate::ast::Ident {
                    span: mock_span.clone(),
                    sym: reflect_name.to_string(),
                },
                fields: vec![],
                methods: vec![],
                implements: vec![],
                extends: None,
                is_ref: false,
                meta: vec![crate::ast::MetaItem {
                    span: mock_span.clone(),
                    name: crate::ast::Ident {
                        span: mock_span.clone(),
                        sym: "reflect".to_string(),
                    },
                    args: vec![],
                }],
            };
            self.classes.insert(reflect_name.to_string(), cls.clone());
            self.scopes[0].insert(reflect_name.to_string(), SymbolKind::Class(cls));
        }

        for item in &module.body {
            match item {
                ModuleItem::Decl(Decl::Meta(m)) => {
                    self.register_meta_def(m)?;
                }
                ModuleItem::Decl(Decl::Class(c)) => {
                    self.classes.insert(c.ident.sym.clone(), c.clone());
                    self.define(
                        c.ident.sym.clone(),
                        SymbolKind::Class(c.clone()),
                        c.ident.span,
                    )?;
                    self.symbol_map
                        .insert(c.ident.span, SymbolKind::Class(c.clone()));
                }
                ModuleItem::Decl(Decl::Trait(t)) => {
                    self.define(
                        t.ident.sym.clone(),
                        SymbolKind::Trait(t.clone()),
                        t.ident.span,
                    )?;
                    self.symbol_map
                        .insert(t.ident.span, SymbolKind::Trait(t.clone()));
                }
                ModuleItem::Decl(Decl::Enum(e)) => {
                    self.define(
                        e.ident.sym.clone(),
                        SymbolKind::Enum(e.clone()),
                        e.ident.span,
                    )?;
                    self.symbol_map
                        .insert(e.ident.span, SymbolKind::Enum(e.clone()));
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

    fn is_const_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Lit(_) => true,
            Expr::Ident(_) => true,
            Expr::Array(elements) => elements.iter().all(Self::is_const_expr),
            Expr::Object(fields) => fields.iter().all(|f| Self::is_const_expr(&f.val)),
            Expr::Dict(fields) => fields.iter().all(|f| Self::is_const_expr(&f.val)),
            _ => false,
        }
    }

    fn register_meta_def(&mut self, m: &MetaDef) -> SemaResult<()> {
        let mut allowed_targets = std::collections::HashSet::new();
        let mut is_once = false;

        self.validate_meta_items(&m.meta, "meta")?;

        for item in &m.meta {
            if item.name.sym.to_lowercase() == "target" {
                for arg in &item.args {
                    let target_name = match arg {
                        MetaArg::Positional(Expr::Ident(id)) => id.sym.to_lowercase(),
                        MetaArg::Named(_, Expr::Ident(id)) => id.sym.to_lowercase(),
                        _ => continue,
                    };
                    allowed_targets.insert(target_name);
                }
            } else if item.name.sym.to_lowercase() == "once" {
                is_once = true;
            }
        }

        if allowed_targets.is_empty() {
            allowed_targets.insert("class".to_string());
            allowed_targets.insert("struct".to_string());
            allowed_targets.insert("function".to_string());
            allowed_targets.insert("method".to_string());
            allowed_targets.insert("field".to_string());
            allowed_targets.insert("param".to_string());
        }

        self.meta_defs.insert(
            m.ident.sym.clone(),
            MetaDefInfo {
                allowed_targets,
                is_once,
            },
        );
        Ok(())
    }

    fn validate_meta_items(&self, items: &[MetaItem], target: &str) -> SemaResult<()> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for item in items {
            *counts.entry(item.name.sym.clone()).or_insert(0) += 1;

            for arg in &item.args {
                let expr = match arg {
                    MetaArg::Positional(e) => e,
                    MetaArg::Named(_, e) => e,
                };
                if !Self::is_const_expr(expr) {
                    return Err(SemaError::MetaArgNotConstant(
                        item.name.sym.clone(),
                        item.span,
                    ));
                }
            }

            if let Some(def) = self.meta_defs.get(&item.name.sym) {
                let matches_target = def.allowed_targets.contains(target)
                    || (target == "struct" && def.allowed_targets.contains("class"))
                    || (target == "class" && def.allowed_targets.contains("struct"))
                    || (target == "method" && def.allowed_targets.contains("function"))
                    || (target == "function" && def.allowed_targets.contains("method"));
                if !matches_target {
                    let mut allowed: Vec<String> = def.allowed_targets.iter().cloned().collect();
                    allowed.sort();
                    return Err(SemaError::MetaInvalidTarget(
                        item.name.sym.clone(),
                        allowed.join(", "),
                        target.to_string(),
                        item.span,
                    ));
                }
            }
        }

        for (name, count) in counts {
            if count > 1 {
                if let Some(def) = self.meta_defs.get(&name) {
                    if def.is_once {
                        let span = items
                            .iter()
                            .find(|i| i.name.sym == name)
                            .map(|i| i.span)
                            .unwrap_or(Span { start: 0, end: 0 });
                        return Err(SemaError::MetaDuplicateOnce(name, span));
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_decl(&mut self, decl: &Decl) -> SemaResult<()> {
        match decl {
            Decl::Meta(m) => {
                self.validate_meta_items(&m.meta, "meta")?;
            }
            Decl::Func(f) => {
                self.validate_meta_items(&f.meta, "function")?;
                for p in &f.params {
                    self.validate_meta_items(&p.meta, "param")?;
                }
                let params = f.params.iter().map(|p| p.ty.clone()).collect();
                let kind = SymbolKind::Func {
                    params,
                    ret: f.return_type.clone(),
                    is_async: f.is_async,
                };
                self.define(f.ident.sym.clone(), kind.clone(), f.ident.span)?;
                self.symbol_map.insert(f.ident.span, kind);

                let old_async = self.current_fn_is_async;
                self.current_fn_is_async = f.is_async;
                self.enter_scope();
                for p in &f.params {
                    let p_kind = SymbolKind::Var {
                        ty: p.ty.clone(),
                        kind: VarDeclKind::Let,
                    };
                    self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span)?;
                    self.symbol_map.insert(p.ident.span, p_kind);
                }
                for stmt in &f.body.body {
                    self.analyze_stmt(stmt)?;
                }
                self.exit_scope();
                self.current_fn_is_async = old_async;
            }
            Decl::Class(c) => {
                let target = if c.is_ref { "struct" } else { "class" };
                self.validate_meta_items(&c.meta, target)?;
                for f in &c.fields {
                    self.validate_meta_items(&f.meta, "field")?;
                }
                for m in &c.methods {
                    self.validate_meta_items(&m.meta, "method")?;
                    for p in &m.params {
                        self.validate_meta_items(&p.meta, "param")?;
                    }
                }
                for m in &c.methods {
                    let old_async = self.current_fn_is_async;
                    self.current_fn_is_async = m.is_async;
                    self.enter_scope();
                    // Define receiver aliases `this` and `_`
                    let self_ty = Type::Ref(c.ident.clone());
                    let _ = self.define(
                        "this".to_string(),
                        SymbolKind::Var {
                            ty: self_ty.clone(),
                            kind: VarDeclKind::Let,
                        },
                        c.ident.span,
                    );
                    let _ = self.define(
                        "_".to_string(),
                        SymbolKind::Var {
                            ty: self_ty.clone(),
                            kind: VarDeclKind::Let,
                        },
                        c.ident.span,
                    );
                    // Define fields & methods from parent hierarchy and current class for omitted `_.` calls
                    let mut cur_opt = Some(c.clone());
                    while let Some(cur_c) = cur_opt {
                        for f in &cur_c.fields {
                            let _ = self.define(
                                f.ident.sym.clone(),
                                SymbolKind::Var {
                                    ty: f.ty.clone(),
                                    kind: VarDeclKind::Let,
                                },
                                f.ident.span,
                            );
                        }
                        for cm in &cur_c.methods {
                            let params = cm
                                .params
                                .iter()
                                .filter(|p| p.ident.sym != "this" && p.ident.sym != "_")
                                .map(|p| p.ty.clone())
                                .collect();
                            let _ = self.define(
                                cm.ident.sym.clone(),
                                SymbolKind::Func {
                                    params,
                                    ret: cm.return_type.clone(),
                                    is_async: cm.is_async,
                                },
                                cm.ident.span,
                            );
                        }
                        cur_opt = cur_c
                            .extends
                            .as_ref()
                            .and_then(|p| self.classes.get(&p.sym).cloned());
                    }
                    // Also define methods from implemented traits
                    for tr_id in &c.implements {
                        if let Some(SymbolKind::Trait(tr_decl)) = self.resolve(&tr_id.sym).cloned()
                        {
                            for tm in &tr_decl.methods {
                                let params = tm
                                    .params
                                    .iter()
                                    .filter(|p| p.ident.sym != "this" && p.ident.sym != "_")
                                    .map(|p| p.ty.clone())
                                    .collect();
                                let _ = self.define(
                                    tm.ident.sym.clone(),
                                    SymbolKind::Func {
                                        params,
                                        ret: tm.return_type.clone(),
                                        is_async: false,
                                    },
                                    tm.ident.span,
                                );
                            }
                        }
                    }
                    for p in &m.params {
                        let p_kind = SymbolKind::Var {
                            ty: p.ty.clone(),
                            kind: VarDeclKind::Let,
                        };
                        self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span)?;
                        self.symbol_map.insert(p.ident.span, p_kind);
                    }
                    for stmt in &m.body.body {
                        self.analyze_stmt(stmt)?;
                    }
                    self.exit_scope();
                    self.current_fn_is_async = old_async;
                }
            }
            Decl::Trait(t) => {
                self.validate_meta_items(&t.meta, "trait")?;
            }
            Decl::Enum(e) => {
                self.validate_meta_items(&e.meta, "enum")?;
            }
            _ => {}
        }
        Ok(())
    }

    fn bind_match_pattern(&mut self, pat: &Pattern) {
        match pat {
            Pattern::Ident(id) => {
                let _ = self.define(
                    id.sym.clone(),
                    SymbolKind::Var {
                        ty: Type::Any,
                        kind: VarDeclKind::Let,
                    },
                    id.span,
                );
            }
            Pattern::Err(inner) => self.bind_match_pattern(inner),
            Pattern::Enum(ep) => {
                if let Some(tpats) = &ep.tuple_pats {
                    for tp in tpats {
                        self.bind_match_pattern(tp);
                    }
                }
                if let Some(props) = &ep.fields {
                    for p in props {
                        if let Some(vp) = &p.val {
                            self.bind_match_pattern(vp);
                        } else {
                            let _ = self.define(
                                p.key.sym.clone(),
                                SymbolKind::Var {
                                    ty: Type::Any,
                                    kind: VarDeclKind::Let,
                                },
                                p.key.span,
                            );
                        }
                    }
                }
            }
            Pattern::Tuple(els) | Pattern::Or(els) => {
                for el in els {
                    self.bind_match_pattern(el);
                }
            }
            Pattern::Object(props) => {
                for p in props {
                    if let Some(vp) = &p.val {
                        self.bind_match_pattern(vp);
                    } else {
                        let _ = self.define(
                            p.key.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Let,
                            },
                            p.key.span,
                        );
                    }
                }
            }
            _ => {}
        }
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
            }
            Stmt::Expr(expr) => {
                self.infer_type(expr)?;
                self.analyze_expr(expr)?;
            }
            Stmt::Block(block) => self.analyze_block(block)?,
            Stmt::If(i) => {
                self.infer_type(&i.test)?;
                self.analyze_expr(&i.test)?;
                self.analyze_stmt(&i.cons)?;
                if let Some(alt) = &i.alt {
                    self.analyze_stmt(alt)?;
                }
            }
            Stmt::While(w) => {
                self.infer_type(&w.test)?;
                self.analyze_expr(&w.test)?;
                self.analyze_stmt(&w.body)?;
            }
            Stmt::Return(r) => {
                if let Some(arg) = &r.arg {
                    self.infer_type(arg)?;
                    self.analyze_expr(arg)?;
                }
            }
            Stmt::Match(m) => {
                self.infer_type(&m.disc)?;
                self.analyze_expr(&m.disc)?;
                for arm in &m.arms {
                    self.enter_scope();
                    self.bind_match_pattern(&arm.pat);
                    if let Some(g) = &arm.guard {
                        self.infer_type(g)?;
                        self.analyze_expr(g)?;
                    }
                    self.analyze_stmt(&arm.body)?;
                    self.exit_scope();
                }
            }
            Stmt::TryCatch {
                try_block,
                err_ident,
                catch_body,
                ..
            } => {
                self.analyze_block(try_block)?;
                if let Some(cb) = catch_body {
                    self.enter_scope();
                    if let Some(cv) = err_ident {
                        let _ = self.define(
                            cv.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Let,
                            },
                            cv.span,
                        );
                    }
                    self.analyze_stmt(cb)?;
                    self.exit_scope();
                }
            }
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
            Expr::Call(call) | Expr::OptCall(call) => {
                self.analyze_expr(&call.callee)?;
                for arg in &call.args {
                    self.analyze_expr(arg)?;
                }
            }
            Expr::New(call) => {
                self.analyze_expr(&call.callee)?;
                for arg in &call.args {
                    self.analyze_expr(arg)?;
                }
            }
            Expr::Member(member) | Expr::OptMember(member) => {
                self.analyze_expr(&member.obj)?;
            }
            Expr::Slice(slice) => {
                self.analyze_expr(&slice.obj)?;
                if let Some(s) = &slice.start {
                    self.analyze_expr(s)?;
                }
                if let Some(e) = &slice.end {
                    self.analyze_expr(e)?;
                }
            }
            Expr::Match(m) => {
                self.analyze_expr(&m.disc)?;
                for arm in &m.arms {
                    self.enter_scope();
                    self.bind_match_pattern(&arm.pat);
                    if let Some(g) = &arm.guard {
                        self.analyze_expr(g)?;
                    }
                    self.analyze_stmt(&arm.body)?;
                    self.exit_scope();
                }
            }
            Expr::Array(elements) => {
                for el in elements {
                    self.analyze_expr(el)?;
                }
            }
            Expr::Object(fields) | Expr::Dict(fields) => {
                for f in fields {
                    self.analyze_expr(&f.val)?;
                }
            }
            Expr::Index { obj, index } => {
                self.analyze_expr(obj)?;
                self.analyze_expr(index)?;
            }
            Expr::Await(inner)
            | Expr::Question(inner)
            | Expr::ErrWrap(inner)
            | Expr::BoxNew(inner)
            | Expr::Move(inner) => {
                self.analyze_expr(inner)?;
            }
            Expr::Borrow(inner, _) => {
                self.analyze_expr(inner)?;
            }
            Expr::QuestionFallback { expr, fallback, .. } => {
                self.analyze_expr(expr)?;
                self.analyze_expr(fallback)?;
            }
            Expr::NullCoalesce { left, right } => {
                self.analyze_expr(left)?;
                self.analyze_expr(right)?;
            }
            Expr::Catch {
                expr,
                err_ident,
                body,
                ..
            } => {
                self.analyze_expr(expr)?;
                self.enter_scope();
                if let Some(ev) = err_ident {
                    let _ = self.define(
                        ev.sym.clone(),
                        SymbolKind::Var {
                            ty: Type::Any,
                            kind: VarDeclKind::Let,
                        },
                        ev.span,
                    );
                }
                self.analyze_stmt(body)?;
                self.exit_scope();
            }
            Expr::Downcast { obj, .. } => {
                self.analyze_expr(obj)?;
            }
            Expr::Builtin(call) => {
                for arg in &call.args {
                    self.analyze_expr(arg)?;
                }
            }
            Expr::ReflectType { span, target_ty } => {
                let type_name = match target_ty {
                    Type::Ref(id) => id.sym.clone(),
                    _ => {
                        return Err(SemaError::ReflectNotEnabled(
                            format!("{:?}", target_ty),
                            *span,
                        ));
                    }
                };
                if let Some(cls) = self.classes.get(&type_name) {
                    let has_reflect = cls.meta.iter().any(|m| m.name.sym == "reflect");
                    if !has_reflect {
                        return Err(SemaError::ReflectNotEnabled(type_name, *span));
                    }
                } else {
                    return Err(SemaError::UndefinedSymbol(type_name, *span));
                }
            }
            Expr::ReflectOf { span, expr } => {
                self.analyze_expr(expr)?;
                let ty = self.infer_type(expr)?;
                let type_name = match ty {
                    Type::Ref(id) => id.sym,
                    Type::BorrowType { inner, .. } => {
                        if let Type::Ref(id) = &*inner {
                            id.sym.clone()
                        } else {
                            return Err(SemaError::ReflectNotEnabled(
                                format!("{:?}", inner),
                                *span,
                            ));
                        }
                    }
                    _ => {
                        return Err(SemaError::ReflectNotEnabled(format!("{:?}", ty), *span));
                    }
                };
                if let Some(cls) = self.classes.get(&type_name) {
                    let has_reflect = cls.meta.iter().any(|m| m.name.sym == "reflect");
                    if !has_reflect {
                        return Err(SemaError::ReflectNotEnabled(type_name, *span));
                    }
                } else {
                    return Err(SemaError::UndefinedSymbol(type_name, *span));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    fn define(&mut self, name: String, kind: SymbolKind, _span: Span) -> SemaResult<()> {
        if self.scopes.last().unwrap().contains_key(&name) {
            // Already defined in current scope
        }
        self.scopes.last_mut().unwrap().insert(name, kind);
        Ok(())
    }
    fn resolve(&self, name: &str) -> Option<&SymbolKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(kind) = scope.get(name) {
                return Some(kind);
            }
        }
        None
    }

    fn types_eq(&self, a: &Type, b: &Type) -> bool {
        if a == b {
            return true;
        }
        if *a == Type::Any || *b == Type::Any || *a == Type::Unknown || *b == Type::Unknown {
            return true;
        }
        if (*a == Type::I32 && *b == Type::U32) || (*a == Type::U32 && *b == Type::I32) {
            return true;
        }
        match (a, b) {
            (Type::Ref(t1), Type::Ref(t2)) => t1.sym == t2.sym,
            (Type::BoxDyn(t1), Type::BoxDyn(t2)) => t1.sym == t2.sym,
            (Type::BoxDyn(_), Type::Ref(_)) | (Type::Ref(_), Type::BoxDyn(_)) => true,
            (Type::BorrowType { inner: i1, .. }, Type::BorrowType { inner: i2, .. }) => {
                self.types_eq(i1, i2)
            }
            (Type::BorrowType { inner: i1, .. }, other)
            | (other, Type::BorrowType { inner: i1, .. }) => self.types_eq(i1, other),
            (Type::Shared(i1), Type::Shared(i2)) => self.types_eq(i1, i2),
            (Type::Shared(i1), other) | (other, Type::Shared(i1)) => self.types_eq(i1, other),
            (Type::Optional(t1), Type::Optional(t2)) => self.types_eq(t1, t2),
            (Type::Optional(t1), other) | (other, Type::Optional(t1)) => self.types_eq(t1, other),
            (Type::ErrorUnion(t1), Type::ErrorUnion(t2)) => self.types_eq(t1, t2),
            (Type::ResultType { ok: o1, .. }, Type::ResultType { ok: o2, .. }) => {
                self.types_eq(o1, o2)
            }
            (Type::ResultType { ok: o1, .. }, other) | (other, Type::ResultType { ok: o1, .. }) => {
                self.types_eq(o1, other)
            }
            (Type::Array(t1), Type::Array(t2)) => self.types_eq(t1, t2),
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return false;
                }
                for (i, p) in t1.iter().enumerate() {
                    if !self.types_eq(p, &t2[i]) {
                        return false;
                    }
                }
                true
            }
            (Type::Object(f1), Type::Object(f2)) => {
                if f1.len() != f2.len() {
                    return false;
                }
                for (i, f) in f1.iter().enumerate() {
                    if f.ident.sym != f2[i].ident.sym || !self.types_eq(&f.ty, &f2[i].ty) {
                        return false;
                    }
                }
                true
            }
            (Type::Fn(p1, r1), Type::Fn(p2, r2)) => {
                if p1.len() != p2.len() {
                    return false;
                }
                for (i, p) in p1.iter().enumerate() {
                    if !self.types_eq(p, &p2[i]) {
                        return false;
                    }
                }
                self.types_eq(r1, r2)
            }
            (Type::Void, Type::Void) => true,
            (Type::Any, _) | (_, Type::Any) => true,
            _ => a == b,
        }
    }

    fn analyze_pattern(&mut self, pat: &Pattern, ty: Type, kind: &VarDeclKind) -> SemaResult<()> {
        match pat {
            Pattern::Ident(ident) => self.define(
                ident.sym.clone(),
                SymbolKind::Var {
                    ty,
                    kind: kind.clone(),
                },
                ident.span,
            ),
            Pattern::Tuple(elements) => {
                let element_types = match ty {
                    Type::Tuple(tys) => tys,
                    _ => {
                        return Err(SemaError::TypeMismatch(
                            Type::Tuple(vec![]),
                            ty,
                            Span { start: 0, end: 0 },
                        ))
                    }
                };
                if elements.len() != element_types.len() {
                    return Err(SemaError::TypeMismatch(
                        Type::Tuple(vec![]),
                        Type::Tuple(element_types),
                        Span { start: 0, end: 0 },
                    ));
                }
                for (i, el) in elements.iter().enumerate() {
                    self.analyze_pattern(el, element_types[i].clone(), kind)?;
                }
                Ok(())
            }
            Pattern::Object(props) => {
                let fields = match ty {
                    Type::Object(fs) => fs,
                    _ => {
                        return Err(SemaError::TypeMismatch(
                            Type::Object(vec![]),
                            ty,
                            Span { start: 0, end: 0 },
                        ))
                    }
                };
                for prop in props {
                    let field = fields
                        .iter()
                        .find(|f| f.ident.sym == prop.key.sym)
                        .ok_or_else(|| {
                            SemaError::UndefinedSymbol(prop.key.sym.clone(), prop.key.span)
                        })?;
                    if let Some(inner_pat) = &prop.val {
                        self.analyze_pattern(inner_pat, field.ty.clone(), kind)?;
                    } else {
                        self.analyze_pattern(
                            &Pattern::Ident(prop.key.clone()),
                            field.ty.clone(),
                            kind,
                        )?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn analyze_block(&mut self, block: &BlockStmt) -> SemaResult<()> {
        self.enter_scope();
        for stmt in &block.body {
            self.analyze_stmt(stmt)?;
        }
        self.exit_scope();
        Ok(())
    }

    fn infer_type(&self, expr: &Expr) -> SemaResult<Type> {
        match expr {
            Expr::Lit(lit) => match lit {
                Lit::Int(_) => Ok(Type::I32),
                Lit::Float(_) => Ok(Type::F32),
                Lit::Bool(_) => Ok(Type::Bool),
                Lit::Str(_) => Ok(Type::String),
                Lit::Null => Ok(Type::Optional(Box::new(Type::Unknown))),
                Lit::Undefined => Ok(Type::Any),
            },
            Expr::Ident(ident) => {
                if ident.sym == "true" || ident.sym == "false" {
                    return Ok(Type::Bool);
                }
                if ident.sym == "null" {
                    return Ok(Type::Optional(Box::new(Type::Unknown)));
                }
                if ident.sym.starts_with('@')
                    || ident.sym == "this"
                    || ident.sym == "_"
                    || ident.sym == "super"
                    || ident.sym == "Box"
                    || ident.sym == "Err"
                    || ident.sym == "Error"
                    || ident.sym == "reflect"
                {
                    return Ok(Type::Any);
                }
                match self.resolve(&ident.sym) {
                    Some(SymbolKind::Var { ty, .. }) => Ok(ty.clone()),
                    Some(SymbolKind::Func {
                        params,
                        ret,
                        is_async: _,
                    }) => Ok(Type::Fn(
                        params.clone(),
                        Box::new(ret.clone().unwrap_or(Type::I32)),
                    )),
                    Some(SymbolKind::Class(c)) => Ok(Type::Ref(c.ident.clone())),
                    Some(SymbolKind::Trait(t)) => Ok(Type::Ref(t.ident.clone())),
                    Some(SymbolKind::Enum(e)) => Ok(Type::Ref(e.ident.clone())),
                    _ => Err(SemaError::UndefinedSymbol(ident.sym.clone(), ident.span)),
                }
            }
            Expr::Bin(bin) => {
                let left = self.infer_type(&bin.left)?;
                let right = self.infer_type(&bin.right)?;
                if bin.op == BinaryOp::Assign {
                    if let Expr::Ident(ident) = &*bin.left {
                        if let Some(SymbolKind::Var {
                            kind: VarDeclKind::Const,
                            ..
                        }) = self.resolve(&ident.sym)
                        {
                            return Err(SemaError::ConstAssignment(ident.sym.clone(), bin.span));
                        }
                    }
                    if !self.types_eq(&left, &right) {
                        return Err(SemaError::TypeMismatch(left, right, bin.span));
                    }
                    Ok(left)
                } else if bin.op == BinaryOp::Lt
                    || bin.op == BinaryOp::LtEq
                    || bin.op == BinaryOp::Gt
                    || bin.op == BinaryOp::GtEq
                    || bin.op == BinaryOp::NotEq
                    || bin.op == BinaryOp::Eq
                {
                    Ok(Type::Bool)
                } else if bin.op == BinaryOp::Add && (left == Type::String || right == Type::String)
                {
                    Ok(Type::String)
                } else {
                    if !self.types_eq(&left, &right) {
                        return Err(SemaError::TypeMismatch(left, right, bin.span));
                    }
                    Ok(left)
                }
            }
            Expr::Call(call) => {
                let callee_ty = self.infer_type(&call.callee)?;
                match callee_ty {
                    Type::Fn(_, ret) => Ok(*ret),
                    Type::Any => Ok(Type::Any),
                    _ => Err(SemaError::NotAFunction(call.span)),
                }
            }
            Expr::OptCall(_) | Expr::OptMember(_) => Ok(Type::Optional(Box::new(Type::Any))),
            Expr::New(call) => {
                if let Expr::Ident(ident) = &*call.callee {
                    if self.classes.contains_key(&ident.sym) {
                        return Ok(Type::Ref(ident.clone()));
                    }
                }
                Err(SemaError::UndefinedSymbol("Class".to_string(), call.span))
            }
            Expr::EnumInit(init) => Ok(Type::Ref(init.enum_name.clone())),
            Expr::Spawn(_) => Ok(Type::PID),
            Expr::Receive(ty) => Ok(ty.as_ref().map(|t| (**t).clone()).unwrap_or(Type::I32)),
            Expr::Slice(slice) => self.infer_type(&slice.obj),
            Expr::Question(inner) => {
                let inner_ty = self.infer_type(inner)?;
                match inner_ty {
                    Type::ResultType { ok, .. } => match *ok {
                        Type::Optional(t) => Ok(*t),
                        other => Ok(other),
                    },
                    Type::ErrorUnion(t) | Type::Optional(t) => Ok(*t),
                    _ => Ok(inner_ty),
                }
            }
            Expr::QuestionFallback { expr, fallback, .. }
            | Expr::NullCoalesce {
                left: expr,
                right: fallback,
            } => {
                let inner_ty = self.infer_type(expr)?;
                let fb_ty = self.infer_type(fallback)?;
                match inner_ty {
                    Type::ResultType { ok, .. } => match *ok {
                        Type::Optional(t) => Ok(*t),
                        other => Ok(other),
                    },
                    Type::ErrorUnion(t) | Type::Optional(t) => {
                        if *t == Type::Unknown || *t == Type::Any {
                            Ok(fb_ty)
                        } else {
                            Ok(*t)
                        }
                    }
                    _ => Ok(fb_ty),
                }
            }
            Expr::Catch { expr, .. } => {
                let inner_ty = self.infer_type(expr)?;
                match inner_ty {
                    Type::ResultType { ok, .. } => Ok(*ok),
                    Type::ErrorUnion(t) => Ok(*t),
                    _ => Ok(inner_ty),
                }
            }
            Expr::ErrWrap(_) | Expr::BoxNew(_) => Ok(Type::Any),
            Expr::Downcast {
                target_ty, is_mut, ..
            } => Ok(Type::Optional(Box::new(Type::BorrowType {
                inner: Box::new(Type::Ref(target_ty.clone())),
                is_mut: *is_mut,
                lifetimes: vec![],
            }))),
            Expr::Object(fields) => {
                let mut fs: Vec<Field> = Vec::new();
                for f in fields {
                    if f.is_spread {
                        let sp_ty = self.infer_type(&f.val)?;
                        if let Type::Object(sub_fs) = sp_ty {
                            for sf in sub_fs {
                                if let Some(pos) = fs
                                    .iter()
                                    .position(|existing| existing.ident.sym == sf.ident.sym)
                                {
                                    fs[pos] = sf;
                                } else {
                                    fs.push(sf);
                                }
                            }
                        }
                    } else if let Some(k) = &f.key {
                        let ty = self.infer_type(&f.val)?;
                        let new_f = Field {
                            span: k.span,
                            ident: k.clone(),
                            ty,
                            meta: Vec::new(),
                        };
                        if let Some(pos) =
                            fs.iter().position(|existing| existing.ident.sym == k.sym)
                        {
                            fs[pos] = new_f;
                        } else {
                            fs.push(new_f);
                        }
                    }
                }
                Ok(Type::Object(fs))
            }
            Expr::Dict(_) => Ok(Type::Dict(Box::new(Type::String), Box::new(Type::Any))),
            Expr::Index { obj, .. } => {
                let o_ty = self.infer_type(obj)?;
                match o_ty {
                    Type::Array(elem) => Ok(*elem),
                    Type::Map(_, v) | Type::Dict(_, v) => Ok(Type::Optional(v)),
                    _ => Ok(Type::Any),
                }
            }
            Expr::Array(elements) => {
                let mut tys = Vec::new();
                for el in elements {
                    tys.push(self.infer_type(el)?);
                }
                Ok(Type::Tuple(tys))
            }
            Expr::Await(inner) => {
                if !self.current_fn_is_async {
                    return Err(SemaError::AwaitOutsideAsync(inner_span(inner)));
                }
                self.infer_type(inner)
            }
            Expr::Borrow(inner, is_mut) => {
                let inner_ty = self.infer_type(inner)?;
                Ok(Type::BorrowType {
                    inner: Box::new(inner_ty),
                    is_mut: *is_mut,
                    lifetimes: vec![],
                })
            }
            Expr::Move(inner) => self.infer_type(inner),
            Expr::Builtin(call) => match call.name.as_str() {
                "@print" | "@fs_write_file" | "@fs_mkdir" | "@fs_remove" | "@fs_copy"
                | "@net_send" | "@net_close" | "@net_listen" => Ok(Type::Void),
                "@fs_read_file" | "@fs_read_async" | "@net_read_async" | "@http_fetch" => {
                    Ok(Type::String)
                }
                "@fs_exists" => Ok(Type::Bool),
                "@net_connect" => Ok(Type::I32),
                "@self" => Ok(Type::PID),
                "@yield_to_beam" | "@json_parse" | "@alloc" => Ok(Type::Any),
                "@get_arena" => Ok(Type::Ref(Ident {
                    span: Span { start: 0, end: 0 },
                    sym: "Arena".to_string(),
                })),
                "@io_read_stdin_line" => Ok(Type::String),
                "@io_write_stdout" | "@io_write_stderr" | "@print_err" | "@free" => Ok(Type::Void),
                "@shared_tensor_init" => Ok(Type::Ref(Ident {
                    span: Span { start: 0, end: 0 },
                    sym: "SharedTensor".to_string(),
                })),
                "@map" => {
                    if call.args.len() > 0 {
                        let inner = self.infer_type(&call.args[0])?;
                        Ok(inner)
                    } else {
                        Ok(Type::Unknown)
                    }
                }
                _ => Ok(Type::Unknown),
            },
            Expr::ReflectType { span, .. } | Expr::ReflectOf { span, .. } => Ok(Type::Ref(Ident {
                span: *span,
                sym: "TypeInfo".to_string(),
            })),
            _ => Ok(Type::Any),
        }
    }
}

fn inner_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident(i) => i.span,
        Expr::Lit(_) => Span { start: 0, end: 0 },
        Expr::Bin(b) => b.span,
        Expr::ReflectType { span, .. } | Expr::ReflectOf { span, .. } => *span,
        _ => Span { start: 0, end: 0 },
    }
}
