use super::*;
use crate::ast::*;
use crate::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

impl Codegen {
    pub(crate) fn compute_last_uses_block(&mut self, block: &BlockStmt, counter: &mut usize) {
        for s in &block.body {
            self.compute_last_uses_stmt(s, counter);
        }
    }

    pub(crate) fn compute_last_uses_stmt(&mut self, stmt: &Stmt, counter: &mut usize) {
        *counter += 1;
        let idx = *counter;
        let mut refs = HashSet::new();
        match stmt {
            Stmt::Var(v) => {
                if let Some(init) = &v.init {
                    crate::ast_visitor::find_referenced_vars_in_expr(init, &mut refs);
                }
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
            }
            Stmt::Expr(e) => {
                if let Expr::Bin(bin) = e {
                    if bin.op == BinaryOp::Assign {
                        crate::ast_visitor::find_referenced_vars_in_expr(&bin.right, &mut refs);
                        match &*bin.left {
                            Expr::Member(mem) => {
                                crate::ast_visitor::find_referenced_vars_in_expr(
                                    &mem.obj, &mut refs,
                                );
                            }
                            Expr::Deref(inner) => {
                                crate::ast_visitor::find_referenced_vars_in_expr(inner, &mut refs);
                            }
                            Expr::Index { obj, index } => {
                                crate::ast_visitor::find_referenced_vars_in_expr(obj, &mut refs);
                                crate::ast_visitor::find_referenced_vars_in_expr(index, &mut refs);
                            }
                            _ => {}
                        }
                        for r in refs {
                            self.last_use_map.insert(r, idx);
                        }
                        return;
                    }
                }
                crate::ast_visitor::find_referenced_vars_in_expr(e, &mut refs);
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
            }
            Stmt::Return(ret) => {
                if let Some(arg) = &ret.arg {
                    crate::ast_visitor::find_referenced_vars_in_expr(arg, &mut refs);
                }
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
            }
            Stmt::Break | Stmt::Continue => {}
            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                self.compute_last_uses_block(b, counter)
            }
            Stmt::If(i) => {
                crate::ast_visitor::find_referenced_vars_in_expr(&i.test, &mut refs);
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
                if let Stmt::Block(b) = &*i.cons {
                    self.compute_last_uses_block(b, counter);
                } else {
                    self.compute_last_uses_stmt(&i.cons, counter);
                }
                if let Some(alt) = &i.alt {
                    if let Stmt::Block(b) = &**alt {
                        self.compute_last_uses_block(b, counter);
                    } else {
                        self.compute_last_uses_stmt(alt, counter);
                    }
                }
            }
            Stmt::While(w) => {
                crate::ast_visitor::find_referenced_vars_in_expr(&w.test, &mut refs);
                if let Some(c) = &w.continue_expr {
                    crate::ast_visitor::find_referenced_vars_in_expr(c, &mut refs);
                }
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
                if let Stmt::Block(b) = &*w.body {
                    self.compute_last_uses_block(b, counter);
                } else {
                    self.compute_last_uses_stmt(&w.body, counter);
                }
            }
            Stmt::For(f) => {
                crate::ast_visitor::find_referenced_vars_in_expr(&f.expr, &mut refs);
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
                if let Stmt::Block(b) = &*f.body {
                    self.compute_last_uses_block(b, counter);
                } else {
                    self.compute_last_uses_stmt(&f.body, counter);
                }
            }
            Stmt::Match(m) => {
                crate::ast_visitor::find_referenced_vars_in_expr(&m.disc, &mut refs);
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
                for arm in &m.arms {
                    if let Some(g) = &arm.guard {
                        let mut grefs = HashSet::new();
                        crate::ast_visitor::find_referenced_vars_in_expr(g, &mut grefs);
                        for r in grefs {
                            self.last_use_map.insert(r, idx);
                        }
                    }
                    if let Stmt::Block(b) = &*arm.body {
                        self.compute_last_uses_block(b, counter);
                    } else {
                        self.compute_last_uses_stmt(&arm.body, counter);
                    }
                }
            }
            Stmt::TryCatch {
                try_block,
                catch_body,
                ..
            } => {
                self.compute_last_uses_block(try_block, counter);
                if let Some(cb) = catch_body {
                    if let Stmt::Block(b) = &**cb {
                        self.compute_last_uses_block(b, counter);
                    } else {
                        self.compute_last_uses_stmt(cb, counter);
                    }
                }
            }
            Stmt::Defer(d) => self.compute_last_uses_stmt(d, counter),
            Stmt::ErrDefer(d) => self.compute_last_uses_stmt(d, counter),
            _ => {}
        }
    }

    pub(crate) fn check_active_borrows_before_move(&self, var_name: &str) {
        for (borrower, (sources, _)) in &self.var_borrow_sources {
            if sources.iter().any(|(s, _)| {
                s == var_name
                    || ((s == "this" || s == "_") && (var_name == "this" || var_name == "_"))
            }) {
                let last_use = self.last_use_map.get(borrower).copied().unwrap_or(0);
                if last_use > self.current_stmt_idx {
                    panic!(
                        "CP Compile Error: Cannot move '{}' because it is currently borrowed by '{}' (active until statement #{})",
                        var_name, borrower, last_use
                    );
                }
            }
        }
    }

    pub(crate) fn check_borrow_conflict(&self, target_var: &str, req_mut: bool) {
        self.check_borrow_conflict_field_excluding(target_var, None, "", req_mut);
    }

    pub(crate) fn check_borrow_conflict_field(
        &self,
        target_var: &str,
        target_field: Option<&str>,
        req_mut: bool,
    ) {
        self.check_borrow_conflict_field_excluding(target_var, target_field, "", req_mut);
    }

    #[allow(dead_code)]
    pub(crate) fn check_borrow_conflict_excluding(
        &self,
        target_var: &str,
        exclude_borrower: &str,
        req_mut: bool,
    ) {
        self.check_borrow_conflict_field_excluding(target_var, None, exclude_borrower, req_mut);
    }

    pub(crate) fn check_borrow_conflict_field_excluding(
        &self,
        target_var: &str,
        target_field: Option<&str>,
        exclude_borrower: &str,
        req_mut: bool,
    ) {
        for (borrower, (sources, is_mut_borrow)) in &self.var_borrow_sources {
            if borrower == exclude_borrower {
                continue;
            }
            let last_use = self.last_use_map.get(borrower).copied().unwrap_or(0);
            if last_use >= self.current_stmt_idx {
                for (src_var, src_field) in sources {
                    if Self::targets_conflict(
                        target_var,
                        target_field,
                        req_mut,
                        src_var,
                        src_field.as_deref(),
                        *is_mut_borrow,
                    ) {
                        let target_desc = if let Some(f) = target_field {
                            format!("{}.{}", target_var, f)
                        } else {
                            target_var.to_string()
                        };
                        panic!(
                            "CP Compile Error: Borrow conflict on '{}': cannot borrow as {} while '{}' is still active",
                            target_desc,
                            if req_mut { "mutable" } else { "immutable" },
                            borrower
                        );
                    }
                }
            }
        }
    }

    pub(crate) fn targets_conflict(
        target_a: &str,
        field_a: Option<&str>,
        is_mut_a: bool,
        target_b: &str,
        field_b: Option<&str>,
        is_mut_b: bool,
    ) -> bool {
        let is_same_root = target_a == target_b
            || ((target_a == "this" || target_a == "_") && (target_b == "this" || target_b == "_"));
        if !is_same_root {
            return false;
        }
        if !is_mut_a && !is_mut_b {
            return false;
        }
        match (field_a, field_b) {
            (None, _) | (_, None) => true,
            (Some(fa), Some(fb)) => {
                if fa == fb {
                    return true;
                }
                if fa.starts_with(fb) && fa.as_bytes().get(fb.len()) == Some(&b'.') {
                    return true;
                }
                if fb.starts_with(fa) && fb.as_bytes().get(fa.len()) == Some(&b'.') {
                    return true;
                }
                false
            }
        }
    }

    pub(crate) fn extract_member_path(&self, expr: &Expr) -> (Option<String>, Option<String>) {
        match expr {
            Expr::Ident(id) => (Some(id.sym.clone()), None),
            Expr::Member(mem) => {
                let mut parts = vec![mem.prop.sym.clone()];
                let mut curr = &*mem.obj;
                loop {
                    match curr {
                        Expr::Member(inner_mem) => {
                            parts.push(inner_mem.prop.sym.clone());
                            curr = &*inner_mem.obj;
                        }
                        Expr::Ident(id) => {
                            parts.reverse();
                            return (Some(id.sym.clone()), Some(parts.join(".")));
                        }
                        Expr::Deref(inner) | Expr::Unwrap(inner) => {
                            curr = inner;
                        }
                        _ => return (None, None),
                    }
                }
            }
            Expr::Deref(inner) | Expr::Unwrap(inner) => self.extract_member_path(inner),
            _ => (None, None),
        }
    }

    pub(crate) fn resolve_borrow_source_field(
        &self,
        root: &str,
        field: Option<&str>,
    ) -> Vec<(String, Option<String>)> {
        if root == "this" || root == "_" {
            return vec![
                ("this".to_string(), field.map(|f| f.to_string())),
                ("_".to_string(), field.map(|f| f.to_string())),
            ];
        }
        if let Some((orig_sources, _)) = self.var_borrow_sources.get(root) {
            if !orig_sources.is_empty() {
                let mut resolved = Vec::new();
                for (orig_root, orig_field) in orig_sources {
                    let combined_field = match (orig_field, field) {
                        (Some(of), Some(f)) => Some(format!("{}.{}", of, f)),
                        (Some(of), None) => Some(of.clone()),
                        (None, Some(f)) => Some(f.to_string()),
                        (None, None) => None,
                    };
                    resolved.push((orig_root.clone(), combined_field));
                }
                return resolved;
            }
        }
        vec![(root.to_string(), field.map(|f| f.to_string()))]
    }

    pub(crate) fn is_zig_heap_ptr_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Map(_, _) | Type::Dict(_, _) | Type::Set(_) => true,
            Type::BorrowType { .. } | Type::RawPtr { .. } => true,
            Type::Ref(cid) => {
                if cid.sym == "Map"
                    || cid.sym == "Dict"
                    || cid.sym == "Set"
                    || cid.sym == "RegExp"
                    || cid.sym == "Task"
                    || cid.sym == "Server"
                    || cid.sym == "Socket"
                    || cid.sym == "Request"
                    || cid.sym == "Response"
                {
                    return true;
                }
                self.class_defs.get(&cid.sym).map_or(false, |c| !c.is_ref)
            }
            _ => false,
        }
    }

    pub(crate) fn check_mutation_allowed(&self, target_expr: &Expr, op_desc: &str) {
        match target_expr {
            Expr::Ident(id) => {
                if id.sym == "this" || id.sym == "_" || self.is_class_field(&id.sym) {
                    if self.moved_vars.contains("this") || self.moved_vars.contains("_") {
                        panic!("CP Compile Error: Use of moved value '{}'", id.sym);
                    }
                    if self.current_zig_fn_name.ends_with("_init") {
                        return;
                    }
                    if self.is_class_field(&id.sym) {
                        if let Some(c_name) = &self.current_class_name {
                            if let Some(cls) = self.class_defs.get(c_name) {
                                if let Some(f) = cls.fields.iter().find(|f| f.ident.sym == id.sym) {
                                    if matches!(
                                        f.ty,
                                        Type::BorrowType { is_mut: true, .. } | Type::RawPtr { .. }
                                    ) {
                                        return;
                                    }
                                    if matches!(f.ty, Type::BorrowType { is_mut: false, .. })
                                        && !op_desc.starts_with("field assignment")
                                    {
                                        panic!(
                                            "CP Compile Error: Cannot perform mutable operation ({}) through read-only borrow field '{}'",
                                            op_desc, id.sym
                                        );
                                    }
                                }
                            }
                        }
                    }
                    if self.current_receiver_mode == ReceiverMode::ReadOnly {
                        panic!(
                            "CP Compile Error: Cannot perform mutable operation ({}) on '{}' in a read-only method (add '*_' or 'mut _' as first parameter)",
                            op_desc, id.sym
                        );
                    }
                    self.check_borrow_conflict("this", true);
                    self.check_borrow_conflict("_", true);
                    return;
                }
                if self.const_vars.contains(&id.sym) && !self.type_namespace_vars.contains(&id.sym)
                {
                    let allow_pointee_mut = op_desc != "assignment '='"
                        && self.var_cp_types.get(&id.sym).map_or(false, |t| {
                            matches!(
                                t,
                                Type::RawPtr { .. }
                                    | Type::ManyRawPtr(_)
                                    | Type::CPtr(_)
                                    | Type::SentinelPtr(_, _)
                                    | Type::BorrowType { is_mut: true, .. }
                                    | Type::Auto
                                    | Type::Array(_)
                            )
                        });
                    if !allow_pointee_mut {
                        panic!(
                            "CP Compile Error: Cannot perform mutable operation ({}) on 'const' variable '{}' (declare with 'let' to allow mutation)",
                            op_desc, id.sym
                        );
                    }
                }
                if self.moved_vars.contains(&id.sym) {
                    panic!("CP Compile Error: Use of moved value '{}'", id.sym);
                }
                if self.loop_borrowed_containers.contains(&id.sym) {
                    panic!(
                        "CP Compile Error: Cannot perform mutable operation ({}) on container '{}' while it is borrowed in a 'for' loop",
                        op_desc, id.sym
                    );
                }
                if let Some((sources, is_mut)) = self.var_borrow_sources.get(&id.sym) {
                    if !*is_mut {
                        panic!(
                            "CP Compile Error: Cannot perform mutable operation ({}) through read-only borrow '{}' (use '*...' for mutable borrow)",
                            op_desc, id.sym
                        );
                    }
                    self.check_borrow_conflict(&id.sym, true);
                    for (src, src_f) in sources {
                        if self.loop_borrowed_containers.contains(src) {
                            panic!(
                                "CP Compile Error: Cannot perform mutable operation ({}) on '{}' while '{}' is borrowed in a 'for' loop",
                                op_desc, id.sym, src
                            );
                        }
                        self.check_borrow_conflict_field_excluding(
                            src,
                            src_f.as_deref(),
                            &id.sym,
                            true,
                        );
                    }
                } else {
                    if self.current_fn_params.contains(&id.sym)
                        && !self.mutable_fn_params.contains(&id.sym)
                    {
                        if op_desc == "index assignment '[] ='" {
                            if self.is_lib_mode {
                                return;
                            }
                            if let Some(param_ty) = self.var_cp_types.get(&id.sym) {
                                if matches!(param_ty, Type::Array(_) | Type::FixedArray(_, _)) {
                                    return;
                                }
                            }
                        }
                        let is_ptr_or_mut_borrow =
                            if let Some(param_ty) = self.var_cp_types.get(&id.sym) {
                                matches!(
                                    param_ty,
                                    Type::RawPtr { .. }
                                        | Type::ManyRawPtr(_)
                                        | Type::CPtr(_)
                                        | Type::SentinelPtr(_, _)
                                        | Type::BorrowType { is_mut: true, .. }
                                        | Type::Auto
                                )
                            } else {
                                false
                            };
                        if !is_ptr_or_mut_borrow {
                            panic!(
                                "CP Compile Error: Cannot perform mutable operation ({}) on read-only parameter '{}' (mark parameter with '*{}' or 'mut {}')",
                                op_desc, id.sym, id.sym, id.sym
                            );
                        }
                    }
                    self.check_borrow_conflict(&id.sym, true);
                }
            }
            Expr::Member(mem) => {
                if let Some(m_ty) = self.infer_cp_type(target_expr) {
                    if matches!(
                        m_ty,
                        Type::BorrowType { is_mut: true, .. } | Type::RawPtr { .. }
                    ) {
                        return;
                    }
                    if matches!(m_ty, Type::BorrowType { is_mut: false, .. })
                        && !op_desc.starts_with("field assignment")
                    {
                        panic!(
                            "CP Compile Error: Cannot perform mutable operation ({}) through read-only borrow field '{}'",
                            op_desc, mem.prop.sym
                        );
                    }
                }
                let (root_opt, field_opt) = self.extract_member_path(target_expr);
                if let Some(root) = root_opt {
                    if self.loop_borrowed_containers.contains(&root) {
                        panic!(
                            "CP Compile Error: Cannot perform mutable operation ({}) on container '{}' while it is borrowed in a 'for' loop",
                            op_desc, root
                        );
                    }
                    if let Some((sources, is_mut)) = self.var_borrow_sources.get(&root) {
                        if !*is_mut {
                            panic!(
                                "CP Compile Error: Cannot perform mutable operation ({}) through read-only borrow '{}'",
                                op_desc, root
                            );
                        }
                        self.check_borrow_conflict_field(&root, field_opt.as_deref(), true);
                        for (src, src_f) in sources {
                            let combined_field = match (src_f, &field_opt) {
                                (Some(sf), Some(f)) => Some(format!("{}.{}", sf, f)),
                                (Some(sf), None) => Some(sf.clone()),
                                (None, Some(f)) => Some(f.clone()),
                                (None, None) => None,
                            };
                            self.check_borrow_conflict_field_excluding(
                                src,
                                combined_field.as_deref(),
                                &root,
                                true,
                            );
                        }
                    } else {
                        if self.current_fn_params.contains(&root)
                            && !self.mutable_fn_params.contains(&root)
                        {
                            if op_desc == "index assignment '[] ='" {
                                if self.is_lib_mode {
                                    return;
                                }
                                if let Some(param_ty) = self.var_cp_types.get(&root) {
                                    if matches!(param_ty, Type::Array(_) | Type::FixedArray(_, _)) {
                                        return;
                                    }
                                }
                            }
                            let is_ptr_or_mut_borrow =
                                if let Some(param_ty) = self.var_cp_types.get(&root) {
                                    matches!(
                                        param_ty,
                                        Type::RawPtr { .. } | Type::BorrowType { is_mut: true, .. }
                                    )
                                } else {
                                    false
                                };
                            if !is_ptr_or_mut_borrow {
                                panic!(
                                    "CP Compile Error: Cannot perform mutable operation ({}) on read-only parameter '{}' (mark parameter with '*{}' or 'mut {}')",
                                    op_desc, root, root, root
                                );
                            }
                        }
                        self.check_borrow_conflict_field(&root, field_opt.as_deref(), true);
                    }
                } else {
                    self.check_mutation_allowed(&mem.obj, op_desc);
                }
            }
            Expr::Deref(inner) | Expr::Unwrap(inner) => {
                self.check_mutation_allowed(inner, op_desc);
            }
            Expr::Index { obj, .. } => {
                self.check_mutation_allowed(obj, op_desc);
            }
            _ => {}
        }
    }

    pub(crate) fn check_read_allowed(&self, target_expr: &Expr) {
        if let Expr::Ident(id) = target_expr {
            if self.moved_vars.contains(&id.sym) {
                panic!("CP Compile Error: Use of moved value '{}'", id.sym);
            }
            if !self.var_borrow_sources.contains_key(&id.sym) {
                self.check_borrow_conflict(&id.sym, false);
            }
        } else if let Expr::Member(_) = target_expr {
            let (root_opt, field_opt) = self.extract_member_path(target_expr);
            if let Some(root) = root_opt {
                if self.moved_vars.contains(&root) {
                    panic!("CP Compile Error: Use of moved value '{}'", root);
                }
                if !self.var_borrow_sources.contains_key(&root) {
                    self.check_borrow_conflict_field(&root, field_opt.as_deref(), false);
                }
            }
        }
    }

    pub(crate) fn extract_expr_borrow_info(
        &self,
        expr: &Expr,
    ) -> (Vec<(String, Option<String>)>, bool, bool) {
        // Returns: (source_vars, is_mut_borrow, is_ref_struct)
        match expr {
            Expr::Downcast { obj, is_mut, .. } => {
                if let Expr::Ident(id) = &**obj {
                    self.check_borrow_conflict(&id.sym, *is_mut);
                    let mut sources = vec![(id.sym.clone(), None)];
                    if let Some((orig, _)) = self.var_borrow_sources.get(&id.sym) {
                        if !orig.is_empty() {
                            sources = orig.clone();
                        }
                    }
                    return (sources, *is_mut, false);
                }
                (vec![], *is_mut, false)
            }
            Expr::Question(inner)
            | Expr::Unwrap(inner)
            | Expr::QuestionFallback { expr: inner, .. }
            | Expr::NullCoalesce { left: inner, .. }
            | Expr::Catch { expr: inner, .. } => self.extract_expr_borrow_info(inner),
            Expr::Borrow(inner, is_mut) => {
                match &**inner {
                    Expr::Ident(id) => {
                        if self.local_trampolines.contains(&id.sym) {
                            return (vec![], false, false);
                        }
                        if id.sym == "this" || id.sym == "_" || self.is_class_field(&id.sym) {
                            if *is_mut && self.current_receiver_mode == ReceiverMode::ReadOnly {
                                panic!(
                                    "CP Compile Error: Cannot take mutable borrow '&mut' of '{}' in a read-only method (add 'mut _' as first parameter)",
                                    id.sym
                                );
                            }
                            if self.is_class_field(&id.sym) {
                                self.check_borrow_conflict_field("this", Some(&id.sym), *is_mut);
                                return (
                                    vec![
                                        ("this".to_string(), Some(id.sym.clone())),
                                        ("_".to_string(), Some(id.sym.clone())),
                                    ],
                                    *is_mut,
                                    false,
                                );
                            }
                            self.check_borrow_conflict("this", *is_mut);
                            return (
                                vec![("this".to_string(), None), ("_".to_string(), None)],
                                *is_mut,
                                false,
                            );
                        }
                        self.check_borrow_conflict(&id.sym, *is_mut);
                        let mut sources = vec![(id.sym.clone(), None)];
                        if let Some((orig, _)) = self.var_borrow_sources.get(&id.sym) {
                            if !orig.is_empty() {
                                sources = orig.clone();
                            }
                        }
                        return (sources, *is_mut, false);
                    }
                    Expr::Member(_) => {
                        let (root_opt, field_opt) = self.extract_member_path(inner);
                        if let Some(root) = root_opt {
                            if root == "this" || root == "_" {
                                if *is_mut && self.current_receiver_mode == ReceiverMode::ReadOnly {
                                    panic!(
                                        "CP Compile Error: Cannot take mutable borrow '&mut {}.{}' in a read-only method (add 'mut _' as first parameter)",
                                        root, field_opt.as_deref().unwrap_or("")
                                    );
                                }
                                self.check_borrow_conflict_field(
                                    "this",
                                    field_opt.as_deref(),
                                    *is_mut,
                                );
                                return (
                                    vec![
                                        ("this".to_string(), field_opt.clone()),
                                        ("_".to_string(), field_opt),
                                    ],
                                    *is_mut,
                                    false,
                                );
                            }
                            let is_obj_ptr =
                                self.locals.get(&root).map_or(false, |t| {
                                    t.starts_with('*')
                                        || t.starts_with("?*")
                                        || t.contains("=> *")
                                        || t.contains("=> ?*")
                                }) || self.var_cp_types.get(&root).map_or(false, |t| {
                                    matches!(
                                        t,
                                        Type::RawPtr { .. }
                                            | Type::BorrowType { .. }
                                            | Type::CPtr(_)
                                            | Type::ManyRawPtr(_)
                                    )
                                });

                            self.check_borrow_conflict_field(&root, field_opt.as_deref(), *is_mut);
                            let resolved =
                                self.resolve_borrow_source_field(&root, field_opt.as_deref());
                            if is_obj_ptr {
                                if let Some((orig, _)) = self.var_borrow_sources.get(&root) {
                                    if !orig.is_empty() {
                                        return (resolved, *is_mut, false);
                                    }
                                }
                                if self.current_fn_params.contains(&root) {
                                    return (resolved, *is_mut, false);
                                }
                                return (vec![], *is_mut, false);
                            }
                            return (resolved, *is_mut, false);
                        }
                    }
                    _ => {}
                }
                (vec![], *is_mut, false)
            }
            Expr::Ident(id) => {
                let is_ref_st = self.var_is_ref_struct.contains(&id.sym);
                if let Some((sources, is_m)) = self.var_borrow_sources.get(&id.sym) {
                    return (sources.clone(), *is_m, is_ref_st);
                }
                if self.current_fn_params.contains(&id.sym) {
                    let is_m = self.mutable_fn_params.contains(&id.sym)
                        || matches!(
                            self.var_cp_types.get(&id.sym),
                            Some(
                                Type::BorrowType { is_mut: true, .. }
                                    | Type::RawPtr { .. }
                                    | Type::ManyRawPtr(_)
                                    | Type::CPtr(_)
                                    | Type::SentinelPtr(_, _)
                            )
                        );
                    return (vec![(id.sym.clone(), None)], is_m, is_ref_st);
                }
                (vec![], false, is_ref_st)
            }
            Expr::New(call) => {
                if let Expr::Ident(cid) = &*call.callee {
                    if self.ref_classes.contains(&cid.sym) {
                        let mut sources = Vec::new();
                        let mut any_mut = false;
                        for arg in &call.args {
                            match arg {
                                Expr::Borrow(inner, is_m) => {
                                    let (root_opt, field_opt) = self.extract_member_path(inner);
                                    if let Some(root) = root_opt {
                                        self.check_borrow_conflict_field(
                                            &root,
                                            field_opt.as_deref(),
                                            *is_m,
                                        );
                                        let resolved = self.resolve_borrow_source_field(
                                            &root,
                                            field_opt.as_deref(),
                                        );
                                        sources.extend(resolved);
                                    }
                                    if *is_m {
                                        any_mut = true;
                                    }
                                }
                                Expr::Ident(aid) => {
                                    if let Some((orig, is_m)) =
                                        self.var_borrow_sources.get(&aid.sym)
                                    {
                                        sources.extend(orig.clone());
                                        if *is_m {
                                            any_mut = true;
                                        }
                                    } else {
                                        sources.push((aid.sym.clone(), None));
                                    }
                                }
                                _ => {}
                            }
                        }
                        return (sources, any_mut, true);
                    }
                }
                (vec![], false, false)
            }
            Expr::Call(call) => {
                if let Expr::Member(mem) = &*call.callee {
                    if let Expr::Ident(obj_id) = &*mem.obj {
                        if let Some(type_name) = self.resolve_var_type_name(&obj_id.sym) {
                            if let Some((params, Some(ret_ty), recv_mode)) =
                                self.lookup_method(&type_name, &mem.prop.sym)
                            {
                                let non_recv_params: Vec<&Param> = params
                                    .iter()
                                    .filter(|p| {
                                        p.ident.sym != "this"
                                            && p.ident.sym != "_"
                                            && p.ident.sym != "self"
                                    })
                                    .collect();
                                match ret_ty {
                                    Type::BorrowType {
                                        is_mut, lifetimes, ..
                                    } => {
                                        let mut sources = Vec::new();
                                        // Tier 1: Method Return Lifetime Elision (default binds to receiver `_`/`this`)
                                        if lifetimes.is_empty()
                                            || lifetimes.iter().any(|l| l == "this" || l == "_")
                                        {
                                            if obj_id.sym == "this" || obj_id.sym == "_" {
                                                sources.push(("this".to_string(), None));
                                                sources.push(("_".to_string(), None));
                                            } else if let Some((orig, _)) =
                                                self.var_borrow_sources.get(&obj_id.sym)
                                            {
                                                sources.extend(orig.clone());
                                            } else {
                                                sources.push((obj_id.sym.clone(), None));
                                            }
                                        }
                                        // Tier 3: Explicit parameter binding on method (e.g. `: &other Tensor`)
                                        for lt_name in &lifetimes {
                                            if let Some(pos) = non_recv_params
                                                .iter()
                                                .position(|p| &p.ident.sym == lt_name)
                                            {
                                                if pos < call.args.len() {
                                                    if let Expr::Ident(arg_id) = &call.args[pos] {
                                                        if let Some((orig, _)) =
                                                            self.var_borrow_sources.get(&arg_id.sym)
                                                        {
                                                            sources.extend(orig.clone());
                                                        } else {
                                                            sources
                                                                .push((arg_id.sym.clone(), None));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        return (sources, is_mut, false);
                                    }
                                    Type::Ref(cid) if self.ref_classes.contains(&cid.sym) => {
                                        let mut sources = Vec::new();
                                        if obj_id.sym == "this" || obj_id.sym == "_" {
                                            sources.push(("this".to_string(), None));
                                            sources.push(("_".to_string(), None));
                                        } else if let Some((orig, _)) =
                                            self.var_borrow_sources.get(&obj_id.sym)
                                        {
                                            sources.extend(orig.clone());
                                        } else {
                                            sources.push((obj_id.sym.clone(), None));
                                        }
                                        return (sources, recv_mode == ReceiverMode::Mut, true);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                } else if let Expr::Ident(fid) = &*call.callee {
                    if fid.sym == "@fieldParentPtr"
                        || fid.sym == "@alignCast"
                        || fid.sym == "@ptrCast"
                    {
                        if let Some(last_arg) = call.args.last() {
                            return self.extract_expr_borrow_info(last_arg);
                        }
                    } else if fid.sym == "@as" {
                        if let Some(val_arg) = call.args.get(1) {
                            return self.extract_expr_borrow_info(val_arg);
                        }
                    }
                    // Check if `fid` is an omitted-receiver method call on current_class_name
                    if let Some(c_name) = &self.current_class_name {
                        if !self.locals.contains_key(&fid.sym) {
                            if let Some((_, Some(ret_ty), recv_mode)) =
                                self.lookup_method(c_name, &fid.sym)
                            {
                                match ret_ty {
                                    Type::BorrowType { is_mut, .. } => {
                                        return (
                                            vec![
                                                ("this".to_string(), None),
                                                ("_".to_string(), None),
                                            ],
                                            is_mut,
                                            false,
                                        );
                                    }
                                    Type::Ref(cid) if self.ref_classes.contains(&cid.sym) => {
                                        return (
                                            vec![
                                                ("this".to_string(), None),
                                                ("_".to_string(), None),
                                            ],
                                            recv_mode == ReceiverMode::Mut,
                                            true,
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    if let Some(sig) = self.fn_sigs.get(&fid.sym).cloned() {
                        // Also check param modifiers (`mut` / `move`) at call site
                        for (idx, param) in sig.params.iter().enumerate() {
                            if idx < call.args.len() {
                                if let Expr::Ident(arg_id) = &call.args[idx] {
                                    if param.is_mut {
                                        self.check_borrow_conflict(&arg_id.sym, true);
                                    } else if !param.is_move {
                                        self.check_borrow_conflict(&arg_id.sym, false);
                                    }
                                }
                            }
                        }
                        if let Some(ret_ty) = &sig.return_type {
                            match ret_ty {
                                Type::BorrowType {
                                    is_mut, lifetimes, ..
                                } => {
                                    let mut sources = Vec::new();
                                    if !lifetimes.is_empty() {
                                        // Tier 3: Explicit parameter name binding (&x T or &(x | y) T)
                                        for lt_name in lifetimes {
                                            if let Some(pos) = sig
                                                .params
                                                .iter()
                                                .position(|p| &p.ident.sym == lt_name)
                                            {
                                                if pos < call.args.len() {
                                                    if let Expr::Ident(arg_id) = &call.args[pos] {
                                                        if let Some((orig, _)) =
                                                            self.var_borrow_sources.get(&arg_id.sym)
                                                        {
                                                            sources.extend(orig.clone());
                                                        } else {
                                                            sources
                                                                .push((arg_id.sym.clone(), None));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // Tier 1 & Tier 2: Automatic Elision (single ref param) or Default Union (all object/ref params)
                                        for (pos, param) in sig.params.iter().enumerate() {
                                            let is_obj_param = matches!(
                                                param.ty,
                                                Type::Ref(_)
                                                    | Type::BorrowType { .. }
                                                    | Type::String
                                                    | Type::Array(_)
                                            );
                                            if is_obj_param && pos < call.args.len() {
                                                if let Expr::Ident(arg_id) = &call.args[pos] {
                                                    if let Some((orig, _)) =
                                                        self.var_borrow_sources.get(&arg_id.sym)
                                                    {
                                                        sources.extend(orig.clone());
                                                    } else {
                                                        sources.push((arg_id.sym.clone(), None));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    return (sources, *is_mut, false);
                                }
                                Type::Ref(cid) if self.ref_classes.contains(&cid.sym) => {
                                    let mut sources = Vec::new();
                                    for (pos, _) in sig.params.iter().enumerate() {
                                        if pos < call.args.len() {
                                            if let Expr::Ident(arg_id) = &call.args[pos] {
                                                if let Some((orig, _)) =
                                                    self.var_borrow_sources.get(&arg_id.sym)
                                                {
                                                    sources.extend(orig.clone());
                                                } else {
                                                    sources.push((arg_id.sym.clone(), None));
                                                }
                                            }
                                        }
                                    }
                                    return (sources, false, true);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                (vec![], false, false)
            }
            _ => (vec![], false, false),
        }
    }
}
