use super::*;
use crate::ast::*;
use crate::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

impl Codegen {
    pub fn block_returns_value(body: &[Stmt]) -> bool {
        for s in body {
            match s {
                Stmt::Return(r) => {
                    if r.arg.is_some() {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    let then_has = if let Stmt::Block(b) = &*if_stmt.cons {
                        Self::block_returns_value(&b.body)
                    } else if let Stmt::Return(r) = &*if_stmt.cons {
                        r.arg.is_some()
                    } else {
                        false
                    };
                    let else_has = if let Some(alt) = &if_stmt.alt {
                        if let Stmt::Block(b) = &**alt {
                            Self::block_returns_value(&b.body)
                        } else if let Stmt::Return(r) = &**alt {
                            r.arg.is_some()
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if then_has || else_has {
                        return true;
                    }
                }
                Stmt::Block(b) => {
                    if Self::block_returns_value(&b.body) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub(crate) fn generate_stmt(&mut self, stmt: &Stmt) {
        self.current_stmt_idx += 1;
        self.record_source_location(stmt.span());
        match stmt {
            Stmt::Var(var) => {
                if let Some(init) = &var.init {
                    let is_null_lit = matches!(init, Expr::Lit(Lit::Null))
                        || matches!(init, Expr::Ident(id) if id.sym == "null");
                    if is_null_lit {
                        if let Some(t) = &var.ty {
                            let allows_null = match t {
                                Type::Optional(_) => true,
                                Type::ResultType { ok, .. } => matches!(&**ok, Type::Optional(_)),
                                _ => false,
                            };
                            if !allows_null {
                                panic!(
                                    "CP Compile Error: Cannot assign 'null' to non-optional type"
                                );
                            }
                        } else {
                            panic!(
                                "CP Compile Error: Cannot assign 'null' without an explicit optional '?T' type annotation"
                            );
                        }
                    }
                    let (borrow_sources, is_mut_borrow, is_ref_st) =
                        self.extract_expr_borrow_info(init);
                    let old_box_trait = self.expected_box_trait.clone();
                    let old_array_elem = self.expected_array_elem_type.clone();
                    if let Some(ty) = &var.ty {
                        match ty {
                            Type::FixedArray(elem, _) | Type::Array(elem) => {
                                self.expected_array_elem_type = Some((**elem).clone());
                            }
                            Type::Ref(id) if id.sym.starts_with('[') => {
                                if let Some(idx) = id.sym.find(']') {
                                    let elem_name =
                                        id.sym[idx + 1..].trim_start_matches("const ").trim();
                                    self.expected_array_elem_type =
                                        Some(Self::type_from_name_static(elem_name));
                                }
                            }
                            _ => {}
                        }
                    }
                    match &var.ty {
                        Some(Type::BoxDyn(tr_id)) => {
                            self.expected_box_trait = Some(tr_id.sym.clone());
                        }
                        Some(Type::Optional(inner)) => {
                            if let Type::BoxDyn(tr_id) = &**inner {
                                self.expected_box_trait = Some(tr_id.sym.clone());
                            }
                        }
                        Some(Type::ResultType { ok, err }) => {
                            if let Type::BoxDyn(tr_id) = &**ok {
                                self.expected_box_trait = Some(tr_id.sym.clone());
                            } else if let ErrorType::BoxDyn(tr_id) = err {
                                self.expected_box_trait = Some(tr_id.sym.clone());
                            }
                        }
                        _ => {}
                    }
                    let expr_code =
                        if let Some(Type::ResultType { err: var_err, .. }) = var.ty.clone() {
                            if let Expr::ErrWrap(err_inner) = init {
                                let ev = self.generate_err_value(err_inner, Some(&var_err));
                                format!(".{{ .err = {} }}", ev)
                            } else if is_null_lit {
                                ".{ .ok = null }".to_string()
                            } else {
                                let init_ty = self.infer_cp_type(init);
                                let raw_c = self.generate_expr(init);
                                if let Some(Type::ResultType { err: init_err, .. }) = init_ty {
                                    self.verify_error_type_compatibility(&init_err, &var_err);
                                    raw_c
                                } else {
                                    format!(".{{ .ok = {} }}", raw_c)
                                }
                            }
                        } else {
                            self.generate_expr(init)
                        };

                    let is_try = expr_code.starts_with("try ");
                    let final_expr = if is_try {
                        expr_code[4..].to_string()
                    } else {
                        expr_code.clone()
                    };
                    if matches!(&var.pat, Pattern::Wildcard)
                        || matches!(&var.pat, Pattern::Ident(ident) if ident.sym == "_")
                    {
                        let init_val = if is_try {
                            format!("try {}", final_expr)
                        } else {
                            expr_code
                        };
                        self.write_to_case(&format!("_ = {};", init_val));
                        return;
                    }
                    match &var.pat {
                        Pattern::Ident(ident) => {
                            if matches!(var.kind, crate::ast::VarDeclKind::Const) {
                                self.const_vars.insert(ident.sym.clone());
                            } else {
                                self.const_vars.remove(&ident.sym);
                            }
                            self.var_scope_depth
                                .insert(ident.sym.clone(), self.current_scope_depth);
                            if let Some(t) = &var.ty {
                                self.var_cp_types.insert(ident.sym.clone(), t.clone());
                            } else if let Some(inferred_ty) = self.infer_cp_type(init) {
                                self.var_cp_types.insert(ident.sym.clone(), inferred_ty);
                            }
                            self.expected_box_trait = old_box_trait;
                            self.expected_array_elem_type = old_array_elem;
                            let mut is_var_borrow = !borrow_sources.is_empty();
                            if let Some(Type::BorrowType { .. }) = &var.ty {
                                is_var_borrow = true;
                            }
                            if !borrow_sources.is_empty() {
                                self.var_borrow_sources.insert(
                                    ident.sym.clone(),
                                    (borrow_sources.clone(), is_mut_borrow),
                                );
                            }
                            if is_ref_st {
                                self.var_is_ref_struct.insert(ident.sym.clone());
                            } else if let Some(Type::Ref(cid)) = &var.ty {
                                if self.ref_classes.contains(&cid.sym) {
                                    self.var_is_ref_struct.insert(ident.sym.clone());
                                }
                            }

                            if expr_code == "undefined" && !self.is_lib_mode {
                                self.uninitialized_vars.insert(ident.sym.clone());
                            }
                            let is_const = var.kind == crate::ast::VarDeclKind::Const
                                || matches!(init, Expr::Builtin(b) if b.name == "@typeOf" || b.name == "@typeInfo" || b.name == "@TypeOf" || b.name == "@TypeInfo")
                                || matches!(init, Expr::Call(c) if matches!(&*c.callee, Expr::Ident(id) if id.sym == "@typeOf" || id.sym == "@TypeOf" || id.sym == "@typeInfo" || id.sym == "@TypeInfo"));
                            let lhs = if self.is_state_machine {
                                format!("ctx.{}", ident.sym)
                            } else {
                                let keyword = if var.is_threadlocal {
                                    "threadlocal var"
                                } else if is_const {
                                    if var.is_comptime {
                                        "comptime const"
                                    } else {
                                        "const"
                                    }
                                } else {
                                    if var.is_comptime {
                                        "comptime var"
                                    } else {
                                        "var"
                                    }
                                };
                                let ty_annot = if expr_code == "undefined" {
                                    if let Some(t) = &var.ty {
                                        format!(": {}", self.map_type(t))
                                    } else {
                                        String::new()
                                    }
                                } else if let Some(t) = &var.ty {
                                    let is_trait_ty = if let Type::Ref(id) = t {
                                        self.trait_defs.contains_key(&id.sym)
                                    } else {
                                        false
                                    };
                                    if let Type::BorrowType { inner, is_mut, .. } = t {
                                        if *is_mut {
                                            if let Type::Array(elem) = &**inner {
                                                format!(": []{}", self.map_type(elem))
                                            } else if let Type::String = &**inner {
                                                ": []u8".to_string()
                                            } else {
                                                format!(": {}", self.map_type(t))
                                            }
                                        } else {
                                            if let Type::Array(elem) = &**inner {
                                                format!(": []const {}", self.map_type(elem))
                                            } else if let Type::String = &**inner {
                                                ": []const u8".to_string()
                                            } else {
                                                format!(": {}", self.map_type(t))
                                            }
                                        }
                                    } else if is_trait_ty {
                                        String::new()
                                    } else if matches!(t, Type::CStr) {
                                        match var.kind {
                                            crate::ast::VarDeclKind::Const => {
                                                ": [*:0]const u8".to_string()
                                            }
                                            _ => ": [*:0]u8".to_string(),
                                        }
                                    } else {
                                        format!(": {}", self.map_type(t))
                                    }
                                } else if matches!(init, Expr::Lit(Lit::Int(_))) {
                                    ": i32".to_string()
                                } else if matches!(init, Expr::Lit(Lit::Float(_))) {
                                    ": f32".to_string()
                                } else if matches!(init, Expr::Lit(Lit::Str(_))) {
                                    ": []const u8".to_string()
                                } else if matches!(init, Expr::Object(_) | Expr::Array(_)) {
                                    if let Some(inferred_ty) = self.infer_cp_type(init) {
                                        format!(": {}", self.map_type(&inferred_ty))
                                    } else {
                                        String::new()
                                    }
                                } else {
                                    String::new()
                                };
                                format!("{} {}{}", keyword, ident.sym, ty_annot)
                            };
                            let init_val = if matches!(var.ty, Some(Type::CStr))
                                && var.kind != crate::ast::VarDeclKind::Const
                                && (final_expr.starts_with('"')
                                    || matches!(init, Expr::Lit(Lit::Str(_))))
                            {
                                format!("@constCast({})", final_expr)
                            } else if matches!(
                                var.ty,
                                Some(Type::ManyRawPtr(_)) | Some(Type::CPtr(_))
                            ) && matches!(init, Expr::Borrow(_, _))
                            {
                                format!("@ptrCast({})", final_expr)
                            } else {
                                final_expr.clone()
                            };
                            self.write_to_case(&format!(
                                "{} = {};",
                                lhs,
                                if is_try {
                                    format!("try {}", init_val)
                                } else {
                                    init_val
                                }
                            ));
                            let needs_drop = if let Some(t) = &var.ty {
                                self.type_has_release(Some(t))
                            } else if let Some(t) = self.infer_cp_type(init) {
                                self.type_has_release(Some(&t))
                            } else if matches!(
                                init,
                                Expr::Lit(_)
                                    | Expr::Slice(_)
                                    | Expr::Cast { .. }
                                    | Expr::Bin(_)
                                    | Expr::Index { .. }
                            ) {
                                false
                            } else if let Expr::Ident(id) = init {
                                if let Some(t) = self.var_cp_types.get(&id.sym) {
                                    self.type_has_release(Some(t))
                                } else {
                                    false
                                }
                            } else {
                                self.class_defs
                                    .values()
                                    .any(|c| c.methods.iter().any(|m| m.ident.sym == "release"))
                            };
                            if !self.is_state_machine && !is_var_borrow && needs_drop {
                                self.drop_flag_vars.insert(ident.sym.clone());
                                self.write_to_case(&format!(
                                    "var __{}_moved = false; _ = &__{}_moved;",
                                    ident.sym, ident.sym
                                ));
                                self.write_to_case(&format!("defer if (!__{0}_moved) {{ const __T = @TypeOf({0}); if (@typeInfo(__T) == .pointer) {{ const Child = @typeInfo(__T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) {{ const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = {0}.release() catch {{}}; }} else {{ {0}.release(); }} }} }} else if (@typeInfo(__T) == .@\"struct\" and @hasDecl(__T, \"release\")) {{ var _drop_val = {0}; _drop_val.release(); }} }};", ident.sym));
                            } else if !self.is_state_machine {
                                if !is_const {
                                    self.write_to_case(&format!("_ = &{};", ident.sym));
                                }
                            }
                            if let Expr::Ident(id) = init {
                                // Only move if `id` is an owned local pointer, Box<dyn Trait>, or `move` parameter (not a borrow)
                                let is_id_borrow = self.var_borrow_sources.contains_key(&id.sym)
                                    || (self.current_fn_params.contains(&id.sym)
                                        && !self.move_fn_params.contains(&id.sym));
                                if !is_id_borrow && self.locals.contains_key(&id.sym) {
                                    let ty_str = &self.locals[&id.sym];
                                    let is_owned_type = (ty_str.starts_with('*')
                                        && ty_str != "*anyopaque")
                                        || self.trait_defs.contains_key(ty_str.as_str());
                                    if is_owned_type {
                                        // NLL Check: cannot move `id` if it has active borrows!
                                        self.check_active_borrows_before_move(&id.sym);
                                        // If `id` is a ref struct, check if it's moving to outer scope
                                        if self.var_is_ref_struct.contains(&id.sym) {
                                            if let Some((srcs, _)) =
                                                self.var_borrow_sources.get(&id.sym)
                                            {
                                                for (s, _) in srcs {
                                                    let s_depth = self
                                                        .var_scope_depth
                                                        .get(s)
                                                        .copied()
                                                        .unwrap_or(1);
                                                    if s_depth > self.current_scope_depth {
                                                        panic!("CP Compile Error: Stack-bound ref struct '{}' cannot escape scope of borrowed value '{}'", id.sym, s);
                                                    }
                                                }
                                            }
                                        }
                                        self.moved_vars.insert(id.sym.clone());
                                        if !self.is_state_machine {
                                            self.write_to_case(&format!(
                                                "__{}_moved = true;",
                                                id.sym
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Pattern::Tuple(elements) => {
                            let tmp_id = self.tmp_counter;
                            self.tmp_counter += 1;
                            self.write_to_case(&format!(
                                "const _tmp_v{} = {};",
                                tmp_id,
                                if is_try {
                                    format!("try {}", final_expr)
                                } else {
                                    final_expr.clone()
                                }
                            ));
                            let mut init_ty = self.infer_cp_type(init);
                            while let Some(Type::BorrowType { inner, .. }) = init_ty {
                                init_ty = Some(*inner);
                            }
                            let rest_pos =
                                elements.iter().position(|p| matches!(p, Pattern::Rest(_)));
                            let is_const = var.kind == crate::ast::VarDeclKind::Const;
                            let keyword = match var.kind {
                                crate::ast::VarDeclKind::Const => "const",
                                _ => "var",
                            };
                            for (idx, el) in elements.iter().enumerate() {
                                match el {
                                    Pattern::Ident(ident) => {
                                        self.var_scope_depth
                                            .insert(ident.sym.clone(), self.current_scope_depth);
                                        if let Some(Type::Array(elem_ty)) = &init_ty {
                                            self.var_cp_types
                                                .insert(ident.sym.clone(), (**elem_ty).clone());
                                        } else if let Some(Type::Tuple(tys)) = &init_ty {
                                            let ty_idx = if let Some(r_idx) = rest_pos {
                                                if idx < r_idx {
                                                    Some(idx)
                                                } else {
                                                    let offset = elements.len() - idx;
                                                    tys.len().checked_sub(offset)
                                                }
                                            } else {
                                                Some(idx)
                                            };
                                            if let Some(ti) = ty_idx {
                                                if let Some(t) = tys.get(ti) {
                                                    self.var_cp_types
                                                        .insert(ident.sym.clone(), t.clone());
                                                }
                                            }
                                        } else if let Some(Type::Object(fields)) = &init_ty {
                                            let ty_idx = if let Some(r_idx) = rest_pos {
                                                if idx < r_idx {
                                                    Some(idx)
                                                } else {
                                                    let offset = elements.len() - idx;
                                                    fields.len().checked_sub(offset)
                                                }
                                            } else {
                                                Some(idx)
                                            };
                                            if let Some(ti) = ty_idx {
                                                if let Some(f) = fields.get(ti) {
                                                    self.var_cp_types
                                                        .insert(ident.sym.clone(), f.ty.clone());
                                                }
                                            }
                                        }
                                        let lhs = if self.is_state_machine {
                                            format!("ctx.{}", ident.sym)
                                        } else {
                                            format!("{} {}", keyword, ident.sym)
                                        };
                                        if let Some(r_idx) = rest_pos {
                                            if idx > r_idx {
                                                let offset_from_end = elements.len() - idx;
                                                self.write_to_case(&format!(
                                                    "{} = cp_unpack_tail(_tmp_v{}, {});",
                                                    lhs, tmp_id, offset_from_end
                                                ));
                                            } else {
                                                self.write_to_case(&format!(
                                                    "{} = cp_unpack_elem(_tmp_v{}, {});",
                                                    lhs, tmp_id, idx
                                                ));
                                            }
                                        } else {
                                            self.write_to_case(&format!(
                                                "{} = cp_unpack_elem(_tmp_v{}, {});",
                                                lhs, tmp_id, idx
                                            ));
                                        }
                                        if !self.is_state_machine {
                                            self.write_to_case(&format!("_ = &{};", ident.sym));
                                        }
                                    }
                                    Pattern::Rest(inner) => {
                                        if let Pattern::Ident(ident) = &**inner {
                                            self.var_scope_depth.insert(
                                                ident.sym.clone(),
                                                self.current_scope_depth,
                                            );
                                            let make_rest_ty = |elem: Type| {
                                                if is_const {
                                                    Type::BorrowType {
                                                        inner: Box::new(Type::Array(Box::new(
                                                            elem,
                                                        ))),
                                                        is_mut: false,
                                                        lifetimes: vec![],
                                                    }
                                                } else {
                                                    Type::Array(Box::new(elem))
                                                }
                                            };
                                            if let Some(Type::Array(elem_ty)) = &init_ty {
                                                self.var_cp_types.insert(
                                                    ident.sym.clone(),
                                                    make_rest_ty((**elem_ty).clone()),
                                                );
                                            } else if let Some(Type::Tuple(tys)) = &init_ty {
                                                let elem_ty = tys
                                                    .get(idx)
                                                    .cloned()
                                                    .or_else(|| tys.first().cloned())
                                                    .unwrap_or(Type::I32);
                                                self.var_cp_types.insert(
                                                    ident.sym.clone(),
                                                    make_rest_ty(elem_ty),
                                                );
                                            } else if let Some(Type::Object(fields)) = &init_ty {
                                                let elem_ty = fields
                                                    .get(idx)
                                                    .map(|f| f.ty.clone())
                                                    .or_else(|| {
                                                        fields.first().map(|f| f.ty.clone())
                                                    })
                                                    .unwrap_or(Type::I32);
                                                self.var_cp_types.insert(
                                                    ident.sym.clone(),
                                                    make_rest_ty(elem_ty),
                                                );
                                            }
                                            let suffix_len = elements.len() - 1 - idx;
                                            let lhs = if self.is_state_machine {
                                                format!("ctx.{}", ident.sym)
                                            } else {
                                                format!("{} {}", keyword, ident.sym)
                                            };
                                            self.write_to_case(&format!(
                                                "{} = cp_unpack_slice(_tmp_v{}, {}, {});",
                                                lhs, tmp_id, idx, suffix_len
                                            ));
                                            if !self.is_state_machine {
                                                self.write_to_case(&format!("_ = &{};", ident.sym));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Pattern::Object(props) => {
                            let tmp_id = self.tmp_counter;
                            self.tmp_counter += 1;
                            self.write_to_case(&format!(
                                "const _tmp_obj{} = {};",
                                tmp_id,
                                if is_try {
                                    format!("try {}", final_expr)
                                } else {
                                    final_expr.clone()
                                }
                            ));
                            let mut init_ty = self.infer_cp_type(init);
                            while let Some(Type::BorrowType { inner, .. }) = init_ty {
                                init_ty = Some(*inner);
                            }
                            let non_rest_keys: HashSet<String> = props
                                .iter()
                                .filter(|p| !p.is_rest)
                                .map(|p| p.key.sym.clone())
                                .collect();

                            for prop in props {
                                if prop.is_rest {
                                    if prop.key.sym == "*"
                                        || matches!(prop.val, Some(Pattern::Wildcard))
                                    {
                                        continue;
                                    }
                                    let bound_name = prop.key.sym.clone();
                                    self.var_scope_depth
                                        .insert(bound_name.clone(), self.current_scope_depth);
                                    let remaining_fields = if let Some(Type::Object(fields)) =
                                        &init_ty
                                    {
                                        fields
                                            .iter()
                                            .filter(|f| !non_rest_keys.contains(&f.ident.sym))
                                            .cloned()
                                            .collect::<Vec<_>>()
                                    } else if let Some(Type::Ref(cname)) = &init_ty {
                                        if let Some(cls) = self.class_defs.get(&cname.sym) {
                                            cls.fields
                                                .iter()
                                                .filter(|f| !non_rest_keys.contains(&f.ident.sym))
                                                .cloned()
                                                .collect::<Vec<_>>()
                                        } else {
                                            vec![]
                                        }
                                    } else {
                                        vec![]
                                    };
                                    let rest_obj_ty = Type::Object(remaining_fields.clone());
                                    self.var_cp_types.insert(bound_name.clone(), rest_obj_ty);

                                    let rhs = if remaining_fields.is_empty() {
                                        ".{}".to_string()
                                    } else {
                                        let inits: Vec<String> = remaining_fields
                                            .iter()
                                            .map(|f| {
                                                format!(
                                                    ".{} = _tmp_obj{}.{}",
                                                    f.ident.sym, tmp_id, f.ident.sym
                                                )
                                            })
                                            .collect();
                                        format!(".{{ {} }}", inits.join(", "))
                                    };

                                    let lhs = if self.is_state_machine {
                                        format!("ctx.{}", bound_name)
                                    } else {
                                        let keyword = match var.kind {
                                            crate::ast::VarDeclKind::Const => "const",
                                            _ => "var",
                                        };
                                        format!("{} {}", keyword, bound_name)
                                    };
                                    self.write_to_case(&format!("{} = {};", lhs, rhs));
                                    if !self.is_state_machine {
                                        self.write_to_case(&format!("_ = &{};", bound_name));
                                    }
                                    continue;
                                }

                                if matches!(prop.val, Some(Pattern::Wildcard)) {
                                    continue;
                                }
                                let bound_name = if let Some(sub_pat) = &prop.val {
                                    if let Pattern::Ident(id) = sub_pat {
                                        id.sym.clone()
                                    } else {
                                        prop.key.sym.clone()
                                    }
                                } else {
                                    prop.key.sym.clone()
                                };
                                self.var_scope_depth
                                    .insert(bound_name.clone(), self.current_scope_depth);
                                if let Some(Type::Object(fields)) = &init_ty {
                                    if let Some(f) =
                                        fields.iter().find(|f| f.ident.sym == prop.key.sym)
                                    {
                                        self.var_cp_types.insert(bound_name.clone(), f.ty.clone());
                                    }
                                }
                                let lhs = if self.is_state_machine {
                                    format!("ctx.{}", bound_name)
                                } else {
                                    let keyword = match var.kind {
                                        crate::ast::VarDeclKind::Const => "const",
                                        _ => "var",
                                    };
                                    format!("{} {}", keyword, bound_name)
                                };
                                self.write_to_case(&format!(
                                    "{} = _tmp_obj{}.{};",
                                    lhs, tmp_id, prop.key.sym
                                ));
                                if !self.is_state_machine {
                                    self.write_to_case(&format!("_ = &{};", bound_name));
                                }
                            }
                        }
                        Pattern::Wildcard => {
                            self.write_to_case(&format!(
                                "_ = {};",
                                if is_try {
                                    format!("try {}", final_expr)
                                } else {
                                    final_expr.clone()
                                }
                            ));
                        }
                        _ => {}
                    }
                }
            }
            Stmt::Expr(expr) => match expr {
                Expr::Bin(bin) if bin.op == BinaryOp::Assign => {
                    if let Expr::Member(mem) = &*bin.left {
                        self.check_mutation_allowed(
                            &mem.obj,
                            &format!("field assignment '.{} ='", mem.prop.sym),
                        );
                    } else if let Expr::Deref(inner) = &*bin.left {
                        self.check_mutation_allowed(inner, "dereference assignment '.* ='");
                    } else if let Expr::Index { obj, .. } = &*bin.left {
                        self.check_mutation_allowed(obj, "index assignment '[] ='");
                    } else if let Expr::Ident(lhs_id) = &*bin.left {
                        if lhs_id.sym != "_" {
                            if self.const_vars.contains(&lhs_id.sym) {
                                panic!(
                                    "CP Compile Error: Cannot assign to 'const' variable '{}' (declare with 'let' to allow mutation)",
                                    lhs_id.sym
                                );
                            }
                            if self.current_fn_params.contains(&lhs_id.sym)
                                && !self.mutable_fn_params.contains(&lhs_id.sym)
                            {
                                panic!(
                                    "CP Compile Error: Cannot mutate read-only variable or captured outer variable '{}'",
                                    lhs_id.sym
                                );
                            }
                            if self.loop_borrowed_containers.contains(&lhs_id.sym) {
                                panic!(
                                    "CP Compile Error: Cannot reassign container '{}' while iterating over it",
                                    lhs_id.sym
                                );
                            }
                            if self.is_class_field(&lhs_id.sym) {
                                if self.current_receiver_mode == ReceiverMode::ReadOnly {
                                    panic!(
                                        "CP Compile Error: Cannot mutate field '{}' in a read-only method (add 'mut _' as first parameter)",
                                        lhs_id.sym
                                    );
                                }
                                self.check_borrow_conflict("this", true);
                                self.check_borrow_conflict("_", true);
                            } else {
                                self.check_borrow_conflict(&lhs_id.sym, true);
                            }
                        }
                    }

                    let is_null_rhs = matches!(&*bin.right, Expr::Lit(Lit::Null))
                        || matches!(&*bin.right, Expr::Ident(id) if id.sym == "null");
                    if is_null_rhs {
                        if let Expr::Ident(lhs_id) = &*bin.left {
                            self.narrowed_null_vars.remove(&lhs_id.sym);
                            if let Some(t) = self.var_cp_types.get(&lhs_id.sym) {
                                if !matches!(t, Type::Optional(_)) {
                                    panic!(
                                        "CP Compile Error: Cannot assign 'null' to non-optional variable '{}'",
                                        lhs_id.sym
                                    );
                                }
                            }
                        }
                    }

                    let (borrow_sources, is_mut_borrow, is_ref_st) =
                        self.extract_expr_borrow_info(&bin.right);
                    let mut expected_box = None;
                    if let Expr::Ident(lhs_id) = &*bin.left {
                        if let Some(Type::BoxDyn(tr_id)) = self.var_cp_types.get(&lhs_id.sym) {
                            expected_box = Some(tr_id.sym.clone());
                        }
                        self.uninitialized_vars.remove(&lhs_id.sym);
                    } else if let Expr::Index { obj, .. } = &*bin.left {
                        if let Expr::Ident(lhs_id) = &**obj {
                            self.uninitialized_vars.remove(&lhs_id.sym);
                        }
                    }
                    if let Expr::Ident(lhs_id) = &*bin.left {
                        let lhs_depth = self.var_scope_depth.get(&lhs_id.sym).copied().unwrap_or(1);
                        for (src, _) in &borrow_sources {
                            let src_depth = self.var_scope_depth.get(src).copied().unwrap_or(1);
                            if src_depth > lhs_depth {
                                if is_ref_st || self.var_is_ref_struct.contains(&lhs_id.sym) {
                                    panic!(
                                        "CP Compile Error: Stack-bound ref struct cannot escape to outer variable '{}' because borrowed value '{}' does not live long enough",
                                        lhs_id.sym, src
                                    );
                                } else {
                                    panic!(
                                        "CP Compile Error: Borrowed value '{}' does not live long enough to be assigned to outer variable '{}'",
                                        src, lhs_id.sym
                                    );
                                }
                            }
                        }
                    }
                    let old_box = self.expected_box_trait.clone();
                    if expected_box.is_some() {
                        self.expected_box_trait = expected_box;
                    }
                    let r = self.generate_expr(&bin.right);
                    self.expected_box_trait = old_box;
                    if let Expr::Ident(lhs_id) = &*bin.left {
                        if !borrow_sources.is_empty() {
                            self.var_borrow_sources
                                .insert(lhs_id.sym.clone(), (borrow_sources, is_mut_borrow));
                        }
                        if is_ref_st {
                            self.var_is_ref_struct.insert(lhs_id.sym.clone());
                        }
                    }
                    if let Expr::Index { obj, index } = &*bin.left {
                        let obj_ty = self.infer_cp_type(obj);
                        let is_map_or_dict =
                            matches!(obj_ty, Some(Type::Map(_, _)) | Some(Type::Dict(_, _)));
                        let obj_code = self.generate_expr(obj);
                        let idx_code = self.generate_expr(index);
                        if is_map_or_dict {
                            self.write_to_case(&format!(
                                "_ = cp_index_assign(&{}, {}, {});",
                                obj_code, idx_code, r
                            ));
                        } else {
                            self.write_to_case(&format!(
                                "{}[cp_cast_int(usize, {})] = {};",
                                obj_code, idx_code, r
                            ));
                        }
                    } else {
                        let l = if let Expr::Ident(lhs_id) = &*bin.left {
                            if lhs_id.sym == "_" {
                                "_".to_string()
                            } else {
                                let was = self.narrowed_null_vars.remove(&lhs_id.sym);
                                let code = self.generate_expr(&bin.left);
                                if was && !is_null_rhs {
                                    self.narrowed_null_vars.insert(lhs_id.sym.clone());
                                }
                                code
                            }
                        } else {
                            self.generate_expr(&bin.left)
                        };
                        self.write_to_case(&format!("{} = {};", l, r));
                    }

                    if let Expr::Ident(id) = &*bin.right {
                        let is_id_borrow = self.var_borrow_sources.contains_key(&id.sym)
                            || (self.current_fn_params.contains(&id.sym)
                                && !self.move_fn_params.contains(&id.sym));
                        if !is_id_borrow && self.locals.contains_key(&id.sym) {
                            let ty_str = &self.locals[&id.sym];
                            let is_owned_type = (ty_str.starts_with('*') && ty_str != "*anyopaque")
                                || self.trait_defs.contains_key(ty_str.as_str());
                            if is_owned_type {
                                self.check_active_borrows_before_move(&id.sym);
                                self.moved_vars.insert(id.sym.clone());
                                if !self.is_state_machine && self.drop_flag_vars.contains(&id.sym) {
                                    self.write_to_case(&format!("__{}_moved = true;", id.sym));
                                }
                            }
                        }
                    }
                }
                Expr::Match(m) => {
                    let sw_code = self.generate_switch(m, false);
                    self.write_to_case(&sw_code);
                }
                Expr::Zig(zig) => {
                    self.write_to_case(&format!("{}\n", lower_struct_body(&zig.code)))
                }
                Expr::Bin(bin)
                    if bin.op == BinaryOp::LogicalOr
                        && matches!(
                            &*bin.right,
                            Expr::Continue(_) | Expr::Break(_, _) | Expr::Return(_, _)
                        ) =>
                {
                    let l = self.generate_expr(&bin.left);
                    match &*bin.right {
                        Expr::Continue(_) => {
                            self.write_to_case(&format!("if (!({})) continue;", l));
                        }
                        Expr::Break(_, val) => {
                            if let Some(v) = val {
                                let v_code = self.generate_expr(v);
                                self.write_to_case(&format!("if (!({})) break {};", l, v_code));
                            } else {
                                self.write_to_case(&format!("if (!({})) break;", l));
                            }
                        }
                        Expr::Return(_, ret_expr) => {
                            let ret_stmt = self.build_ret_stmt_for_fallback(ret_expr.as_deref());
                            self.write_to_case(&format!("if (!({})) {{ {} }}", l, ret_stmt));
                        }
                        _ => unreachable!(),
                    }
                }
                Expr::LabeledBreak(_, label, val) => {
                    if let Some(v) = val {
                        let vc = self.generate_expr(v);
                        self.write_to_case(&format!("break :{} {};", label.sym, vc));
                    } else {
                        self.write_to_case(&format!("break :{};", label.sym));
                    }
                }
                Expr::LabeledContinue(_, label) => {
                    self.write_to_case(&format!("continue :{};", label.sym));
                }
                Expr::Break(_, val) => {
                    let cur_blk = self.current_block_label.clone();
                    if let Some(v) = val {
                        let vc = self.generate_expr(v);
                        if let Some(blk) = &cur_blk {
                            self.write_to_case(&format!("break :{} {};", blk, vc));
                        } else {
                            self.write_to_case(&format!("break {};", vc));
                        }
                    } else {
                        if let Some(blk) = &cur_blk {
                            self.write_to_case(&format!("break :{};", blk));
                        } else {
                            self.write_to_case("break;");
                        }
                    }
                }
                _ => {
                    let e = self.generate_expr(expr);
                    if e.starts_with("@branchHint") {
                        self.write_to_case(&format!("{};", e));
                    } else {
                        self.write_to_case(&format!("_ = {};", e));
                    }
                }
            },
            Stmt::Block(b) | Stmt::UnsafeBlock(b) => {
                self.current_scope_depth += 1;
                if !self.is_state_machine {
                    self.write_to_case("{");
                    self.indent();
                }
                for s in &b.body {
                    self.generate_stmt(s);
                }
                if !self.is_state_machine {
                    self.dedent();
                    self.write_to_case("}");
                }
                self.current_scope_depth -= 1;
            }
            Stmt::ArenaBlock(b) => {
                self.current_scope_depth += 1;
                let arena_id = self.tmp_counter;
                self.tmp_counter += 1;
                let arena_var = format!("__arena_{}", arena_id);
                let alloc_var = format!("__arena_alloc_{}", arena_id);
                self.write_to_case("{");
                self.indent();
                self.write_to_case(&format!(
                    "var {} = std.heap.ArenaAllocator.init(std.heap.page_allocator);",
                    arena_var
                ));
                self.write_to_case(&format!("defer {}.deinit();", arena_var));
                self.write_to_case(&format!(
                    "const {} = {}.allocator(); _ = &{};",
                    alloc_var, arena_var, alloc_var
                ));
                self.arena_allocator_stack.push(alloc_var);
                for s in &b.body {
                    self.generate_stmt(s);
                }
                self.arena_allocator_stack.pop();
                self.dedent();
                self.write_to_case("}");
                self.current_scope_depth -= 1;
            }
            Stmt::ComptimeBlock(b) => {
                let saved = self.current_case_content.clone();
                let saved_sm = self.is_state_machine;
                let saved_comptime = self.is_comptime;
                self.is_state_machine = false;
                self.is_comptime = true;
                self.current_case_content.clear();
                for s in &b.body {
                    self.generate_stmt(s);
                }
                let body_str = self.current_case_content.clone();
                self.current_case_content = saved;
                self.is_state_machine = saved_sm;
                self.is_comptime = saved_comptime;
                self.write_to_case(&format!("comptime {{\n{}\n}}", body_str));
            }
            Stmt::Defer(defer_stmt) => {
                self.write_to_case("defer {");
                self.generate_stmt(defer_stmt);
                self.write_to_case("}");
            }
            Stmt::ErrDefer(errdefer_stmt) => {
                self.write_to_case("errdefer {");
                self.generate_stmt(errdefer_stmt);
                self.write_to_case("}");
            }
            Stmt::If(i) => {
                let not_null_vars = Self::extract_not_null_vars(&i.test);
                let is_null_vars = Self::extract_is_null_vars(&i.test);
                if i.is_comptime {
                    let test = self.generate_expr(&i.test);
                    self.write_to_case(&format!("if (comptime {}) {{", test));
                    self.indent();
                    let old_narrowed = self.narrowed_null_vars.clone();
                    self.narrowed_null_vars.extend(not_null_vars);
                    if let Stmt::Block(b) = &*i.cons {
                        for s in &b.body {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(&i.cons);
                    }
                    self.narrowed_null_vars = old_narrowed.clone();
                    self.dedent();
                    if let Some(alt) = &i.alt {
                        self.write_to_case("} else {");
                        self.indent();
                        self.narrowed_null_vars.extend(is_null_vars);
                        if let Stmt::Block(b) = &**alt {
                            for s in &b.body {
                                self.generate_stmt(s);
                            }
                        } else {
                            self.generate_stmt(alt);
                        }
                        self.narrowed_null_vars = old_narrowed;
                        self.dedent();
                    }
                    self.write_to_case("}");
                    return;
                }
                let test = self.generate_expr(&i.test);
                let old_uninit = self.uninitialized_vars.clone();
                let old_narrowed = self.narrowed_null_vars.clone();
                if test == "true" && i.capture.is_none() {
                    self.narrowed_null_vars.extend(not_null_vars);
                    self.generate_stmt(&i.cons);
                    self.narrowed_null_vars = old_narrowed;
                } else if test == "false" && i.capture.is_none() {
                    if let Some(alt) = &i.alt {
                        self.narrowed_null_vars.extend(is_null_vars);
                        self.generate_stmt(alt);
                        self.narrowed_null_vars = old_narrowed;
                    }
                } else {
                    let (saved_cap_local, saved_cap_cp, was_mut_loop) =
                        if let Some(cap) = &i.capture {
                            self.var_scope_depth
                                .insert(cap.sym.clone(), self.current_scope_depth + 1);
                            self.block_locals.insert(cap.sym.clone());
                            let was_mut = if i.is_ptr_capture {
                                !self.mut_borrow_loop_vars.insert(cap.sym.clone())
                            } else {
                                false
                            };
                            let ptr_prefix = if i.is_ptr_capture { "*" } else { "" };
                            self.write_to_case(&format!(
                                "if ({}) |{}{}| {{",
                                test, ptr_prefix, cap.sym
                            ));
                            let test_cp_ty = self.infer_cp_type(&i.test);
                            let unwrapped_ty = match &test_cp_ty {
                                Some(Type::Optional(inner)) => (**inner).clone(),
                                Some(Type::ErrorUnion(inner)) => (**inner).clone(),
                                Some(Type::ResultType { ok, .. }) => (**ok).clone(),
                                Some(other) => other.clone(),
                                None => Type::Any,
                            };
                            let final_ty = if i.is_ptr_capture {
                                Type::BorrowType {
                                    is_mut: true,
                                    inner: Box::new(unwrapped_ty.clone()),
                                    lifetimes: vec![],
                                }
                            } else {
                                unwrapped_ty.clone()
                            };
                            let zig_ty = self.map_type(&final_ty);
                            let prev_cp = self.var_cp_types.insert(cap.sym.clone(), final_ty);
                            let prev_l = self.locals.insert(cap.sym.clone(), zig_ty);
                            (prev_l, prev_cp, was_mut)
                        } else {
                            self.write_to_case(&format!("if ({}) {{", test));
                            (None, None, false)
                        };
                    self.indent();
                    if let Some(cap) = &i.capture {
                        if cap.sym != "_" {
                            self.write_to_case(&format!("_ = &{};", cap.sym));
                        }
                    }
                    self.current_scope_depth += 1;
                    self.narrowed_null_vars.extend(not_null_vars);
                    if let Stmt::Block(b) = &*i.cons {
                        for s in &b.body {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(&i.cons);
                    }
                    if let Some(cap) = &i.capture {
                        self.block_locals.remove(&cap.sym);
                        if i.is_ptr_capture && !was_mut_loop {
                            self.mut_borrow_loop_vars.remove(&cap.sym);
                        }
                        if let Some(prev) = saved_cap_local {
                            self.locals.insert(cap.sym.clone(), prev);
                        } else {
                            self.locals.remove(&cap.sym);
                        }
                        if let Some(prev_cp) = saved_cap_cp {
                            self.var_cp_types.insert(cap.sym.clone(), prev_cp);
                        } else {
                            self.var_cp_types.remove(&cap.sym);
                        }
                    }
                    self.narrowed_null_vars = old_narrowed.clone();
                    self.current_scope_depth -= 1;
                    self.dedent();
                    let uninit_cons = self.uninitialized_vars.clone();
                    self.uninitialized_vars = old_uninit.clone();

                    if let Some(alt) = &i.alt {
                        self.write_to_case("} else {");
                        self.indent();
                        self.current_scope_depth += 1;
                        self.narrowed_null_vars.extend(is_null_vars);
                        if let Stmt::Block(b) = &**alt {
                            for s in &b.body {
                                self.generate_stmt(s);
                            }
                        } else {
                            self.generate_stmt(alt);
                        }
                        self.narrowed_null_vars = old_narrowed;
                        self.current_scope_depth -= 1;
                        self.dedent();
                        let uninit_alt = self.uninitialized_vars.clone();
                        // Union of uninitialized variables
                        self.uninitialized_vars = uninit_cons;
                        self.uninitialized_vars.extend(uninit_alt);
                        self.write_to_case("}");
                    } else {
                        self.write_to_case("}");
                        // If there is no else, the var could remain uninitialized
                        let uninit_cons = self.uninitialized_vars.clone();
                        self.uninitialized_vars = old_uninit;
                        self.uninitialized_vars.extend(uninit_cons);
                    }
                }
            }
            Stmt::While(w) => {
                let not_null_vars = Self::extract_not_null_vars(&w.test);
                let test = self.generate_expr(&w.test);
                let cont_suffix = if let Some(c) = &w.continue_expr {
                    format!(" : ({})", self.generate_expr(c))
                } else {
                    String::new()
                };
                let old_uninit = self.uninitialized_vars.clone();
                let old_narrowed = self.narrowed_null_vars.clone();
                let lbl_prefix = if let Some(lbl) = &w.label {
                    format!("{}: ", lbl.sym)
                } else {
                    String::new()
                };
                let (saved_cap_local, saved_cap_cp, was_mut_loop) = if let Some(cap) = &w.capture {
                    self.var_scope_depth
                        .insert(cap.sym.clone(), self.current_scope_depth + 1);
                    self.block_locals.insert(cap.sym.clone());
                    let was_mut = if w.is_ptr_capture {
                        !self.mut_borrow_loop_vars.insert(cap.sym.clone())
                    } else {
                        false
                    };
                    let inline_prefix = if w.is_inline { "inline " } else { "" };
                    let ptr_prefix = if w.is_ptr_capture { "*" } else { "" };
                    self.write_to_case(&format!(
                        "{}{}while ({}) |{}{}|{} {{",
                        lbl_prefix, inline_prefix, test, ptr_prefix, cap.sym, cont_suffix
                    ));
                    let test_cp_ty = self.infer_cp_type(&w.test);
                    let unwrapped_ty = match &test_cp_ty {
                        Some(Type::Optional(inner)) => (**inner).clone(),
                        Some(Type::ErrorUnion(inner)) => (**inner).clone(),
                        Some(Type::ResultType { ok, .. }) => (**ok).clone(),
                        Some(other) => other.clone(),
                        None => Type::Any,
                    };
                    let final_ty = if w.is_ptr_capture {
                        Type::BorrowType {
                            is_mut: true,
                            inner: Box::new(unwrapped_ty.clone()),
                            lifetimes: vec![],
                        }
                    } else {
                        unwrapped_ty.clone()
                    };
                    let zig_ty = self.map_type(&final_ty);
                    let prev_cp = self.var_cp_types.insert(cap.sym.clone(), final_ty);
                    let prev_l = self.locals.insert(cap.sym.clone(), zig_ty);
                    (prev_l, prev_cp, was_mut)
                } else {
                    let inline_prefix = if w.is_inline { "inline " } else { "" };
                    self.write_to_case(&format!(
                        "{}{}while ({}){} {{",
                        lbl_prefix, inline_prefix, test, cont_suffix
                    ));
                    (None, None, false)
                };
                self.indent();
                if let Some(cap) = &w.capture {
                    if !self.last_use_map.contains_key(&cap.sym) {
                        self.write_to_case(&format!("_ = &{};", cap.sym));
                    }
                }
                self.current_scope_depth += 1;
                self.narrowed_null_vars.extend(not_null_vars);
                if let Stmt::Block(b) = &*w.body {
                    for s in &b.body {
                        self.generate_stmt(s);
                    }
                } else {
                    self.generate_stmt(&w.body);
                }
                if let Some(cap) = &w.capture {
                    self.block_locals.remove(&cap.sym);
                    if w.is_ptr_capture && !was_mut_loop {
                        self.mut_borrow_loop_vars.remove(&cap.sym);
                    }
                    if let Some(prev) = saved_cap_local {
                        self.locals.insert(cap.sym.clone(), prev);
                    } else {
                        self.locals.remove(&cap.sym);
                    }
                    if let Some(prev_cp) = saved_cap_cp {
                        self.var_cp_types.insert(cap.sym.clone(), prev_cp);
                    } else {
                        self.var_cp_types.remove(&cap.sym);
                    }
                }
                self.narrowed_null_vars = old_narrowed;
                self.current_scope_depth -= 1;
                self.dedent();
                self.write_to_case("}");
                // The loop might not execute, so variables could remain uninitialized
                let uninit_body = self.uninitialized_vars.clone();
                self.uninitialized_vars = old_uninit;
                self.uninitialized_vars.extend(uninit_body);
            }
            Stmt::For(f) => {
                let lbl_prefix = if let Some(lbl) = &f.label {
                    format!("{}: ", lbl.sym)
                } else {
                    String::new()
                };
                if let Some(raw) = &f.raw_header {
                    let pat_str = match &f.pat {
                        Pattern::Ident(id) => id.sym.clone(),
                        _ => "item".to_string(),
                    };
                    let inline_prefix = if f.is_comptime { "inline " } else { "" };
                    self.write_to_case(&format!(
                        "{}{}for ({}) |{}| {{",
                        lbl_prefix, inline_prefix, raw, pat_str
                    ));
                    self.indent();
                    if let Stmt::Block(b) = &*f.body {
                        for s in &b.body {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(&f.body);
                    }
                    self.dedent();
                    self.write_to_case("}");
                    return;
                }
                if f.is_comptime {
                    if let Expr::Range {
                        start,
                        end,
                        inclusive,
                        ..
                    } = &f.expr
                    {
                        let start_code = self.generate_expr(start);
                        let end_code = self.generate_expr(end);
                        let bound_id = match &f.pat {
                            Pattern::Ident(id) => id.sym.clone(),
                            _ => "i".to_string(),
                        };
                        let range_end = if *inclusive {
                            format!("({} + 1)", end_code)
                        } else {
                            end_code
                        };
                        self.write_to_case(&format!(
                            "inline for ({}..{}) |{}| {{",
                            start_code, range_end, bound_id
                        ));
                        self.indent();
                        if let Stmt::Block(b) = &*f.body {
                            for s in &b.body {
                                self.generate_stmt(s);
                            }
                        } else {
                            self.generate_stmt(&f.body);
                        }
                        self.dedent();
                        self.write_to_case("}");
                        return;
                    } else {
                        let coll_code = self.generate_expr(&f.expr);
                        let bound_id = match &f.pat {
                            Pattern::Ident(id) => id.sym.clone(),
                            _ => "item".to_string(),
                        };
                        self.write_to_case(&format!(
                            "inline for ({}) |{}| {{",
                            coll_code, bound_id
                        ));
                        self.indent();
                        if let Stmt::Block(b) = &*f.body {
                            for s in &b.body {
                                self.generate_stmt(s);
                            }
                        } else {
                            self.generate_stmt(&f.body);
                        }
                        self.dedent();
                        self.write_to_case("}");
                        return;
                    }
                }
                let tmp_id = self.tmp_counter;
                self.tmp_counter += 1;
                let old_uninit = self.uninitialized_vars.clone();

                if let Expr::Range {
                    start,
                    end,
                    inclusive,
                    ..
                } = &f.expr
                {
                    let start_code = self.generate_expr(start);
                    let end_code = self.generate_expr(end);
                    let op = if *inclusive { "<=" } else { "<" };

                    if let Pattern::Ident(id) = &f.pat {
                        self.var_scope_depth
                            .insert(id.sym.clone(), self.current_scope_depth);
                        self.var_cp_types.insert(id.sym.clone(), Type::I32);
                        self.write_to_case("{");
                        self.indent();
                        self.write_to_case(&format!("var {}: i32 = {};", id.sym, start_code));
                        self.write_to_case(&format!("_ = &{};", id.sym));
                        self.write_to_case(&format!("const __end_{} = {};", tmp_id, end_code));
                        self.write_to_case(&format!(
                            "while ({} {} __end_{}) : ({} += 1) {{",
                            id.sym, op, tmp_id, id.sym
                        ));
                    } else {
                        panic!("CP Compile Error: Range loop only supports identifier pattern");
                    }

                    self.indent();
                    self.current_scope_depth += 1;
                    if let Stmt::Block(b) = &*f.body {
                        for s in &b.body {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(&f.body);
                    }
                    self.current_scope_depth -= 1;
                    self.dedent();
                    self.write_to_case("}");
                    self.dedent();
                    self.write_to_case("}");
                    let uninit_body = self.uninitialized_vars.clone();
                    self.uninitialized_vars = old_uninit;
                    self.uninitialized_vars.extend(uninit_body);
                    return;
                }

                match f.mode {
                    IterMode::MutBorrow => {
                        self.check_mutation_allowed(&f.expr, "mutable 'for *' iteration");
                    }
                    IterMode::Borrow => {
                        self.check_read_allowed(&f.expr);
                    }
                    IterMode::Move => {
                        if let Expr::Ident(coll_id) = &f.expr {
                            self.check_active_borrows_before_move(&coll_id.sym);
                        }
                    }
                }
                let expr_code = self.generate_expr(&f.expr);
                let old_mut_loop_vars = self.mut_borrow_loop_vars.clone();
                let old_loop_borrowed_containers = self.loop_borrowed_containers.clone();
                if f.mode != IterMode::Move {
                    if let Expr::Ident(coll_id) = &f.expr {
                        self.loop_borrowed_containers.insert(coll_id.sym.clone());
                        if let Some((srcs, _)) = self.var_borrow_sources.get(&coll_id.sym) {
                            for (s, _) in srcs.clone() {
                                self.loop_borrowed_containers.insert(s);
                            }
                        }
                    }
                }

                // Iterator Stream Path (if target type implements a `next()` method)
                let expr_ty = self.infer_cp_type(&f.expr);
                let clean_type_name = expr_ty.as_ref().and_then(|t| match t {
                    Type::Ref(id) => Some(id.sym.clone()),
                    Type::BorrowType { inner, .. } => match &**inner {
                        Type::Ref(id) => Some(id.sym.clone()),
                        _ => None,
                    },
                    _ => None,
                });
                let is_builtin_container_stream = matches!(
                    clean_type_name.as_deref(),
                    Some("VecDeque" | "BinaryHeap" | "BTreeMap" | "BTreeSet")
                );
                let has_next_method = clean_type_name
                    .as_ref()
                    .and_then(|c_name| self.lookup_method(c_name, "next"));

                if has_next_method.is_some() || is_builtin_container_stream {
                    let ret_ty_opt = has_next_method.and_then(|(_, r, _)| r);
                    let iter_var = if matches!(f.expr, Expr::Ident(_)) {
                        expr_code.clone()
                    } else {
                        let tmp_name = format!("__iter_{}", tmp_id);
                        self.write_to_case(&format!("var {} = {};", tmp_name, expr_code));
                        tmp_name
                    };
                    let item_ty = match ret_ty_opt {
                        Some(Type::Optional(inner)) => *inner,
                        Some(other) => other,
                        None => Type::Any,
                    };

                    if let Pattern::Ident(id) = &f.pat {
                        self.var_scope_depth
                            .insert(id.sym.clone(), self.current_scope_depth);
                        self.var_cp_types.insert(id.sym.clone(), item_ty);
                        self.write_to_case(&format!("while ({}.next()) |{}| {{", iter_var, id.sym));
                        self.indent();
                        self.current_scope_depth += 1;
                        self.write_to_case(&format!("_ = &{};", id.sym));
                        if let Stmt::Block(b) = &*f.body {
                            for s in &b.body {
                                self.generate_stmt(s);
                            }
                        } else {
                            self.generate_stmt(&f.body);
                        }
                        self.current_scope_depth -= 1;
                        self.dedent();
                        self.write_to_case("}");
                        let uninit_body = self.uninitialized_vars.clone();
                        self.uninitialized_vars = old_uninit;
                        self.uninitialized_vars.extend(uninit_body);
                        return;
                    } else {
                        let item_var = format!("__item_{}", tmp_id);
                        self.write_to_case(&format!(
                            "while ({}.next()) |{}| {{",
                            iter_var, item_var
                        ));
                        self.indent();
                        self.current_scope_depth += 1;
                        match &f.pat {
                            Pattern::Tuple(elems) => {
                                let rest_pos =
                                    elems.iter().position(|p| matches!(p, Pattern::Rest(_)));
                                for (idx, p) in elems.iter().enumerate() {
                                    if let Pattern::Ident(id) = p {
                                        self.var_scope_depth
                                            .insert(id.sym.clone(), self.current_scope_depth);
                                        let val_expr = if let Some(r_idx) = rest_pos {
                                            if idx > r_idx {
                                                let offset_from_end = elems.len() - idx;
                                                format!(
                                                    "cp_unpack_elem_from_end({}, {})",
                                                    item_var, offset_from_end
                                                )
                                            } else if idx < r_idx {
                                                format!("cp_unpack_elem({}, {})", item_var, idx)
                                            } else {
                                                let left_count = r_idx;
                                                let right_count = elems.len() - 1 - r_idx;
                                                format!(
                                                    "cp_unpack_rest({}, {}, {})",
                                                    item_var, left_count, right_count
                                                )
                                            }
                                        } else {
                                            format!("cp_unpack_elem({}, {})", item_var, idx)
                                        };
                                        self.write_to_case(&format!(
                                            "const {} = {};",
                                            id.sym, val_expr
                                        ));
                                        self.write_to_case(&format!("_ = &{};", id.sym));
                                    }
                                }
                            }
                            _ => {}
                        }
                        if let Stmt::Block(b) = &*f.body {
                            for s in &b.body {
                                self.generate_stmt(s);
                            }
                        } else {
                            self.generate_stmt(&f.body);
                        }
                        self.current_scope_depth -= 1;
                        self.dedent();
                        self.write_to_case("}");
                        let uninit_body = self.uninitialized_vars.clone();
                        self.uninitialized_vars = old_uninit;
                        self.uninitialized_vars.extend(uninit_body);
                        return;
                    }
                }

                let is_native_slice = f.is_comptime
                    || expr_code.ends_with(".items")
                    || expr_code.ends_with(".values()");
                let (for_target, ptr_star) = match f.mode {
                    IterMode::MutBorrow => {
                        if is_native_slice {
                            (expr_code.clone(), "*")
                        } else {
                            (format!("cp_mut_slice(&{})", expr_code), "*")
                        }
                    }
                    IterMode::Borrow => {
                        if is_native_slice {
                            (expr_code.clone(), "")
                        } else {
                            (format!("cp_iter_slice({})", expr_code), "")
                        }
                    }
                    IterMode::Move => {
                        if let Expr::Ident(coll_id) = &f.expr {
                            self.moved_vars.insert(coll_id.sym.clone());
                            if !self.is_state_machine && self.drop_flag_vars.contains(&coll_id.sym)
                            {
                                self.write_to_case(&format!("__{}_moved = true;", coll_id.sym));
                            }
                        }
                        if is_native_slice {
                            (expr_code.clone(), "")
                        } else {
                            (format!("cp_iter_slice({})", expr_code), "")
                        }
                    }
                };

                let inline_prefix = if f.is_comptime { "inline " } else { "" };

                if let Pattern::Ident(id) = &f.pat {
                    self.var_scope_depth
                        .insert(id.sym.clone(), self.current_scope_depth);
                    if f.mode == IterMode::MutBorrow {
                        self.mut_borrow_loop_vars.insert(id.sym.clone());
                    }
                    if let Some(ty) = self.infer_cp_type(&f.expr) {
                        let base_ty = match ty {
                            Type::BorrowType { inner, .. } => *inner,
                            other => other,
                        };
                        if let Type::Array(inner) = base_ty {
                            self.var_cp_types.insert(id.sym.clone(), *inner);
                        }
                    }
                    self.write_to_case(&format!(
                        "{}for ({}) |{}{}| {{",
                        inline_prefix, for_target, ptr_star, id.sym
                    ));
                    self.indent();
                    self.current_scope_depth += 1;
                    self.write_to_case(&format!("_ = &{};", id.sym));
                    if f.mode == IterMode::Move {
                        self.write_to_case(&format!(
                            "var __{0}_moved = false; _ = &__{0}_moved;",
                            id.sym
                        ));
                        self.write_to_case(&format!("defer if (!__{0}_moved) {{ const __T = @TypeOf({0}); if (@typeInfo(__T) == .pointer) {{ const Child = @typeInfo(__T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) {{ const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = {0}.release() catch {{}}; }} else {{ {0}.release(); }} }} }} else if (@typeInfo(__T) == .@\"struct\" and @hasDecl(__T, \"release\")) {{ var _drop_val = {0}; _drop_val.release(); }} }};", id.sym));
                    }
                } else {
                    let item_var = format!("__item_{}", tmp_id);
                    self.write_to_case(&format!(
                        "{}for ({}) |{}{}| {{",
                        inline_prefix, for_target, ptr_star, item_var
                    ));
                    self.indent();
                    self.current_scope_depth += 1;

                    match &f.pat {
                        Pattern::Tuple(elems) => {
                            let rest_pos = elems.iter().position(|p| matches!(p, Pattern::Rest(_)));
                            for (idx, p) in elems.iter().enumerate() {
                                match p {
                                    Pattern::Ident(id) => {
                                        self.var_scope_depth
                                            .insert(id.sym.clone(), self.current_scope_depth);
                                        if f.mode == IterMode::MutBorrow {
                                            self.mut_borrow_loop_vars.insert(id.sym.clone());
                                        }
                                        let val_expr = if let Some(r_idx) = rest_pos {
                                            if idx > r_idx {
                                                let offset_from_end = elems.len() - idx;
                                                format!(
                                                    "cp_unpack_tail({}, {})",
                                                    item_var, offset_from_end
                                                )
                                            } else {
                                                format!("(if (comptime (@typeInfo(@TypeOf({0})) == .pointer and @hasField(@typeInfo(@TypeOf({0})).pointer.child, \"key\")) or (!(@typeInfo(@TypeOf({0})) == .pointer) and @hasField(@TypeOf({0}), \"key\"))) (if ({1} == 0) {0}.key else {0}.value) else cp_unpack_elem({0}, {1}))", item_var, idx)
                                            }
                                        } else {
                                            format!("(if (comptime (@typeInfo(@TypeOf({0})) == .pointer and @hasField(@typeInfo(@TypeOf({0})).pointer.child, \"key\")) or (!(@typeInfo(@TypeOf({0})) == .pointer) and @hasField(@TypeOf({0}), \"key\"))) (if ({1} == 0) {0}.key else {0}.value) else cp_unpack_elem({0}, {1}))", item_var, idx)
                                        };
                                        self.write_to_case(&format!(
                                            "const {} = {}; _ = &{};",
                                            id.sym, val_expr, id.sym
                                        ));
                                    }
                                    Pattern::Rest(inner) => {
                                        if let Pattern::Ident(id) = &**inner {
                                            self.var_scope_depth
                                                .insert(id.sym.clone(), self.current_scope_depth);
                                            let suffix_len = elems.len() - 1 - idx;
                                            let val_expr = format!(
                                                "cp_unpack_slice({}, {}, {})",
                                                item_var, idx, suffix_len
                                            );
                                            self.write_to_case(&format!(
                                                "const {} = {}; _ = &{};",
                                                id.sym, val_expr, id.sym
                                            ));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Pattern::Object(props) => {
                            let non_rest_keys: HashSet<String> = props
                                .iter()
                                .filter(|p| !p.is_rest)
                                .map(|p| p.key.sym.clone())
                                .collect();

                            for prop in props {
                                if prop.is_rest {
                                    if prop.key.sym == "*"
                                        || matches!(prop.val, Some(Pattern::Wildcard))
                                    {
                                        continue;
                                    }
                                    let bound_name = prop.key.sym.clone();
                                    self.var_scope_depth
                                        .insert(bound_name.clone(), self.current_scope_depth);
                                    let coll_ty = self.infer_cp_type(&f.expr);
                                    let remaining_fields = match coll_ty {
                                        Some(Type::Array(inner)) => match *inner {
                                            Type::Object(fields) => fields
                                                .iter()
                                                .filter(|f| !non_rest_keys.contains(&f.ident.sym))
                                                .cloned()
                                                .collect::<Vec<_>>(),
                                            _ => vec![],
                                        },
                                        _ => vec![],
                                    };
                                    let rhs = if remaining_fields.is_empty() {
                                        ".{}".to_string()
                                    } else {
                                        let inits: Vec<String> = remaining_fields
                                            .iter()
                                            .map(|f| {
                                                format!(
                                                    ".{} = {}.{}",
                                                    f.ident.sym, item_var, f.ident.sym
                                                )
                                            })
                                            .collect();
                                        format!(".{{ {} }}", inits.join(", "))
                                    };
                                    self.write_to_case(&format!(
                                        "const {} = {}; _ = &{};",
                                        bound_name, rhs, bound_name
                                    ));
                                    continue;
                                }

                                if matches!(prop.val, Some(Pattern::Wildcard)) {
                                    continue;
                                }
                                let bound_name = if let Some(Pattern::Ident(id)) = &prop.val {
                                    id.sym.clone()
                                } else {
                                    prop.key.sym.clone()
                                };
                                self.var_scope_depth
                                    .insert(bound_name.clone(), self.current_scope_depth);
                                if f.mode == IterMode::MutBorrow {
                                    self.mut_borrow_loop_vars.insert(bound_name.clone());
                                }
                                self.write_to_case(&format!(
                                    "const {} = {}.{}; _ = &{};",
                                    bound_name, item_var, prop.key.sym, bound_name
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                if let Stmt::Block(b) = &*f.body {
                    for s in &b.body {
                        self.generate_stmt(s);
                    }
                } else {
                    self.generate_stmt(&f.body);
                }

                self.mut_borrow_loop_vars = old_mut_loop_vars;
                self.loop_borrowed_containers = old_loop_borrowed_containers;
                self.current_scope_depth -= 1;
                self.dedent();
                self.write_to_case("}");
                let uninit_body = self.uninitialized_vars.clone();
                self.uninitialized_vars = old_uninit;
                self.uninitialized_vars.extend(uninit_body);
            }
            Stmt::Return(ret) => {
                if let Some(blk_lbl) = self.inline_closure_blk.clone() {
                    if let Some(arg) = &ret.arg {
                        let e = self.generate_expr(arg);
                        self.write_to_case(&format!("break :{} {};", blk_lbl, e));
                    } else {
                        self.write_to_case(&format!("break :{} {{}};", blk_lbl));
                    }
                    return;
                }
                if let Some(arg) = &ret.arg {
                    let is_null_ret = matches!(arg, Expr::Lit(Lit::Null))
                        || matches!(arg, Expr::Ident(id) if id.sym == "null");
                    if is_null_ret {
                        let allows_null = match &self.current_fn_ret_ty {
                            Some(Type::Optional(_)) => true,
                            Some(Type::ResultType { ok, .. }) => matches!(&**ok, Type::Optional(_)),
                            Some(Type::ErrorUnion(inner)) => matches!(&**inner, Type::Optional(_)),
                            Some(Type::Ref(id))
                                if id.sym.contains('?') || id.sym.starts_with("switch") =>
                            {
                                true
                            }
                            None => true,
                            _ => false,
                        };
                        if !allows_null {
                            panic!(
                                "CP Compile Error: Cannot return 'null' from function '{}' whose return type is not optional '?T' (actual ret ty: {:?})",
                                self.current_fn_zig_name, self.current_fn_ret_ty
                            );
                        }
                    }

                    // Check for dangling local reference return or ref struct escape!
                    let (borrow_sources, _, is_ref_st) = self.extract_expr_borrow_info(arg);
                    for (src, _) in &borrow_sources {
                        if self.local_trampolines.contains(src)
                            || self.globals.contains(src)
                            || self.global_var_types.contains_key(src)
                        {
                            continue;
                        }
                        if !self.current_fn_params.contains(src) {
                            if is_ref_st {
                                panic!(
                                    "CP Compile Error: Cannot return stack-bound ref struct borrowing local variable '{}'",
                                    src
                                );
                            } else {
                                panic!(
                                    "CP Compile Error: Cannot return reference to local variable '{}'",
                                    src
                                );
                            }
                        }
                        if let Some(Type::BorrowType { lifetimes, .. }) = &self.current_fn_ret_ty {
                            if !lifetimes.is_empty() {
                                let matches_lt = if src == "this" || src == "_" {
                                    lifetimes.iter().any(|l| l == "this" || l == "_")
                                } else {
                                    lifetimes.contains(src)
                                };
                                if !matches_lt {
                                    panic!(
                                        "CP Compile Error: Returned reference borrows from '{}', which does not match lifetime annotation '&{}'",
                                        src,
                                        lifetimes.join(" | ")
                                    );
                                }
                            }
                        }
                    }
                    if is_ref_st && borrow_sources.is_empty() {
                        panic!("CP Compile Error: Stack-bound ref struct cannot escape function via return");
                    }
                    // If returning an owned local variable or move receiver/param, mark it as moved so defer doesn't release it before caller uses it!
                    let ret_ident_opt = match arg {
                        Expr::Ident(ret_id) => Some(ret_id),
                        Expr::ErrWrap(inner) => {
                            if let Expr::Ident(ret_id) = &**inner {
                                Some(ret_id)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(ret_id) = ret_ident_opt {
                        if (ret_id.sym == "this" || ret_id.sym == "_")
                            && self.current_receiver_mode == ReceiverMode::Move
                        {
                            self.check_active_borrows_before_move("this");
                            self.check_active_borrows_before_move("_");
                            if !self.is_state_machine {
                                self.write_to_case("__self_moved = true;");
                            }
                        } else if self.move_fn_params.contains(&ret_id.sym) {
                            self.check_active_borrows_before_move(&ret_id.sym);
                            if !self.is_state_machine {
                                self.write_to_case(&format!("__{}_moved = true;", ret_id.sym));
                            }
                        } else {
                            let is_borrow = self.var_borrow_sources.contains_key(&ret_id.sym);
                            if !is_borrow && !self.current_fn_params.contains(&ret_id.sym) {
                                self.check_active_borrows_before_move(&ret_id.sym);
                                if !self.is_state_machine && self.locals.contains_key(&ret_id.sym) {
                                    if self.drop_flag_vars.contains(&ret_id.sym) {
                                        self.write_to_case(&format!(
                                            "__{}_moved = true;",
                                            ret_id.sym
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    if matches!(self.current_fn_ret_ty, Some(Type::ErrorUnion(_))) {
                        if let Expr::ErrWrap(_) = arg {
                            panic!("CP Compile Error: Function return type is 'T!' (AnyErr), which only allows static error tags (e.g. err.xxx or FileErr.xxx). When returning dynamic Err(code, msg), explicitly declare return type as 'T ! Err' (or 'T ! err').");
                        }
                    }

                    if let Some(Type::ResultType {
                        ok: fn_ok,
                        err: fn_err,
                    }) = self.current_fn_ret_ty.clone()
                    {
                        let err_str = self.map_error_type(&fn_err);
                        let is_native_err_union = self.is_lib_mode
                            || err_str.contains('.')
                            || err_str == "anyerror"
                            || self.error_sets.contains_key(&err_str)
                            || err_str.starts_with('(');
                        if is_native_err_union {
                            if let Expr::ErrWrap(err_inner) = arg {
                                if err_str == "anyerror" {
                                    panic!("CP Compile Error: Function return type is 'T!' (AnyErr), which only allows static error tags (e.g. err.xxx or FileErr.xxx). When returning dynamic Err(code, msg), explicitly declare return type as 'T ! Err' (or 'T ! err').");
                                }
                                let err_val = self.generate_err_value(err_inner, Some(&fn_err));
                                self.write_to_case(&format!("return {};", err_val));
                                return;
                            }
                            if let Expr::Member(mem) = arg {
                                if let Expr::Ident(obj_id) = &*mem.obj {
                                    if self.error_sets.contains_key(&obj_id.sym) {
                                        self.write_to_case(&format!(
                                            "return error.{};",
                                            mem.prop.sym
                                        ));
                                        return;
                                    }
                                }
                            }
                            if let Expr::EnumInit(ei) = arg {
                                if ei.enum_name.sym.is_empty() {
                                    self.write_to_case(&format!(
                                        "return error.{};",
                                        ei.variant_name.sym
                                    ));
                                    return;
                                }
                            }
                        } else {
                            if let Expr::ErrWrap(err_inner) = arg {
                                let err_val = self.generate_err_value(err_inner, Some(&fn_err));
                                self.write_to_case(&format!("return .{{ .err = {} }};", err_val));
                                return;
                            } else if let Expr::Member(mem) = arg {
                                if let Expr::Ident(obj_id) = &*mem.obj {
                                    if self.error_sets.contains_key(&obj_id.sym) {
                                        let target_err = self.map_error_type(&fn_err);
                                        self.write_to_case(&format!(
                                            "return .{{ .err = cp_convert_err({}, error.{}) }};",
                                            target_err, mem.prop.sym
                                        ));
                                        return;
                                    }
                                }
                            } else if let Expr::EnumInit(ei) = arg {
                                if ei.enum_name.sym.is_empty() {
                                    let target_err = self.map_error_type(&fn_err);
                                    self.write_to_case(&format!(
                                        "return .{{ .err = cp_convert_err({}, error.{}) }};",
                                        target_err, ei.variant_name.sym
                                    ));
                                    return;
                                }
                            } else if is_null_ret {
                                self.write_to_case("return .{ .ok = null };");
                                return;
                            } else {
                                let old_box = self.expected_box_trait.clone();
                                if let Type::BoxDyn(tr_id) = &*fn_ok {
                                    self.expected_box_trait = Some(tr_id.sym.clone());
                                } else if let Type::Optional(opt_in) = &*fn_ok {
                                    if let Type::BoxDyn(tr_id) = &**opt_in {
                                        self.expected_box_trait = Some(tr_id.sym.clone());
                                    }
                                }
                                let e = self.generate_expr(arg);
                                self.expected_box_trait = old_box;
                                self.write_to_case(&format!("return .{{ .ok = {} }};", e));
                                return;
                            }
                        }
                    }

                    let old_box = self.expected_box_trait.clone();
                    if let Some(Type::BoxDyn(tr_id)) = &self.current_fn_ret_ty {
                        self.expected_box_trait = Some(tr_id.sym.clone());
                    } else if let Some(Type::Optional(opt_in)) = &self.current_fn_ret_ty {
                        if let Type::BoxDyn(tr_id) = &**opt_in {
                            self.expected_box_trait = Some(tr_id.sym.clone());
                        }
                    }
                    let e = self.generate_expr(arg);
                    self.expected_box_trait = old_box;

                    if self.current_fn_is_async {
                        if self.is_beam_mode {
                            self.write_to_case(&format!("return {};", e));
                        } else {
                            self.write_to_case(&format!("if (ctx._parent) |p| {{ const base: *BaseContext = @ptrCast(@alignCast(p)); base.await_result = cp_message_from({}); if (ctx._parent_fn) |pfn| {{ schedule(pfn, p); }} }} _ = {}; return;", e, e));
                        }
                    } else {
                        let ret_val = if e.starts_with("lambda_") {
                            format!("@ptrCast(@constCast(&{}))", e)
                        } else {
                            e
                        };
                        self.write_to_case(&format!("return {};", ret_val));
                    }
                } else {
                    if let Some(Type::ResultType { ok, err }) = &self.current_fn_ret_ty {
                        let err_str = self.map_error_type(err);
                        let is_native_err_union =
                            self.is_lib_mode || err_str.contains('.') || err_str == "anyerror";
                        if !is_native_err_union && **ok == Type::Void {
                            self.write_to_case("return .{ .ok = {} };");
                            return;
                        }
                    }
                    if self.is_main {
                        self.write_to_case("return;");
                    } else if self.is_beam_mode {
                        self.write_to_case(r#"return beam.Beam.makeAtom(ctx.self.env.?, "ok");"#);
                    } else if self.current_fn_is_async {
                        self.write_to_case(
                            "if (ctx._parent_fn) |pfn| { schedule(pfn, ctx._parent.?); } return;",
                        );
                    } else {
                        self.write_to_case("return;");
                    }
                }
            }
            Stmt::Break => {
                self.write_to_case("break;");
            }
            Stmt::Continue => {
                self.write_to_case("continue;");
            }
            Stmt::Match(m) => {
                let sw_code = self.generate_switch(m, false);
                self.write_to_case(&sw_code);
            }
            Stmt::TryCatch {
                try_block,
                err_ident,
                catch_body,
                ..
            } => {
                if let Some(cb) = catch_body {
                    let mut err_tys = Vec::new();
                    for s in &try_block.body {
                        self.collect_stmt_error_types(s, &mut err_tys);
                    }
                    let try_err_ty = err_tys
                        .into_iter()
                        .next()
                        .unwrap_or(ErrorType::DefaultError);
                    let err_zig_ty = self.map_error_type(&try_err_ty);
                    let tc_id = self.tmp_counter;
                    self.tmp_counter += 1;
                    let blk_label = format!("__try_blk_{}", tc_id);
                    let err_var = format!("__try_err_{}", tc_id);
                    let flag_var = format!("__try_failed_{}", tc_id);

                    self.write_to_case(&format!(
                        "var {}: {} = undefined; _ = &{};",
                        err_var, err_zig_ty, err_var
                    ));
                    self.write_to_case(&format!(
                        "var {}: bool = false; _ = &{};",
                        flag_var, flag_var
                    ));
                    self.write_to_case(&format!("{}: {{", blk_label));
                    self.indent();
                    let old_in_try = self.in_try_region;
                    let old_target = self.try_catch_target.clone();
                    self.in_try_region = true;
                    self.try_catch_target = Some((
                        blk_label.clone(),
                        err_var.clone(),
                        flag_var.clone(),
                        try_err_ty.clone(),
                    ));
                    for s in &try_block.body {
                        self.generate_stmt(s);
                    }
                    self.in_try_region = old_in_try;
                    self.try_catch_target = old_target;
                    self.dedent();
                    self.write_to_case("}");

                    self.write_to_case(&format!("if ({}) {{", flag_var));
                    self.indent();
                    let old_vars = self.var_cp_types.clone();
                    if let Some(eid) = err_ident {
                        self.write_to_case(&format!(
                            "const {0} = {1}; _ = &{0};",
                            eid.sym, err_var
                        ));
                        self.var_cp_types
                            .insert(eid.sym.clone(), self.error_type_to_cp_type(&try_err_ty));
                    }
                    if let Stmt::Block(b) = &**cb {
                        for s in &b.body {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(cb);
                    }
                    self.var_cp_types = old_vars;
                    self.dedent();
                    self.write_to_case("}");
                } else {
                    let old_in_try = self.in_try_region;
                    self.in_try_region = true;
                    for s in &try_block.body {
                        self.generate_stmt(s);
                    }
                    self.in_try_region = old_in_try;
                }
            }
            Stmt::Func(f) => {
                let trampoline_name = f.ident.sym.clone();
                self.local_trampolines.insert(trampoline_name.clone());
                let struct_name = format!("__cp_trampoline_{}", trampoline_name);
                self.write_to_case(&format!("const {} = struct {{", struct_name));
                self.indent();

                let mut params_str = String::new();
                for p in &f.params {
                    let mapped_ty = match &p.ty {
                        Type::Ref(id) if self.trait_defs.contains_key(&id.sym) => {
                            "anytype".to_string()
                        }
                        Type::CStr => {
                            if p.is_mut {
                                "[*:0]u8".to_string()
                            } else {
                                "[*:0]const u8".to_string()
                            }
                        }
                        _ => self.map_type(&p.ty),
                    };
                    self.locals.insert(p.ident.sym.clone(), mapped_ty.clone());
                    if !params_str.is_empty() {
                        params_str.push_str(", ");
                    }
                    let comptime_prefix = if p.is_comptime { "comptime " } else { "" };
                    params_str.push_str(&format!(
                        "{}{}: {}",
                        comptime_prefix, p.ident.sym, mapped_ty
                    ));
                }

                let is_inline = f.meta.iter().any(|m| m.name.sym == "inline");
                let inline_kw = if is_inline { "inline " } else { "" };
                let is_c = f
                    .meta
                    .iter()
                    .any(|m| m.name.sym == "c" || m.name.sym == "extern");
                let callconv_clause = if is_c {
                    "callconv(.c) ".to_string()
                } else if let Some(m) = f.meta.iter().find(|m| m.name.sym == "callconv") {
                    let conv = match m.args.first() {
                        Some(MetaArg::Positional(Expr::Lit(Lit::Str(s)))) => {
                            if s.starts_with('.') {
                                s.clone()
                            } else {
                                format!(".{}", s)
                            }
                        }
                        Some(MetaArg::Positional(Expr::Ident(id))) => {
                            if id.sym.starts_with('.') {
                                id.sym.clone()
                            } else {
                                format!(".{}", id.sym)
                            }
                        }
                        _ => ".c".to_string(),
                    };
                    format!("callconv({}) ", conv)
                } else {
                    String::new()
                };

                let zig_ret_ty = f
                    .return_type
                    .as_ref()
                    .map(|t| self.map_type(t))
                    .unwrap_or_else(|| "void".to_string());

                self.write_to_case(&format!(
                    "pub {}fn {}({}) {}{} {{",
                    inline_kw, f.ident.sym, params_str, callconv_clause, zig_ret_ty
                ));
                self.indent();

                for p in &f.params {
                    let is_used = Self::block_contains_ident(&f.body, &p.ident.sym);
                    if !p.is_comptime && !is_used {
                        self.write_to_case(&format!("_ = {};", p.ident.sym));
                    }
                }

                let saved_sm = self.is_state_machine;
                self.is_state_machine = false;
                for s in &f.body.body {
                    self.generate_stmt(s);
                }
                self.is_state_machine = saved_sm;

                self.dedent();
                self.write_to_case("}");
                self.dedent();
                self.write_to_case("};");
            }
            _ => {}
        }
    }
}
