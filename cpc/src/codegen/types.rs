use super::*;
use crate::ast::*;
use crate::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

impl Codegen {
    pub fn unwrap_optional_type(ty: &Type) -> Type {
        match ty {
            Type::Optional(inner) => *inner.clone(),
            Type::BorrowType {
                inner,
                is_mut,
                lifetimes,
            } => {
                if let Type::Optional(opt_inner) = &**inner {
                    Type::BorrowType {
                        inner: opt_inner.clone(),
                        is_mut: *is_mut,
                        lifetimes: lifetimes.clone(),
                    }
                } else {
                    ty.clone()
                }
            }
            _ => ty.clone(),
        }
    }

    pub fn extract_not_null_vars(expr: &Expr) -> Vec<String> {
        let mut vars = Vec::new();
        match expr {
            Expr::Bin(bin) => {
                if bin.op == BinaryOp::NotEq {
                    if matches!(&*bin.right, Expr::Lit(Lit::Null))
                        || matches!(&*bin.right, Expr::Ident(id) if id.sym == "null")
                    {
                        if let Expr::Ident(id) = &*bin.left {
                            if id.sym != "null" && id.sym != "undefined" {
                                vars.push(id.sym.clone());
                            }
                        }
                    } else if matches!(&*bin.left, Expr::Lit(Lit::Null))
                        || matches!(&*bin.left, Expr::Ident(id) if id.sym == "null")
                    {
                        if let Expr::Ident(id) = &*bin.right {
                            if id.sym != "null" && id.sym != "undefined" {
                                vars.push(id.sym.clone());
                            }
                        }
                    }
                } else if bin.op == BinaryOp::LogicalAnd {
                    vars.extend(Self::extract_not_null_vars(&bin.left));
                    vars.extend(Self::extract_not_null_vars(&bin.right));
                }
            }
            Expr::Builtin(call) if call.name == "@not" && call.args.len() == 1 => {
                vars.extend(Self::extract_is_null_vars(&call.args[0]));
            }
            _ => {}
        }
        vars
    }

    pub fn extract_is_null_vars(expr: &Expr) -> Vec<String> {
        let mut vars = Vec::new();
        match expr {
            Expr::Bin(bin) => {
                if bin.op == BinaryOp::Eq {
                    if matches!(&*bin.right, Expr::Lit(Lit::Null))
                        || matches!(&*bin.right, Expr::Ident(id) if id.sym == "null")
                    {
                        if let Expr::Ident(id) = &*bin.left {
                            if id.sym != "null" && id.sym != "undefined" {
                                vars.push(id.sym.clone());
                            }
                        }
                    } else if matches!(&*bin.left, Expr::Lit(Lit::Null))
                        || matches!(&*bin.left, Expr::Ident(id) if id.sym == "null")
                    {
                        if let Expr::Ident(id) = &*bin.right {
                            if id.sym != "null" && id.sym != "undefined" {
                                vars.push(id.sym.clone());
                            }
                        }
                    }
                } else if bin.op == BinaryOp::LogicalOr {
                    vars.extend(Self::extract_is_null_vars(&bin.left));
                    vars.extend(Self::extract_is_null_vars(&bin.right));
                }
            }
            Expr::Builtin(call) if call.name == "@not" && call.args.len() == 1 => {
                vars.extend(Self::extract_not_null_vars(&call.args[0]));
            }
            _ => {}
        }
        vars
    }

    pub fn type_from_name_static(name: &str) -> Type {
        match name {
            "i8" => Type::I8,
            "u8" => Type::U8,
            "i16" => Type::I16,
            "u16" => Type::U16,
            "i32" => Type::I32,
            "u32" => Type::U32,
            "i64" => Type::I64,
            "u64" => Type::U64,
            "isize" => Type::ISize,
            "usize" => Type::USize,
            "f32" => Type::F32,
            "f64" => Type::F64,
            "bool" => Type::Bool,
            "string" => Type::String,
            _ => Type::Ref(Ident {
                sym: name.to_string(),
                span: Span { start: 0, end: 0 },
            }),
        }
    }

    pub fn fnv1a_type_id(name: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for b in name.as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    pub fn is_numeric_type(ty: &Type) -> bool {
        matches!(
            ty,
            Type::I8
                | Type::U8
                | Type::I16
                | Type::U16
                | Type::I32
                | Type::U32
                | Type::I64
                | Type::U64
                | Type::I128
                | Type::U128
                | Type::ISize
                | Type::USize
                | Type::F32
                | Type::F64
        )
    }

    pub fn is_scalar_or_primitive_type(ty: &Type) -> bool {
        match ty {
            Type::I8
            | Type::U8
            | Type::I16
            | Type::U16
            | Type::I32
            | Type::U32
            | Type::I64
            | Type::U64
            | Type::I128
            | Type::U128
            | Type::ISize
            | Type::USize
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Void
            | Type::String
            | Type::RawPtr { .. } => true,
            Type::BorrowType { .. } => true,
            Type::FixedArray(inner, _) | Type::Array(inner) => {
                Self::is_scalar_or_primitive_type(inner)
            }
            Type::Optional(inner) => Self::is_scalar_or_primitive_type(inner),
            _ => false,
        }
    }

    pub fn type_has_release(&self, ty: Option<&Type>) -> bool {
        let Some(mut t) = ty else {
            return false;
        };
        while let Type::BorrowType { inner, .. } = t {
            t = inner;
        }
        while let Type::RawPtr { inner, .. } = t {
            t = inner;
        }
        while let Type::Optional(inner) = t {
            t = inner;
        }
        if Self::is_scalar_or_primitive_type(t) {
            return false;
        }
        match t {
            Type::Ref(id) => {
                if let Some(cls) = self.class_defs.get(&id.sym) {
                    cls.methods.iter().any(|m| m.ident.sym == "release")
                } else {
                    id.sym.starts_with("LockGuard")
                        || id.sym.starts_with("ReadGuard")
                        || id.sym.starts_with("WriteGuard")
                        || matches!(
                            id.sym.as_str(),
                            "Map"
                                | "Dict"
                                | "Set"
                                | "HashMap"
                                | "HashSet"
                                | "VecDeque"
                                | "BinaryHeap"
                                | "BTreeMap"
                                | "BTreeSet"
                                | "RegExp"
                        )
                }
            }
            Type::BoxDyn(_) => true,
            _ => false,
        }
    }

    pub fn map_error_type(&self, err: &ErrorType) -> String {
        match err {
            ErrorType::Named(id) => {
                if id.sym == "Err" || id.sym == "Error" || id.sym == "err" || id.sym == "error" {
                    "Err".to_string()
                } else if id.sym == "AnyErr" || id.sym == "anyErr" || id.sym == "anyerror" {
                    "anyerror".to_string()
                } else if self.ref_classes.contains(&id.sym) || self.value_classes.contains(&id.sym)
                {
                    id.sym.clone()
                } else if self.class_defs.contains_key(&id.sym) {
                    format!("*{}", id.sym)
                } else {
                    id.sym.clone()
                }
            }
            ErrorType::Union(idents) => {
                let has_err = idents
                    .iter()
                    .any(|i| i.sym == "Err" || i.sym == "Error" || i.sym == "err");
                if has_err {
                    "Err".to_string()
                } else if idents.iter().all(|i| {
                    self.error_sets.contains_key(&i.sym)
                        || i.sym.starts_with("error{")
                        || i.sym == "anyerror"
                }) {
                    let names: Vec<String> = idents
                        .iter()
                        .map(|i| {
                            if i.sym == "anyerror" {
                                "anyerror".to_string()
                            } else {
                                i.sym.clone()
                            }
                        })
                        .collect();
                    format!("({})", names.join(" || "))
                } else {
                    let names: Vec<String> = idents.iter().map(|i| i.sym.clone()).collect();
                    format!("__UnionErr_{}", names.join("_"))
                }
            }
            ErrorType::AnonSet(idents) => {
                let names: Vec<String> = idents.iter().map(|i| i.sym.clone()).collect();
                format!("__AnonErr_{}", names.join("_"))
            }
            ErrorType::BoxDyn(id) => id.sym.clone(),
            ErrorType::DefaultError => "Err".to_string(),
            ErrorType::AnyErr => "anyerror".to_string(),
        }
    }

    pub fn check_type_validity_and_collect_errors(&mut self, ty: &Type) {
        match ty {
            Type::BareDyn(tr) => {
                panic!(
                    "CP Compile Error: Bare 'dyn {}' cannot be placed on the stack; wrap it in 'Box<dyn {}>'",
                    tr.sym, tr.sym
                );
            }
            Type::ResultType { ok, err } => {
                self.check_type_validity_and_collect_errors(ok);
                match err {
                    ErrorType::AnonSet(idents) => {
                        let zig_name = self.map_error_type(err);
                        if !self.anon_err_sets.iter().any(|(n, _)| n == &zig_name) {
                            let variants = idents.iter().map(|i| i.sym.clone()).collect();
                            self.anon_err_sets.push((zig_name, variants));
                        }
                    }
                    ErrorType::Union(idents) => {
                        let has_err = idents
                            .iter()
                            .any(|i| i.sym == "Err" || i.sym == "Error" || i.sym == "err");
                        let all_err_sets = idents.iter().all(|i| {
                            self.error_sets.contains_key(&i.sym)
                                || i.sym.starts_with("error{")
                                || i.sym == "anyerror"
                        });
                        if !has_err && !all_err_sets {
                            let zig_name = self.map_error_type(err);
                            if !self.union_err_sets.iter().any(|(n, _)| n == &zig_name) {
                                let sub_names = idents.iter().map(|i| i.sym.clone()).collect();
                                self.union_err_sets.push((zig_name, sub_names));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Type::Optional(inner)
            | Type::Array(inner)
            | Type::Shared(inner)
            | Type::ErrorUnion(inner)
            | Type::RawPtr { inner, .. }
            | Type::BorrowType { inner, .. } => {
                self.check_type_validity_and_collect_errors(inner);
            }
            Type::Tuple(tys) => {
                for t in tys {
                    self.check_type_validity_and_collect_errors(t);
                }
            }
            Type::Object(fields) => {
                for f in fields {
                    self.check_type_validity_and_collect_errors(&f.ty);
                }
            }
            _ => {}
        }
    }

    pub fn scan_stmt_types(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(v) => {
                if let Some(t) = &v.ty {
                    self.check_type_validity_and_collect_errors(t);
                }
            }
            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                for s in &b.body {
                    self.scan_stmt_types(s);
                }
            }
            Stmt::If(i) => {
                self.scan_stmt_types(&i.cons);
                if let Some(alt) = &i.alt {
                    self.scan_stmt_types(alt);
                }
            }
            Stmt::While(w) => self.scan_stmt_types(&w.body),
            Stmt::For(f) => self.scan_stmt_types(&f.body),
            Stmt::Match(m) => {
                for arm in &m.arms {
                    self.scan_stmt_types(&arm.body);
                }
            }
            Stmt::TryCatch {
                try_block,
                catch_body,
                ..
            } => {
                for s in &try_block.body {
                    self.scan_stmt_types(s);
                }
                if let Some(cb) = catch_body {
                    self.scan_stmt_types(cb);
                }
            }
            _ => {}
        }
    }

    pub fn verify_error_type_compatibility(&self, src: &ErrorType, dst: &ErrorType) {
        if src == dst {
            return;
        }
        let src_name = self.map_error_type(src);
        let dst_name = self.map_error_type(dst);
        if src_name == dst_name
            || dst_name == "Error"
            || dst_name == "Err"
            || dst_name == "anyerror"
            || dst_name == "AnyErr"
        {
            return;
        }
        match (src, dst) {
            (ErrorType::Named(s_id), ErrorType::Union(d_ids)) => {
                if d_ids.iter().any(|d| d.sym == s_id.sym) {
                    return;
                }
            }
            (ErrorType::Union(s_ids), ErrorType::Union(d_ids)) => {
                if s_ids.iter().all(|s| d_ids.iter().any(|d| d.sym == s.sym)) {
                    return;
                }
            }
            (ErrorType::AnonSet(s_ids), ErrorType::AnonSet(d_ids)) => {
                if s_ids.iter().all(|s| d_ids.iter().any(|d| d.sym == s.sym)) {
                    return;
                }
            }
            (ErrorType::Named(s_id), ErrorType::Named(d_id)) => {
                if let Some(en) = self.enum_defs.get(&d_id.sym) {
                    if en
                        .variants
                        .iter()
                        .any(|v| v.from_err.as_ref().map_or(false, |f| f.sym == s_id.sym))
                    {
                        return;
                    }
                }
                if self.trait_defs.contains_key(&d_id.sym) {
                    if let Some(cls) = self.class_defs.get(&s_id.sym) {
                        if cls.implements.iter().any(|i| i.sym == d_id.sym) {
                            return;
                        }
                    }
                }
            }
            (ErrorType::Named(s_id), ErrorType::BoxDyn(tr_id)) => {
                if s_id.sym == tr_id.sym {
                    return;
                }
                if let Some(cls) = self.class_defs.get(&s_id.sym) {
                    if cls.implements.iter().any(|i| i.sym == tr_id.sym) {
                        return;
                    }
                }
            }
            _ => {}
        }
        panic!(
            "CP Compile Error: Error type '{}' cannot be automatically converted to '{}' (missing 'from {}' in target error enum)",
            src_name, dst_name, src_name
        );
    }

    pub fn is_class_field(&self, field_name: &str) -> bool {
        if let Some(c_name) = &self.current_class_name {
            if let Some(cls) = self.class_defs.get(c_name) {
                return cls.fields.iter().any(|f| f.ident.sym == field_name);
            }
        }
        false
    }

    pub fn lookup_method(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<(Vec<Param>, Option<Type>, ReceiverMode)> {
        let clean = type_name.trim_start_matches('?').trim_start_matches('*');
        let base_name = if let Some(pos) = clean.find('<') {
            &clean[..pos]
        } else {
            clean
        };
        if let Some(cls) = self.class_defs.get(base_name) {
            if let Some(m) = cls.methods.iter().find(|m| m.ident.sym == method_name) {
                return Some((
                    m.params.clone(),
                    m.return_type.clone(),
                    Self::get_func_receiver_mode(m),
                ));
            }
        }
        if let Some(tr) = self.trait_defs.get(base_name) {
            if let Some(tm) = tr.methods.iter().find(|m| m.ident.sym == method_name) {
                return Some((
                    tm.params.clone(),
                    tm.return_type.clone(),
                    Self::get_sig_receiver_mode(tm),
                ));
            }
        }
        None
    }

    pub fn resolve_var_type_name(&self, var_name: &str) -> Option<String> {
        if var_name == "this" || var_name == "_" || var_name == "self" {
            return self.current_class_name.clone();
        }
        if let Some(ty) = self.var_cp_types.get(var_name) {
            match ty {
                Type::Ref(id) | Type::BoxDyn(id) => return Some(id.sym.clone()),
                Type::RawPtr { inner, .. } => {
                    if let Type::Ref(id) | Type::BoxDyn(id) = &**inner {
                        return Some(id.sym.clone());
                    }
                }
                Type::BorrowType { inner, .. } => {
                    if let Type::Ref(id) | Type::BoxDyn(id) = &**inner {
                        return Some(id.sym.clone());
                    }
                }
                Type::Optional(inner) => match &**inner {
                    Type::Ref(id) | Type::BoxDyn(id) => return Some(id.sym.clone()),
                    Type::RawPtr { inner: r_in, .. } => {
                        if let Type::Ref(id) | Type::BoxDyn(id) = &**r_in {
                            return Some(id.sym.clone());
                        }
                    }
                    Type::BorrowType { inner: b_in, .. } => {
                        if let Type::Ref(id) | Type::BoxDyn(id) = &**b_in {
                            return Some(id.sym.clone());
                        }
                    }
                    _ => {}
                },
                Type::Shared(inner) => {
                    if let Type::Ref(id) = &**inner {
                        return Some(id.sym.clone());
                    }
                }
                _ => {}
            }
        }

        if let Some(zig_ty) = self.locals.get(var_name) {
            let clean = zig_ty.trim_start_matches('?').trim_start_matches('*');
            if clean == "Error"
                || self.class_defs.contains_key(clean)
                || self.trait_defs.contains_key(clean)
            {
                return Some(clean.to_string());
            }
        }
        None
    }

    pub(crate) fn infer_cp_type(&self, expr: &Expr) -> Option<Type> {
        self.infer_cp_type_with_mock(expr, &HashMap::new())
    }

    pub(crate) fn infer_cp_type_with_mock(
        &self,
        expr: &Expr,
        mock_types: &HashMap<String, Type>,
    ) -> Option<Type> {
        match expr {
            Expr::Lit(Lit::Int(_)) => Some(Type::I32),
            Expr::Lit(Lit::Float(_)) => Some(Type::F32),
            Expr::Lit(Lit::Str(_)) => Some(Type::String),
            Expr::Lit(Lit::Bool(_)) => Some(Type::Bool),
            Expr::Template(_) => Some(Type::String),
            Expr::Ident(id) => {
                if let Some(t) = mock_types.get(&id.sym) {
                    return Some(t.clone());
                }
                if id.sym == "true" || id.sym == "false" {
                    return Some(Type::Bool);
                }
                if id.sym == "this" || id.sym == "_" || id.sym == "self" {
                    return self.current_class_name.as_ref().map(|c| {
                        Type::Ref(Ident {
                            span: Span { start: 0, end: 0 },
                            sym: c.clone(),
                        })
                    });
                }
                if let Some(t) = self
                    .var_cp_types
                    .get(&id.sym)
                    .or_else(|| self.global_var_types.get(&id.sym))
                {
                    if self.narrowed_null_vars.contains(&id.sym) {
                        return Some(Self::unwrap_optional_type(t));
                    }
                    return Some(t.clone());
                }
                if let Some(c_name) = &self.current_class_name {
                    if let Some(cls) = self.class_defs.get(c_name) {
                        if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == id.sym) {
                            return Some(f.ty.clone());
                        }
                    }
                }
                None
            }
            Expr::New(nc) => {
                if let Expr::Ident(cid) = &*nc.callee {
                    return Some(Type::Ref(cid.clone()));
                }
                None
            }
            Expr::BoxNew(_) => self.expected_box_trait.as_ref().map(|tr| {
                Type::BoxDyn(Ident {
                    span: Span { start: 0, end: 0 },
                    sym: tr.clone(),
                })
            }),
            Expr::Downcast {
                target_ty, is_mut, ..
            } => Some(Type::Optional(Box::new(Type::BorrowType {
                inner: Box::new(Type::Ref(target_ty.clone())),
                is_mut: *is_mut,
                lifetimes: vec![],
            }))),
            Expr::EnumInit(ei) => Some(Type::Ref(ei.enum_name.clone())),
            Expr::Cast {
                expr, target_ty, ..
            } => {
                if let Type::RawPtr { inner, depth } = target_ty {
                    if **inner == Type::Any {
                        if let Expr::Borrow(b, _) = &**expr {
                            if let Some(mut base_ty) = self.infer_cp_type_with_mock(b, mock_types) {
                                for _ in 1..*depth {
                                    if let Type::RawPtr {
                                        inner: sub,
                                        depth: d,
                                    } = base_ty
                                    {
                                        if d > 1 {
                                            base_ty = Type::RawPtr {
                                                inner: sub,
                                                depth: d - 1,
                                            };
                                        } else {
                                            base_ty = *sub;
                                        }
                                    }
                                }
                                return Some(Type::RawPtr {
                                    inner: Box::new(base_ty),
                                    depth: *depth,
                                });
                            }
                        }
                    }
                }
                Some(target_ty.clone())
            }
            Expr::Deref(inner) => {
                let inner_ty = self.infer_cp_type_with_mock(inner, mock_types)?;
                match inner_ty {
                    Type::RawPtr { inner, depth } => {
                        if depth > 1 {
                            Some(Type::RawPtr {
                                inner,
                                depth: depth - 1,
                            })
                        } else {
                            Some(*inner)
                        }
                    }
                    Type::BorrowType { inner, .. } => Some(*inner),
                    other => Some(other),
                }
            }
            Expr::BitNot(inner) => self.infer_cp_type_with_mock(inner, mock_types),
            Expr::Move(inner) => self.infer_cp_type_with_mock(inner, mock_types),
            Expr::Borrow(inner, is_mut) => {
                let inner_ty = self.infer_cp_type_with_mock(inner, mock_types)?;
                match inner_ty {
                    Type::BorrowType { inner: sub, .. } => Some(Type::BorrowType {
                        inner: sub,
                        is_mut: *is_mut,
                        lifetimes: vec![],
                    }),
                    other => Some(Type::BorrowType {
                        inner: Box::new(other),
                        is_mut: *is_mut,
                        lifetimes: vec![],
                    }),
                }
            }
            Expr::Await(inner) => self.infer_cp_type_with_mock(inner, mock_types),
            Expr::Call(c) => {
                if let Expr::Ident(fid) = &*c.callee {
                    if fid.sym == "join" || fid.sym == "@task_join" {
                        let mut tup_tys = Vec::new();
                        for _ in &c.args {
                            tup_tys.push(Type::String);
                        }
                        return Some(Type::Tuple(tup_tys));
                    }
                    if fid.sym == "spawn"
                        || fid.sym == "@task_spawn"
                        || fid.sym == "@task_spawn_cpu"
                    {
                        return Some(Type::Ref(Ident {
                            span: fid.span,
                            sym: "Task".to_string(),
                        }));
                    }
                    if fid.sym == "next" {
                        return self.current_fn_ret_ty.clone();
                    }
                    if fid.sym == "Error" {
                        return Some(Type::Ref(Ident {
                            span: Span { start: 0, end: 0 },
                            sym: "Error".to_string(),
                        }));
                    }
                    if fid.sym == "Mutex" {
                        if c.args.is_empty() {
                            return Some(Type::Ref(Ident {
                                sym: "Mutex".to_string(),
                                span: fid.span,
                            }));
                        } else {
                            let inner = self
                                .infer_cp_type_with_mock(&c.args[0], mock_types)
                                .unwrap_or(Type::I32);
                            return Some(Type::Ref(Ident {
                                sym: format!("Mutex<{}>", type_to_display_string(&inner)),
                                span: fid.span,
                            }));
                        }
                    }
                    if fid.sym == "RwLock" {
                        if c.args.is_empty() {
                            return Some(Type::Ref(Ident {
                                sym: "RwLock".to_string(),
                                span: fid.span,
                            }));
                        } else {
                            let inner = self
                                .infer_cp_type_with_mock(&c.args[0], mock_types)
                                .unwrap_or(Type::I32);
                            return Some(Type::Ref(Ident {
                                sym: format!("RwLock<{}>", type_to_display_string(&inner)),
                                span: fid.span,
                            }));
                        }
                    }
                    if fid.sym == "Atomic" {
                        let inner = if !c.args.is_empty() {
                            self.infer_cp_type_with_mock(&c.args[0], mock_types)
                                .unwrap_or(Type::I32)
                        } else {
                            Type::I32
                        };
                        return Some(Type::Ref(Ident {
                            sym: format!("Atomic<{}>", type_to_display_string(&inner)),
                            span: fid.span,
                        }));
                    }
                    if fid.sym == "Shared" {
                        if !c.args.is_empty() {
                            let inner = self
                                .infer_cp_type_with_mock(&c.args[0], mock_types)
                                .unwrap_or(Type::I32);
                            return Some(Type::Shared(Box::new(inner)));
                        }
                    }
                    if (fid.sym == "string" || fid.sym == "String") && c.args.len() == 1 {
                        return Some(Type::String);
                    }
                    if fid.sym == "@alloc" {
                        return Some(Type::ManyRawPtr(Box::new(Type::U8)));
                    }
                    match fid.sym.as_str() {
                        "i8" => return Some(Type::I8),
                        "u8" => return Some(Type::U8),
                        "i16" => return Some(Type::I16),
                        "u16" => return Some(Type::U16),
                        "i32" => return Some(Type::I32),
                        "u32" => return Some(Type::U32),
                        "i64" => return Some(Type::I64),
                        "u64" => return Some(Type::U64),
                        "i128" => return Some(Type::I128),
                        "u128" => return Some(Type::U128),
                        "isize" => return Some(Type::ISize),
                        "usize" => return Some(Type::USize),
                        "f32" => return Some(Type::F32),
                        "f64" => return Some(Type::F64),
                        "bool" => return Some(Type::Bool),
                        _ => {}
                    }
                    if self.enum_defs.contains_key(&fid.sym) && c.args.len() == 1 {
                        return Some(Type::Ref(fid.clone()));
                    }
                    if let Some(sig) = self.fn_sigs.get(&fid.sym) {
                        if let Some(rt) = &sig.return_type {
                            if let Type::Ref(rid) = rt {
                                for (idx, p) in sig.params.iter().enumerate() {
                                    let mut p_ty = &p.ty;
                                    while let Type::BorrowType { inner, .. } = p_ty {
                                        p_ty = inner;
                                    }
                                    if let Type::Ref(pid) = p_ty {
                                        if pid.sym == rid.sym && idx < c.args.len() {
                                            if let Some(arg_ty) = self
                                                .infer_cp_type_with_mock(&c.args[idx], mock_types)
                                            {
                                                return Some(arg_ty);
                                            }
                                        }
                                    }
                                }
                            }
                            if self.in_try_region {
                                if let Type::ResultType { ok, .. } = rt {
                                    return Some((**ok).clone());
                                }
                            }
                            return Some(rt.clone());
                        }
                    } else if let Some(c_name) = &self.current_class_name {
                        if let Some((_, Some(rt), _)) = self.lookup_method(c_name, &fid.sym) {
                            if self.in_try_region {
                                if let Type::ResultType { ok, .. } = &rt {
                                    return Some((**ok).clone());
                                }
                            }
                            return Some(rt);
                        }
                    }
                } else if let Expr::Member(mem) = &*c.callee {
                    if mem.prop.sym == "toString" && c.args.is_empty() {
                        return Some(Type::String);
                    }
                    if let Some(mut oty) = self.infer_cp_type_with_mock(&mem.obj, mock_types) {
                        while let Type::BorrowType { inner, .. } = oty {
                            oty = *inner;
                        }
                        while let Type::Shared(inner) = oty {
                            oty = *inner;
                        }
                        while let Type::RawPtr { inner, .. } = oty {
                            oty = *inner;
                        }
                        if let Type::Ref(id) = &oty {
                            let prop = mem.prop.sym.as_str();
                            if id.sym.starts_with("Mutex<") {
                                let inner_sym = &id.sym[6..id.sym.len() - 1];
                                if prop == "lock" {
                                    return Some(Type::Ref(Ident {
                                        sym: format!("LockGuard<{}>", inner_sym),
                                        span: mem.prop.span,
                                    }));
                                }
                                if prop == "tryLock" {
                                    return Some(Type::Optional(Box::new(Type::Ref(Ident {
                                        sym: format!("LockGuard<{}>", inner_sym),
                                        span: mem.prop.span,
                                    }))));
                                }
                                if prop == "isLocked" {
                                    return Some(Type::Bool);
                                }
                                if prop == "withLock" && !c.args.is_empty() {
                                    if let Expr::Arrow(arr) = &c.args[0] {
                                        let mut sub_mock = mock_types.clone();
                                        if !arr.params.is_empty() {
                                            if let Pattern::Ident(pid) = &arr.params[0] {
                                                sub_mock.insert(
                                                    pid.sym.clone(),
                                                    Self::type_from_name_static(inner_sym),
                                                );
                                            }
                                        }
                                        match &*arr.body {
                                            ArrowBody::Expr(e) => {
                                                return self.infer_cp_type_with_mock(e, &sub_mock)
                                            }
                                            ArrowBody::Block(b) => {
                                                if let Some(Stmt::Return(r)) = b.body.last() {
                                                    if let Some(a) = &r.arg {
                                                        return self
                                                            .infer_cp_type_with_mock(a, &sub_mock);
                                                    }
                                                }
                                                return Some(Type::Void);
                                            }
                                        }
                                    }
                                }
                            } else if id.sym == "Mutex" {
                                if prop == "tryLock" || prop == "isLocked" {
                                    return Some(Type::Bool);
                                }
                                if prop == "lock" || prop == "unlock" {
                                    return Some(Type::Void);
                                }
                            } else if id.sym.starts_with("RwLock<") {
                                let inner_sym = &id.sym[7..id.sym.len() - 1];
                                if prop == "read" {
                                    return Some(Type::Ref(Ident {
                                        sym: format!("ReadGuard<{}>", inner_sym),
                                        span: mem.prop.span,
                                    }));
                                }
                                if prop == "tryRead" {
                                    return Some(Type::Optional(Box::new(Type::Ref(Ident {
                                        sym: format!("ReadGuard<{}>", inner_sym),
                                        span: mem.prop.span,
                                    }))));
                                }
                                if prop == "write" {
                                    return Some(Type::Ref(Ident {
                                        sym: format!("WriteGuard<{}>", inner_sym),
                                        span: mem.prop.span,
                                    }));
                                }
                                if prop == "tryWrite" {
                                    return Some(Type::Optional(Box::new(Type::Ref(Ident {
                                        sym: format!("WriteGuard<{}>", inner_sym),
                                        span: mem.prop.span,
                                    }))));
                                }
                                if prop == "isReadLocked" || prop == "isWriteLocked" {
                                    return Some(Type::Bool);
                                }
                                if (prop == "withRead" || prop == "withWrite") && !c.args.is_empty()
                                {
                                    if let Expr::Arrow(arr) = &c.args[0] {
                                        let mut sub_mock = mock_types.clone();
                                        if !arr.params.is_empty() {
                                            if let Pattern::Ident(pid) = &arr.params[0] {
                                                sub_mock.insert(
                                                    pid.sym.clone(),
                                                    Self::type_from_name_static(inner_sym),
                                                );
                                            }
                                        }
                                        match &*arr.body {
                                            ArrowBody::Expr(e) => {
                                                return self.infer_cp_type_with_mock(e, &sub_mock)
                                            }
                                            ArrowBody::Block(b) => {
                                                if let Some(Stmt::Return(r)) = b.body.last() {
                                                    if let Some(a) = &r.arg {
                                                        return self
                                                            .infer_cp_type_with_mock(a, &sub_mock);
                                                    }
                                                }
                                                return Some(Type::Void);
                                            }
                                        }
                                    }
                                }
                            } else if id.sym == "RwLock" {
                                if prop == "tryLock"
                                    || prop == "tryLockShared"
                                    || prop == "isReadLocked"
                                    || prop == "isWriteLocked"
                                {
                                    return Some(Type::Bool);
                                }
                                if prop == "lock"
                                    || prop == "unlock"
                                    || prop == "lockShared"
                                    || prop == "unlockShared"
                                {
                                    return Some(Type::Void);
                                }
                            } else if id.sym.starts_with("Atomic<") {
                                let inner_sym = &id.sym[7..id.sym.len() - 1];
                                let inner_ty = Self::type_from_name_static(inner_sym);
                                if prop == "load"
                                    || prop == "swap"
                                    || prop == "add"
                                    || prop == "sub"
                                    || prop == "inc"
                                    || prop == "dec"
                                    || prop == "and"
                                    || prop == "or"
                                    || prop == "xor"
                                {
                                    return Some(inner_ty);
                                }
                                if prop == "cas" || prop == "compareAndSwap" || prop == "testAndSet"
                                {
                                    return Some(Type::Bool);
                                }
                                if prop == "store" || prop == "clear" {
                                    return Some(Type::Void);
                                }
                            } else if id.sym.starts_with("LockGuard<")
                                || id.sym.starts_with("ReadGuard<")
                                || id.sym.starts_with("WriteGuard<")
                            {
                                if prop == "release" || prop == "unlock" {
                                    return Some(Type::Void);
                                }
                            }
                        }
                    }
                    if let Expr::Ident(oid) = &*mem.obj {
                        if (oid.sym == "string" || oid.sym == "String") && mem.prop.sym == "from" {
                            return Some(Type::String);
                        }
                        if oid.sym == "reflect" {
                            if mem.prop.sym == "hasMethod"
                                || mem.prop.sym == "hasField"
                                || mem.prop.sym == "has"
                            {
                                return Some(Type::Bool);
                            }
                        }
                        if oid.sym == "spawn" && mem.prop.sym == "cpu" {
                            return Some(Type::Ref(Ident {
                                span: mem.prop.span,
                                sym: "Task".to_string(),
                            }));
                        }
                        let is_enum = self.enum_defs.contains_key(&oid.sym)
                            || (!self.locals.contains_key(&oid.sym)
                                && !mock_types.contains_key(&oid.sym)
                                && !self.global_var_types.contains_key(&oid.sym)
                                && oid
                                    .sym
                                    .chars()
                                    .next()
                                    .map_or(false, |c| c.is_ascii_uppercase())
                                && !self.class_defs.contains_key(&oid.sym));
                        if is_enum {
                            if mem.prop.sym == "fromName"
                                || mem.prop.sym == "fromString"
                                || mem.prop.sym == "get"
                            {
                                return Some(Type::Optional(Box::new(Type::Ref(oid.clone()))));
                            }
                            if mem.prop.sym == "has" {
                                return Some(Type::Bool);
                            }
                            return Some(Type::Ref(oid.clone()));
                        }
                        if mem.prop.sym == "get" {
                            if let Some(mut oty) =
                                self.infer_cp_type_with_mock(&mem.obj, mock_types)
                            {
                                while let Type::BorrowType { inner, .. } = oty {
                                    oty = *inner;
                                }
                                if let Type::Map(_, val_ty) | Type::Dict(_, val_ty) = oty {
                                    return Some(Type::Optional(val_ty));
                                }
                            }
                            return Some(Type::Optional(Box::new(Type::Ref(Ident {
                                span: Span { start: 0, end: 0 },
                                sym: "Message".to_string(),
                            }))));
                        }
                        if mem.prop.sym == "push"
                            || mem.prop.sym == "clear"
                            || mem.prop.sym == "put_all"
                        {
                            return Some(Type::Void);
                        }
                        if mem.prop.sym == "has"
                            || mem.prop.sym == "delete"
                            || mem.prop.sym == "includes"
                        {
                            return Some(Type::Bool);
                        }
                        if mem.prop.sym == "pop" {
                            if let Some(mut oty) =
                                self.infer_cp_type_with_mock(&mem.obj, mock_types)
                            {
                                while let Type::BorrowType { inner, .. } = oty {
                                    oty = *inner;
                                }
                                if let Type::Array(elem) = oty {
                                    return Some(Type::Optional(elem));
                                }
                            }
                        }
                        if mem.prop.sym == "getField" {
                            return Some(Type::Optional(Box::new(Type::Ref(Ident {
                                span: Span { start: 0, end: 0 },
                                sym: "FieldInfo".to_string(),
                            }))));
                        }
                        if mem.prop.sym == "getMethod" {
                            return Some(Type::Optional(Box::new(Type::Ref(Ident {
                                span: Span { start: 0, end: 0 },
                                sym: "MethodInfo".to_string(),
                            }))));
                        }
                        let tname_opt = if let Some(mty) = mock_types.get(&oid.sym) {
                            match mty {
                                Type::Ref(id) | Type::BoxDyn(id) => Some(id.sym.clone()),
                                Type::BorrowType { inner, .. } => {
                                    if let Type::Ref(id) | Type::BoxDyn(id) = &**inner {
                                        Some(id.sym.clone())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            }
                        } else {
                            self.resolve_var_type_name(&oid.sym).or_else(|| {
                                if self.class_defs.contains_key(&oid.sym) {
                                    Some(oid.sym.clone())
                                } else {
                                    None
                                }
                            })
                        };
                        if let Some(tname) = tname_opt {
                            if let Some((_, Some(rt), _)) =
                                self.lookup_method(&tname, &mem.prop.sym)
                            {
                                if self.in_try_region {
                                    if let Type::ResultType { ok, .. } = &rt {
                                        return Some((**ok).clone());
                                    }
                                }
                                return Some(rt);
                            }
                        }
                    }
                }
                if let Some(mut callee_ty) = self.infer_cp_type_with_mock(&c.callee, mock_types) {
                    while let Type::BorrowType { inner, .. } = callee_ty {
                        callee_ty = *inner;
                    }
                    if let Type::Ref(rid) = &callee_ty {
                        if let Some(target) = self.type_aliases.get(&rid.sym) {
                            callee_ty = target.target_ty.clone();
                        }
                    }
                    if let Type::Fn(_, ret) | Type::CFn(_, ret) = callee_ty {
                        return Some(*ret);
                    }
                }
                None
            }
            Expr::Member(mem) => {
                if mem.prop.sym == "length" || mem.prop.sym == "len" {
                    return Some(Type::USize);
                }
                if let Some(mut obj_ty) = self.infer_cp_type_with_mock(&mem.obj, mock_types) {
                    while let Type::BorrowType { inner, .. } = obj_ty {
                        obj_ty = *inner;
                    }
                    while let Type::RawPtr { inner, .. } = obj_ty {
                        obj_ty = *inner;
                    }
                    while let Type::Shared(inner) = obj_ty {
                        obj_ty = *inner;
                    }
                    if mem.prop.sym == "size"
                        && matches!(obj_ty, Type::Map(_, _) | Type::Dict(_, _) | Type::Set(_))
                    {
                        return Some(Type::I32);
                    }
                    if let Type::Object(fields) = &obj_ty {
                        if let Some(f) = fields.iter().find(|f| f.ident.sym == mem.prop.sym) {
                            return Some(f.ty.clone());
                        }
                    }
                    if let Type::Ref(id) = &obj_ty {
                        if self.enum_defs.contains_key(&id.sym) {
                            if mem.prop.sym == "name" {
                                return Some(Type::String);
                            }
                            if mem.prop.sym == "value" {
                                return Some(Type::I32);
                            }
                        }
                        let tname = &id.sym;
                        let guard_inner = if tname.starts_with("LockGuard<") {
                            Some(&tname[10..tname.len() - 1])
                        } else if tname.starts_with("ReadGuard<") {
                            Some(&tname[10..tname.len() - 1])
                        } else if tname.starts_with("WriteGuard<") {
                            Some(&tname[11..tname.len() - 1])
                        } else {
                            None
                        };
                        if let Some(inner_cls) = guard_inner {
                            if let Some(cls) = self.class_defs.get(inner_cls) {
                                if let Some(f) =
                                    cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym)
                                {
                                    return Some(f.ty.clone());
                                }
                            }
                        }
                        if let Some(cls) = self.class_defs.get(tname) {
                            if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym)
                            {
                                return Some(f.ty.clone());
                            }
                        }
                        if tname == "TypeInfo" {
                            if mem.prop.sym == "fields" {
                                return Some(Type::Array(Box::new(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "FieldInfo".to_string(),
                                }))));
                            }
                            if mem.prop.sym == "methods" {
                                return Some(Type::Array(Box::new(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "MethodInfo".to_string(),
                                }))));
                            }
                            if mem.prop.sym == "name" {
                                return Some(Type::String);
                            }
                            if mem.prop.sym == "isRef" {
                                return Some(Type::Bool);
                            }
                            if mem.prop.sym == "meta" {
                                return Some(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Map".to_string(),
                                }));
                            }
                        }
                        if tname == "FieldInfo" {
                            if mem.prop.sym == "name" || mem.prop.sym == "typeName" {
                                return Some(Type::String);
                            }
                            if mem.prop.sym == "meta" {
                                return Some(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Map".to_string(),
                                }));
                            }
                        }
                        if tname == "MethodInfo" {
                            if mem.prop.sym == "name" || mem.prop.sym == "returnTypeName" {
                                return Some(Type::String);
                            }
                            if mem.prop.sym == "isAsync" || mem.prop.sym == "isStatic" {
                                return Some(Type::Bool);
                            }
                            if mem.prop.sym == "paramNames" {
                                return Some(Type::Array(Box::new(Type::String)));
                            }
                            if mem.prop.sym == "meta" {
                                return Some(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Map".to_string(),
                                }));
                            }
                        }
                    }
                }
                if let Expr::Ident(oid) = &*mem.obj {
                    if let Some(mty) = mock_types.get(&oid.sym) {
                        let tname_opt = match mty {
                            Type::Ref(id) | Type::BoxDyn(id) => Some(id.sym.clone()),
                            Type::BorrowType { inner, .. } => {
                                if let Type::Ref(id) | Type::BoxDyn(id) = &**inner {
                                    Some(id.sym.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };
                        if let Some(tname) = tname_opt {
                            if let Some(cls) = self.class_defs.get(&tname) {
                                if let Some(f) =
                                    cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym)
                                {
                                    return Some(f.ty.clone());
                                }
                            }
                        }
                    }
                    if let Some(mut vty) = self.var_cp_types.get(&oid.sym) {
                        while let Type::BorrowType { inner, .. } = vty {
                            vty = inner;
                        }
                        if let Type::Object(fields) = vty {
                            if let Some(f) = fields.iter().find(|f| f.ident.sym == mem.prop.sym) {
                                return Some(f.ty.clone());
                            }
                        }
                    }
                    if let Some(cls) = self.class_defs.get(&oid.sym) {
                        if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym) {
                            return Some(f.ty.clone());
                        }
                    }
                    if self.enum_defs.contains_key(&oid.sym)
                        || self.error_sets.contains_key(&oid.sym)
                    {
                        return Some(Type::Ref(oid.clone()));
                    }
                    if let Some(tname) = self.resolve_var_type_name(&oid.sym) {
                        if tname == "Error" {
                            if mem.prop.sym == "code" {
                                return Some(Type::I32);
                            }
                            if mem.prop.sym == "msg" {
                                return Some(Type::String);
                            }
                        }
                        if tname == "TypeInfo" {
                            if mem.prop.sym == "fields" {
                                return Some(Type::Array(Box::new(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "FieldInfo".to_string(),
                                }))));
                            }
                            if mem.prop.sym == "methods" {
                                return Some(Type::Array(Box::new(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "MethodInfo".to_string(),
                                }))));
                            }
                            if mem.prop.sym == "name" {
                                return Some(Type::String);
                            }
                            if mem.prop.sym == "isRef" {
                                return Some(Type::Bool);
                            }
                            if mem.prop.sym == "meta" {
                                return Some(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Map".to_string(),
                                }));
                            }
                        }
                        if tname == "FieldInfo" {
                            if mem.prop.sym == "name" || mem.prop.sym == "typeName" {
                                return Some(Type::String);
                            }
                            if mem.prop.sym == "meta" {
                                return Some(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Map".to_string(),
                                }));
                            }
                        }
                        if tname == "MethodInfo" {
                            if mem.prop.sym == "name" || mem.prop.sym == "returnTypeName" {
                                return Some(Type::String);
                            }
                            if mem.prop.sym == "isAsync" || mem.prop.sym == "isStatic" {
                                return Some(Type::Bool);
                            }
                            if mem.prop.sym == "paramNames" {
                                return Some(Type::Array(Box::new(Type::String)));
                            }
                            if mem.prop.sym == "meta" {
                                return Some(Type::Ref(Ident {
                                    span: Span { start: 0, end: 0 },
                                    sym: "Map".to_string(),
                                }));
                            }
                        }
                        if let Some(cls) = self.class_defs.get(&tname) {
                            if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym)
                            {
                                return Some(f.ty.clone());
                            }
                        }
                    }
                }
                None
            }
            Expr::OptMember(mem) => {
                if let Some(obj_ty) = self.infer_cp_type_with_mock(&mem.obj, mock_types) {
                    let mut base_ty = &obj_ty;
                    while let Type::Optional(inner) | Type::BorrowType { inner, .. } = base_ty {
                        base_ty = inner;
                    }
                    if let Type::Ref(cid) = base_ty {
                        if let Some(cls) = self.class_defs.get(&cid.sym) {
                            if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym)
                            {
                                return match &f.ty {
                                    Type::Optional(_) => Some(f.ty.clone()),
                                    other => Some(Type::Optional(Box::new(other.clone()))),
                                };
                            }
                        }
                    }
                }
                None
            }
            Expr::OptCall(call) => {
                if let Expr::Member(mem) | Expr::OptMember(mem) = &*call.callee {
                    if let Some(obj_ty) = self.infer_cp_type_with_mock(&mem.obj, mock_types) {
                        let mut base_ty = &obj_ty;
                        while let Type::Optional(inner) | Type::BorrowType { inner, .. } = base_ty {
                            base_ty = inner;
                        }
                        if let Type::Ref(cid) | Type::BoxDyn(cid) = base_ty {
                            if let Some((_, Some(rt), _)) =
                                self.lookup_method(&cid.sym, &mem.prop.sym)
                            {
                                return match rt {
                                    Type::Optional(_) => Some(rt),
                                    other => Some(Type::Optional(Box::new(other))),
                                };
                            }
                        }
                    }
                }
                None
            }
            Expr::Question(inner) => {
                if let Some(inner_ty) = self.infer_cp_type_with_mock(inner, mock_types) {
                    let fn_bubbles_null = self.try_catch_target.is_none()
                        && match &self.current_fn_ret_ty {
                            Some(Type::ResultType { ok: fn_ok, .. }) => {
                                matches!(&**fn_ok, Type::Optional(_))
                            }
                            Some(Type::Optional(_)) => true,
                            _ => false,
                        };
                    return match inner_ty {
                        Type::ResultType { ok, .. } => match *ok {
                            Type::Optional(opt_in) if fn_bubbles_null => Some(*opt_in),
                            other => Some(other),
                        },
                        Type::Optional(opt_in) => Some(*opt_in),
                        other => Some(other),
                    };
                }
                None
            }
            Expr::QuestionFallback {
                expr: inner,
                fallback,
                is_return_fallback,
            } => {
                if let Some(inner_ty) = self.infer_cp_type_with_mock(inner, mock_types) {
                    return match inner_ty {
                        Type::ResultType { ok, .. } => match *ok {
                            Type::Optional(opt_in) => Some(*opt_in),
                            other => Some(other),
                        },
                        Type::Optional(opt_in) => Some(*opt_in),
                        other => Some(other),
                    };
                }
                if !*is_return_fallback {
                    return self.infer_cp_type_with_mock(fallback, mock_types);
                }
                None
            }
            Expr::NullCoalesce { left, right } => {
                if let Some(l_ty) = self.infer_cp_type_with_mock(left, mock_types) {
                    if let Type::Optional(opt_in) = l_ty {
                        return Some(*opt_in);
                    }
                    return Some(l_ty);
                }
                self.infer_cp_type_with_mock(right, mock_types)
            }
            Expr::Catch { expr: inner, .. } => {
                if let Some(inner_ty) = self.infer_cp_type_with_mock(inner, mock_types) {
                    if let Type::ResultType { ok, .. } = inner_ty {
                        return Some(*ok);
                    }
                    if let Type::ErrorUnion(t) = inner_ty {
                        return Some(*t);
                    }
                    return Some(inner_ty);
                }
                None
            }
            Expr::Bin(bin) => {
                if bin.op == BinaryOp::Pipe {
                    let l_ty = self.infer_cp_type_with_mock(&bin.left, mock_types);
                    let transformed = transform_pipeline_rhs(&bin.right, "__pipe_mock");
                    let mut next_mocks = mock_types.clone();
                    if let Some(ty) = l_ty {
                        next_mocks.insert("__pipe_mock".to_string(), ty);
                    }
                    return self.infer_cp_type_with_mock(&transformed, &next_mocks);
                }
                if bin.op == BinaryOp::Add {
                    let l_ty = self.infer_cp_type_with_mock(&bin.left, mock_types);
                    let r_ty = self.infer_cp_type_with_mock(&bin.right, mock_types);
                    if matches!(l_ty, Some(Type::String)) || matches!(r_ty, Some(Type::String)) {
                        return Some(Type::String);
                    }
                    return l_ty.or(r_ty);
                }
                if matches!(
                    bin.op,
                    BinaryOp::Eq
                        | BinaryOp::NotEq
                        | BinaryOp::Lt
                        | BinaryOp::Gt
                        | BinaryOp::LtEq
                        | BinaryOp::GtEq
                        | BinaryOp::LogicalAnd
                        | BinaryOp::LogicalOr
                ) {
                    return Some(Type::Bool);
                }
                self.infer_cp_type_with_mock(&bin.left, mock_types)
            }
            Expr::Match(m) => {
                let disc_ty = self.infer_cp_type_with_mock(&m.disc, mock_types);
                for arm in &m.arms {
                    let mut arm_mocks = mock_types.clone();
                    match &arm.pat {
                        Pattern::TypeMatch { ident, ty } => {
                            arm_mocks.insert(ident.sym.clone(), ty.clone());
                        }
                        Pattern::Err(ip) => match &**ip {
                            Pattern::TypeMatch { ident, ty } => {
                                arm_mocks.insert(ident.sym.clone(), ty.clone());
                            }
                            Pattern::Binding { ident, subpat } => {
                                if let Pattern::Ident(tid) = &**subpat {
                                    arm_mocks.insert(ident.sym.clone(), Type::Ref(tid.clone()));
                                }
                            }
                            Pattern::Ident(id) => {
                                if let Some(Type::ResultType { err, .. }) = &disc_ty {
                                    arm_mocks
                                        .insert(id.sym.clone(), self.error_type_to_cp_type(err));
                                }
                            }
                            _ => {}
                        },
                        Pattern::Ident(id) => {
                            if let Some(Type::ResultType { ok, .. }) = &disc_ty {
                                arm_mocks.insert(id.sym.clone(), (**ok).clone());
                            } else if let Some(dt) = &disc_ty {
                                arm_mocks.insert(id.sym.clone(), dt.clone());
                            }
                        }
                        _ => {}
                    }
                    if let Stmt::Expr(e) = &*arm.body {
                        if let Some(t) = self.infer_cp_type_with_mock(e, &arm_mocks) {
                            return Some(t);
                        }
                        if let (Pattern::Ident(pid), Expr::Ident(eid)) = (&arm.pat, e) {
                            if pid.sym == eid.sym {
                                if let Some(Type::ResultType { ok, .. }) = &disc_ty {
                                    return match &**ok {
                                        Type::Optional(inner) => Some((**inner).clone()),
                                        other => Some(other.clone()),
                                    };
                                } else if let Some(Type::Optional(inner)) = &disc_ty {
                                    return Some((**inner).clone());
                                } else if let Some(dt) = &disc_ty {
                                    return Some(dt.clone());
                                }
                            }
                        }
                    } else if let Stmt::Block(b) = &*arm.body {
                        if let Some(Stmt::Expr(e)) = b.body.last() {
                            if let Some(t) = self.infer_cp_type_with_mock(e, mock_types) {
                                return Some(t);
                            }
                        }
                    }
                }
                None
            }
            Expr::Object(fields) => {
                let mut fs: Vec<Field> = Vec::new();
                for f in fields {
                    if f.is_spread {
                        if let Some(Type::Object(sub_fs)) =
                            self.infer_cp_type_with_mock(&f.val, mock_types)
                        {
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
                        let ty = self
                            .infer_cp_type_with_mock(&f.val, mock_types)
                            .unwrap_or(Type::Any);
                        let new_f = Field {
                            span: k.span,
                            ident: k.clone(),
                            ty,
                            meta: Vec::new(),
                            default: None,
                            is_pub: true,
                            is_static: false,
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
                Some(Type::Object(fs))
            }
            Expr::Array(items) => {
                if let Some(first) = items.first() {
                    if let Some(elem_ty) = self.infer_cp_type_with_mock(first, mock_types) {
                        return Some(Type::Array(Box::new(elem_ty)));
                    }
                }
                None
            }
            Expr::ArrayRepeat(val, count) => {
                let val_ty = self
                    .infer_cp_type_with_mock(val, mock_types)
                    .unwrap_or(Type::U8);
                if let Expr::Lit(Lit::Int(n)) = &**count {
                    Some(Type::FixedArray(
                        Box::new(val_ty),
                        ArraySize::Literal(*n as usize),
                    ))
                } else if let Expr::Ident(id) = &**count {
                    Some(Type::FixedArray(
                        Box::new(val_ty),
                        ArraySize::Ident(id.clone()),
                    ))
                } else {
                    Some(Type::Array(Box::new(val_ty)))
                }
            }
            Expr::Slice(s) => {
                let obj_ty = self.infer_cp_type_with_mock(&s.obj, mock_types)?;
                match obj_ty {
                    Type::ManyRawPtr(elem) => Some(Type::Array(elem)),
                    Type::FixedArray(elem, _) => Some(Type::Array(elem)),
                    Type::CStr => Some(Type::String),
                    Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr" => {
                        Some(Type::Array(Box::new(Type::U8)))
                    }
                    other => Some(other),
                }
            }
            Expr::Dict(_) => Some(Type::Ref(Ident {
                span: Span { start: 0, end: 0 },
                sym: "Dict".to_string(),
            })),
            Expr::ComptimeMap {
                target_ty, entries, ..
            } => {
                let val_ty = if let Some(ty) = target_ty {
                    ty.clone()
                } else if let Some(first) = entries.first() {
                    self.infer_cp_type_with_mock(&first.val, mock_types)
                        .unwrap_or(Type::Any)
                } else {
                    Type::Any
                };
                Some(Type::Map(Box::new(Type::String), Box::new(val_ty)))
            }
            Expr::InlineAsm { .. } => Some(Type::Any),
            Expr::Index { obj, .. } => {
                let mut obj_ty = self.infer_cp_type_with_mock(obj, mock_types);
                while let Some(Type::BorrowType { inner, .. }) = obj_ty {
                    obj_ty = Some(*inner);
                }
                match obj_ty {
                    Some(Type::Dict(_, v)) | Some(Type::Map(_, v)) => Some(Type::Optional(v)),
                    Some(Type::Array(inner)) => Some(*inner),
                    Some(Type::FixedArray(inner, _)) => Some(*inner),
                    Some(Type::String) => Some(Type::U8),
                    Some(Type::CStr) => Some(Type::U8),
                    Some(Type::ManyRawPtr(inner)) | Some(Type::CPtr(inner)) => Some(*inner),
                    Some(Type::Ref(id)) if id.sym == "cstring" || id.sym == "cstr" => {
                        Some(Type::U8)
                    }
                    Some(Type::Ref(id)) if id.sym == "Dict" || id.sym == "Map" => {
                        Some(Type::Optional(Box::new(Type::Ref(Ident {
                            span: Span { start: 0, end: 0 },
                            sym: "Message".to_string(),
                        }))))
                    }
                    _ => None,
                }
            }
            Expr::ReflectType { .. } | Expr::ReflectOf { .. } => Some(Type::Ref(Ident {
                span: Span { start: 0, end: 0 },
                sym: "TypeInfo".to_string(),
            })),
            Expr::Range { .. } => Some(Type::I32),
            Expr::Comptime(b) | Expr::Unsafe(b) | Expr::Arena(b) => {
                let mut local_mocks = mock_types.clone();
                for s in &b.body {
                    if let Stmt::Var(v) = s {
                        if let Pattern::Ident(id) = &v.pat {
                            let init_ty = v
                                .init
                                .as_ref()
                                .and_then(|init| self.infer_cp_type_with_mock(init, &local_mocks));
                            let vty = v.ty.clone().or(init_ty);
                            if let Some(t) = vty {
                                local_mocks.insert(id.sym.clone(), t);
                            }
                        }
                    }
                }
                if let Some(Stmt::Expr(e)) = b.body.last() {
                    return self.infer_cp_type_with_mock(e, &local_mocks);
                } else if let Some(Stmt::Return(r)) = b.body.last() {
                    if let Some(arg) = &r.arg {
                        return self.infer_cp_type_with_mock(arg, &local_mocks);
                    }
                }
                Some(Type::Void)
            }
            Expr::Unwrap(inner) => {
                let inner_ty = self.infer_cp_type_with_mock(inner, mock_types);
                match inner_ty {
                    Some(Type::Optional(unwrapped)) => Some(*unwrapped),
                    other => other,
                }
            }
            Expr::If { cons, alt, .. } => {
                if let Some(t) = self.infer_cp_type_with_mock(cons, mock_types) {
                    return Some(t);
                }
                if let Some(a) = alt {
                    return self.infer_cp_type_with_mock(a, mock_types);
                }
                None
            }
            _ => None,
        }
    }

    pub(crate) fn infer_zig_type(&self, ty: Option<&Type>, init: Option<&Expr>) -> String {
        if let Some(t) = ty {
            return self.map_type(t);
        }
        if let Some(expr) = init {
            if let Expr::New(call) = expr {
                if let Expr::Ident(id) = &*call.callee {
                    if id.sym == "Error" {
                        return "Error".to_string();
                    }
                    return format!("*{}", self.map_type_name(&id.sym));
                }
            }
            if let Expr::Slice(s) = expr {
                let obj_t = self.infer_zig_type(None, Some(&s.obj));
                let elem_t = if let Some(Type::Array(elem)) = self.infer_cp_type(expr) {
                    self.map_type(&elem)
                } else {
                    "u8".to_string()
                };
                if obj_t.contains("const") || obj_t.contains("[:0]const") {
                    return format!("[]const {}", elem_t);
                } else {
                    return format!("[]{}", elem_t);
                }
            }
            if let Some(cp_ty) = self.infer_cp_type(expr) {
                return self.map_type(&cp_ty);
            }
            match expr {
                Expr::Lit(crate::ast::Lit::Int(_)) => return "i32".to_string(),
                Expr::Lit(crate::ast::Lit::Float(_)) => return "f32".to_string(),
                Expr::Lit(crate::ast::Lit::Str(_)) => return "[]const u8".to_string(),
                Expr::Lit(crate::ast::Lit::Bool(_)) => return "bool".to_string(),
                Expr::Template(_) => return "[]const u8".to_string(),
                Expr::New(call) => {
                    if let Expr::Ident(id) = &*call.callee {
                        if id.sym == "Error" {
                            return "Error".to_string();
                        }
                        return format!("*{}", self.map_type_name(&id.sym));
                    }
                }
                Expr::Call(call) => {
                    if let Expr::Ident(id) = &*call.callee {
                        if (id.sym == "string" || id.sym == "String") && call.args.len() == 1 {
                            return "[]const u8".to_string();
                        }
                        match id.sym.as_str() {
                            "i8" => return "i8".to_string(),
                            "u8" => return "u8".to_string(),
                            "i16" => return "i16".to_string(),
                            "u16" => return "u16".to_string(),
                            "i32" => return "i32".to_string(),
                            "u32" => return "u32".to_string(),
                            "i64" => return "i64".to_string(),
                            "u64" => return "u64".to_string(),
                            "i128" => return "i128".to_string(),
                            "u128" => return "u128".to_string(),
                            "isize" => return "isize".to_string(),
                            "usize" => return "usize".to_string(),
                            "f32" => return "f32".to_string(),
                            "f64" => return "f64".to_string(),
                            "bool" => return "bool".to_string(),
                            _ => {}
                        }
                        if id.sym == "next" {
                            if let Some(ret_ty) = &self.current_fn_ret_ty {
                                return self.map_type(ret_ty);
                            } else {
                                return "void".to_string();
                            }
                        }
                        if let Some(sig) = self.fn_sigs.get(&id.sym) {
                            if let Some(ret_ty) = &sig.return_type {
                                return self.map_type(ret_ty);
                            }
                        }
                        if let Some(c_name) = &self.current_class_name {
                            if let Some((_, Some(ret_ty), _)) = self.lookup_method(c_name, &id.sym)
                            {
                                return self.map_type(&ret_ty);
                            }
                        }
                        if id.sym == "listen" {
                            return "*Server".to_string();
                        }
                        if id.sym == "serve" {
                            return "*Server".to_string();
                        }
                        if id.sym == "connect" {
                            return "*Socket".to_string();
                        }
                        if id.sym == "fetch" {
                            return "*Response".to_string();
                        }
                        if id.sym == "json" {
                            return "*Response".to_string();
                        }
                        if id.sym == "readFileSync" {
                            return "[]const u8".to_string();
                        }
                        if id.sym == "readFile" {
                            return "[]const u8".to_string();
                        }
                        if id.sym == "randomUUID" {
                            return "[]const u8".to_string();
                        }
                        if id.sym == "hash" {
                            return "[]const u8".to_string();
                        }
                        if id.sym == "@alloc" {
                            return "[*]u8".to_string();
                        }
                        if id.sym == "self_pid" || id.sym == "sender" || id.sym == "spawn" {
                            return "PID".to_string();
                        }
                        if id.sym == "receive" {
                            return "Message".to_string();
                        }
                        if id.sym == "send" || id.sym == "reply" {
                            return "bool".to_string();
                        }
                        if id.sym == "sleep" {
                            return "void".to_string();
                        }
                    } else if let Expr::Member(mem) = &*call.callee {
                        if mem.prop.sym == "getField" {
                            return "?FieldInfo".to_string();
                        }
                        if mem.prop.sym == "getMethod" {
                            return "?MethodInfo".to_string();
                        }
                        if let Expr::Ident(obj_id) = &*mem.obj {
                            if obj_id.sym == "super" {
                                if let Some(c_name) = &self.current_class_name {
                                    if let Some((_, Some(ret_ty), _)) = self
                                        .lookup_method(c_name, &format!("__super_{}", mem.prop.sym))
                                    {
                                        return self.map_type(&ret_ty);
                                    }
                                }
                            } else if let Some(tname) = self.resolve_var_type_name(&obj_id.sym) {
                                if let Some((_, Some(ret_ty), _)) =
                                    self.lookup_method(&tname, &mem.prop.sym)
                                {
                                    return self.map_type(&ret_ty);
                                }
                            } else if self.class_defs.contains_key(&obj_id.sym) {
                                if let Some((_, Some(ret_ty), _)) =
                                    self.lookup_method(&obj_id.sym, &mem.prop.sym)
                                {
                                    return self.map_type(&ret_ty);
                                }
                            }
                        }
                        let prop = mem.prop.sym.as_str();
                        if (prop == "indexOf" || prop == "lastIndexOf" || prop == "charCodeAt")
                            && call.args.len() == 1
                        {
                            return "i32".to_string();
                        } else if prop == "includes"
                            || prop == "startsWith"
                            || prop == "endsWith"
                            || prop == "test"
                        {
                            return "bool".to_string();
                        } else if prop == "slice"
                            || prop == "substring"
                            || prop == "toLowerCase"
                            || prop == "toUpperCase"
                            || prop == "replace"
                            || prop == "match"
                            || prop == "exec"
                            || prop == "toString"
                        {
                            return "[]const u8".to_string();
                        }
                        if mem.prop.sym == "accept" {
                            return "*Socket".to_string();
                        }
                        if mem.prop.sym == "get"
                            || mem.prop.sym == "peek"
                            || mem.prop.sym == "peek_front"
                            || mem.prop.sym == "peek_back"
                            || mem.prop.sym == "pop"
                            || mem.prop.sym == "pop_front"
                            || mem.prop.sym == "pop_back"
                            || mem.prop.sym == "min_key"
                            || mem.prop.sym == "max_key"
                            || mem.prop.sym == "min"
                            || mem.prop.sym == "max"
                            || mem.prop.sym == "insert"
                            || mem.prop.sym == "remove"
                        {
                            return "?Message".to_string();
                        }
                        if mem.prop.sym == "has"
                            || mem.prop.sym == "delete"
                            || mem.prop.sym == "is_empty"
                        {
                            return "bool".to_string();
                        }
                        if mem.prop.sym == "size" {
                            return "i32".to_string();
                        }
                        if mem.prop.sym == "len" {
                            return "usize".to_string();
                        }
                        if mem.prop.sym == "keys" || mem.prop.sym == "values" {
                            return "[]Message".to_string();
                        }
                        if mem.prop.sym == "entries" {
                            return "[]MessageEntry".to_string();
                        }
                        if mem.prop.sym == "read" {
                            return "[]const u8".to_string();
                        }
                    }
                    if let Expr::Member(mem) = &*call.callee {
                        if let Expr::Ident(id) = &*mem.obj {
                            if id.sym == "Math" {
                                return "f32".to_string();
                            }
                        }
                    }
                }

                Expr::Builtin(call) => {
                    if call.name == "@self"
                        || call.name == "@sender"
                        || call.name == "@worker_spawn"
                        || call.name == "@spawn"
                    {
                        return "PID".to_string();
                    }
                    if call.name == "@alloc" {
                        return "[*]u8".to_string();
                    }
                    if call.name == "@receive" {
                        return "Message".to_string();
                    }
                    if call.name == "@send" || call.name == "@reply" || call.name == "@not" {
                        return "bool".to_string();
                    }
                    if call.name == "@http_serve" {
                        return "*Server".to_string();
                    }
                    if call.name == "@json_stringify"
                        || call.name == "@crypto_uuid"
                        || call.name == "@crypto_hash"
                    {
                        return "[]const u8".to_string();
                    }
                    if call.name == "@json_parse" {
                        if call.args.len() > 1 {
                            if let Expr::Ident(id) = &call.args[1] {
                                return format!("*{}", id.sym);
                            }
                        }
                        return "std.json.Value".to_string();
                    }
                }
                Expr::Await(inner) => {
                    let inner_type = self.infer_zig_type(None, Some(inner));
                    if inner_type == "Message" {
                        return "Message".to_string();
                    }
                    if inner_type == "Unknown" || inner_type.is_empty() {
                        return "?Message".to_string();
                    }
                    return inner_type;
                }
                Expr::Move(inner) => {
                    return self.infer_zig_type(None, Some(inner));
                }
                Expr::Borrow(inner, _) => {
                    if let Some(cp_ty) = self.infer_cp_type(expr) {
                        return self.map_type(&cp_ty);
                    }
                    let inner_z = self.infer_zig_type(None, Some(inner));
                    if inner_z.starts_with('*') {
                        return inner_z;
                    }
                    return format!("*{}", inner_z);
                }
                Expr::Ident(id) => {
                    if id.sym == "true" || id.sym == "false" {
                        return "bool".to_string();
                    }
                    if id.sym == "@self" || id.sym == "@sender" {
                        return "PID".to_string();
                    }
                    if id.sym == "this" || id.sym == "_" {
                        if let Some(t) = self.locals.get("self") {
                            return t.clone();
                        }
                    }
                    if let Some(t) = self.locals.get(&id.sym) {
                        if self.narrowed_null_vars.contains(&id.sym) && t.starts_with('?') {
                            return t[1..].to_string();
                        }
                        return t.clone();
                    }
                    if let Some(c_name) = &self.current_class_name {
                        if let Some(cls) = self.class_defs.get(c_name) {
                            if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == id.sym) {
                                return self.map_type(&f.ty);
                            }
                        }
                    }
                }
                Expr::Member(mem) => {
                    if let Some(cp_ty) = self.infer_cp_type(expr) {
                        return self.map_type(&cp_ty);
                    }
                    if let Expr::Ident(id) = &*mem.obj {
                        if id.sym == "Math" {
                            return "f32".to_string();
                        }
                        if let Some(tname) = self.resolve_var_type_name(&id.sym) {
                            if let Some(cls) = self.class_defs.get(&tname) {
                                if let Some(f) =
                                    cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym)
                                {
                                    return self.map_type(&f.ty);
                                }
                            }
                        }
                    }
                    if mem.prop.sym == "node_id" || mem.prop.sym == "local_id" {
                        return "u32".to_string();
                    }
                    if mem.prop.sym == "string" {
                        return "[]const u8".to_string();
                    }
                    if mem.prop.sym == "int" {
                        return "i32".to_string();
                    }
                }
                Expr::ReflectType { .. } | Expr::ReflectOf { .. } => {
                    return "*const TypeInfo".to_string();
                }
                Expr::Index { .. } => {
                    if let Some(cp_ty) = self.infer_cp_type(expr) {
                        return self.map_type(&cp_ty);
                    }
                }
                Expr::Comptime(_) | Expr::Unsafe(_) | Expr::Arena(_) => {
                    if let Some(cp_ty) = self.infer_cp_type(expr) {
                        return self.map_type(&cp_ty);
                    }
                }
                _ => {}
            }
        }

        "*anyopaque".to_string() // Fallback
    }

    pub(crate) fn map_type_name(&self, name: &str) -> String {
        match name {
            "i8" => "i8".to_string(),
            "u8" => "u8".to_string(),
            "i16" => "i16".to_string(),
            "u16" => "u16".to_string(),
            "i32" => "i32".to_string(),
            "u32" => "u32".to_string(),
            "i64" => "i64".to_string(),
            "u64" => "u64".to_string(),
            "isize" => "isize".to_string(),
            "usize" => "usize".to_string(),
            "f32" => "f32".to_string(),
            "f64" => "f64".to_string(),
            "bool" => "bool".to_string(),
            "string" => "[]const u8".to_string(),
            _ => {
                if name.starts_with("Mutex<") {
                    let inner = &name[6..name.len() - 1];
                    format!("CpMutex({})", self.map_type_name(inner))
                } else if name.starts_with("RwLock<") {
                    let inner = &name[7..name.len() - 1];
                    format!("CpRwLock({})", self.map_type_name(inner))
                } else if name.starts_with("Atomic<") {
                    let inner = &name[7..name.len() - 1];
                    format!("CpAtomic({})", self.map_type_name(inner))
                } else if let Some(pos) = name.find('<') {
                    if name.ends_with('>') {
                        let base = &name[..pos];
                        let args = &name[pos + 1..name.len() - 1];
                        let mapped_args = Self::map_generic_args_str(args);
                        format!("{}({})", base, mapped_args)
                    } else {
                        name.to_string()
                    }
                } else {
                    name.to_string()
                }
            }
        }
    }

    pub(crate) fn map_type(&self, ty: &Type) -> String {
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
            Type::String => "[]const u8".to_string(),
            Type::PID => "PID".to_string(),
            Type::Void => "void".to_string(),
            Type::Any => "*anyopaque".to_string(),
            Type::Auto => "anytype".to_string(),
            Type::Array(inner) => format!("[]const {}", self.map_type(inner)),
            Type::FixedArray(inner, size) => format!("[{}]{}", size, self.map_type(inner)),
            Type::Map(_, _) => "*Map".to_string(),
            Type::Dict(_, _) => "*Dict".to_string(),
            Type::Set(_) => "*Set".to_string(),
            Type::Ref(ident) => {
                if ident.sym == "TypeInfo" {
                    return "*const TypeInfo".to_string();
                }
                if ident.sym.starts_with("Mutex<") {
                    let inner = &ident.sym[6..ident.sym.len() - 1];
                    return format!("CpMutex({})", self.map_type_name(inner));
                }
                if ident.sym == "Mutex"
                    && !self.type_aliases.contains_key("Mutex")
                    && !self.class_defs.contains_key("Mutex")
                {
                    return "CpBareMutex".to_string();
                }
                if ident.sym.starts_with("RwLock<") {
                    let inner = &ident.sym[7..ident.sym.len() - 1];
                    return format!("CpRwLock({})", self.map_type_name(inner));
                }
                if ident.sym == "RwLock"
                    && !self.type_aliases.contains_key("RwLock")
                    && !self.class_defs.contains_key("RwLock")
                {
                    return "CpBareRwLock".to_string();
                }
                if ident.sym.starts_with("Atomic<") {
                    let inner = &ident.sym[7..ident.sym.len() - 1];
                    return format!("CpAtomic({})", self.map_type_name(inner));
                }
                if ident.sym == "Atomic" {
                    return "CpAtomic(i32)".to_string();
                }
                if ident.sym.starts_with("LockGuard<") {
                    let inner = &ident.sym[10..ident.sym.len() - 1];
                    return format!("LockGuard({})", self.map_type_name(inner));
                }
                if ident.sym.starts_with("ReadGuard<") {
                    let inner = &ident.sym[10..ident.sym.len() - 1];
                    return format!("ReadGuard({})", self.map_type_name(inner));
                }
                if ident.sym.starts_with("WriteGuard<") {
                    let inner = &ident.sym[11..ident.sym.len() - 1];
                    return format!("WriteGuard({})", self.map_type_name(inner));
                }
                if ident.sym == "FieldInfo" || ident.sym == "MethodInfo" {
                    return ident.sym.clone();
                }
                if let Some(pos) = ident.sym.find('<') {
                    if ident.sym.ends_with('>') {
                        let base = &ident.sym[..pos];
                        let args = &ident.sym[pos + 1..ident.sym.len() - 1];
                        let mapped_args = Self::map_generic_args_str(args);
                        return format!("{}({})", base, mapped_args);
                    }
                }
                if ident.sym == "cstring" {
                    return "[:0]const u8".to_string();
                }
                if ident.sym.starts_with('[') {
                    return ident.sym.clone();
                }
                let is_custom_int = (ident.sym.starts_with('u') || ident.sym.starts_with('i'))
                    && ident.sym.len() > 1
                    && ident.sym[1..].chars().all(|c| c.is_ascii_digit());
                if is_custom_int {
                    return ident.sym.clone();
                }
                let base_name = ident.sym.split('(').next().unwrap_or(&ident.sym);
                if ident.sym == "PID"
                    || ident.sym == "Message"
                    || ident.sym == "Error"
                    || ident.sym == "anytype"
                    || ident.sym == "anyopaque"
                    || ident.sym == "type"
                    || ident.sym == "anyerror"
                    || ident.sym == "noreturn"
                    || (!ident.sym.is_empty() && ident.sym.chars().all(|c| c.is_ascii_uppercase()))
                    || ident.sym.starts_with("c_")
                    || ident.sym.starts_with("Many")
                    || ident.sym.contains('.')
                    || ident.sym.contains('(')
                    || ident.sym.starts_with("__AnonErr_")
                    || ident.sym.starts_with("__UnionErr_")
                    || self.ref_classes.contains(&ident.sym)
                    || self.value_classes.contains(&ident.sym)
                    || self.value_classes.contains(base_name)
                    || self.enum_defs.contains_key(&ident.sym)
                    || self.error_sets.contains_key(&ident.sym)
                    || self.trait_defs.contains_key(&ident.sym)
                {
                    return ident.sym.clone();
                }
                if let Some(cls) = self.class_defs.get(base_name) {
                    if !cls.generic_params.is_empty() {
                        return ident.sym.clone();
                    }
                } else if !self.ref_classes.contains(base_name) {
                    return ident.sym.clone();
                }
                if let Some(ta) = self.type_aliases.get(&ident.sym) {
                    let target_base = match &ta.target_ty {
                        Type::Ref(tid) => tid.sym.split('<').next().unwrap_or(&tid.sym),
                        _ => "",
                    };
                    if let Some(cls) = self.class_defs.get(target_base) {
                        if !cls.generic_params.is_empty()
                            || cls.is_ref
                            || self.value_classes.contains(target_base)
                        {
                            return ident.sym.clone();
                        }
                    } else if !self.class_defs.contains_key(target_base) {
                        return ident.sym.clone();
                    }
                }
                format!("*{}", ident.sym)
            }
            Type::BorrowType { inner, is_mut, .. } => {
                if !*is_mut {
                    if matches!(**inner, Type::CStr) {
                        return "[*:0]const u8".to_string();
                    }
                    if let Type::Optional(opt_inner) = &**inner {
                        let opt_zig = self.map_type(opt_inner);
                        if opt_zig.starts_with("*const ") {
                            return format!("?{}", opt_zig);
                        }
                        let clean_opt = if opt_zig.starts_with('*') {
                            &opt_zig[1..]
                        } else {
                            &opt_zig
                        };
                        return format!("?*const {}", clean_opt);
                    }
                    let inner_zig = self.map_type(inner);
                    if inner_zig.starts_with("*const ") {
                        return inner_zig;
                    }
                    let clean_inner = if inner_zig.starts_with('*') {
                        &inner_zig[1..]
                    } else {
                        &inner_zig
                    };
                    return format!("*const {}", clean_inner);
                }
                if *is_mut {
                    if let Type::Array(elem) = &**inner {
                        return format!("[]{}", self.map_type(elem));
                    }
                    if let Type::String = &**inner {
                        return "[]u8".to_string();
                    }
                    if let Type::Ref(id) = &**inner {
                        if id.sym == "cstring" {
                            return "[:0]u8".to_string();
                        }
                    }
                    if matches!(**inner, Type::CStr) {
                        return "[*:0]u8".to_string();
                    }
                    if let Type::Optional(opt_inner) = &**inner {
                        if let Type::Array(elem) = &**opt_inner {
                            return format!("?[]{}", self.map_type(elem));
                        }
                        if let Type::String = &**opt_inner {
                            return "?[]u8".to_string();
                        }
                    }
                }
                if let Type::Optional(opt_inner) = &**inner {
                    let opt_inner_zig = self.map_type(opt_inner);
                    if opt_inner_zig.starts_with('*') {
                        return format!("*?{}", opt_inner_zig);
                    }
                    return format!("?*{}", opt_inner_zig);
                }
                let inner_zig = self.map_type(inner);
                let is_trait = if let Type::Ref(id) = &**inner {
                    self.trait_defs.contains_key(&id.sym)
                } else {
                    false
                };
                if inner_zig.starts_with('*') || is_trait || inner_zig == "anytype" {
                    inner_zig
                } else {
                    format!("*{}", inner_zig)
                }
            }
            Type::Shared(inner) => {
                let inner_zig = self.map_type(inner);
                if inner_zig.starts_with('*') {
                    inner_zig
                } else {
                    format!("*{}", inner_zig)
                }
            }
            Type::Optional(inner) => format!("?{}", self.map_type(inner)),
            Type::ErrorUnion(inner) => {
                if self.is_lib_mode {
                    format!("!{}", self.map_type(inner))
                } else {
                    format!("anyerror!{}", self.map_type(inner))
                }
            }
            Type::ResultType { ok, err } => {
                let err_str = self.map_error_type(err);
                let ok_str = self.map_type(ok);
                if self.is_lib_mode
                    || err_str.contains('.')
                    || err_str == "anyerror"
                    || self.error_sets.contains_key(&err_str)
                    || err_str.starts_with('(')
                {
                    format!("{}!{}", err_str, ok_str)
                } else {
                    format!("CpResult({}, {})", ok_str, err_str)
                }
            }
            Type::BoxDyn(id) => id.sym.clone(),
            Type::BareDyn(id) => {
                panic!(
                    "CP Compile Error: Bare 'dyn {}' cannot be placed on the stack or used without Box; use 'Box<dyn {}>' instead",
                    id.sym, id.sym
                )
            }
            Type::Tuple(types) => {
                let mapped: Vec<String> = types.iter().map(|t| self.map_type(t)).collect();
                format!("struct {{ {} }}", mapped.join(", "))
            }
            Type::Object(fields) => {
                let mapped: Vec<String> = fields
                    .iter()
                    .map(|f| format!("{}: {}", f.ident.sym, self.map_type(&f.ty)))
                    .collect();
                let body = format!("struct {{ {} }}", mapped.join(", "));
                let mut name = String::from("__CpObj");
                for f in fields {
                    let clean_ty: String = self
                        .map_type(&f.ty)
                        .chars()
                        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                        .collect();
                    name.push_str(&format!("_{}_{}", f.ident.sym, clean_ty));
                }
                self.anon_obj_structs
                    .borrow_mut()
                    .insert(name.clone(), body);
                name
            }
            Type::RawPtr { inner, depth } => {
                let base = self.map_raw_ptr_base(inner);
                format!("{}{}", "*".repeat(*depth), base)
            }
            Type::ManyRawPtr(inner) => {
                let base = self.map_raw_ptr_base(inner);
                format!("[*]{}", base)
            }
            Type::CPtr(inner) => {
                let base = self.map_raw_ptr_base(inner);
                format!("[*c]{}", base)
            }
            Type::SentinelPtr(inner, val) => {
                let base = self.map_raw_ptr_base(inner);
                format!("[*:{val}]{}", base)
            }
            Type::SentinelSlice(inner, val) => {
                if let Some(s) = val.strip_suffix(":const") {
                    format!("[:{s}]const {}", self.map_type(inner))
                } else {
                    format!("[:{val}]{}", self.map_type(inner))
                }
            }
            Type::CStr => "[*:0]const u8".to_string(),
            Type::CFn(params, ret) => {
                let mapped: Vec<String> = params.iter().map(|p| self.map_type(p)).collect();
                format!(
                    "*const fn ({}) callconv(.c) {}",
                    mapped.join(", "),
                    self.map_type(ret)
                )
            }
            Type::Fn(params, ret) => {
                let mapped: Vec<String> = params.iter().map(|p| self.map_type(p)).collect();
                format!("*const fn ({}) {}", mapped.join(", "), self.map_type(ret))
            }
            Type::Conditional(cond, cons, alt) => {
                let cond_str = self.expr_to_zig_pure(cond);
                let cons_str = self.map_type(cons);
                let alt_str = self.map_type(alt);
                format!("if ({}) {} else {}", cond_str, cons_str, alt_str)
            }
            _ => "i32".to_string(),
        }
    }

    pub(crate) fn split_generic_args_str(args: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut cur = String::new();
        let mut depth = 0;
        for c in args.chars() {
            match c {
                '[' | '(' | '<' | '{' => {
                    depth += 1;
                    cur.push(c);
                }
                ']' | ')' | '>' | '}' => {
                    depth -= 1;
                    cur.push(c);
                }
                ',' if depth == 0 => {
                    parts.push(cur.trim().to_string());
                    cur.clear();
                }
                _ => cur.push(c),
            }
        }
        if !cur.trim().is_empty() {
            parts.push(cur.trim().to_string());
        }
        parts
    }

    pub(crate) fn map_generic_args_str(args: &str) -> String {
        let parts = Self::split_generic_args_str(args);
        let types = [
            "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "usize", "isize", "f32", "f64",
            "bool", "string",
        ];
        let mapped_parts: Vec<String> = parts
            .into_iter()
            .map(|p| {
                if p.starts_with('[') && p.ends_with(']') {
                    let inner = p[1..p.len() - 1].trim();
                    if types.contains(&inner) {
                        format!("[]{}", inner)
                    } else {
                        format!(".{{ {} }}", inner)
                    }
                } else {
                    p
                }
            })
            .collect();

        mapped_parts.join(", ")
    }

    pub(crate) fn map_raw_ptr_base(&self, ty: &Type) -> String {
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
            Type::String => "[]const u8".to_string(),
            Type::CStr => "[*:0]u8".to_string(),
            Type::Void | Type::Any => "anyopaque".to_string(),
            Type::BorrowType { inner, is_mut, .. } => {
                if !*is_mut {
                    format!("const {}", self.map_raw_ptr_base(inner))
                } else {
                    self.map_raw_ptr_base(inner)
                }
            }
            Type::Ref(id) => {
                if id.sym == "anyopaque" || id.sym == "any" {
                    "anyopaque".to_string()
                } else {
                    id.sym.clone()
                }
            }
            _ => self.map_type(ty),
        }
    }
}

pub(crate) fn type_to_display_string(ty: &Type) -> String {
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
        Type::PID => "PID".to_string(),
        Type::Array(inner) => format!("Array<{}>", type_to_display_string(inner)),
        Type::FixedArray(inner, size) => format!("{}[{}]", type_to_display_string(inner), size),
        Type::Ref(id) => id.sym.clone(),
        Type::Optional(inner) => format!("?{}", type_to_display_string(inner)),
        Type::ResultType { ok, .. } => type_to_display_string(ok),
        Type::ErrorUnion(inner) => format!("{}!", type_to_display_string(inner)),
        Type::BorrowType { inner, .. } => type_to_display_string(inner),
        Type::Shared(inner) => type_to_display_string(inner),
        Type::RawPtr { inner, depth } => {
            format!(
                "{}{}{}",
                "*".repeat(*depth),
                type_to_display_string(inner),
                "!"
            )
        }
        Type::ManyRawPtr(inner) => format!("[*]{}!", type_to_display_string(inner)),
        Type::CStr => "cstr".to_string(),
        Type::CFn(params, ret) => {
            let p_strs: Vec<String> = params.iter().map(|p| type_to_display_string(p)).collect();
            format!(
                "@c fn({}) -> {}",
                p_strs.join(", "),
                type_to_display_string(ret)
            )
        }
        Type::Fn(params, ret) => {
            let p_strs: Vec<String> = params.iter().map(|p| type_to_display_string(p)).collect();
            format!(
                "fn({}) -> {}",
                p_strs.join(", "),
                type_to_display_string(ret)
            )
        }
        _ => "any".to_string(),
    }
}
