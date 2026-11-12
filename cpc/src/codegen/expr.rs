use super::*;
use crate::ast::*;
use crate::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

impl Codegen {
    pub(crate) fn maybe_wrap_try_call(
        &mut self,
        call_code: String,
        raw_ret_ty: Option<&Type>,
    ) -> String {
        if !self.in_try_region {
            return call_code;
        }
        if let Some(Type::ResultType { err: call_err, .. }) = raw_ret_ty {
            let id = self.lambda_counter;
            self.lambda_counter += 1;
            if let Some((blk_label, err_var, flag_var, target_err)) = self.try_catch_target.clone()
            {
                self.verify_error_type_compatibility(call_err, &target_err);
                let tgt_zig = self.map_error_type(&target_err);
                return format!(
                    "(blk_try_{id}: {{ const __r_{id} = {call_code}; switch (__r_{id}) {{ .err => |__e_{id}| {{ {err_var} = cp_convert_err({tgt_zig}, __e_{id}); {flag_var} = true; break :{blk_label}; }}, .ok => |__v_{id}| break :blk_try_{id} __v_{id} }} }})"
                );
            } else if let Some(Type::ResultType { err: fn_err, .. }) =
                self.current_fn_ret_ty.clone()
            {
                self.verify_error_type_compatibility(call_err, &fn_err);
                let tgt_zig = self.map_error_type(&fn_err);
                return format!(
                    "(blk_try_{id}: {{ const __r_{id} = {call_code}; switch (__r_{id}) {{ .err => |__e_{id}| return .{{ .err = cp_convert_err({tgt_zig}, __e_{id}) }}, .ok => |__v_{id}| break :blk_try_{id} __v_{id} }} }})"
                );
            } else {
                panic!("CP Compile Error: Cannot auto-bubble error in 'try' block inside a function without '!Error' return type or 'catch' handler");
            }
        }
        call_code
    }

    pub(crate) fn build_ret_stmt_for_fallback(&mut self, fallback_expr: Option<&Expr>) -> String {
        let is_null_fb = match fallback_expr {
            Some(Expr::Lit(Lit::Null)) => true,
            Some(Expr::Ident(id)) if id.sym == "null" => true,
            None => false,
            _ => false,
        };

        if let Some(Type::ResultType { ok, err: fn_err }) = self.current_fn_ret_ty.clone() {
            if let Some(Expr::ErrWrap(err_inner)) = fallback_expr {
                let ev = self.generate_err_value(err_inner, Some(&fn_err));
                format!("return .{{ .err = {} }};", ev)
            } else if is_null_fb || (fallback_expr.is_none() && matches!(&*ok, Type::Optional(_))) {
                if !matches!(&*ok, Type::Optional(_)) {
                    panic!("CP Compile Error: Cannot return 'null' from function '{}' whose return type is not optional '?T' (actual ret ty: {:?})", self.current_fn_zig_name, self.current_fn_ret_ty);
                }
                "return .{ .ok = null };".to_string()
            } else if let Some(fb) = fallback_expr {
                let fb_c = self.generate_expr(fb);
                format!("return .{{ .ok = {} }};", fb_c)
            } else if matches!(&*ok, Type::Void) {
                "return .{ .ok = {} };".to_string()
            } else {
                "return;".to_string()
            }
        } else {
            let allows_null = match &self.current_fn_ret_ty {
                Some(Type::Optional(_)) => true,
                Some(Type::ErrorUnion(inner)) => matches!(&**inner, Type::Optional(_)),
                _ => false,
            };
            if is_null_fb && !allows_null {
                panic!("CP Compile Error: Cannot return 'null' from function '{}' whose return type is not optional '?T' (actual ret ty: {:?})", self.current_fn_zig_name, self.current_fn_ret_ty);
            }
            if let Some(fb) = fallback_expr {
                let fb_c = self.generate_expr(fb);
                format!("return {};", fb_c)
            } else {
                "return;".to_string()
            }
        }
    }

    pub(crate) fn generate_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(Lit::Int(v)) => v.to_string(),
            Expr::Lit(Lit::Float(f)) => {
                let s = f.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Expr::Lit(Lit::Str(s)) => format!("\"{}\"", Self::escape_zig_string(s)),
            Expr::Lit(Lit::Bool(b)) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Expr::Lit(Lit::Null) => "null".to_string(),
            Expr::Lit(Lit::Undefined) => "undefined".to_string(),
            Expr::Ident(i) => {
                if self.moved_vars.contains(&i.sym)
                    || ((i.sym == "this" || i.sym == "_")
                        && (self.moved_vars.contains("this") || self.moved_vars.contains("_")))
                {
                    panic!("CP Compile Error: Use of moved value '{}'", i.sym);
                }
                if !self.is_lib_mode && self.uninitialized_vars.contains(&i.sym) {
                    panic!(
                        "CP Compile Error: Variable '{}' read before initialization",
                        i.sym
                    );
                }
                if i.sym == "@is_beam" {
                    return if self.is_beam_mode {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    };
                }
                if i.sym == "@self" {
                    return if self.is_state_machine {
                        "ctx._self".to_string()
                    } else {
                        "PID{ .node_id = current_node_id, .local_id = 1 }".to_string()
                    };
                }
                if i.sym == "@sender" {
                    return if self.is_state_machine {
                        "ctx.received_sender.?".to_string()
                    } else {
                        "PID{ .node_id = 0, .local_id = 0 }".to_string()
                    };
                }
                if i.sym == "this" || i.sym == "_" {
                    if self.is_state_machine {
                        "ctx.self".to_string()
                    } else if self.is_class_method {
                        "self".to_string()
                    } else if self.locals.contains_key(&i.sym) {
                        i.sym.clone()
                    } else {
                        "self".to_string()
                    }
                } else if i.sym == "true"
                    || i.sym == "false"
                    || i.sym == "null"
                    || i.sym.starts_with('@')
                    || i.sym.starts_with('[')
                    || i.sym.starts_with("__pipe_")
                    || i.sym.ends_with("_Context")
                    || self.is_comptime
                    || self.globals.contains(&i.sym)
                {
                    i.sym.clone()
                } else if self.locals.contains_key(&i.sym) {
                    let base = if self.is_state_machine
                        && !self.inline_closure_params.contains(&i.sym)
                        && !self.block_locals.contains(&i.sym)
                    {
                        format!("ctx.{}", i.sym)
                    } else {
                        i.sym.clone()
                    };
                    if self.mut_borrow_loop_vars.contains(&i.sym) {
                        format!("({}.*)", base)
                    } else if self.narrowed_null_vars.contains(&i.sym) {
                        format!("({}.?)", base)
                    } else {
                        base
                    }
                } else if self.is_class_method && self.is_class_field(&i.sym) {
                    if self.moved_vars.contains("this") || self.moved_vars.contains("_") {
                        panic!(
                            "CP Compile Error: Use of moved value '_' when accessing field '{}'",
                            i.sym
                        );
                    }
                    let base = if self.is_state_machine {
                        format!("ctx.self.{}", i.sym)
                    } else {
                        format!("self.{}", i.sym)
                    };
                    if self.narrowed_null_vars.contains(&i.sym) {
                        format!("({}.?)", base)
                    } else {
                        base
                    }
                } else if self.local_trampolines.contains(&i.sym) {
                    format!("__cp_trampoline_{}.{}", i.sym, i.sym)
                } else if self.mut_borrow_loop_vars.contains(&i.sym) {
                    format!("({}.*)", i.sym)
                } else if self.narrowed_null_vars.contains(&i.sym) {
                    format!("({}.?)", i.sym)
                } else {
                    i.sym.clone()
                }
            }
            Expr::Borrow(inner, is_mut) => {
                if let Expr::Ident(i) = &**inner {
                    if self.local_trampolines.contains(&i.sym) {
                        return format!("&__cp_trampoline_{}.{}", i.sym, i.sym);
                    }
                    self.uninitialized_vars.remove(&i.sym);
                    if i.sym == "this" || i.sym == "_" || self.is_class_field(&i.sym) {
                        if *is_mut && self.current_receiver_mode == ReceiverMode::ReadOnly {
                            panic!(
                                "CP Compile Error: Cannot take mutable borrow '&mut' of '{}' in a read-only method (add 'mut _' as first parameter)",
                                i.sym
                            );
                        }
                        self.check_borrow_conflict("this", *is_mut);
                        self.check_borrow_conflict("_", *is_mut);
                    } else {
                        if *is_mut {
                            if let Some((_, is_m)) = self.var_borrow_sources.get(&i.sym) {
                                if !*is_m {
                                    panic!(
                                        "CP Compile Error: Cannot take mutable borrow '&mut {}' from read-only borrow",
                                        i.sym
                                    );
                                }
                            } else if self.current_fn_params.contains(&i.sym)
                                && !self.mutable_fn_params.contains(&i.sym)
                            {
                                panic!(
                                    "CP Compile Error: Cannot take mutable borrow '&mut {}' from read-only parameter",
                                    i.sym
                                );
                            }
                        }
                        self.check_borrow_conflict(&i.sym, *is_mut);
                    }
                }
                let inner_code = self.generate_expr(inner);
                let is_type_name = match &**inner {
                    Expr::Ident(id) => {
                        !self.locals.contains_key(&id.sym)
                            && !self.var_cp_types.contains_key(&id.sym)
                            && (self.class_defs.contains_key(&id.sym)
                                || self.value_classes.contains(&id.sym)
                                || self.type_aliases.contains_key(&id.sym)
                                || self.trait_defs.contains_key(&id.sym)
                                || id.sym.chars().next().map_or(false, |c| c.is_ascii_uppercase()))
                    }
                    Expr::Member(mem) => {
                        mem.prop.sym.chars().next().map_or(false, |c| c.is_ascii_uppercase())
                    }
                    _ => false,
                };
                if is_type_name {
                    return if *is_mut {
                        format!("*{}", inner_code)
                    } else {
                        format!("*const {}", inner_code)
                    };
                }
                let already_ptr = if matches!(&**inner, Expr::Member(_) | Expr::Index { .. }) {
                    false
                } else if let Expr::Ident(id) = &**inner {
                    if let Some(zig_ty) = self.locals.get(&id.sym) {
                        let clean_zig_ty = if self.narrowed_null_vars.contains(&id.sym) {
                            zig_ty.trim_start_matches('?')
                        } else {
                            zig_ty.as_str()
                        };
                        clean_zig_ty.starts_with('*')
                    } else if let Some(ty) = self.var_cp_types.get(&id.sym) {
                        let clean_ty = if self.narrowed_null_vars.contains(&id.sym) {
                            Self::unwrap_optional_type(ty)
                        } else {
                            ty.clone()
                        };
                        self.is_zig_heap_ptr_type(&clean_ty)
                    } else {
                        false
                    }
                } else if let Some(ty) = self.infer_cp_type(inner) {
                    self.is_zig_heap_ptr_type(&ty)
                } else {
                    false
                };
                if already_ptr {
                    inner_code
                } else {
                    format!("(&{})", inner_code)
                }
            }
            Expr::Move(expr) => {
                if let Expr::Ident(i) = &**expr {
                    if i.sym == "this" || i.sym == "_" {
                        if self.current_receiver_mode != ReceiverMode::Move {
                            panic!(
                                "CP Compile Error: Cannot move '{}' in a non-move method (add 'move _' as first parameter)",
                                i.sym
                            );
                        }
                        self.check_active_borrows_before_move("this");
                        self.check_active_borrows_before_move("_");
                        let gen = self.generate_expr(expr);
                        self.moved_vars.insert("this".to_string());
                        self.moved_vars.insert("_".to_string());
                        if !self.is_state_machine {
                            format!("(blk: {{ __self_moved = true; break :blk {}; }})", gen)
                        } else {
                            gen
                        }
                    } else {
                        if self.var_borrow_sources.contains_key(&i.sym)
                            || (self.current_fn_params.contains(&i.sym)
                                && !self.move_fn_params.contains(&i.sym))
                        {
                            panic!(
                                "CP Compile Error: Cannot move borrowed reference or read-only parameter '{}'",
                                i.sym
                            );
                        }
                        self.check_active_borrows_before_move(&i.sym);
                        let gen = self.generate_expr(expr);
                        self.moved_vars.insert(i.sym.clone());
                        if !self.is_state_machine {
                            format!("(blk: {{ __{}_moved = true; break :blk {}; }})", i.sym, gen)
                        } else {
                            gen
                        }
                    }
                } else {
                    panic!("CP Compile Error: move keyword can only be applied to identifiers");
                }
            }
            Expr::Bin(bin) => {
                if bin.op == BinaryOp::Pipe {
                    return self.generate_pipeline(&bin.left, &bin.right);
                }
                if bin.op == BinaryOp::Assign {
                    if let Expr::Ident(id) = &*bin.left {
                        self.uninitialized_vars.remove(&id.sym);
                        if matches!(&*bin.right, Expr::Lit(Lit::Null))
                            || matches!(&*bin.right, Expr::Ident(rid) if rid.sym == "null")
                        {
                            self.narrowed_null_vars.remove(&id.sym);
                        }
                    }
                }
                let is_null_cmp = matches!(&*bin.left, Expr::Lit(Lit::Null))
                    || matches!(&*bin.right, Expr::Lit(Lit::Null))
                    || matches!(&*bin.left, Expr::Ident(id) if id.sym == "null")
                    || matches!(&*bin.right, Expr::Ident(id) if id.sym == "null");
                let l = if bin.op == BinaryOp::Assign {
                    if let Expr::Ident(id) = &*bin.left {
                        let was = self.narrowed_null_vars.remove(&id.sym);
                        let code = self.generate_expr(&bin.left);
                        if was {
                            self.narrowed_null_vars.insert(id.sym.clone());
                        }
                        code
                    } else {
                        self.generate_expr(&bin.left)
                    }
                } else if is_null_cmp {
                    let l_id = if let Expr::Ident(id) = &*bin.left {
                        Some(id.sym.clone())
                    } else {
                        None
                    };
                    let was_l = l_id.as_ref().map_or(false, |s| self.narrowed_null_vars.remove(s));
                    let code = self.generate_expr(&bin.left);
                    if was_l {
                        self.narrowed_null_vars.insert(l_id.unwrap());
                    }
                    code
                } else {
                    self.generate_expr(&bin.left)
                };
                let r = if is_null_cmp {
                    let r_id = if let Expr::Ident(id) = &*bin.right {
                        Some(id.sym.clone())
                    } else {
                        None
                    };
                    let was_r = r_id.as_ref().map_or(false, |s| self.narrowed_null_vars.remove(s));
                    let code = self.generate_expr(&bin.right);
                    if was_r {
                        self.narrowed_null_vars.insert(r_id.unwrap());
                    }
                    code
                } else if bin.op == BinaryOp::LogicalAnd {
                    let not_null_vars = Self::extract_not_null_vars(&bin.left);
                    let old_narrowed = self.narrowed_null_vars.clone();
                    self.narrowed_null_vars.extend(not_null_vars);
                    let code = self.generate_expr(&bin.right);
                    self.narrowed_null_vars = old_narrowed;
                    code
                } else {
                    self.generate_expr(&bin.right)
                };
                let l_ty = self.infer_cp_type(&bin.left);
                let r_ty = self.infer_cp_type(&bin.right);
                let are_both_numeric = match (&l_ty, &r_ty) {
                    (Some(t1), Some(t2)) => Self::is_numeric_type(t1) && Self::is_numeric_type(t2),
                    (Some(t1), None) => {
                        Self::is_numeric_type(t1)
                            && matches!(
                                &*bin.right,
                                Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_)) | Expr::Index { .. }
                            )
                    }
                    (None, Some(t2)) => {
                        Self::is_numeric_type(t2)
                            && matches!(
                                &*bin.left,
                                Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_)) | Expr::Index { .. }
                            )
                    }
                    _ => {
                        matches!(&*bin.left, Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_)))
                            || matches!(
                                &*bin.right,
                                Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_))
                            )
                            || (matches!(&*bin.left, Expr::Index { .. })
                                && matches!(&*bin.right, Expr::Index { .. }))
                    }
                };
                let is_null_cmp = matches!(&*bin.left, Expr::Lit(Lit::Null))
                    || matches!(&*bin.right, Expr::Lit(Lit::Null))
                    || matches!(&*bin.left, Expr::Ident(id) if id.sym == "null")
                    || matches!(&*bin.right, Expr::Ident(id) if id.sym == "null");
                let is_enum_cmp = matches!(&*bin.left, Expr::EnumInit(_))
                    || matches!(&*bin.right, Expr::EnumInit(_))
                    || match (&l_ty, &r_ty) {
                        (Some(Type::Ref(id)), _) if self.enum_defs.contains_key(&id.sym) => true,
                        (_, Some(Type::Ref(id))) if self.enum_defs.contains_key(&id.sym) => true,
                        _ => false,
                    };
                let is_bool_cmp = match (&l_ty, &r_ty) {
                    (Some(Type::Bool), _) | (_, Some(Type::Bool)) => true,
                    _ => {
                        matches!(&*bin.left, Expr::Lit(Lit::Bool(_)))
                            || matches!(&*bin.right, Expr::Lit(Lit::Bool(_)))
                    }
                };
                match bin.op {
                    BinaryOp::Eq => {
                        let has_num = are_both_numeric
                            || matches!(&*bin.left, Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_)))
                            || matches!(&*bin.right, Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_)));
                        if self.is_lib_mode || has_num || is_null_cmp || is_enum_cmp || is_bool_cmp {
                            format!("({} == {})", l, r)
                        } else {
                            format!("cp_eq({}, {})", l, r)
                        }
                    }
                    BinaryOp::NotEq => {
                        let has_num = are_both_numeric
                            || matches!(&*bin.left, Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_)))
                            || matches!(&*bin.right, Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_)));
                        if self.is_lib_mode || has_num || is_null_cmp || is_enum_cmp || is_bool_cmp {
                            format!("({} != {})", l, r)
                        } else {
                            format!("cp_neq({}, {})", l, r)
                        }
                    }
                    BinaryOp::Add => {
                        let is_str_concat = matches!(l_ty, Some(Type::String))
                            || matches!(r_ty, Some(Type::String))
                            || matches!(&*bin.left, Expr::Lit(Lit::Str(_)))
                            || matches!(&*bin.right, Expr::Lit(Lit::Str(_)));
                        if self.is_lib_mode || !is_str_concat || are_both_numeric {
                            format!("({} + {})", l, r)
                        } else {
                            format!("cp_add({}, {})", l, r)
                        }
                    }
                    BinaryOp::Lt => {
                        let is_str_cmp = matches!(l_ty, Some(Type::String))
                            && matches!(r_ty, Some(Type::String));
                        if self.is_lib_mode || !is_str_cmp || are_both_numeric {
                            format!("({} < {})", l, r)
                        } else {
                            format!("cp_lt({}, {})", l, r)
                        }
                    }
                    BinaryOp::Gt => {
                        let is_str_cmp = matches!(l_ty, Some(Type::String))
                            && matches!(r_ty, Some(Type::String));
                        if self.is_lib_mode || !is_str_cmp || are_both_numeric {
                            format!("({} > {})", l, r)
                        } else {
                            format!("cp_gt({}, {})", l, r)
                        }
                    }
                    BinaryOp::LtEq => {
                        let is_str_cmp = matches!(l_ty, Some(Type::String))
                            && matches!(r_ty, Some(Type::String));
                        if self.is_lib_mode || !is_str_cmp || are_both_numeric {
                            format!("({} <= {})", l, r)
                        } else {
                            format!("cp_lte({}, {})", l, r)
                        }
                    }
                    BinaryOp::GtEq => {
                        let is_str_cmp = matches!(l_ty, Some(Type::String))
                            && matches!(r_ty, Some(Type::String));
                        if self.is_lib_mode || !is_str_cmp || are_both_numeric {
                            format!("({} >= {})", l, r)
                        } else {
                            format!("cp_gte({}, {})", l, r)
                        }
                    }
                    BinaryOp::Mod => format!("({} % {})", l, r),
                    BinaryOp::BitAnd => format!("({} & {})", l, r),
                    BinaryOp::BitOr => format!("({} | {})", l, r),
                    BinaryOp::BitXor => format!("({} ^ {})", l, r),
                    BinaryOp::Shl => format!("cp_shl({}, {})", l, r),
                    BinaryOp::Shr => format!("cp_shr({}, {})", l, r),
                    BinaryOp::LogicalAnd => format!("({} and {})", l, r),
                    BinaryOp::LogicalOr => match &*bin.right {
                        Expr::Continue(_) => {
                            let id = self.lambda_counter;
                            self.lambda_counter += 1;
                            format!("(blk_{id}: {{ if (!({})) continue; break :blk_{id} true; }})", l)
                        }
                        Expr::Break(_, val) => {
                            let id = self.lambda_counter;
                            self.lambda_counter += 1;
                            if let Some(v) = val {
                                let v_code = self.generate_expr(v);
                                format!(
                                    "(blk_{id}: {{ if (!({})) break {}; break :blk_{id} true; }})",
                                    l, v_code
                                )
                            } else {
                                format!("(blk_{id}: {{ if (!({})) break; break :blk_{id} true; }})", l)
                            }
                        }
                        Expr::Return(_, ret_expr) => {
                            let id = self.lambda_counter;
                            self.lambda_counter += 1;
                            let ret_stmt = self.build_ret_stmt_for_fallback(ret_expr.as_deref());
                            format!("(blk_{id}: {{ if (!({})) {{ {} }}; break :blk_{id} true; }})", l, ret_stmt)
                        }
                        _ => format!("({} or {})", l, r),
                    },
                    BinaryOp::Assign => {
                        if let Expr::Ident(id) = &*bin.left {
                            if id.sym == "_" {
                                format!("_ = {}", r)
                            } else {
                                format!("{} = {}", l, r)
                            }
                        } else {
                            format!("{} = {}", l, r)
                        }
                    }
                    _ => {
                        let op = match bin.op {
                            BinaryOp::Sub => "-",
                            BinaryOp::Mul => "*",
                            BinaryOp::Div => "/",
                            BinaryOp::WrappingAdd => "+%",
                            BinaryOp::WrappingSub => "-%",
                            BinaryOp::WrappingMul => "*%",
                            BinaryOp::SaturatingAdd => "+|",
                            BinaryOp::SaturatingSub => "-|",
                            _ => "",
                        };
                        format!("({} {} {})", l, op, r)
                    }
                }
            }
            Expr::Call(call) => {
                if let Expr::Member(mem) = &*call.callee {
                    let prop = mem.prop.sym.as_str();

                    if let Expr::Ident(id) = &*mem.obj {
                        if id.sym == "reflect" {
                            if prop == "hasMethod" && call.args.len() == 2 {
                                let target = self.generate_expr(&call.args[0]);
                                let name = self.generate_expr(&call.args[1]);
                                return format!("@hasDecl({}, {})", target, name);
                            }
                            if prop == "hasField" && call.args.len() == 2 {
                                let target = self.generate_expr(&call.args[0]);
                                let name = self.generate_expr(&call.args[1]);
                                return format!("@hasField({}, {})", target, name);
                            }
                            if prop == "has" && call.args.len() == 2 {
                                let target = self.generate_expr(&call.args[0]);
                                let name = self.generate_expr(&call.args[1]);
                                return format!("(@hasDecl({0}, {1}) or @hasField({0}, {1}))", target, name);
                            }
                        }
                        if (id.sym == "string" || id.sym == "String") && prop == "from" && call.args.len() == 1 {
                            let arg = self.generate_expr(&call.args[0]);
                            return format!("cp_string_from({})", arg);
                        }
                        let is_enum = self.enum_defs.contains_key(&id.sym)
                            || (!self.locals.contains_key(&id.sym)
                                && !self.global_var_types.contains_key(&id.sym)
                                && id.sym.chars().next().map_or(false, |c| c.is_ascii_uppercase())
                                && !self.class_defs.contains_key(&id.sym));
                        let is_custom_method = self.enum_defs.get(&id.sym).map_or(false, |ed| {
                            ed.methods.iter().any(|m| m.ident.sym == prop)
                        });
                        if is_enum && !is_custom_method && (prop == "get" || prop == "fromName" || prop == "fromString") && call.args.len() == 1 {
                            let arg = self.generate_expr(&call.args[0]);
                            return format!("{}.get({})", id.sym, arg);
                        }
                        if is_enum && prop == "has" && call.args.len() == 1 {
                            let arg = self.generate_expr(&call.args[0]);
                            return format!("{}.has({})", id.sym, arg);
                        }
                        if id.sym == "Shared" && prop == "new" && !call.args.is_empty() {
                            let arg = self.generate_expr(&call.args[0]);
                            return format!(
                                "(blk: {{ const _val = {}; const _T = @TypeOf(_val); if (comptime @typeInfo(_T) == .pointer) break :blk _val else {{ const _p = std.heap.page_allocator.create(_T) catch unreachable; _p.* = _val; break :blk _p; }} }})",
                                arg
                            );
                        }
                        if id.sym == "Mutex" && prop == "new" {
                            if call.args.is_empty() {
                                return "CpBareMutex{}".to_string();
                            } else {
                                let arg = self.generate_expr(&call.args[0]);
                                return format!("cp_make_mutex({})", arg);
                            }
                        }
                        if id.sym == "RwLock" && prop == "new" {
                            if call.args.is_empty() {
                                return "CpBareRwLock{}".to_string();
                            } else {
                                let arg = self.generate_expr(&call.args[0]);
                                return format!("cp_make_rwlock({})", arg);
                            }
                        }
                        if id.sym == "Atomic" && prop == "new" {
                            if call.args.is_empty() {
                                return "cp_make_atomic(@as(i32, 0))".to_string();
                            } else {
                                let arg = self.generate_expr(&call.args[0]);
                                return format!("cp_make_atomic({})", arg);
                            }
                        }
                    }

                    let mut raw_obj_ty = self.infer_cp_type(&mem.obj);
                    loop {
                        match raw_obj_ty {
                            Some(Type::Optional(inner))
                            | Some(Type::BorrowType { inner, .. })
                            | Some(Type::Shared(inner))
                            | Some(Type::RawPtr { inner, .. }) => {
                                raw_obj_ty = Some(*inner);
                            }
                            _ => break,
                        }
                    }

                    if (prop == "withLock" || prop == "withRead" || prop == "withWrite") && !call.args.is_empty() {
                        let obj_code = self.generate_expr(&mem.obj);
                        let blk_id = self.lambda_counter;
                        self.lambda_counter += 1;
                        let (lock_fn, unlock_fn) = match prop {
                            "withLock" => ("lock", "unlock"),
                            "withRead" => ("lockShared", "unlockShared"),
                            "withWrite" => ("lockExclusive", "unlockExclusive"),
                            _ => unreachable!(),
                        };
                        if let Expr::Arrow(arrow) = &call.args[0] {
                            let mut param_bind = String::new();
                            let mut saved_local = None;
                            let mut saved_cp_ty = None;
                            let mut param_ident = None;
                            if !arrow.params.is_empty() {
                                if let Pattern::Ident(pid) = &arrow.params[0] {
                                    let pname = pid.sym.clone();
                                    param_ident = Some(pname.clone());
                                    self.inline_closure_params.insert(pname.clone());
                                    param_bind = format!("    const {0} = &(_target.data); _ = &{0};\n", pname);
                                    saved_local = self.locals.get(&pname).cloned();
                                    saved_cp_ty = self.var_cp_types.get(&pname).cloned();
                                    if let Some(Type::Ref(id)) = &raw_obj_ty {
                                        let inner_sym = if id.sym.starts_with("Mutex<") {
                                            &id.sym[6..id.sym.len() - 1]
                                        } else if id.sym.starts_with("RwLock<") {
                                            &id.sym[7..id.sym.len() - 1]
                                        } else {
                                            ""
                                        };
                                        if !inner_sym.is_empty() {
                                            self.locals.insert(pname.clone(), format!("*{}", inner_sym));
                                            self.var_cp_types.insert(
                                                pname.clone(),
                                                Type::BorrowType {
                                                    inner: Box::new(Self::type_from_name_static(inner_sym)),
                                                    is_mut: prop != "withRead",
                                                    lifetimes: vec![],
                                                },
                                            );
                                        }
                                    }
                                }
                            }

                            let res = match &*arrow.body {
                                ArrowBody::Expr(e) => {
                                    let body_expr = self.generate_expr(e);
                                    format!(
                                        "(blk_sync_{0}: {{\n    const _target = if (comptime @typeInfo(@TypeOf({1})) == .pointer) ({1}) else &({1});\n    _target.raw.{2}();\n    defer _target.raw.{3}();\n{4}    break :blk_sync_{0} {5};\n}})",
                                        blk_id, obj_code, lock_fn, unlock_fn, param_bind, body_expr
                                    )
                                }
                                ArrowBody::Block(b) => {
                                    let old_blk = self.inline_closure_blk.clone();
                                    self.inline_closure_blk = Some(format!("blk_sync_{}", blk_id));
                                    let saved_content = self.current_case_content.clone();
                                    self.current_case_content.clear();

                                    for s in &b.body {
                                        self.generate_stmt(s);
                                    }
                                    if !Self::block_returns_value(&b.body) {
                                        self.write_to_case(&format!("break :blk_sync_{} {{}};", blk_id));
                                    }

                                    let body_stmts = self.current_case_content.clone();
                                    self.current_case_content = saved_content;
                                    self.inline_closure_blk = old_blk;

                                    format!(
                                        "(blk_sync_{0}: {{\n    const _target = if (comptime @typeInfo(@TypeOf({1})) == .pointer) ({1}) else &({1});\n    _target.raw.{2}();\n    defer _target.raw.{3}();\n{4}{5}\n}})",
                                        blk_id, obj_code, lock_fn, unlock_fn, param_bind, body_stmts
                                    )
                                }
                            };

                            if let Some(pname) = param_ident {
                                self.inline_closure_params.remove(&pname);
                                if let Some(l) = saved_local {
                                    self.locals.insert(pname.clone(), l);
                                } else {
                                    self.locals.remove(&pname);
                                }
                                if let Some(t) = saved_cp_ty {
                                    self.var_cp_types.insert(pname, t);
                                } else {
                                    self.var_cp_types.remove(&pname);
                                }
                            }
                            return res;
                        }
                    }

                    let is_sync_type = matches!(&raw_obj_ty, Some(Type::Ref(id)) if id.sym == "Mutex" || id.sym.starts_with("Mutex<") || id.sym == "RwLock" || id.sym.starts_with("RwLock<") || id.sym == "Atomic" || id.sym.starts_with("Atomic<") || id.sym.starts_with("LockGuard<") || id.sym.starts_with("ReadGuard<") || id.sym.starts_with("WriteGuard<"));
                    if is_sync_type || matches!(prop, "cas" | "compareAndSwap" | "testAndSet" | "tryLock" | "isLocked" | "tryRead" | "tryWrite" | "isReadLocked" | "isWriteLocked" | "release") {
                        let obj_raw = self.generate_expr(&mem.obj);
                        let obj_ty = self.infer_zig_type(None, Some(&mem.obj));
                        let obj = if obj_ty.starts_with('?') {
                            format!("({}).?", obj_raw)
                        } else {
                            obj_raw
                        };
                        if prop == "and" || prop == "or" || prop == "xor" {
                            let arg = self.generate_expr(&call.args[0]);
                            return format!("(if (comptime @typeInfo(@TypeOf({0})) == .pointer) ({0}) else &({0})).@\"{1}\"({2})", obj, prop, arg);
                        }
                        if prop == "cas" || prop == "compareAndSwap" {
                            let arg0 = self.generate_expr(&call.args[0]);
                            let arg1 = self.generate_expr(&call.args[1]);
                            return format!("(if (comptime @typeInfo(@TypeOf({0})) == .pointer) ({0}) else &({0})).cas({1}, {2})", obj, arg0, arg1);
                        }
                        if matches!(prop, "load" | "store" | "swap" | "add" | "sub" | "inc" | "dec" | "testAndSet" | "clear") {
                            let mut args_code = Vec::new();
                            for a in &call.args {
                                args_code.push(self.generate_expr(a));
                            }
                            return format!("(if (comptime @typeInfo(@TypeOf({0})) == .pointer) ({0}) else &({0})).{1}({2})", obj, prop, args_code.join(", "));
                        }
                        if matches!(prop, "lock" | "unlock" | "tryLock" | "isLocked" | "read" | "write" | "tryRead" | "tryWrite" | "isReadLocked" | "isWriteLocked" | "release") {
                            return format!("(if (comptime @typeInfo(@TypeOf({0})) == .pointer) ({0}) else &({0})).{1}()", obj, prop);
                        }
                    }

                    let _is_builtin_container = matches!(
                        raw_obj_ty,
                        Some(Type::Ref(ref id)) if id.sym == "BinaryHeap" || id.sym == "VecDeque" || id.sym == "BTreeMap" || id.sym == "BTreeSet" || id.sym == "Map" || id.sym == "Set" || id.sym == "HashMap" || id.sym == "HashSet"
                    );
                    let is_array_type = matches!(raw_obj_ty, Some(Type::Array(_)));
                    if prop == "push" && is_array_type {
                        self.check_mutation_allowed(&mem.obj, "container method 'push'");
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_array_push(&{}, {})", obj, arg);
                    } else if prop == "pop" && is_array_type {
                        self.check_mutation_allowed(&mem.obj, "container method 'pop'");
                        let obj = self.generate_expr(&mem.obj);
                        return format!("cp_array_pop(&{})", obj);
                    } else if prop == "clear" && is_array_type {
                        self.check_mutation_allowed(&mem.obj, "container method 'clear'");
                        let obj = self.generate_expr(&mem.obj);
                        return format!("cp_container_clear(&{})", obj);
                    } else if prop == "push"
                        || prop == "push_back"
                        || prop == "push_front"
                        || prop == "pop"
                        || prop == "pop_back"
                        || prop == "pop_front"
                        || prop == "clear"
                        || prop == "set"
                        || prop == "put_all"
                        || prop == "add"
                        || prop == "delete"
                        || prop == "insert"
                        || prop == "remove"
                    {
                        self.check_mutation_allowed(
                            &mem.obj,
                            &format!("container method '{}'", prop),
                        );
                        let obj = self.generate_expr(&mem.obj);
                        let mut args_code = Vec::new();
                        for a in &call.args {
                            args_code.push(self.generate_expr(a));
                        }
                        return format!("{}.{}({})", obj, prop, args_code.join(", "));
                    } else if prop == "get"
                        || prop == "has"
                        || prop == "keys"
                        || prop == "values"
                        || prop == "entries"
                        || prop == "peek"
                        || prop == "peek_front"
                        || prop == "peek_back"
                        || prop == "len"
                        || prop == "is_empty"
                        || prop == "min_key"
                        || prop == "max_key"
                        || prop == "min"
                        || prop == "max"
                    {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let mut args_code = Vec::new();
                        for a in &call.args {
                            args_code.push(self.generate_expr(a));
                        }
                        return format!("{}.{}({})", obj, prop, args_code.join(", "));
                    } else if prop == "indexOf" && call.args.len() == 1 {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_index_of({}, {})", obj, arg);
                    } else if prop == "lastIndexOf" && call.args.len() == 1 {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_last_index_of({}, {})", obj, arg);
                    } else if prop == "includes" && call.args.len() == 1 {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_includes({}, {})", obj, arg);
                    } else if prop == "startsWith" && call.args.len() == 1 {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("@import(\"std\").mem.startsWith(u8, {}, {})", obj, arg);
                    } else if prop == "endsWith" && call.args.len() == 1 {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("@import(\"std\").mem.endsWith(u8, {}, {})", obj, arg);
                    } else if (prop == "slice" || prop == "substring") && !call.args.is_empty() {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let start = self.generate_expr(&call.args[0]);
                        if call.args.len() > 1 {
                            let end = self.generate_expr(&call.args[1]);
                            return format!("cp_iter_slice({})[@intCast({})..@intCast({})]", obj, start, end);
                        } else {
                            return format!("cp_iter_slice({})[@intCast({})..]", obj, start);
                        }
                    } else if prop == "toLowerCase" && call.args.is_empty() {
                        let obj = self.generate_expr(&mem.obj);
                        return format!("(blk: {{ var out = {}.alloc(u8, {}.len) catch unreachable; for ({}, 0..) |c, i| out[i] = @import(\"std\").ascii.toLower(c); break :blk out; }})", self.current_allocator(), obj, obj);
                    } else if prop == "toUpperCase" && call.args.is_empty() {
                        let obj = self.generate_expr(&mem.obj);
                        return format!("(blk: {{ var out = {}.alloc(u8, {}.len) catch unreachable; for ({}, 0..) |c, i| out[i] = @import(\"std\").ascii.toUpper(c); break :blk out; }})", self.current_allocator(), obj, obj);
                    } else if prop == "replace" && call.args.len() == 2 {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        let replacement = self.generate_expr(&call.args[1]);
                        return format!("@import(\"std\").mem.replaceOwned(u8, {}, {}, {}, {}) catch unreachable", self.current_allocator(), obj, arg, replacement);
                    } else if prop == "charCodeAt" && !call.args.is_empty() {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("@as(i32, @intCast({}[@intCast({})]))", obj, arg);
                    } else if prop == "match" && call.args.len() == 1 {
                        let is_class_or_enum = match &*mem.obj {
                            Expr::Ident(id) => self.class_defs.contains_key(&id.sym) || self.enum_defs.contains_key(&id.sym) || id.sym.chars().next().map_or(false, |c| c.is_uppercase()),
                            _ => false,
                        };
                        if !is_class_or_enum {
                            let obj = self.generate_expr(&mem.obj);
                            let arg = self.generate_expr(&call.args[0]);
                            return format!("{}.exec({})", arg, obj);
                        }
                    } else if prop == "test" && !call.args.is_empty() {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("{}.@\"test\"({})", obj, arg);
                    } else if prop == "exec" && !call.args.is_empty() {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("{}.exec({})", obj, arg);
                    } else if prop == "toString" && call.args.is_empty() {
                        let obj = self.generate_expr(&mem.obj);
                        if let Some(Type::Ref(id)) = self.infer_cp_type(&mem.obj) {
                            if self.enum_defs.contains_key(&id.sym) {
                                return format!("@tagName({})", obj);
                            }
                        }
                        return format!("cp_string_from({})", obj);
                    }
                }

                if let Expr::Member(mem) = &*call.callee {
                    if let Expr::Ident(id) = &*mem.obj {
                        if (id.sym == "string" || id.sym == "String") && mem.prop.sym == "from" && call.args.len() == 1 {
                            let arg_code = self.generate_expr(&call.args[0]);
                            return format!("cp_string_from({})", arg_code);
                        }
                        if let Some(ed) = self.enum_defs.get(&id.sym) {
                            let is_method = ed.methods.iter().any(|m| m.ident.sym == mem.prop.sym)
                                || ed.extra_zig.iter().any(|z| z.contains(&format!("fn {}(", mem.prop.sym)));
                            let is_variant = ed.variants.iter().any(|v| v.ident.sym == mem.prop.sym);
                            if is_method || !is_variant {
                                let mut args_code = Vec::new();
                                for a in &call.args {
                                    args_code.push(self.generate_expr(a));
                                }
                                return format!("{}.{}({})", id.sym, mem.prop.sym, args_code.join(", "));
                            }
                            let arg_code = if !call.args.is_empty() {
                                self.generate_expr(&call.args[0])
                            } else {
                                "{}".to_string()
                            };
                            return format!("{}{{ .{} = {} }}", id.sym, mem.prop.sym, arg_code);
                        }
                        if id.sym == "Math" {
                            let prop = mem.prop.sym.as_str();
                            if prop == "abs" {
                                return format!("@abs({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "ceil" {
                                return format!("@ceil({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "floor" {
                                return format!("@floor({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "round" {
                                return format!("@round({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "trunc" {
                                return format!("@trunc({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "max" {
                                return format!(
                                    "@max({}, {})",
                                    self.generate_expr(&call.args[0]),
                                    self.generate_expr(&call.args[1])
                                );
                            }
                            if prop == "min" {
                                return format!(
                                    "@min({}, {})",
                                    self.generate_expr(&call.args[0]),
                                    self.generate_expr(&call.args[1])
                                );
                            }
                            if prop == "sqrt" {
                                return format!("@sqrt({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "sin" {
                                return format!("@sin({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "cos" {
                                return format!("@cos({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "tan" {
                                return format!("@tan({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "exp" {
                                return format!("@exp({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "log" {
                                return format!("@log({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "log10" {
                                return format!("@log10({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "log2" {
                                return format!("@log2({})", self.generate_expr(&call.args[0]));
                            }
                            if prop == "pow" {
                                return format!(
                                    "@import(\"std\").math.pow(f32, {}, {})",
                                    self.generate_expr(&call.args[0]),
                                    self.generate_expr(&call.args[1])
                                );
                            }
                            if prop == "asin" {
                                return format!(
                                    "@import(\"std\").math.asin({})",
                                    self.generate_expr(&call.args[0])
                                );
                            }
                            if prop == "acos" {
                                return format!(
                                    "@import(\"std\").math.acos({})",
                                    self.generate_expr(&call.args[0])
                                );
                            }
                            if prop == "atan" {
                                return format!(
                                    "@import(\"std\").math.atan({})",
                                    self.generate_expr(&call.args[0])
                                );
                            }
                            if prop == "atan2" {
                                return format!(
                                    "@import(\"std\").math.atan2({}, {})",
                                    self.generate_expr(&call.args[0]),
                                    self.generate_expr(&call.args[1])
                                );
                            }
                            if prop == "random" {
                                return "cp_math_random()".to_string();
                            }
                        }

                        // 1. Handle `super.method(...)` in single-inheritance subclass
                        if id.sym == "super" {
                            let c_name = self.current_class_name.clone().expect(
                                "CP Compile Error: 'super' can only be used inside a class method",
                            );
                            let super_method_name = format!("__super_{}", mem.prop.sym);
                            if let Some((_params, ret_ty, target_recv_mode)) =
                                self.lookup_method(&c_name, &super_method_name)
                            {
                                if self.moved_vars.contains("this") || self.moved_vars.contains("_")
                                {
                                    panic!("CP Compile Error: Use of moved value '_'");
                                }
                                match target_recv_mode {
                                    ReceiverMode::Mut => {
                                        if self.current_receiver_mode == ReceiverMode::ReadOnly {
                                            panic!(
                                                "CP Compile Error: Cannot call mutable parent method 'super.{}()' from a read-only method (add 'mut _' as first parameter)",
                                                mem.prop.sym
                                            );
                                        }
                                        self.check_borrow_conflict("this", true);
                                        self.check_borrow_conflict("_", true);
                                    }
                                    ReceiverMode::Move => {
                                        if self.current_receiver_mode != ReceiverMode::Move {
                                            panic!(
                                                "CP Compile Error: Cannot call consuming parent method 'super.{}()' from a non-move method (add 'move _' as first parameter)",
                                                mem.prop.sym
                                            );
                                        }
                                        self.check_active_borrows_before_move("this");
                                        self.check_active_borrows_before_move("_");
                                    }
                                    ReceiverMode::ReadOnly => {
                                        self.check_borrow_conflict("this", false);
                                        self.check_borrow_conflict("_", false);
                                    }
                                    ReceiverMode::Static => {}
                                }
                                let mut args = String::new();
                                for (idx, arg) in call.args.iter().enumerate() {
                                    args.push_str(&self.generate_expr(arg));
                                    if idx < call.args.len() - 1 {
                                        args.push_str(", ");
                                    }
                                }
                                let self_str = if self.is_state_machine {
                                    "ctx.self"
                                } else {
                                    "self"
                                };
                                let call_res = if target_recv_mode == ReceiverMode::Move {
                                    self.moved_vars.insert("this".to_string());
                                    self.moved_vars.insert("_".to_string());
                                    if !self.is_state_machine {
                                        format!(
                                            "(blk: {{ __self_moved = true; break :blk {}.{}({}); }})",
                                            self_str, super_method_name, args
                                        )
                                    } else {
                                        format!("{}.{}({})", self_str, super_method_name, args)
                                    }
                                } else {
                                    format!("{}.{}({})", self_str, super_method_name, args)
                                };
                                return self.maybe_wrap_try_call(call_res, ret_ty.as_ref());
                            } else {
                                panic!(
                                    "CP Compile Error: Parent method 'super.{}()' not found in class '{}'",
                                    mem.prop.sym, c_name
                                );
                            }
                        }

                        // 2. Handle `_.method(...)` / `this.method(...)` or `obj.method(...)`
                        if let Some(type_name) = self.resolve_var_type_name(&id.sym) {
                            if let Some((params, ret_ty, target_recv_mode)) =
                                self.lookup_method(&type_name, &mem.prop.sym)
                            {
                                let is_self_recv = id.sym == "this" || id.sym == "_";
                                if is_self_recv {
                                    if self.moved_vars.contains("this")
                                        || self.moved_vars.contains("_")
                                    {
                                        panic!(
                                            "CP Compile Error: Use of moved value '{}'",
                                            id.sym
                                        );
                                    }
                                    match target_recv_mode {
                                        ReceiverMode::Mut => {
                                            if self.current_receiver_mode == ReceiverMode::ReadOnly
                                            {
                                                panic!(
                                                    "CP Compile Error: Cannot call mutable method '{}.{}()' inside a read-only method (add 'mut _' as first parameter)",
                                                    id.sym, mem.prop.sym
                                                );
                                            }
                                            self.check_borrow_conflict("this", true);
                                            self.check_borrow_conflict("_", true);
                                        }
                                        ReceiverMode::Move => {
                                            if self.current_receiver_mode != ReceiverMode::Move {
                                                panic!(
                                                    "CP Compile Error: Cannot call consuming 'move _' method '{}.{}()' inside a non-move method (add 'move _' as first parameter)",
                                                    id.sym, mem.prop.sym
                                                );
                                            }
                                            self.check_active_borrows_before_move("this");
                                            self.check_active_borrows_before_move("_");
                                        }
                                        ReceiverMode::ReadOnly => {
                                            self.check_borrow_conflict("this", false);
                                            self.check_borrow_conflict("_", false);
                                        }
                                        ReceiverMode::Static => {}
                                    }
                                } else {
                                    if self.moved_vars.contains(&id.sym) {
                                        panic!(
                                            "CP Compile Error: Use of moved value '{}'",
                                            id.sym
                                        );
                                    }
                                    match target_recv_mode {
                                        ReceiverMode::Mut => {
                                            if let Some((_, is_mut)) =
                                                self.var_borrow_sources.get(&id.sym)
                                            {
                                                if !*is_mut {
                                                    panic!(
                                                        "CP Compile Error: Cannot call mutable method '{}()' on read-only borrow '{}'",
                                                        mem.prop.sym, id.sym
                                                    );
                                                }
                                            } else if self.current_fn_params.contains(&id.sym)
                                                && !self.mutable_fn_params.contains(&id.sym)
                                            {
                                                panic!(
                                                    "CP Compile Error: Cannot call mutable method '{}()' on read-only parameter '{}' (mark parameter as 'mut {}')",
                                                    mem.prop.sym, id.sym, id.sym
                                                );
                                            }
                                            self.check_borrow_conflict(&id.sym, true);
                                        }
                                        ReceiverMode::Move => {
                                            if self.var_borrow_sources.contains_key(&id.sym) {
                                                panic!(
                                                    "CP Compile Error: Cannot call consuming 'move _' method '{}()' on borrowed reference '{}'",
                                                    mem.prop.sym, id.sym
                                                );
                                            }
                                            if self.current_fn_params.contains(&id.sym)
                                                && !self.move_fn_params.contains(&id.sym)
                                            {
                                                panic!(
                                                    "CP Compile Error: Cannot call consuming 'move _' method '{}()' on borrowed parameter '{}' (mark parameter as 'move {}')",
                                                    mem.prop.sym, id.sym, id.sym
                                                );
                                            }
                                            self.check_active_borrows_before_move(&id.sym);
                                        }
                                        ReceiverMode::ReadOnly => {
                                            self.check_borrow_conflict(&id.sym, false);
                                        }
                                        ReceiverMode::Static => {}
                                    }
                                }

                                let non_recv_params: Vec<Param> = params
                                    .into_iter()
                                    .filter(|p| p.ident.sym != "this" && p.ident.sym != "_")
                                    .collect();
                                let obj_code = self.generate_expr(&mem.obj);
                                let mut args = String::new();
                                for (idx, arg) in call.args.iter().enumerate() {
                                    let mut arg_is_move_param = false;
                                    let old_box = self.expected_box_trait.clone();
                                    if idx < non_recv_params.len() {
                                        let param = &non_recv_params[idx];
                                        if let Type::BoxDyn(tr_id) = &param.ty {
                                            self.expected_box_trait = Some(tr_id.sym.clone());
                                        }
                                        if let Type::Ref(t_id) = &param.ty {
                                            if self.trait_defs.contains_key(&t_id.sym) {
                                                if let Expr::Ident(arg_id) = arg {
                                                    if let Some(arg_cls) =
                                                        self.resolve_var_type_name(&arg_id.sym)
                                                    {
                                                        let implements = self
                                                            .class_defs
                                                            .get(&arg_cls)
                                                            .map_or(false, |c| {
                                                                c.implements
                                                                    .iter()
                                                                    .any(|i| i.sym == t_id.sym)
                                                            });
                                                        if !implements {
                                                            panic!(
                                                                "CP Compile Error: Type '{}' does not implement trait '{}'",
                                                                arg_cls, t_id.sym
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if let Expr::Ident(arg_id) = arg {
                                            if param.is_mut {
                                                if let Some((_, is_m)) =
                                                    self.var_borrow_sources.get(&arg_id.sym)
                                                {
                                                    if !*is_m {
                                                        panic!(
                                                            "CP Compile Error: Cannot pass read-only borrow '{}' to mutable parameter '{}'",
                                                            arg_id.sym, param.ident.sym
                                                        );
                                                    }
                                                } else if self.current_fn_params.contains(&arg_id.sym)
                                                    && !self.mutable_fn_params.contains(&arg_id.sym)
                                                {
                                                    panic!(
                                                        "CP Compile Error: Cannot pass read-only parameter '{}' to mutable parameter '{}'",
                                                        arg_id.sym, param.ident.sym
                                                    );
                                                }
                                                self.check_borrow_conflict(&arg_id.sym, true);
                                            } else if param.is_move {
                                                if self.var_borrow_sources.contains_key(&arg_id.sym)
                                                    || (self.current_fn_params.contains(&arg_id.sym)
                                                        && !self
                                                            .move_fn_params
                                                            .contains(&arg_id.sym))
                                                {
                                                    panic!(
                                                        "CP Compile Error: Cannot move borrowed reference or read-only parameter '{}' into consuming parameter '{}'",
                                                        arg_id.sym, param.ident.sym
                                                    );
                                                }
                                                self.check_active_borrows_before_move(&arg_id.sym);
                                                arg_is_move_param = true;
                                            } else {
                                                self.check_borrow_conflict(&arg_id.sym, false);
                                            }
                                        }
                                    }
                                    let mut gen_arg = self.generate_expr(arg);
                                    if gen_arg.starts_with("lambda_") {
                                        gen_arg = format!("@ptrCast(@constCast(&{}))", gen_arg);
                                    }
                                    self.expected_box_trait = old_box;
                                    if arg_is_move_param {
                                        if let Expr::Ident(arg_id) = arg {
                                            self.moved_vars.insert(arg_id.sym.clone());
                                            if !self.is_state_machine {
                                                args.push_str(&format!(
                                                    "(blk: {{ __{}_moved = true; break :blk {}; }})",
                                                    arg_id.sym, gen_arg
                                                ));
                                            } else {
                                                args.push_str(&gen_arg);
                                            }
                                        } else {
                                            args.push_str(&gen_arg);
                                        }
                                    } else {
                                        args.push_str(&gen_arg);
                                    }
                                    if idx < call.args.len() - 1 {
                                        args.push_str(", ");
                                    }
                                }

                                let prop_name = if matches!(mem.prop.sym.as_str(), "and" | "or" | "orelse") {
                                    format!("@\"{}\"", mem.prop.sym)
                                } else {
                                    mem.prop.sym.clone()
                                };
                                let call_res = if target_recv_mode == ReceiverMode::Move {
                                    if is_self_recv {
                                        self.moved_vars.insert("this".to_string());
                                        self.moved_vars.insert("_".to_string());
                                        if !self.is_state_machine {
                                            format!(
                                                "(blk: {{ __self_moved = true; break :blk {}.{}({}); }})",
                                                obj_code, prop_name, args
                                            )
                                        } else {
                                            format!("{}.{}({})", obj_code, prop_name, args)
                                        }
                                    } else {
                                        self.moved_vars.insert(id.sym.clone());
                                        if !self.is_state_machine {
                                            format!(
                                                "(blk: {{ __{}_moved = true; break :blk {}.{}({}); }})",
                                                id.sym, obj_code, prop_name, args
                                            )
                                        } else {
                                            format!("{}.{}({})", obj_code, prop_name, args)
                                        }
                                    }
                                } else {
                                    format!("{}.{}({})", obj_code, prop_name, args)
                                };
                                return self.maybe_wrap_try_call(call_res, ret_ty.as_ref());
                            }
                        }
                    }
                }

                // 3. Check omitted-receiver sibling method call inside a class method (`helper()` -> `self.helper()`)
                if let Expr::Ident(ident) = &*call.callee {
                    if self.local_trampolines.contains(&ident.sym) {
                        let mut args = String::new();
                        for (idx, arg) in call.args.iter().enumerate() {
                            args.push_str(&self.generate_expr(arg));
                            if idx + 1 < call.args.len() {
                                args.push_str(", ");
                            }
                        }
                        return format!("__cp_trampoline_{}.{}({})", ident.sym, ident.sym, args);
                    }
                    if self.enum_defs.contains_key(&ident.sym) && call.args.len() == 1 {
                        let arg_code = self.generate_expr(&call.args[0]);
                        return format!("@as({}, @enumFromInt({}))", ident.sym, arg_code);
                    }
                    if (ident.sym == "string" || ident.sym == "String") && call.args.len() == 1 {
                        let arg_code = self.generate_expr(&call.args[0]);
                        if let Some(Type::Ref(id)) = self.infer_cp_type(&call.args[0]) {
                            if self.enum_defs.contains_key(&id.sym) {
                                return format!("@tagName({})", arg_code);
                            }
                        }
                        return format!("cp_string_from({})", arg_code);
                    }
                    let is_primitive = matches!(
                        ident.sym.as_str(),
                        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "i128" | "u128"
                            | "isize" | "usize" | "f32" | "f64" | "bool"
                    );
                    if is_primitive && call.args.len() == 1 {
                        let target_ty = match ident.sym.as_str() {
                            "i8" => Type::I8,
                            "u8" => Type::U8,
                            "i16" => Type::I16,
                            "u16" => Type::U16,
                            "i32" => Type::I32,
                            "u32" => Type::U32,
                            "i64" => Type::I64,
                            "u64" => Type::U64,
                            "i128" => Type::I128,
                            "u128" => Type::U128,
                            "isize" => Type::ISize,
                            "usize" => Type::USize,
                            "f32" => Type::F32,
                            "f64" => Type::F64,
                            "bool" => Type::Bool,
                            _ => unreachable!(),
                        };
                        return self.generate_expr(&Expr::Cast {
                            expr: Box::new(call.args[0].clone()),
                            target_ty,
                            span: call.span,
                        });
                    }
                    if let Some(c_name) = self.current_class_name.clone() {
                        if self.current_receiver_mode != ReceiverMode::Static
                            && !self.locals.contains_key(&ident.sym)
                            && !self.fn_sigs.contains_key(&ident.sym)
                        {
                            if let Some((params, ret_ty, target_recv_mode)) =
                                self.lookup_method(&c_name, &ident.sym)
                            {
                                if self.moved_vars.contains("this") || self.moved_vars.contains("_")
                                {
                                    panic!("CP Compile Error: Use of moved value '_'");
                                }
                                match target_recv_mode {
                                    ReceiverMode::Mut => {
                                        if self.current_receiver_mode == ReceiverMode::ReadOnly {
                                            panic!(
                                                "CP Compile Error: Cannot call mutable method '{}()' inside a read-only method (add 'mut _' as first parameter)",
                                                ident.sym
                                            );
                                        }
                                        self.check_borrow_conflict("this", true);
                                        self.check_borrow_conflict("_", true);
                                    }
                                    ReceiverMode::Move => {
                                        if self.current_receiver_mode != ReceiverMode::Move {
                                            panic!(
                                                "CP Compile Error: Cannot call consuming 'move _' method '{}()' inside a non-move method (add 'move _' as first parameter)",
                                                ident.sym
                                            );
                                        }
                                        self.check_active_borrows_before_move("this");
                                        self.check_active_borrows_before_move("_");
                                    }
                                    ReceiverMode::ReadOnly => {
                                        self.check_borrow_conflict("this", false);
                                        self.check_borrow_conflict("_", false);
                                    }
                                    ReceiverMode::Static => {}
                                }
                                let non_recv_params: Vec<Param> = params
                                    .into_iter()
                                    .filter(|p| p.ident.sym != "this" && p.ident.sym != "_")
                                    .collect();
                                let mut args = String::new();
                                for (idx, arg) in call.args.iter().enumerate() {
                                    let mut arg_is_move_param = false;
                                    let old_box = self.expected_box_trait.clone();
                                    if idx < non_recv_params.len() {
                                        let param = &non_recv_params[idx];
                                        if let Type::BoxDyn(tr_id) = &param.ty {
                                            self.expected_box_trait = Some(tr_id.sym.clone());
                                        }
                                        if let Expr::Ident(arg_id) = arg {
                                            if param.is_mut {
                                                self.check_borrow_conflict(&arg_id.sym, true);
                                            } else if param.is_move {
                                                self.check_active_borrows_before_move(&arg_id.sym);
                                                arg_is_move_param = true;
                                            } else {
                                                self.check_borrow_conflict(&arg_id.sym, false);
                                            }
                                        }
                                    }
                                    let mut gen_arg = self.generate_expr(arg);
                                    if gen_arg.starts_with("lambda_") {
                                        gen_arg = format!("@ptrCast(@constCast(&{}))", gen_arg);
                                    }
                                    self.expected_box_trait = old_box;
                                    if arg_is_move_param {
                                        if let Expr::Ident(arg_id) = arg {
                                            self.moved_vars.insert(arg_id.sym.clone());
                                            if !self.is_state_machine {
                                                args.push_str(&format!(
                                                    "(blk: {{ __{}_moved = true; break :blk {}; }})",
                                                    arg_id.sym, gen_arg
                                                ));
                                            } else {
                                                args.push_str(&gen_arg);
                                            }
                                        } else {
                                            args.push_str(&gen_arg);
                                        }
                                    } else {
                                        args.push_str(&gen_arg);
                                    }
                                    if idx < call.args.len() - 1 {
                                        args.push_str(", ");
                                    }
                                }
                                if target_recv_mode == ReceiverMode::Static {
                                    let call_res = format!("{}.{}({})", c_name, ident.sym, args);
                                    return self.maybe_wrap_try_call(call_res, ret_ty.as_ref());
                                }
                                let self_str = if self.is_state_machine {
                                    "ctx.self"
                                } else {
                                    "self"
                                };
                                let call_res = if target_recv_mode == ReceiverMode::Move {
                                    self.moved_vars.insert("this".to_string());
                                    self.moved_vars.insert("_".to_string());
                                    if !self.is_state_machine {
                                        format!(
                                            "(blk: {{ __self_moved = true; break :blk {}.{}({}); }})",
                                            self_str, ident.sym, args
                                        )
                                    } else {
                                        format!("{}.{}({})", self_str, ident.sym, args)
                                    }
                                } else {
                                    format!("{}.{}({})", self_str, ident.sym, args)
                                };
                                return self.maybe_wrap_try_call(call_res, ret_ty.as_ref());
                            }
                        }
                    }
                }

                let is_spawn_cpu = if let Expr::Member(mem) = &*call.callee {
                    if let Expr::Ident(obj_id) = &*mem.obj {
                        obj_id.sym == "spawn" && mem.prop.sym == "cpu"
                    } else {
                        false
                    }
                } else {
                    false
                };
                let callee = if is_spawn_cpu {
                    "@task_spawn_cpu".to_string()
                } else {
                    match &*call.callee {
                        Expr::Ident(ident) => {
                            if self.is_state_machine && self.locals.contains_key(&ident.sym) && !self.inline_closure_params.contains(&ident.sym) {
                                format!("ctx.{}", ident.sym)
                            } else {
                                ident.sym.clone()
                            }
                        }
                        _ => self.generate_expr(&call.callee),
                    }
                };
                if (callee == "spawn" || callee == "@task_spawn" || callee == "@task_spawn_cpu")
                    && call.args.len() == 1
                {
                    let task_fn = match &call.args[0] {
                        Expr::Ident(id) => id.sym.clone(),
                        Expr::Arrow(_) => self.generate_expr(&call.args[0]),
                        _ => "Unknown".to_string(),
                    };
                    let mut captures = String::new();
                    let moved_in_closure = self.lambda_moved_vars.get(&task_fn).cloned().unwrap_or_default();
                    if let Some(cv) = self.captured_vars.get(&task_fn).cloned() {
                        for (r, _) in &cv {
                            if self.var_is_ref_struct.contains(r) {
                                panic!("CP Compile Error: Stack-bound ref struct '{}' cannot be captured across parallel task spawn boundary", r);
                            }
                            if self.var_borrow_sources.contains_key(r) {
                                panic!("CP Compile Error: Temporary borrow '{}' cannot be captured across parallel task spawn boundary (use move or Shared<T>)", r);
                            }
                            if !moved_in_closure.contains(r) {
                                if self.moved_vars.contains(r) {
                                    panic!("CP Compile Error: Use of moved variable '{}'", r);
                                }
                                self.check_borrow_conflict(r, false);
                            }
                            let src = if self.is_state_machine {
                                format!("ctx.{}", r)
                            } else {
                                r.clone()
                            };
                            captures.push_str(&format!("sub_ctx.{} = {}; ", r, src));
                        }
                    }
                    if callee == "@task_spawn_cpu" {
                        return format!("(blk: {{ _ = global_active_tasks.fetchAdd(1, .acquire); const arena_ptr = std.heap.page_allocator.create(std.heap.ArenaAllocator) catch unreachable; arena_ptr.* = std.heap.ArenaAllocator.init(std.heap.page_allocator); const task_ptr = arena_ptr.allocator().create(CpTask) catch unreachable; task_ptr.* = .{{ .arena = arena_ptr }}; const sub_ctx = arena_ptr.allocator().create({0}_Context) catch unreachable; sub_ctx.* = undefined; sub_ctx.pc = 0; sub_ctx.arena = arena_ptr; sub_ctx._parent = @ptrCast(task_ptr); sub_ctx._parent_fn = @ptrCast(&__cp_task_bridge); {1} const p = PID{{ .node_id = current_node_id, .local_id = next_local_id }}; next_local_id += 1; sub_ctx._self = p; const _CpuRunner = struct {{ pub fn run(c: *anyopaque) void {{ {0}(c) catch {{}}; }} }}; const _th = std.Thread.spawn(.{{}}, _CpuRunner.run, .{{@as(*anyopaque, @ptrCast(sub_ctx))}}) catch unreachable; _th.detach(); break :blk task_ptr; }})", task_fn, captures);
                    }
                    return format!("(blk: {{ _ = global_active_tasks.fetchAdd(1, .acquire); const arena_ptr = std.heap.page_allocator.create(std.heap.ArenaAllocator) catch unreachable; arena_ptr.* = std.heap.ArenaAllocator.init(std.heap.page_allocator); const task_ptr = arena_ptr.allocator().create(CpTask) catch unreachable; task_ptr.* = .{{ .arena = arena_ptr }}; const sub_ctx = arena_ptr.allocator().create({0}_Context) catch unreachable; sub_ctx.* = undefined; sub_ctx.pc = 0; sub_ctx.arena = arena_ptr; sub_ctx._parent = @ptrCast(task_ptr); sub_ctx._parent_fn = @ptrCast(&__cp_task_bridge); {1} const p = PID{{ .node_id = current_node_id, .local_id = next_local_id }}; next_local_id += 1; sub_ctx._self = p; schedule(@ptrCast(&{0}), @ptrCast(sub_ctx)); break :blk task_ptr; }})", task_fn, captures);
                }
                if callee == "Shared" && !call.args.is_empty() {
                    let arg = self.generate_expr(&call.args[0]);
                    return format!(
                        "(blk: {{ const _val = {}; const _T = @TypeOf(_val); if (comptime @typeInfo(_T) == .pointer) break :blk _val else {{ const _p = std.heap.page_allocator.create(_T) catch unreachable; _p.* = _val; break :blk _p; }} }})",
                        arg
                    );
                }
                if callee == "Mutex" {
                    if call.args.is_empty() {
                        return "CpBareMutex{}".to_string();
                    } else {
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_make_mutex({})", arg);
                    }
                }
                if callee == "RwLock" {
                    if call.args.is_empty() {
                        return "CpBareRwLock{}".to_string();
                    } else {
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_make_rwlock({})", arg);
                    }
                }
                if callee == "Atomic" {
                    if call.args.is_empty() {
                        return "cp_make_atomic(@as(i32, 0))".to_string();
                    } else {
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_make_atomic({})", arg);
                    }
                }
                if callee == "Error" {
                    let code = if !call.args.is_empty() {
                        self.generate_expr(&call.args[0])
                    } else {
                        "0".to_string()
                    };
                    let msg = if call.args.len() > 1 {
                        self.generate_expr(&call.args[1])
                    } else {
                        "\"\"".to_string()
                    };
                    return format!("Error.init({}, {})", code, msg);
                }
                if callee == "test" && call.args.len() == 2 {
                    let name = self.generate_expr(&call.args[0]);
                    let lambda = self.generate_expr(&call.args[1]);
                    return format!("(blk: {{ __cap_test_total += 1; const __start_fails = __cap_test_failed; @import(\"std\").debug.print(\"\\n▶ RUN   {{s}}\\n\", .{{{}}}); var test_ctx: {}_Context = undefined; test_ctx.pc = 0; test_ctx._self = PID{{ .node_id = current_node_id, .local_id = next_local_id }}; next_local_id += 1; test_ctx._parent = null; test_ctx._parent_fn = null; test_ctx.arena = null; test_ctx.received_msg = null; test_ctx.received_sender = null; test_ctx.await_result = null; {}(&test_ctx) catch |e| {{ if (e != error.TestFailed) {{ @import(\"std\").debug.print(\"❌ Panic: {{any}}\\n\", .{{e}}); __cap_test_failed += 1; }} }}; if (__cap_test_failed == __start_fails) {{ @import(\"std\").debug.print(\"✔ PASS  {{s}}\\n\", .{{{}}}); __cap_test_passed += 1; }} else {{ @import(\"std\").debug.print(\"❌ FAIL  {{s}}\\n\", .{{{}}}); }} break :blk {{}}; }})", name, lambda, lambda, name, name);
                }
                if callee == "expect_eq" && call.args.len() == 2 {
                    let a = self.generate_expr(&call.args[0]);
                    let b = self.generate_expr(&call.args[1]);
                    return format!("(blk: {{ const _a = {}; const _b = {}; const is_eq = cp_eq(_a, _b); if (!is_eq) {{ @import(\"std\").debug.print(\"\\n    ❌ Assertion Failed!\\n    Expected: {{any}}\\n    Got:      {{any}}\\n\", .{{ _b, _a }}); __cap_test_failed += 1; return error.TestFailed; }} break :blk {{}}; }})", a, b);
                }
                if callee == "serve" && !call.args.is_empty() {
                    return self.generate_http_serve(&call.args[0]);
                }
                if callee == "json" && call.args.len() == 1 {
                    let arg = self.generate_expr(&call.args[0]);
                    return format!(
                        r#"(blk: {{
    const _val = {};
    const _fmt = if (@typeInfo(@TypeOf(_val)) == .pointer) _val.* else _val;
    const _raw_json = @import("std").fmt.allocPrint(std.heap.page_allocator, "{{f}}", .{{@import("std").json.fmt(_fmt, .{{}})}}) catch unreachable;
    const _res = Response._create(std.heap.page_allocator) catch unreachable;
    _res.init(_raw_json);
    _res.contentType = "application/json";
    break :blk _res;
}})"#,
                        arg
                    );
                }

                if callee == "next" {
                    if let Some(next_fn) = self.current_next_callee.clone() {
                        let mut args_str = String::new();
                        if call.args.is_empty() {
                            for p in &self.current_forward_params {
                                if !args_str.is_empty() {
                                    args_str.push_str(", ");
                                }
                                args_str.push_str(p);
                            }
                        } else {
                            for arg in &call.args {
                                if !args_str.is_empty() {
                                    args_str.push_str(", ");
                                }
                                args_str.push_str(&self.generate_expr(arg));
                            }
                        }
                        if self.is_class_method {
                            return format!("self.{}({})", next_fn, args_str);
                        } else if let Some(c) = &self.current_class_name {
                            return format!("{}.{}({})", c, next_fn, args_str);
                        } else {
                            return format!("{}({})", next_fn, args_str);
                        }
                    }
                }

                // 4. Free function call: check Trait conformance and `mut` / `move` parameter modifiers
                let fn_sig_opt = self.fn_sigs.get(&callee).cloned();
                let mut args = String::new();
                for (idx, arg) in call.args.iter().enumerate() {
                    let mut arg_is_move_param = false;
                    let old_box = self.expected_box_trait.clone();
                    if let Some(sig) = &fn_sig_opt {
                        if idx < sig.params.len() {
                            let param = &sig.params[idx];
                            if let Type::BoxDyn(tr_id) = &param.ty {
                                self.expected_box_trait = Some(tr_id.sym.clone());
                            }
                            if let Type::Ref(t_id) = &param.ty {
                                if self.trait_defs.contains_key(&t_id.sym) {
                                    let inner_arg = match arg {
                                        Expr::Move(inner) | Expr::Borrow(inner, _) => &**inner,
                                        other => other,
                                    };
                                    if let Expr::Ident(arg_id) = inner_arg {
                                        if let Some(arg_cls) =
                                            self.resolve_var_type_name(&arg_id.sym)
                                        {
                                            let implements = self
                                                .class_defs
                                                .get(&arg_cls)
                                                .map_or(false, |c| {
                                                    c.implements
                                                        .iter()
                                                        .any(|i| i.sym == t_id.sym)
                                                        || if let Some(tr) = self.trait_defs.get(&t_id.sym) {
                                                            tr.methods.iter().all(|tm| tm.default_body.is_some() || c.methods.iter().any(|m| m.ident.sym == tm.ident.sym))
                                                        } else {
                                                            false
                                                        }
                                                });
                                            if !implements {
                                                panic!(
                                                    "CP Compile Error: Type '{}' does not implement trait '{}'",
                                                    arg_cls, t_id.sym
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            if let Expr::Ident(arg_id) = arg {
                                if param.is_mut {
                                    if let Some((_, is_m)) = self.var_borrow_sources.get(&arg_id.sym)
                                    {
                                        if !*is_m {
                                            panic!(
                                                "CP Compile Error: Cannot pass read-only borrow '{}' to mutable parameter '{}'",
                                                arg_id.sym, param.ident.sym
                                            );
                                        }
                                    } else if self.current_fn_params.contains(&arg_id.sym)
                                        && !self.mutable_fn_params.contains(&arg_id.sym)
                                    {
                                        panic!(
                                            "CP Compile Error: Cannot pass read-only parameter '{}' to mutable parameter '{}'",
                                            arg_id.sym, param.ident.sym
                                        );
                                    }
                                    self.check_borrow_conflict(&arg_id.sym, true);
                                } else if param.is_move {
                                    if self.var_borrow_sources.contains_key(&arg_id.sym)
                                        || (self.current_fn_params.contains(&arg_id.sym)
                                            && !self.move_fn_params.contains(&arg_id.sym))
                                    {
                                        panic!(
                                            "CP Compile Error: Cannot move borrowed reference or read-only parameter '{}' into consuming parameter '{}'",
                                            arg_id.sym, param.ident.sym
                                        );
                                    }
                                    self.check_active_borrows_before_move(&arg_id.sym);
                                    arg_is_move_param = true;
                                } else {
                                    self.check_borrow_conflict(&arg_id.sym, false);
                                }
                            }
                        }
                    }
                    let mut gen_arg = self.generate_expr(arg);
                    if gen_arg.starts_with("lambda_") {
                        gen_arg = format!("@ptrCast(@constCast(&{}))", gen_arg);
                    }
                    self.expected_box_trait = old_box;
                    if arg_is_move_param {
                        if let Expr::Ident(arg_id) = arg {
                            self.moved_vars.insert(arg_id.sym.clone());
                            if !self.is_state_machine && self.drop_flag_vars.contains(&arg_id.sym) {
                                args.push_str(&format!(
                                    "(blk: {{ __{}_moved = true; break :blk {}; }})",
                                    arg_id.sym, gen_arg
                                ));
                            } else {
                                args.push_str(&gen_arg);
                            }
                        } else {
                            args.push_str(&gen_arg);
                        }
                    } else {
                        args.push_str(&gen_arg);
                    }
                    if idx < call.args.len() - 1 {
                        args.push_str(", ");
                    }
                }
                let call_res = if call.is_comptime {
                    format!("comptime {}({})", callee, args)
                } else {
                    format!("{}({})", callee, args)
                };
                self.maybe_wrap_try_call(
                    call_res,
                    fn_sig_opt.as_ref().and_then(|s| s.return_type.as_ref()),
                )
            }
            Expr::Question(inner) => {
                let old_in_try = self.in_try_region;
                self.in_try_region = false;
                let inner_ty = self.infer_cp_type(inner);
                let exp = self.generate_expr(inner);
                self.in_try_region = old_in_try;

                if self.is_lib_mode && self.try_catch_target.is_none() {
                    return format!("try {}", exp);
                }

                let id = self.lambda_counter;
                self.lambda_counter += 1;

                match inner_ty {
                    Some(Type::ResultType { ok, err: src_err }) => {
                        if self.is_lib_mode && self.try_catch_target.is_none() {
                            return format!("try {}", exp);
                        }
                        let is_fn_native = match &self.current_fn_ret_ty {
                            Some(Type::ResultType { err: fn_err, .. }) => {
                                let s = self.map_error_type(fn_err);
                                self.is_lib_mode || s.contains('.') || s == "anyerror" || self.error_sets.contains_key(&s)
                            }
                            Some(Type::ErrorUnion(_)) => true,
                            _ => false,
                        };

                        let err_stmt = if let Some((blk_label, err_var, flag_var, target_err)) =
                            self.try_catch_target.clone()
                        {
                            self.verify_error_type_compatibility(&src_err, &target_err);
                            let tgt_zig = self.map_error_type(&target_err);
                            format!(
                                "const __e_{id} = cp_unwrap_err_val(__r_{id}); {err_var} = cp_convert_err({tgt_zig}, __e_{id}); {flag_var} = true; break :{blk_label};"
                            )
                        } else if let Some(Type::ResultType { err: fn_err, .. }) =
                            self.current_fn_ret_ty.clone()
                        {
                            self.verify_error_type_compatibility(&src_err, &fn_err);
                            let tgt_zig = self.map_error_type(&fn_err);
                            if is_fn_native {
                                format!("const __e_{id} = cp_unwrap_err_val(__r_{id}); return cp_convert_err({tgt_zig}, __e_{id});")
                            } else {
                                format!("const __e_{id} = cp_unwrap_err_val(__r_{id}); return .{{ .err = cp_convert_err({tgt_zig}, __e_{id}) }};")
                            }
                        } else if is_fn_native {
                            format!("const __e_{id} = cp_unwrap_err_val(__r_{id}); return __e_{id};")
                        } else {
                            panic!("CP Compile Error: Operator '?' cannot be used on '!Error' expression inside a function that does not return '!Error'");
                        };

                        let ok_handling = if matches!(*ok, Type::Optional(_))
                            && self.try_catch_target.is_none()
                        {
                            match &self.current_fn_ret_ty {
                                Some(Type::ResultType { ok: fn_ok, .. })
                                    if matches!(&**fn_ok, Type::Optional(_)) =>
                                {
                                    format!(
                                        "const __v_{id} = cp_unwrap_ok_val(__r_{id}); if (__v_{id} == null) return .{{ .ok = null }}; break :blk_q_{id} __v_{id}.?;"
                                    )
                                }
                                Some(Type::Optional(_)) => {
                                    format!(
                                        "const __v_{id} = cp_unwrap_ok_val(__r_{id}); if (__v_{id} == null) return null; break :blk_q_{id} __v_{id}.?;"
                                    )
                                }
                                _ => format!("break :blk_q_{id} cp_unwrap_ok_val(__r_{id});"),
                            }
                        } else {
                            format!("break :blk_q_{id} cp_unwrap_ok_val(__r_{id});")
                        };

                        format!(
                            "(blk_q_{id}: {{ const __r_{id} = {exp}; if (cp_has_ok_val(__r_{id})) {{ {ok_handling} }} {err_stmt} }})"
                        )
                    }
                    Some(Type::Optional(_)) => {
                        let null_ret = match &self.current_fn_ret_ty {
                            Some(Type::ResultType { ok: fn_ok, .. })
                                if matches!(&**fn_ok, Type::Optional(_)) =>
                            {
                                "return .{ .ok = null };"
                            }
                            Some(Type::Optional(_)) => "return null;",
                            _ => panic!(
                                "CP Compile Error: Operator '?' cannot bubble 'null' from '?T' inside a function that does not return an optional '?U' type"
                            ),
                        };
                        format!(
                            "(blk_q_{id}: {{ const __v_{id} = {exp}; if (__v_{id} == null) {null_ret} break :blk_q_{id} __v_{id}.?; }})"
                        )
                    }
                    _ => format!("try {}", exp),
                }
            }
            Expr::QuestionFallback {
                expr: inner,
                fallback,
                is_return_fallback,
            } => {
                let old_in_try = self.in_try_region;
                self.in_try_region = false;
                let inner_ty = self.infer_cp_type(inner);
                let exp = self.generate_expr(inner);
                self.in_try_region = old_in_try;
                let id = self.lambda_counter;
                self.lambda_counter += 1;

                let is_inner_optional = matches!(inner_ty, Some(Type::Optional(_)));

                if *is_return_fallback || matches!(&**fallback, Expr::Return(_, _)) {
                    let ret_inner = match &**fallback {
                        Expr::Return(_, val) => val.as_deref(),
                        other => Some(other),
                    };
                    let ret_stmt = self.build_ret_stmt_for_fallback(ret_inner);

                    if self.is_lib_mode {
                        if is_inner_optional {
                            format!("(({exp}) orelse {{ {ret_stmt} }})")
                        } else {
                            format!("(({exp}) catch {{ {ret_stmt} }})")
                        }
                    } else if is_inner_optional {
                        format!(
                            "(blk_qf_{id}: {{ const __v_{id} = {exp}; if (__v_{id} == null) {{ {ret_stmt} }} break :blk_qf_{id} __v_{id}.?; }})"
                        )
                    } else {
                        format!(
                            "(blk_qf_{id}: {{ const __r_{id} = {exp}; if (cp_has_ok_val(__r_{id})) break :blk_qf_{id} cp_unwrap_ok_val(__r_{id}); {ret_stmt} }})"
                        )
                    }
                } else if matches!(&**fallback, Expr::Continue(_)) {
                    if self.is_lib_mode {
                        if is_inner_optional {
                            format!("(({exp}) orelse continue)")
                        } else {
                            format!("(({exp}) catch continue)")
                        }
                    } else if is_inner_optional {
                        format!(
                            "(blk_qf_{id}: {{ const __v_{id} = {exp}; if (__v_{id} == null) continue; break :blk_qf_{id} __v_{id}.?; }})"
                        )
                    } else {
                        format!(
                            "(blk_qf_{id}: {{ const __r_{id} = {exp}; if (cp_has_ok_val(__r_{id})) break :blk_qf_{id} cp_unwrap_ok_val(__r_{id}); continue; }})"
                        )
                    }
                } else if let Expr::Break(_, val) = &**fallback {
                    let brk_action = if let Some(v) = val {
                        format!("break {}", self.generate_expr(v))
                    } else {
                        "break".to_string()
                    };
                    if self.is_lib_mode {
                        if is_inner_optional {
                            format!("(({exp}) orelse {brk_action})")
                        } else {
                            format!("(({exp}) catch {brk_action})")
                        }
                    } else if is_inner_optional {
                        format!(
                            "(blk_qf_{id}: {{ const __v_{id} = {exp}; if (__v_{id} == null) {brk_action}; break :blk_qf_{id} __v_{id}.?; }})"
                        )
                    } else {
                        format!(
                            "(blk_qf_{id}: {{ const __r_{id} = {exp}; if (cp_has_ok_val(__r_{id})) break :blk_qf_{id} cp_unwrap_ok_val(__r_{id}); {brk_action}; }})"
                        )
                    }
                } else {
                    let fb_c = self.generate_expr(fallback);
                    if self.is_lib_mode {
                        if is_inner_optional {
                            format!("(({exp}) orelse {fb_c})")
                        } else {
                            format!("(({exp}) catch {fb_c})")
                        }
                    } else if is_inner_optional {
                        format!(
                            "(blk_qf_{id}: {{ const __v_{id} = {exp}; if (__v_{id} == null) break :blk_qf_{id} cp_val({fb_c}); break :blk_qf_{id} __v_{id}.?; }})"
                        )
                    } else {
                        format!(
                            "(blk_qf_{id}: {{ const __r_{id} = {exp}; if (cp_has_ok_val(__r_{id})) break :blk_qf_{id} cp_unwrap_ok_val(__r_{id}); break :blk_qf_{id} cp_val({fb_c}); }})"
                        )
                    }
                }
            }
            Expr::NullCoalesce { left, right } => {
                let old_in_try = self.in_try_region;
                self.in_try_region = false;
                let l_ty = self.infer_cp_type(left);
                if matches!(
                    l_ty,
                    Some(Type::ResultType { .. }) | Some(Type::ErrorUnion(_))
                ) {
                    panic!(
                        "CP Compile Error: Operator '??' can only be used on optional '?T' for null coalescing; for '!Error' types use '? default' or 'catch'"
                    );
                }
                let l_code = self.generate_expr(left);
                self.in_try_region = old_in_try;

                match &**right {
                    Expr::Continue(_) => {
                        format!("(({}) orelse continue)", l_code)
                    }
                    Expr::Break(_, val) => {
                        if let Some(v) = val {
                            let v_code = self.generate_expr(v);
                            format!("(({}) orelse break {})", l_code, v_code)
                        } else {
                            format!("(({}) orelse break)", l_code)
                        }
                    }
                    Expr::LabeledBreak(_, label, val) => {
                        if let Some(v) = val {
                            let v_code = self.generate_expr(v);
                            format!("(({}) orelse break :{} {})", l_code, label.sym, v_code)
                        } else {
                            format!("(({}) orelse break :{})", l_code, label.sym)
                        }
                    }
                    Expr::Return(_, ret_expr) => {
                        let ret_stmt = self.build_ret_stmt_for_fallback(ret_expr.as_deref());
                        format!("(({}) orelse {{ {} }})", l_code, ret_stmt)
                    }
                    _ => {
                        let r_code = self.generate_expr(right);
                        if self.is_lib_mode {
                            format!("(({}) orelse ({}))", l_code, r_code)
                        } else {
                            format!("cp_null_coalesce({}, {})", l_code, r_code)
                        }
                    }
                }
            }
            Expr::Catch {
                expr: inner,
                err_ident,
                body,
                ..
            } => {
                let old_in_try = self.in_try_region;
                self.in_try_region = false;
                let inner_ty = self.infer_cp_type(inner);
                let exp = self.generate_expr(inner);
                self.in_try_region = old_in_try;
                let id = self.lambda_counter;
                self.lambda_counter += 1;
                let blk_label = format!("blk_catch_{}", id);
                let err_bind = err_ident
                    .as_ref()
                    .map(|i| i.sym.clone())
                    .unwrap_or_else(|| format!("__ce_{}", id));

                let old_vars = self.var_cp_types.clone();
                let old_locals = self.locals.clone();
                if let Some(Type::ResultType { err, .. }) = &inner_ty {
                    if let Some(eid) = err_ident {
                        self.var_cp_types
                            .insert(eid.sym.clone(), self.error_type_to_cp_type(err));
                        self.locals
                            .insert(eid.sym.clone(), self.map_error_type(err));
                    }
                }

                let target_payload_ty = match &inner_ty {
                    Some(Type::ErrorUnion(t)) => Some(self.map_type(t)),
                    Some(Type::ResultType { ok, .. }) => Some(self.map_type(ok)),
                    _ => None,
                };

                let catch_arm_body = match &**body {
                    Stmt::Expr(e) => {
                        if matches!(e, Expr::Bin(bin) if bin.op == BinaryOp::Assign) {
                            let saved = self.current_case_content.clone();
                            self.current_case_content.clear();
                            self.generate_stmt(body);
                            let default_val = if let Some(ty_str) = &target_payload_ty {
                                match ty_str.as_str() {
                                    "bool" => "false",
                                    "void" => "{}",
                                    s if s.starts_with('?') => "null",
                                    _ => "undefined",
                                }
                            } else {
                                "{}"
                            };
                            let brk = if let Some(ty_str) = &target_payload_ty {
                                format!("break :{} @as({}, {});", blk_label, ty_str, default_val)
                            } else {
                                format!("break :{} {};", blk_label, default_val)
                            };
                            self.write_to_case(&brk);
                            let code = self.current_case_content.trim().to_string();
                            self.current_case_content = saved;
                            code
                        } else {
                            let ec = self.generate_expr(e);
                            if ec == "unreachable" || ec.starts_with("@panic") || ec.starts_with("@compileError") || ec.starts_with("break") || ec.starts_with("return") || ec.starts_with("continue") {
                                format!("{};", ec)
                            } else if let Some(ty_str) = &target_payload_ty {
                                format!("break :{} @as({}, {});", blk_label, ty_str, ec)
                            } else if self.is_lib_mode {
                                format!("break :{} {};", blk_label, ec)
                            } else {
                                format!("break :{} cp_val({});", blk_label, ec)
                            }
                        }
                    }
                    Stmt::Return(_) => {
                        let saved = self.current_case_content.clone();
                        self.current_case_content.clear();
                        self.generate_stmt(body);
                        let ret_s = self.current_case_content.trim().to_string();
                        self.current_case_content = saved;
                        ret_s
                    }
                    Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                        let saved = self.current_case_content.clone();
                        self.current_case_content.clear();
                        let len = b.body.len();
                        for (idx, s) in b.body.iter().enumerate() {
                            if idx + 1 == len {
                                if let Stmt::Expr(e) = s {
                                    if matches!(e, Expr::Bin(bin) if bin.op == BinaryOp::Assign) {
                                        self.generate_stmt(s);
                                        if let Some(ty_str) = &target_payload_ty {
                                            let default_val = match ty_str.as_str() {
                                                "bool" => "false",
                                                "void" => "{}",
                                                s if s.starts_with('?') => "null",
                                                _ => "undefined",
                                            };
                                            self.write_to_case(&format!(
                                                "break :{} @as({}, {});",
                                                blk_label, ty_str, default_val
                                            ));
                                        } else if self.is_lib_mode {
                                            self.write_to_case(&format!("break :{} {{}};", blk_label));
                                        } else {
                                            self.write_to_case(&format!("break :{} cp_val({{}});", blk_label));
                                        }
                                        continue;
                                    }
                                    let ec = self.generate_expr(e);
                                    if ec == "unreachable" || ec.starts_with("@panic") || ec.starts_with("@compileError") {
                                        self.write_to_case(&format!("{};", ec));
                                    } else if let Some(ty_str) = &target_payload_ty {
                                        self.write_to_case(&format!(
                                            "break :{} @as({}, {});",
                                            blk_label, ty_str, ec
                                        ));
                                    } else if self.is_lib_mode {
                                        self.write_to_case(&format!(
                                            "break :{} {};",
                                            blk_label, ec
                                        ));
                                    } else {
                                        self.write_to_case(&format!(
                                            "break :{} cp_val({});",
                                            blk_label, ec
                                        ));
                                    }
                                } else {
                                    self.generate_stmt(s);
                                }
                            } else {
                                self.generate_stmt(s);
                            }
                        }
                        let blk_s = self.current_case_content.trim().to_string();
                        self.current_case_content = saved;
                        blk_s
                    }
                    _ => {
                        let saved = self.current_case_content.clone();
                        self.current_case_content.clear();
                        self.generate_stmt(body);
                        let s_str = self.current_case_content.trim().to_string();
                        self.current_case_content = saved;
                        s_str
                    }
                };

                self.var_cp_types = old_vars;
                self.locals = old_locals;

                if matches!(inner_ty, Some(Type::ErrorUnion(_))) || self.is_lib_mode || inner_ty.is_none() {
                    if catch_arm_body == "unreachable;" && err_ident.is_none() {
                        return format!("(({exp}) catch unreachable)");
                    }
                    return format!(
                        "({blk_label}: {{ break :{blk_label} (({exp}) catch |{err_bind}| {{ _ = &{err_bind}; {catch_arm_body} }}); }})"
                    );
                }

                format!(
                    "({blk_label}: {{ const __r_{id} = {exp}; switch (__r_{id}) {{ .ok => |__v_{id}| break :{blk_label} __v_{id}, .err => |{err_bind}| {{ _ = &{err_bind}; {catch_arm_body} }} }} }})"
                )
            }
            Expr::ErrWrap(inner) => self.generate_err_value(inner, None),
            Expr::BoxNew(inner) => {
                let trait_name = self.expected_box_trait.clone().unwrap_or_else(|| {
                    panic!("CP Compile Error: Box.new(...) requires a known Box<dyn Trait> target type")
                });
                let inner_unwrapped = match &**inner {
                    Expr::Move(mv) => &**mv,
                    other => other,
                };
                let cls_name = match inner_unwrapped {
                    Expr::Ident(id) => {
                        if self.moved_vars.contains(&id.sym) {
                            panic!("CP Compile Error: Use of moved value '{}'", id.sym);
                        }
                        if self.var_borrow_sources.contains_key(&id.sym)
                            || (self.current_fn_params.contains(&id.sym)
                                && !self.move_fn_params.contains(&id.sym))
                        {
                            panic!(
                                "CP Compile Error: Box.new() requires ownership of '{}'; cannot box a borrowed reference",
                                id.sym
                            );
                        }
                        self.check_active_borrows_before_move(&id.sym);
                        self.resolve_var_type_name(&id.sym).unwrap_or_else(|| {
                            panic!(
                                "CP Compile Error: Cannot determine concrete type of '{}' for Box.new()",
                                id.sym
                            )
                        })
                    }
                    Expr::New(nc) => match &*nc.callee {
                        Expr::Ident(cid) => cid.sym.clone(),
                        _ => panic!("CP Compile Error: Invalid New expression inside Box.new()"),
                    },
                    other => {
                        if let Some(Type::Ref(cid)) = self.infer_cp_type(other) {
                            cid.sym.clone()
                        } else {
                            panic!("CP Compile Error: Cannot determine concrete type for Box.new()")
                        }
                    }
                };
                let implements = self
                    .class_defs
                    .get(&cls_name)
                    .map_or(false, |c| c.implements.iter().any(|i| i.sym == trait_name));
                if !implements {
                    panic!(
                        "CP Compile Error: Type '{}' does not implement trait '{}' required by Box<dyn {}>",
                        cls_name, trait_name, trait_name
                    );
                }
                let inner_code = self.generate_expr(inner);
                let move_stmt = if let Expr::Ident(id) = &**inner {
                    self.moved_vars.insert(id.sym.clone());
                    if !self.is_state_machine && self.drop_flag_vars.contains(&id.sym) {
                        format!("__{}_moved = true; ", id.sym)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let id = self.lambda_counter;
                self.lambda_counter += 1;
                format!(
                    "(blk_box_{id}: {{ const __b_ptr_{id} = {inner_code}; {move_stmt}break :blk_box_{id} {trait_name}{{ .ptr = @ptrCast(__b_ptr_{id}), .vtable = &vtable_{trait_name}_{cls_name} }}; }})"
                )
            }
            Expr::Downcast {
                obj,
                target_ty,
                is_mut,
            } => {
                if let Expr::Ident(id) = &**obj {
                    if *is_mut {
                        if let Some((_, is_m)) = self.var_borrow_sources.get(&id.sym) {
                            if !*is_m {
                                panic!(
                                    "CP Compile Error: Cannot take mutable downcast borrow from read-only borrow '{}'",
                                    id.sym
                                );
                            }
                        } else if self.current_fn_params.contains(&id.sym)
                            && !self.mutable_fn_params.contains(&id.sym)
                        {
                            panic!(
                                "CP Compile Error: Cannot take mutable downcast borrow from read-only parameter '{}'",
                                id.sym
                            );
                        }
                    }
                    self.check_borrow_conflict(&id.sym, *is_mut);
                }
                let obj_code = self.generate_expr(obj);
                format!("{}.downcast({})", obj_code, target_ty.sym)
            }
            Expr::EnumInit(ei) => {
                let fields_slice = ei.fields.as_deref().unwrap_or(&[]);
                let tuple_slice = ei.tuple_args.as_deref().unwrap_or(&[]);
                if ei.variant_name.sym.is_empty() && !ei.enum_name.sym.is_empty() {
                    let mut target_sym = ei.enum_name.sym.clone();
                    while let Some(ta) = self.type_aliases.get(&target_sym) {
                        if let Type::Ref(ref tid) = ta.target_ty {
                            target_sym = tid.sym.clone();
                        } else {
                            break;
                        }
                    }
                    let base_name = target_sym.split('<').next().unwrap_or(&target_sym);
                    let cls_opt = self.class_defs.get(base_name).cloned();
                    if let Some(ref cls) = cls_opt {
                        if cls.is_static {
                            panic!("CP Compile Error: Cannot instantiate static class or struct '{}'", ei.enum_name.sym);
                        }
                    }
                    let mut subst_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                    if let Some(pos) = target_sym.find('<') {
                        if target_sym.ends_with('>') {
                            let args_str = &target_sym[pos + 1..target_sym.len() - 1];
                            let parts = Self::split_generic_args_str(args_str);
                            if let Some(ref cls) = cls_opt {
                                for (gp, arg) in cls.generic_params.iter().zip(parts.iter()) {
                                    let param_name = match gp {
                                        GenericParam::Type { ident, .. } => &ident.sym,
                                        GenericParam::Const { ident, .. } => &ident.sym,
                                    };
                                    subst_map.insert(param_name.clone(), arg.clone());
                                }
                            }
                        }
                    }
                    let mapped_name = self.map_type_name(&ei.enum_name.sym);
                    let mut s = format!("{}{{ ", mapped_name);
                    for (idx, f) in fields_slice.iter().enumerate() {
                        let old_array_elem = self.expected_array_elem_type.clone();
                        if let Some(ref cls) = cls_opt {
                            if let Some(cf) = cls.fields.iter().find(|fl| fl.ident.sym == f.key.sym) {
                                if let Type::FixedArray(elem, _) | Type::Array(elem) = &cf.ty {
                                    let mut el = (**elem).clone();
                                    if let Type::Ref(ref mut id) = el {
                                        if let Some(sub) = subst_map.get(&id.sym) {
                                            id.sym = sub.clone();
                                        }
                                    }
                                    self.expected_array_elem_type = Some(el);
                                }
                            }
                        }
                        let val_code = self.generate_expr(&f.val);
                        self.expected_array_elem_type = old_array_elem;
                        s.push_str(&format!(".{} = {}", f.key.sym, val_code));
                        if idx + 1 < fields_slice.len() {
                            s.push_str(", ");
                        }
                    }
                    s.push_str(" }");
                    return s;
                }
                if ei.enum_name.sym.is_empty() {
                    if !tuple_slice.is_empty() {
                        let is_expected_tuple = self.enum_defs.values().any(|en| {
                            en.variants.iter().any(|v| {
                                &v.ident.sym == &ei.variant_name.sym
                                    && matches!(v.tuple_type, Some(Type::Tuple(_)))
                            })
                        });
                        if tuple_slice.len() == 1 {
                            if is_expected_tuple {
                                if let Expr::Array(items) = &tuple_slice[0] {
                                    let args_str: Vec<String> =
                                        items.iter().map(|a| self.generate_expr(a)).collect();
                                    format!(
                                        ".{{ .{} = .{{ {} }} }}",
                                        ei.variant_name.sym,
                                        args_str.join(", ")
                                    )
                                } else {
                                    format!(
                                        ".{{ .{} = {} }}",
                                        ei.variant_name.sym,
                                        self.generate_expr(&tuple_slice[0])
                                    )
                                }
                            } else {
                                format!(
                                    ".{{ .{} = {} }}",
                                    ei.variant_name.sym,
                                    self.generate_expr(&tuple_slice[0])
                                )
                            }
                        } else {
                            let args_str: Vec<String> =
                                tuple_slice.iter().map(|a| self.generate_expr(a)).collect();
                            format!(
                                ".{{ .{} = .{{ {} }} }}",
                                ei.variant_name.sym,
                                args_str.join(", ")
                            )
                        }
                    } else if fields_slice.is_empty() {
                        if ei.variant_name.sym.is_empty() {
                            ".{}".to_string()
                        } else {
                            format!(".{}", ei.variant_name.sym)
                        }
                    } else {
                        let mut f_str = String::new();
                        for (idx, f) in fields_slice.iter().enumerate() {
                            f_str.push_str(&format!(
                                ".{} = {}",
                                f.key.sym,
                                self.generate_expr(&f.val)
                            ));
                            if idx + 1 < fields_slice.len() {
                                f_str.push_str(", ");
                            }
                        }
                        if ei.variant_name.sym.is_empty() {
                            format!(".{{ {} }}", f_str)
                        } else {
                            format!(".{{ .{} = .{{ {} }} }}", ei.variant_name.sym, f_str)
                        }
                    }
                } else if !tuple_slice.is_empty() {
                    let is_enum = self.enum_defs.contains_key(&ei.enum_name.sym);
                    if !is_enum {
                        let args_str: Vec<String> =
                            tuple_slice.iter().map(|a| self.generate_expr(a)).collect();
                        format!(
                            "{}.{}({})",
                            ei.enum_name.sym,
                            ei.variant_name.sym,
                            args_str.join(", ")
                        )
                    } else {
                        let is_expected_tuple = self.enum_defs.get(&ei.enum_name.sym).map_or(false, |en| {
                            en.variants.iter().any(|v| {
                                &v.ident.sym == &ei.variant_name.sym
                                    && matches!(v.tuple_type, Some(Type::Tuple(_)))
                            })
                        });
                        if tuple_slice.len() == 1 {
                            if is_expected_tuple {
                                if let Expr::Array(items) = &tuple_slice[0] {
                                    let args_str: Vec<String> =
                                        items.iter().map(|a| self.generate_expr(a)).collect();
                                    format!(
                                        "{}{{ .{} = .{{ {} }} }}",
                                        ei.enum_name.sym,
                                        ei.variant_name.sym,
                                        args_str.join(", ")
                                    )
                                } else {
                                    format!(
                                        "{}{{ .{} = {} }}",
                                        ei.enum_name.sym,
                                        ei.variant_name.sym,
                                        self.generate_expr(&tuple_slice[0])
                                    )
                                }
                            } else {
                                format!(
                                    "{}{{ .{} = {} }}",
                                    ei.enum_name.sym,
                                    ei.variant_name.sym,
                                    self.generate_expr(&tuple_slice[0])
                                )
                            }
                        } else {
                            let args_str: Vec<String> =
                                tuple_slice.iter().map(|a| self.generate_expr(a)).collect();
                            format!(
                                "{}{{ .{} = .{{ {} }} }}",
                                ei.enum_name.sym,
                                ei.variant_name.sym,
                                args_str.join(", ")
                            )
                        }
                    }
                } else if fields_slice.is_empty() {
                    let is_enum = self.enum_defs.contains_key(&ei.enum_name.sym);
                    if !is_enum {
                        format!("{}.{}", ei.enum_name.sym, ei.variant_name.sym)
                    } else {
                        let is_pure_enum = self.enum_defs.get(&ei.enum_name.sym).map_or(false, |en| {
                            en.variants.iter().all(|v| {
                                v.fields.is_none() && v.tuple_type.is_none() && v.from_err.is_none()
                            })
                        });
                        if is_pure_enum {
                            format!("{}.{}", ei.enum_name.sym, ei.variant_name.sym)
                        } else {
                            format!("{}{{ .{} = {{}} }}", ei.enum_name.sym, ei.variant_name.sym)
                        }
                    }
                } else {
                    let mut f_str = String::new();
                    for (idx, f) in fields_slice.iter().enumerate() {
                        f_str.push_str(&format!(".{} = {}", f.key.sym, self.generate_expr(&f.val)));
                        if idx + 1 < fields_slice.len() {
                            f_str.push_str(", ");
                        }
                    }
                    if ei.variant_name.sym.is_empty() {
                        format!("{}{{ {} }}", ei.enum_name.sym, f_str)
                    } else {
                        format!(
                            "{}{{ .{} = .{{ {} }} }}",
                            ei.enum_name.sym, ei.variant_name.sym, f_str
                        )
                    }
                }
            }
            Expr::OptMember(mem) => {
                let obj_code = self.generate_expr(&mem.obj);
                let prop = match mem.prop.sym.as_str() {
                    "length" => "len",
                    other => other,
                };
                let id = self.lambda_counter;
                self.lambda_counter += 1;
                format!(
                    "(blk_om_{id}: {{ const __o_{id} = {obj_code}; const __R_{id} = CpOptType(@TypeOf(cp_opt_unwrap(__o_{id}).{prop})); if (cp_opt_is_null(__o_{id})) break :blk_om_{id} @as(__R_{id}, null); break :blk_om_{id} @as(__R_{id}, cp_opt_unwrap(__o_{id}).{prop}); }})"
                )
            }
            Expr::OptCall(call) => {
                if let Expr::OptMember(mem) | Expr::Member(mem) = &*call.callee {
                    let obj_code = self.generate_expr(&mem.obj);
                    let prop = &mem.prop.sym;
                    let mut args = String::new();
                    for (idx, arg) in call.args.iter().enumerate() {
                        args.push_str(&self.generate_expr(arg));
                        if idx + 1 < call.args.len() {
                            args.push_str(", ");
                        }
                    }
                    let id = self.lambda_counter;
                    self.lambda_counter += 1;
                    format!(
                        "(blk_oc_{id}: {{ const __o_{id} = {obj_code}; const __R_{id} = CpOptType(@TypeOf(cp_opt_unwrap(__o_{id}).{prop}({args}))); if (cp_opt_is_null(__o_{id})) break :blk_oc_{id} @as(__R_{id}, null); break :blk_oc_{id} @as(__R_{id}, cp_opt_unwrap(__o_{id}).{prop}({args})); }})"
                    )
                } else {
                    "null".to_string()
                }
            }
            Expr::Match(m) => self.generate_switch(m, true),
            Expr::If {
                test,
                cons,
                alt,
                is_comptime,
                ..
            } => {
                let not_null_vars = Self::extract_not_null_vars(test);
                let is_null_vars = Self::extract_is_null_vars(test);
                let test_s = self.generate_expr(test);
                let old_narrowed = self.narrowed_null_vars.clone();
                self.narrowed_null_vars.extend(not_null_vars);
                let cons_s = self.generate_expr(cons);
                self.narrowed_null_vars = old_narrowed.clone();
                let alt_s = if let Some(a) = alt {
                    self.narrowed_null_vars.extend(is_null_vars);
                    let res = self.generate_expr(a);
                    self.narrowed_null_vars = old_narrowed;
                    res
                } else {
                    "null".to_string()
                };
                if *is_comptime {
                    format!("(if (comptime {}) {} else {})", test_s, cons_s, alt_s)
                } else {
                    format!("(if ({}) {} else {})", test_s, cons_s, alt_s)
                }
            }
            Expr::New(call) => {
                let raw_class_name = match &*call.callee {
                    Expr::Ident(id) => id.sym.clone(),
                    _ => "Unknown".to_string(),
                };
                let class_name = self.map_type_name(&raw_class_name);
                let base_name = if let Some(pos) = raw_class_name.find('<') {
                    &raw_class_name[..pos]
                } else {
                    &raw_class_name
                };
                if let Some(cls) = self.class_defs.get(base_name) {
                    if cls.is_static {
                        panic!("CP Compile Error: Cannot instantiate static class or struct '{}' with 'new'", raw_class_name);
                    }
                }
                if class_name == "Error" {
                    let code = if !call.args.is_empty() {
                        self.generate_expr(&call.args[0])
                    } else {
                        "0".to_string()
                    };
                    let msg = if call.args.len() > 1 {
                        self.generate_expr(&call.args[1])
                    } else {
                        "\"\"".to_string()
                    };
                    return format!("Error.init({}, {})", code, msg);
                }
                let init_params: Vec<Param> = self
                    .lookup_method(base_name, "init")
                    .map(|(p, _, _)| {
                        p.into_iter()
                            .filter(|param| param.ident.sym != "this" && param.ident.sym != "_")
                            .collect()
                    })
                    .unwrap_or_default();
                let mut args = String::new();
                for (idx, arg) in call.args.iter().enumerate() {
                    let mut arg_is_move_param = false;
                    if idx < init_params.len() {
                        let param = &init_params[idx];
                        if let Expr::Ident(arg_id) = arg {
                            if param.is_mut {
                                self.check_borrow_conflict(&arg_id.sym, true);
                            } else if param.is_move {
                                self.check_active_borrows_before_move(&arg_id.sym);
                                arg_is_move_param = true;
                            } else {
                                self.check_borrow_conflict(&arg_id.sym, false);
                            }
                        }
                    }
                    let gen_arg = self.generate_expr(arg);
                    if arg_is_move_param {
                        if let Expr::Ident(arg_id) = arg {
                            self.moved_vars.insert(arg_id.sym.clone());
                            if !self.is_state_machine && self.drop_flag_vars.contains(&arg_id.sym) {
                                args.push_str(&format!(
                                    "(blk_arg: {{ __{}_moved = true; break :blk_arg {}; }})",
                                    arg_id.sym, gen_arg
                                ));
                            } else {
                                args.push_str(&gen_arg);
                            }
                        } else {
                            args.push_str(&gen_arg);
                        }
                    } else {
                        args.push_str(&gen_arg);
                    }
                    if idx < call.args.len() - 1 {
                        args.push_str(", ");
                    }
                }
                format!("(blk: {{ const _obj = {}._create({}) catch unreachable; if (@hasDecl({}, \"init\")) {{ const RType = @typeInfo(@TypeOf({}.init)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _obj.init({}) catch unreachable; }} else {{ _obj.init({}); }} }} break :blk _obj; }})", class_name, self.current_allocator(), class_name, class_name, args, args)
            }
            Expr::Await(inner) => {
                if let Expr::Call(call) = &**inner {
                    if let Expr::Ident(id) = &*call.callee {
                        if id.sym == "join" || id.sym == "@task_join" {
                            if self.is_state_machine {
                                let next_pc = self.pc_counter + 1;
                                let current_fn = if let Some(c) = &self.current_class_name {
                                    format!("{}.{}", c, self.current_fn_zig_name)
                                } else {
                                    self.current_zig_fn_name.clone()
                                };
                                let mut reg_tasks = String::new();
                                for (idx, arg) in call.args.iter().enumerate() {
                                    let arg_code = self.generate_expr(arg);
                                    reg_tasks.push_str(&format!(
                                        "const __t{idx}: *CpTask = @ptrCast(@alignCast({arg_code})); \
                                        __t{idx}.lock.acquire(); \
                                        if (__t{idx}.state == 2) {{ \
                                            const __res{idx} = __t{idx}.base.await_result orelse Message{{ .string = \"\" }}; \
                                            __t{idx}.lock.release(); \
                                            barrier.on_task_done({idx}, __res{idx}); \
                                        }} else {{ \
                                            __t{idx}.join_barrier = barrier; \
                                            __t{idx}.join_idx = {idx}; \
                                            __t{idx}.lock.release(); \
                                        }} "
                                    ));
                                }
                                let logic = format!(
                                    "const barrier = JoinBarrier.init(ctx.arena.?.allocator(), {0}) catch unreachable; \
                                    barrier.waiter_ctx = @ptrCast(ctx); \
                                    barrier.waiter_fn = @ptrCast(&{1}); \
                                    {2} \
                                    ctx.pc = {3}; \
                                    if (barrier.finish_setup_and_should_suspend()) return; \
                                    continue;",
                                    call.args.len(), current_fn, reg_tasks, next_pc
                                );
                                self.write_to_case(&logic);
                                self.next_state();
                                return "ctx.await_result".to_string();
                            }
                        }
                    }
                    if self.is_state_machine {
                        let (callee_init, callee_fn) = match &*call.callee {
                            Expr::Ident(id) => (id.sym.clone(), id.sym.clone()),
                            Expr::Member(mem) => {
                                let obj_type = self.infer_zig_type(None, Some(&mem.obj));
                                let base = obj_type.trim_start_matches('*');
                                let prop = &mem.prop;
                                (
                                    format!("{}.{}_{}", base, base, prop.sym),
                                    format!("{}.{}", base, prop.sym),
                                )
                            }
                            _ => ("Unknown".to_string(), "Unknown".to_string()),
                        };
                        let mut args_str = String::new();
                        if let Expr::Member(mem) = &*call.callee {
                            args_str.push_str(", ");
                            args_str.push_str(&self.generate_expr(&mem.obj));
                        }
                        for arg in &call.args {
                            args_str.push_str(", ");
                            args_str.push_str(&self.generate_expr(arg));
                        }
                        let next_pc = self.pc_counter + 1;
                        let current_fn = if let Some(c) = &self.current_class_name {
                            format!("{}.{}", c, self.current_fn_zig_name)
                        } else {
                            self.current_zig_fn_name.clone()
                        };
                        let logic = format!(
                            "const child_ctx = {}_init(ctx.arena.?, @ptrCast(ctx), @ptrCast(&{}){}); \
                            schedule(@ptrCast(&{}), @ptrCast(child_ctx)); \
                            ctx.pc = {}; \
                            return;",
                            callee_init, current_fn, args_str, callee_fn, next_pc
                        );
                        self.write_to_case(&logic);
                        self.next_state();

                        let inferred = self.infer_zig_type(None, Some(inner));
                        if inferred == "void" {
                            "(blk: { break :blk {}; })".to_string()
                        } else if inferred == "[]const u8" {
                            "ctx.await_result.?.string".to_string()
                        } else if inferred == "i32" || inferred == "usize" || inferred == "u32" {
                            "@intCast(ctx.await_result.?.int)".to_string()
                        } else if inferred.starts_with('*') {
                            format!(
                                "@as({}, @ptrCast(@alignCast(ctx.await_result.?.any)))",
                                inferred
                            )
                        } else if inferred.starts_with("struct {") {
                            format!(
                                "@as(*{}, @ptrCast(@alignCast(ctx.await_result.?.any))).*",
                                inferred
                            )
                        } else {
                            "ctx.await_result".to_string()
                        }
                    } else {
                        self.generate_expr(inner)
                    }
                } else if self.is_state_machine {
                    let task_code = self.generate_expr(inner);
                    let next_pc = self.pc_counter + 1;
                    let current_fn = if let Some(c) = &self.current_class_name {
                        format!("{}.{}", c, self.current_fn_zig_name)
                    } else {
                        self.current_zig_fn_name.clone()
                    };
                    let logic = format!(
                        "const __single_task: *CpTask = @ptrCast(@alignCast({0})); \
                        __single_task.lock.acquire(); \
                        if (__single_task.state == 2) {{ \
                            ctx.await_result = __single_task.base.await_result; \
                            __single_task.lock.release(); \
                            ctx.pc = {2}; \
                            continue; \
                        }} else {{ \
                            __single_task.waiter_ctx = @ptrCast(ctx); \
                            __single_task.waiter_fn = @ptrCast(&{1}); \
                            ctx.pc = {2}; \
                            __single_task.lock.release(); \
                            return; \
                        }}",
                        task_code, current_fn, next_pc
                    );
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.await_result".to_string()
                } else {
                    self.generate_expr(inner)
                }
            }
            Expr::Member(member) => {
                if let Expr::Ident(id) = &*member.obj {
                    if id.sym == "error" && member.prop.sym.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                        return format!("error.{}", member.prop.sym);
                    }
                    if id.sym == "Math" {
                        if member.prop.sym == "PI" {
                            return "@import(\"std\").math.pi".to_string();
                        }
                        if member.prop.sym == "E" {
                            return "@import(\"std\").math.e".to_string();
                        }
                    }
                    if let Some(en) = self.enum_defs.get(&id.sym) {
                        let is_pure_enum = en.variants.iter().all(|v| {
                            v.fields.is_none() && v.tuple_type.is_none() && v.from_err.is_none()
                        });
                        if is_pure_enum {
                            return format!("{}.{}", id.sym, member.prop.sym);
                        } else {
                            return format!("{}{{ .{} = {{}} }}", id.sym, member.prop.sym);
                        }
                    }
                    if self.error_sets.contains_key(&id.sym) {
                        return format!("{}.{}", id.sym, member.prop.sym);
                    }
                    if id.sym.contains('<') && id.sym.ends_with('>') {
                        let mapped = self.map_type_name(&id.sym);
                        return format!("{}.{}", mapped, member.prop.sym);
                    }
                }
                if matches!(&*member.obj, Expr::OptMember(_) | Expr::OptCall(_)) {
                    let obj_code = self.generate_expr(&member.obj);
                    let prop = match member.prop.sym.as_str() {
                        "length" => "len",
                        other => other,
                    };
                    let id = self.lambda_counter;
                    self.lambda_counter += 1;
                    return format!(
                        "(blk_om_{id}: {{ const __o_{id} = {obj_code}; const __R_{id} = CpOptType(@TypeOf(cp_opt_unwrap(__o_{id}).{prop})); if (cp_opt_is_null(__o_{id})) break :blk_om_{id} @as(__R_{id}, null); break :blk_om_{id} @as(__R_{id}, cp_opt_unwrap(__o_{id}).{prop}); }})"
                    );
                }
                let obj = self.generate_expr(&member.obj);
                let cp_ty = self.infer_cp_type(&member.obj);
                let obj_ty = self.infer_zig_type(None, Some(&member.obj));

                let is_array_or_string = match &cp_ty {
                    Some(Type::String) | Some(Type::Array(_)) | Some(Type::FixedArray(_, _)) => true,
                    _ => false,
                };
                let prop_str = if is_array_or_string && member.prop.sym == "length" {
                    "len".to_string()
                } else {
                    member.prop.sym.clone()
                };
                let prop = prop_str.as_str();
                let is_guard = obj_ty.contains("LockGuard(")
                    || obj_ty.contains("ReadGuard(")
                    || obj_ty.contains("WriteGuard(");
                let safe_prop = if matches!(prop, "and" | "or" | "orelse") {
                    format!("@\"{}\"", prop)
                } else {
                    prop.to_string()
                };
                if is_guard && prop != "release" && prop != "unlock" {
                    return if obj_ty.starts_with('?') {
                        format!("({}).?.data.{}", obj, safe_prop)
                    } else {
                        format!("{}.data.{}", obj, safe_prop)
                    };
                }

                let is_struct_or_class = match &cp_ty {
                    Some(Type::Ref(id)) => self.class_defs.contains_key(&id.sym),
                    Some(Type::BorrowType { inner, .. }) => match &**inner {
                        Type::Ref(id) => self.class_defs.contains_key(&id.sym),
                        _ => false,
                    },
                    _ => false,
                };

                let is_enum = match &cp_ty {
                    Some(Type::Ref(id)) => self.enum_defs.contains_key(&id.sym),
                    _ => false,
                };
                if !is_struct_or_class {
                    if is_enum && member.prop.sym == "name" {
                        return format!("@tagName({})", obj);
                    }
                    if is_enum && member.prop.sym == "value" {
                        return format!("@intFromEnum({})", obj);
                    }
                    if member.prop.sym == "name" {
                        return format!("(if (comptime @typeInfo(@TypeOf({0})) == .@\"enum\" or @typeInfo(@TypeOf({0})) == .@\"union\") @tagName({0}) else {0}.name)", obj);
                    }
                    if member.prop.sym == "value" {
                        return format!("(if (comptime @typeInfo(@TypeOf({0})) == .@\"enum\") @intFromEnum({0}) else {0}.value)", obj);
                    }
                }

                let is_cstr = matches!(cp_ty, Some(Type::CStr)) || obj_ty.contains("[*:0]");
                if is_cstr && (member.prop.sym == "len" || member.prop.sym == "length") {
                    return format!("std.mem.span({}).len", obj);
                }

                if obj_ty.starts_with('?') {
                    format!("({}).?.{}", obj, safe_prop)
                } else if obj_ty.starts_with("*[]") {
                    format!("{}.*.{}", obj, safe_prop)
                } else {
                    format!("{}.{}", obj, safe_prop)
                }
            }
            Expr::Builtin(call) => match call.name.as_str() {
                "@hash" => {
                    if call.args.is_empty() {
                        "0".to_string()
                    } else if call.args.len() == 1 {
                        format!("cp_hash_val({})", self.generate_expr(&call.args[0]))
                    } else {
                        let mut block = String::from("(blk: { var _h: usize = 2166136261; ");
                        for arg in &call.args {
                            let gen = self.generate_expr(arg);
                            block.push_str(&format!("_h = (_h *% 16777619) ^ cp_hash_val({}); ", gen));
                        }
                        block.push_str("break :blk _h; })");
                        block
                    }
                }
                "@not" => {
                    let inner = self.generate_expr(&call.args[0]);
                    if self.is_lib_mode {
                        format!("(!{})", inner)
                    } else {
                        format!("cp_not({})", inner)
                    }
                }
                "@print" => {
                    let fmt = self.generate_expr(&call.args[0]);
                    if call.args.len() == 1 {
                        format!("std.debug.print(\"{{s}}\\n\", .{{ {} }})", fmt)
                    } else {
                        let mut args = String::from(".{ ");
                        for idx in 1..call.args.len() {
                            args.push_str(&self.generate_expr(&call.args[idx]));
                            if idx < call.args.len() - 1 {
                                args.push_str(", ");
                            }
                        }
                        args.push_str(" }");
                        format!("std.debug.print({} ++ \"\\n\", {})", fmt, args)
                    }
                }
                "@spawn" => {
                    let callee = match &call.args[0] {
                        Expr::Ident(id) => id.sym.clone(),
                        Expr::Arrow(_) => self.generate_expr(&call.args[0]),
                        _ => "Unknown".to_string(),
                    };
                    let mut captures = String::new();
                    if let Some(cv) = self.captured_vars.get(&callee) {
                        for (r, _) in cv {
                            if self.var_is_ref_struct.contains(r) {
                                panic!("CP Compile Error: Stack-bound ref struct '{}' cannot be captured across Actor @spawn boundary", r);
                            }
                            if self.var_borrow_sources.contains_key(r) {
                                panic!("CP Compile Error: Temporary borrow '{}' cannot be captured across Actor @spawn boundary (use move or Shared<T>)", r);
                            }
                            let src = if self.is_state_machine {
                                format!("ctx.{}", r)
                            } else {
                                r.clone()
                            };
                            captures.push_str(&format!("sub_ctx.{} = {}; ", r, src));
                        }
                    }
                    format!("(blk: {{ const arena_ptr = std.heap.page_allocator.create(std.heap.ArenaAllocator) catch unreachable; arena_ptr.* = std.heap.ArenaAllocator.init(std.heap.page_allocator); const sub_ctx = arena_ptr.allocator().create({0}_Context) catch unreachable; sub_ctx.* = undefined; sub_ctx.pc = 0; sub_ctx.arena = arena_ptr; sub_ctx._parent = null; sub_ctx._parent_fn = null; {1} const p = PID{{ .node_id = current_node_id, .local_id = next_local_id }}; next_local_id += 1; sub_ctx._self = p; schedule(@ptrCast(&{0}), @ptrCast(sub_ctx)); break :blk p; }})", callee, captures)
                }
                "@net_accept_async" => {
                    if !self.is_state_machine {
                        return "0".to_string();
                    }
                    let handle = self.generate_expr(&call.args[0]);
                    let next_pc = self.pc_counter + 1;
                    let logic = if self.is_beam_mode {
                        "unreachable;".to_string()
                    } else {
                        format!("const h: std.posix.fd_t = @intCast({}); const rc = std.os.linux.accept4(h, null, null, std.posix.SOCK.NONBLOCK | std.posix.SOCK.CLOEXEC); const err = std.posix.errno(rc); if (err == .SUCCESS) {{ ctx.received_msg = .{{ .int = @intCast(rc) }}; ctx.pc = {}; continue; }} else if (err == .AGAIN or err == .WOULDBLOCK) {{ register_fd(h, @ptrCast(&{}), @ptrCast(ctx), std.posix.POLL.IN); return; }} else {{ schedule(@ptrCast(&{}), @ptrCast(ctx)); return; }}", handle, next_pc, self.current_zig_fn_name, self.current_zig_fn_name)
                    };
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.received_msg.?.int".to_string()
                }
                "@net_read_async" => {
                    if !self.is_state_machine {
                        return "0".to_string();
                    }
                    let handle = self.generate_expr(&call.args[0]);
                    let next_pc = self.pc_counter + 1;
                    let logic = if self.is_beam_mode {
                        "unreachable;".to_string()
                    } else {
                        format!("const h: std.posix.fd_t = @intCast({}); var buf = ctx.arena.?.allocator().alloc(u8, 4096) catch unreachable; if (std.posix.read(h, buf)) |bytes_read| {{ if (bytes_read > 0) {{ ctx.received_msg = .{{ .string = buf[0..bytes_read] }}; ctx.pc = {}; continue; }} else {{ schedule(@ptrCast(&{}), @ptrCast(ctx)); return; }} }} else |err| switch (err) {{ error.WouldBlock => {{ register_fd(h, @ptrCast(&{}), @ptrCast(ctx), std.posix.POLL.IN); return; }}, else => {{ schedule(@ptrCast(&{}), @ptrCast(ctx)); return; }}, }}", handle, next_pc, self.current_zig_fn_name, self.current_zig_fn_name, self.current_zig_fn_name)
                    };
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.received_msg.?.string".to_string()
                }
                "@net_write_async" => {
                    if !self.is_state_machine {
                        return "0".to_string();
                    }
                    let handle = self.generate_expr(&call.args[0]);
                    let data = self.generate_expr(&call.args[1]);
                    let next_pc = self.pc_counter + 1;
                    let logic = if self.is_beam_mode {
                        "unreachable;".to_string()
                    } else {
                        format!("const h: std.posix.fd_t = @intCast({}); const data_slice: []const u8 = {}; const rc = std.os.linux.write(h, data_slice.ptr, data_slice.len); const err = std.posix.errno(rc); if (err == .SUCCESS) {{ ctx.received_msg = .{{ .int = @intCast(rc) }}; ctx.pc = {}; continue; }} else if (err == .AGAIN or err == .WOULDBLOCK) {{ register_fd(h, @ptrCast(&{}), @ptrCast(ctx), std.posix.POLL.OUT); return; }} else {{ schedule(@ptrCast(&{}), @ptrCast(ctx)); return; }}", handle, data, next_pc, self.current_zig_fn_name, self.current_zig_fn_name)
                    };
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.received_msg.?.int".to_string()
                }
                "@fs_read_async" => {
                    if !self.is_state_machine {
                        return "0".to_string();
                    }
                    let path = self.generate_expr(&call.args[0]);
                    let next_pc = self.pc_counter + 1;
                    let logic = if self.is_beam_mode {
                        format!("const io_msg = try beam.Beam.makeFastMessage(ctx.self.env.?, 99990001, @intFromEnum(beam.SystemAction.FS_READ), {}); ctx.pc = {}; const ctx_bin = try beam.Beam.makeBinary(ctx.self.env.?, std.mem.asBytes(ctx)); return beam.Beam.makeTuple(ctx.self.env.?, &[_]beam.Term{{ beam.Beam.makeAtom(ctx.self.env.?, \"yield\"), io_msg, ctx_bin }});", path, next_pc)
                    } else {
                        format!("const f = std.fs.cwd().openFile({}, .{{}}) catch unreachable; defer f.close(); const data = f.readToEndAlloc(ctx.arena.?.allocator(), 10 * 1024 * 1024) catch unreachable; ctx.received_msg = .{{ .string = data }}; ctx.pc = {}; continue;", path, next_pc)
                    };
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.received_msg.?.string".to_string()
                }
                "@net_listen" => {
                    let port = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const addr = @import(\"std\").Io.net.IpAddress.parse(\"127.0.0.1\", @intCast({})) catch unreachable; const _server = addr.listen(global_io, .{{}}) catch unreachable; break :blk @as(i32, @intCast(_server.socket.handle)); }})", port)
                }
                "@net_close" => {
                    let handle = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const h: std.posix.fd_t = @intCast({}); _ = @import(\"std\").os.linux.close(h); break :blk {{}}; }})", handle)
                }
                "@yield_to_beam" => {
                    if !self.is_state_machine || !self.is_beam_mode {
                        return "0".to_string();
                    }
                    let action_id = self.generate_expr(&call.args[0]);
                    let payload = self.generate_expr(&call.args[1]);
                    let next_pc = self.pc_counter + 1;
                    let logic = format!(
                        "const io_msg = try beam.Beam.makeFastMessage(ctx.self.env.?, 99990001, @intCast({}), {}); \
                        ctx.pc = {}; \
                        const ctx_bin = try beam.Beam.makeBinary(ctx.self.env.?, std.mem.asBytes(ctx)); \
                        return beam.Beam.makeTuple(ctx.self.env.?, &[_]beam.Term{{ beam.Beam.makeAtom(ctx.self.env.?, \"yield\"), io_msg, ctx_bin }});",
                        action_id, payload, next_pc
                    );
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.received_msg.?.string".to_string()
                }
                "@print_err" => {
                    let fmt = self.generate_expr(&call.args[0]);
                    let mut args = String::from(".{ ");
                    for idx in 1..call.args.len() {
                        args.push_str(&self.generate_expr(&call.args[idx]));
                        if idx < call.args.len() - 1 {
                            args.push_str(", ");
                        }
                    }
                    args.push_str(" }");
                    format!("std.debug.print({} ++ \"\\n\", {})", fmt, args)
                }
                "@io_write_stdout" => {
                    let fmt = self.generate_expr(&call.args[0]);
                    let mut args = String::from(".{ ");
                    for idx in 1..call.args.len() {
                        args.push_str(&self.generate_expr(&call.args[idx]));
                        if idx < call.args.len() - 1 {
                            args.push_str(", ");
                        }
                    }
                    args.push_str(" }");
                    format!("(blk: {{ const out = std.io.getStdOut().writer(); try out.print({} ++ \"\\n\", {}); break :blk {{}}; }})", fmt, args)
                }
                "@io_write_stderr" => {
                    let fmt = self.generate_expr(&call.args[0]);
                    let mut args = String::from(".{ ");
                    for idx in 1..call.args.len() {
                        args.push_str(&self.generate_expr(&call.args[idx]));
                        if idx < call.args.len() - 1 {
                            args.push_str(", ");
                        }
                    }
                    args.push_str(" }");
                    format!("(blk: {{ const err_w = std.io.getStdErr().writer(); try err_w.print({} ++ \"\\n\", {}); break :blk {{}}; }})", fmt, args)
                }
                "@io_read_stdin_line" => r#"(blk: { var buf: [1024]u8 = undefined; const line = (try std.io.getStdIn().reader().readUntilDelimiterOrEof(&buf, '\n')) orelse ""; break :blk try std.heap.page_allocator.dupe(u8, line); })"#.to_string(),
                "@http_fetch" => {
                    if !self.is_state_machine {
                        return "0".to_string();
                    }
                    let url = self.generate_expr(&call.args[0]);
                    let next_pc = self.pc_counter + 1;
                    let logic = if self.is_beam_mode {
                        format!("const payload_bin = try beam.Beam.makeBinary(ctx.self.env.?, {}); const io_msg = try beam.Beam.makeFastMessage(ctx.self.env.?, 99990001, @intFromEnum(beam.SystemAction.HTTP_FETCH), payload_bin); ctx.pc = {}; const ctx_bin = try beam.Beam.makeBinary(ctx.self.env.?, std.mem.asBytes(ctx)); return beam.Beam.makeTuple(ctx.self.env.?, &[_]beam.Term{{ beam.Beam.makeAtom(ctx.self.env.?, \"yield\"), io_msg, ctx_bin }});", url, next_pc)
                    } else {
                        format!("ctx.received_msg = .{{ .string = \"Mock HTTP Res\" }}; ctx.pc = {}; continue;", next_pc)
                    };
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.received_msg.?.string".to_string()
                }
                "@fs_write_file" => {
                    let p = self.generate_expr(&call.args[0]);
                    let d = self.generate_expr(&call.args[1]);
                    format!("(blk: {{ const file_out = @import(\"std\").Io.Dir.createFile(.cwd(), global_io, {}, .{{}}) catch unreachable; defer @import(\"std\").Io.File.close(file_out, global_io); @import(\"std\").Io.File.writeStreamingAll(file_out, global_io, {}) catch unreachable; break :blk {{}}; }})", p, d)
                }
                "@fs_mkdir" => {
                    let p = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ @import(\"std\").Io.Dir.createDirPath(.cwd(), global_io, {}) catch unreachable; break :blk {{}}; }})", p)
                }
                "@fs_stat_size" => {
                    let p = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const stat = @import(\"std\").Io.Dir.statFile(.cwd(), global_io, {}, .{{}}) catch unreachable; break :blk @as(i32, @intCast(stat.size)); }})", p)
                }
                "@fs_is_dir" => {
                    let p = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const stat = @import(\"std\").Io.Dir.statFile(.cwd(), global_io, {}, .{{}}) catch unreachable; break :blk stat.kind == .directory; }})", p)
                }
                "@fs_remove" => {
                    let p = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ @import(\"std\").Io.Dir.deleteFile(.cwd(), global_io, {}) catch unreachable; break :blk {{}}; }})", p)
                }
                "@fs_rmdir" => {
                    let p = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ @import(\"std\").Io.Dir.deleteDir(.cwd(), global_io, {}) catch unreachable; break :blk {{}}; }})", p)
                }
                "@fs_rename" => {
                    let old_p = self.generate_expr(&call.args[0]);
                    let new_p = self.generate_expr(&call.args[1]);
                    format!("(blk: {{ @import(\"std\").Io.Dir.rename(.cwd(), {}, .cwd(), {}, global_io) catch unreachable; break :blk {{}}; }})", old_p, new_p)
                }
                "@fs_copy_file" => {
                    let src = self.generate_expr(&call.args[0]);
                    let dest = self.generate_expr(&call.args[1]);
                    format!("(blk: {{ @import(\"std\").Io.Dir.copyFile(.cwd(), {}, .cwd(), {}, global_io, .{{}}) catch unreachable; break :blk {{}}; }})", src, dest)
                }
                "@fs_exists" => {
                    let p = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const is_exists = if (@import(\"std\").Io.Dir.statFile(.cwd(), global_io, {}, .{{}})) |_| true else |err| if (err == error.FileNotFound) false else unreachable; break :blk is_exists; }})", p)
                }
                "@crypto_uuid" => {
                    let allocator = if self.is_state_machine {
                        "ctx.arena.?.allocator()"
                    } else {
                        "std.heap.page_allocator"
                    };
                    format!("(blk: {{ var uuid: [16]u8 = undefined; _ = @import(\"std\").os.linux.getrandom(&uuid, 16, 0); uuid[6] = (uuid[6] & 0x0f) | 0x40; uuid[8] = (uuid[8] & 0x3f) | 0x80; const s = @import(\"std\").fmt.allocPrint({}, \"{{x:0>2}}{{x:0>2}}{{x:0>2}}{{x:0>2}}-{{x:0>2}}{{x:0>2}}-{{x:0>2}}{{x:0>2}}-{{x:0>2}}{{x:0>2}}-{{x:0>2}}{{x:0>2}}{{x:0>2}}{{x:0>2}}{{x:0>2}}{{x:0>2}}\", .{{ uuid[0], uuid[1], uuid[2], uuid[3], uuid[4], uuid[5], uuid[6], uuid[7], uuid[8], uuid[9], uuid[10], uuid[11], uuid[12], uuid[13], uuid[14], uuid[15] }}) catch unreachable; break :blk s; }})", allocator)
                }
                "@crypto_hash" => {
                    let allocator = if self.is_state_machine {
                        "ctx.arena.?.allocator()"
                    } else {
                        "std.heap.page_allocator"
                    };
                    let data = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ var out: [32]u8 = undefined; @import(\"std\").crypto.hash.sha2.Sha256.hash({}, &out, .{{}}); const hex = @import(\"std\").fmt.bytesToHex(out, .lower); const hex_str = {}.dupe(u8, &hex) catch unreachable; break :blk hex_str; }})", data, allocator)
                }
                "@fs_read_file" => {
                    let p = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const file_in = @import(\"std\").Io.Dir.openFile(.cwd(), global_io, {}, .{{}}) catch unreachable; defer @import(\"std\").Io.File.close(file_in, global_io); const stat = @import(\"std\").Io.File.stat(file_in, global_io) catch unreachable; const data = std.heap.page_allocator.alloc(u8, @intCast(stat.size)) catch unreachable; _ = @import(\"std\").Io.File.readPositionalAll(file_in, global_io, data, 0) catch unreachable; break :blk data; }})", p)
                }
                "@net_connect" => "(blk: { break :blk 1; })".to_string(),
                "@net_send" => "(blk: { break :blk {}; })".to_string(),
                "@worker_spawn" => {
                    let entry = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const wid = global_next_worker_id.fetchAdd(1, .monotonic); const _Runner = struct {{ pub fn run(w_id: u32) void {{ current_node_id = w_id; next_local_id = 2; const worker_ctx = std.heap.page_allocator.create({0}_Context) catch unreachable; worker_ctx.* = undefined; worker_ctx.pc = 0; worker_ctx._self = PID{{ .node_id = w_id, .local_id = 1 }}; worker_ctx.arena = null; worker_ctx._parent = null; worker_ctx._parent_fn = null; worker_ctx.received_msg = null; worker_ctx.received_sender = null; schedule(@ptrCast(&{0}), @ptrCast(worker_ctx)); while (true) {{ drain_cross_thread_mailbox(); while (queue_len > 0) {{ const task = task_queue[0]; for (task_queue[1..queue_len], 0..) |t, j| {{ task_queue[j] = t; }} queue_len -= 1; task.run(task.ctx) catch unreachable; }} drain_cross_thread_mailbox(); if (active_fds == 0 and queue_len == 0 and waiting_count == 0) break; if (queue_len == 0 and waiting_count > 0 and active_fds == 0) {{ const ts = std.os.linux.timespec{{ .sec = 0, .nsec = 500_000 }}; _ = std.os.linux.nanosleep(&ts, null); }} if (active_fds > 0) {{ const ready_count = std.posix.poll(poll_fds[0..active_fds], 10) catch 0; if (ready_count > 0) {{ var i: usize = 0; while (i < active_fds) {{ if (poll_fds[i].revents != 0) {{ schedule(fd_to_run[i], fd_to_ctx[i]); poll_fds[i] = poll_fds[active_fds - 1]; fd_to_ctx[i] = fd_to_ctx[active_fds - 1]; fd_to_run[i] = fd_to_run[active_fds - 1]; active_fds -= 1; }} else {{ i += 1; }} }} }} }} }} }} }}; const _t = @import(\"std\").Thread.spawn(.{{}}, _Runner.run, .{{wid}}) catch unreachable; _t.detach(); break :blk PID{{ .node_id = wid, .local_id = 1 }}; }})", entry)
                }
                "@create_pending_task" => {
                    format!("(blk: {{ const task_ptr = std.heap.page_allocator.create(CpTask) catch unreachable; task_ptr.* = .{{}}; break :blk task_ptr; }})")
                }
                "@complete_task" => {
                    let task = self.generate_expr(&call.args[0]);
                    let val = self.generate_expr(&call.args[1]);
                    format!("(blk: {{ const __cp_target_task: *CpTask = @ptrCast(@alignCast({0})); __cp_target_task.lock.acquire(); __cp_target_task.state = 2; __cp_target_task.base.await_result = cp_message_from({1}); const __jb = __cp_target_task.join_barrier; const __j_idx = __cp_target_task.join_idx; const __w_ctx_opt = __cp_target_task.waiter_ctx; const __w_fn_opt = __cp_target_task.waiter_fn; __cp_target_task.waiter_ctx = null; __cp_target_task.waiter_fn = null; const __res = __cp_target_task.base.await_result; __cp_target_task.lock.release(); if (__jb) |b| {{ b.on_task_done(__j_idx, __res orelse Message{{ .string = \"\" }}); }} if (__w_ctx_opt) |w_ctx| {{ const w_fn = __w_fn_opt.?; const base_waiter: *BaseContext = @ptrCast(@alignCast(w_ctx)); base_waiter.await_result = __res; schedule(w_fn, w_ctx); }} break :blk {{}}; }})", task, val)
                }
                "@send" | "@reply" => {
                    let target = self.generate_expr(&call.args[0]);
                    let msg = self.generate_expr(&call.args[1]);
                    let sender = if self.is_state_machine {
                        "ctx._self"
                    } else {
                        "PID{ .node_id = current_node_id, .local_id = 1 }"
                    };
                    format!("cp_send({}, {}, cp_message_from({}))", sender, target, msg)
                }
                "@self" => {
                    if self.is_state_machine {
                        "ctx._self".to_string()
                    } else {
                        "PID{ .node_id = current_node_id, .local_id = 1 }".to_string()
                    }
                }
                "@sender" => {
                    if self.is_state_machine {
                        "ctx.received_sender.?".to_string()
                    } else {
                        "PID{ .node_id = 0, .local_id = 0 }".to_string()
                    }
                }
                "@http_serve" => self.generate_http_serve(&call.args[0]),
                "@receive" => {
                    if !self.is_state_machine {
                        return "null".to_string();
                    }
                    let check_pc = self.pc_counter + 1;
                    self.write_to_case(&format!("ctx.pc = {}; continue;\n", check_pc));
                    self.next_state();
                    let resume_pc = check_pc + 1;
                    let logic = if self.is_beam_mode {
                        format!("ctx.pc = {}; const ctx_bin = try beam.Beam.makeBinary(ctx.self.env.?, std.mem.asBytes(ctx)); return beam.Beam.makeTuple(ctx.self.env.?, &[_]beam.Term{{ beam.Beam.makeAtom(ctx.self.env.?, \"yield\"), beam.Beam.makeAtom(ctx.self.env.?, \"receive\"), ctx_bin }});", resume_pc)
                    } else {
                        format!(
                            "if (local_mailboxes[ctx._self.local_id % 1024].pop()) |item| {{ ctx.received_msg = item.msg; ctx.received_sender = item.sender; ctx.pc = {}; continue; }} else {{ ctx.received_msg = null; ctx.received_sender = null; waiting_actors[ctx._self.local_id % 1024] = ctx; waiting_actor_fns[ctx._self.local_id % 1024] = @ptrCast(&{}); waiting_count += 1; ctx.pc = {}; return; }}",
                            resume_pc, self.current_zig_fn_name, check_pc
                        )
                    };
                    self.write_to_case(&logic);
                    self.next_state();
                    "ctx.received_msg.?".to_string()
                }
                "@alloc" => {
                    let s = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const __alloc_res = {}.alloc(u8, {}) catch unreachable; break :blk __alloc_res.ptr; }})", self.current_allocator(), s)
                }
                "@free" => "(blk: { break :blk {}; })".to_string(),
                "@expect_eq" if call.args.len() == 2 => {
                    let a = self.generate_expr(&call.args[0]);
                    let b = self.generate_expr(&call.args[1]);
                    format!("(blk: {{ const _a = {}; const _b = {}; const is_eq = cp_eq(_a, _b); if (!is_eq) {{ @import(\"std\").debug.print(\"\\n    ❌ Assertion Failed!\\n    Expected: {{any}}\\n    Got:      {{any}}\\n\", .{{ _b, _a }}); __cap_test_failed += 1; return error.TestFailed; }} break :blk {{}}; }})", a, b)
                }
                "@test" if call.args.len() == 2 => {
                    let name = self.generate_expr(&call.args[0]);
                    let lambda = self.generate_expr(&call.args[1]);
                    format!("(blk: {{ _ = {}; _ = {}; break :blk {{}}; }})", name, lambda)
                }
                "@get_arena" | "@arena" => {
                    format!("(blk: {{ break :blk {}; }})", self.current_allocator())
                }
                "@shared_tensor_init" => "(blk: { break :blk 0; })".to_string(),
                "@json_parse" => {
                    let json_str = self.generate_expr(&call.args[0]);
                    let type_name = if call.args.len() > 1 {
                        if let Expr::Ident(id) = &call.args[1] {
                            id.sym.clone()
                        } else {
                            "std.json.Value".to_string()
                        }
                    } else {
                        "std.json.Value".to_string()
                    };

                    if type_name == "std.json.Value" {
                        format!("(blk: {{ const parsed = try @import(\"std\").json.parseFromSlice(std.json.Value, std.heap.page_allocator, {}, .{{ .ignore_unknown_fields = true }}); break :blk parsed.value; }})", json_str)
                    } else {
                        format!("(blk: {{ const parsed = try @import(\"std\").json.parseFromSlice({}, std.heap.page_allocator, {}, .{{ .ignore_unknown_fields = true }}); const json_ptr = try std.heap.page_allocator.create({}); json_ptr.* = parsed.value; break :blk json_ptr; }})", type_name, json_str, type_name)
                    }
                }
                "@json_stringify" => {
                    let obj = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ break :blk try @import(\"std\").fmt.allocPrint(std.heap.page_allocator, \"{{f}}\", .{{@import(\"std\").json.fmt({}.*, .{{}})}}); }})", obj)
                }
                "@compileError" => {
                    let msg = self.generate_expr(&call.args[0]);
                    format!("@compileError({})", msg)
                }
                "@comptime" if call.args.len() == 1 => {
                    let expr = self.generate_expr(&call.args[0]);
                    format!("comptime ({})", expr)
                }
                "@compileLog" => {
                    let mut args_code = Vec::new();
                    for a in &call.args {
                        args_code.push(self.generate_expr(a));
                    }
                    format!("@compileLog({})", args_code.join(", "))
                }
                "@typeOf" => {
                    let val = self.generate_expr(&call.args[0]);
                    format!("@TypeOf({})", val)
                }
                "@typeName" => {
                    let t = self.generate_expr(&call.args[0]);
                    format!("@typeName({})", t)
                }
                "@fieldsOf" => {
                    let t = self.generate_expr(&call.args[0]);
                    format!("@typeInfo({}).@\"struct\".fields", t)
                }
                "@field" => {
                    let obj = self.generate_expr(&call.args[0]);
                    let f = self.generate_expr(&call.args[1]);
                    let f = strip_redundant_comptime(&f);
                    format!("@field({}, {})", obj, f)
                }
                "@hasField" => {
                    let t = self.generate_expr(&call.args[0]);
                    let f = self.generate_expr(&call.args[1]);
                    let f = strip_redundant_comptime(&f);
                    format!("@hasField({}, {})", t, f)
                }
                "@sizeOf" => {
                    let t = self.generate_expr(&call.args[0]);
                    format!("@sizeOf({})", t)
                }
                "@alignOf" => {
                    let t = self.generate_expr(&call.args[0]);
                    format!("@alignOf({})", t)
                }
                _ => {
                    let mut args_code = Vec::new();
                    for a in &call.args {
                        args_code.push(self.generate_expr(a));
                    }
                    format!("{}({})", call.name, args_code.join(", "))
                }
            },
            Expr::Arrow(arrow) => {
                let lambda_name = format!("lambda_{}", self.lambda_counter);
                self.lambda_counter += 1;
                let old_fn = self.current_zig_fn_name.clone();
                let mut lambda_output = String::new();
                let mut lambda_gen = Codegen::new();
                lambda_gen.globals = self.globals.clone();
                lambda_gen.class_defs = self.class_defs.clone();
                lambda_gen.ref_classes = self.ref_classes.clone();
                lambda_gen.value_classes = self.value_classes.clone();
                lambda_gen.enum_defs = self.enum_defs.clone();
                lambda_gen.error_sets = self.error_sets.clone();
                lambda_gen.fn_sigs = self.fn_sigs.clone();
                lambda_gen.trait_defs = self.trait_defs.clone();
                lambda_gen.var_cp_types = self.var_cp_types.clone();
                lambda_gen.global_var_types = self.global_var_types.clone();
                lambda_gen.anon_obj_structs = self.anon_obj_structs.clone();
                lambda_gen.type_aliases = self.type_aliases.clone();
                lambda_gen.is_state_machine = true;
                lambda_gen.current_fn_is_async = true;
                lambda_gen.current_zig_fn_name = lambda_name.clone();
                let ctx_name = format!("{}_Context", lambda_name);
                writeln!(lambda_output, "const {} = struct {{ pc: u32 = 0, _self: PID = undefined, _parent: ?*anyopaque = null, _parent_fn: ?*const fn(*anyopaque) anyerror!void = null, arena: ?*std.heap.ArenaAllocator = null, received_msg: ?Message = null, received_sender: ?PID = null, await_result: ?Message = null,", ctx_name).unwrap();

                let mut refs = std::collections::HashSet::new();
                match &*arrow.body {
                    ArrowBody::Expr(e) => {
                        crate::ast_visitor::find_referenced_vars_in_expr(e, &mut refs)
                    }
                    ArrowBody::Block(b) => {
                        crate::ast_visitor::find_referenced_vars_in_block(b, &mut refs)
                    }
                }
                let decls = match &*arrow.body {
                    ArrowBody::Expr(_) => std::collections::HashSet::new(),
                    ArrowBody::Block(b) => crate::ast_visitor::get_declared_vars_in_block(b),
                };
                for d in decls {
                    refs.remove(&d);
                }
                let mut captured = Vec::new();
                for r in refs {
                    if r == "this" || r == "_" {
                        if let Some(self_ty) = self.locals.get("self") {
                            captured.push(("self".to_string(), self_ty.clone()));
                        } else if let Some(c) = &self.current_class_name {
                            captured.push(("self".to_string(), format!("*{}", c)));
                        }
                    } else if let Some(ty) = self.locals.get(&r) {
                        if self.moved_vars.contains(&r) {
                            panic!("CP Compile Error: Use of moved variable '{}'", r);
                        }
                        captured.push((r.clone(), ty.clone()));
                    }
                }
                self.captured_vars
                    .insert(lambda_name.clone(), captured.clone());

                for (r, ty) in &captured {
                    lambda_gen.locals.insert(r.clone(), ty.clone());
                    if r != "self" || self.current_receiver_mode != ReceiverMode::Mut {
                        lambda_gen.current_fn_params.insert(r.clone());
                    }
                    if !self.var_borrow_sources.contains_key(r)
                        && (!self.current_fn_params.contains(r) || self.move_fn_params.contains(r))
                    {
                        lambda_gen.move_fn_params.insert(r.clone());
                    }
                    writeln!(lambda_output, "    {}: {},", r, ty).unwrap();
                }

                for param in &arrow.params {
                    if let Pattern::Ident(pid) = param {
                        lambda_gen.locals.insert(pid.sym.clone(), "?*anyopaque".to_string());
                        writeln!(lambda_output, "    {}: ?*anyopaque = null,", pid.sym).unwrap();
                    }
                }

                if let ArrowBody::Block(b) = &*arrow.body {
                    lambda_gen.extract_local_vars_to_str(b, &mut lambda_output);
                }
                writeln!(lambda_output, "}};").unwrap();
                writeln!(
                    lambda_output,
                    "pub fn {}(ptr: *anyopaque) anyerror!void {{",
                    lambda_name
                )
                .unwrap();
                writeln!(
                    lambda_output,
                    "    const ctx: *{} = @ptrCast(@alignCast(ptr)); while (true) {{ switch (ctx.pc) {{",
                    ctx_name
                )
                .unwrap();
                lambda_gen.indent_level = 2;
                match &*arrow.body {
                    ArrowBody::Expr(e) => {
                        let ec = lambda_gen.generate_expr(e);
                        lambda_gen.write_to_case(&format!(
                            "const __ret_val = {}; if (ctx._parent) |p| {{ const base: *BaseContext = @ptrCast(@alignCast(p)); base.await_result = cp_message_from(__ret_val); schedule(ctx._parent_fn.?, p); }} return;",
                            ec
                        ));
                    }
                    ArrowBody::Block(b) => {
                        for s in &b.body {
                            lambda_gen.generate_stmt(s);
                        }
                    }
                }
                lambda_gen.emit_current_case();
                for case in &lambda_gen.cases {
                    lambda_output.push_str(case);
                }
                writeln!(lambda_output, "        else => unreachable,\n    }} }} }}\n").unwrap();
                self.lambda_funcs.push(lambda_output);

                self.lambda_moved_vars
                    .insert(lambda_name.clone(), lambda_gen.moved_vars.clone());
                for moved in &lambda_gen.moved_vars {
                    if self.locals.contains_key(moved) {
                        self.check_active_borrows_before_move(moved);
                        self.moved_vars.insert(moved.clone());
                    }
                }

                self.current_zig_fn_name = old_fn;
                lambda_name
            }
            Expr::Object(fields) => {
                if !fields.is_empty() && fields.iter().all(|f| f.key.is_none() && !f.is_spread) {
                    let mut s = String::from(".{ ");
                    for (idx, f) in fields.iter().enumerate() {
                        s.push_str(&self.generate_expr(&f.val));
                        if idx + 1 < fields.len() {
                            s.push_str(", ");
                        }
                    }
                    s.push_str(" }");
                    return s;
                }
                let mut prop_map: Vec<(String, String)> = Vec::new();
                for field in fields {
                    if field.is_spread {
                        let base_code = self.generate_expr(&field.val);
                        let base_ty = self.infer_cp_type(&field.val);
                        if let Some(Type::Object(sub_fs)) = base_ty {
                            for sf in sub_fs {
                                let key = sf.ident.sym.clone();
                                let val_code = format!("{}.{}", base_code, key);
                                if let Some(pos) = prop_map.iter().position(|(k, _)| k == &key) {
                                    prop_map[pos] = (key, val_code);
                                } else {
                                    prop_map.push((key, val_code));
                                }
                            }
                        } else if let Some(Type::Ref(cid)) = base_ty {
                            if let Some(cls) = self.class_defs.get(&cid.sym).cloned() {
                                for fl in cls.fields {
                                    let key = fl.ident.sym.clone();
                                    let val_code = format!("{}.{}", base_code, key);
                                    if let Some(pos) = prop_map.iter().position(|(k, _)| k == &key) {
                                        prop_map[pos] = (key, val_code);
                                    } else {
                                        prop_map.push((key, val_code));
                                    }
                                }
                            }
                        }
                    } else if let Some(key) = &field.key {
                        let val_code = self.generate_expr(&field.val);
                        if let Some(pos) = prop_map.iter().position(|(k, _)| k == &key.sym) {
                            prop_map[pos] = (key.sym.clone(), val_code);
                        } else {
                            prop_map.push((key.sym.clone(), val_code));
                        }
                    }
                }
                if prop_map.is_empty() {
                    return ".{}".to_string();
                }
                let mut s = String::from(".{ ");
                for (idx, (k, v)) in prop_map.iter().enumerate() {
                    let val_out = if v == ".{}" && k == "none" { "{}" } else { v.as_str() };
                    s.push_str(&format!(".{} = {}", k, val_out));
                    if idx + 1 < prop_map.len() {
                        s.push_str(", ");
                    }
                }
                s.push_str(" }");
                s
            }
            Expr::Dict(fields) => {
                let mut sets = String::new();
                for field in fields {
                    if field.is_spread {
                        let base_code = self.generate_expr(&field.val);
                        sets.push_str(&format!("_dict.put_all({}); ", base_code));
                    } else if let Some(key) = &field.key {
                        let val_code = self.generate_expr(&field.val);
                        sets.push_str(&format!("_ = _dict.set(\"{}\", {}); ", key.sym, val_code));
                    }
                }
                format!("(blk: {{ const _dict = Dict._create({}) catch unreachable; {}break :blk _dict; }})", self.current_allocator(), sets)
            }
            Expr::Index { obj, index } => {
                self.check_read_allowed(obj);
                let obj_code = self.generate_expr(obj);
                let idx_code = self.generate_expr(index);
                let is_collection = match self.infer_cp_type(obj) {
                    Some(Type::Dict(_, _)) | Some(Type::Map(_, _)) => true,
                    Some(Type::Ref(id))
                        if id.sym == "Dict" || id.sym == "Map" || id.sym == "VecDeque" =>
                    {
                        true
                    }
                    _ => false,
                };
                if !is_collection {
                    if matches!(&**index, Expr::Lit(Lit::Int(_))) {
                        format!("{}[{}]", obj_code, idx_code)
                    } else {
                        format!("{}[@as(usize, @intCast({}))]", obj_code, idx_code)
                    }
                } else {
                    format!("cp_index_get({}, {})", obj_code, idx_code)
                }
            }
            Expr::Array(items) => {
                let mut s = String::from(".{ ");
                for (idx, item) in items.iter().enumerate() {
                    s.push_str(&self.generate_expr(item));
                    if idx < items.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s.push_str(" }");
                if self.expected_array_elem_type.is_some() || self.is_comptime || self.is_lib_mode {
                    s
                } else if !items.is_empty() {
                    let elem_zig_ty = self
                        .infer_cp_type(&items[0])
                        .map(|t| self.map_type(&t))
                        .unwrap_or_else(|| "i32".to_string());
                    format!("cp_array_lit({}, &{})", elem_zig_ty, s)
                } else {
                    s
                }
            },
            Expr::ArrayRepeat(val, count) => {
                let val_code = self.generate_expr(val);
                let count_code = self.generate_expr(count);
                let elem_zig_ty = self
                    .expected_array_elem_type
                    .clone()
                    .or_else(|| self.infer_cp_type(val))
                    .map(|t| self.map_type(&t))
                    .unwrap_or_else(|| "u8".to_string());
                format!("@as([{}]{}, @splat(@as({}, {})))", count_code, elem_zig_ty, elem_zig_ty, val_code)
            },
            Expr::Slice(s) => {
                let obj_code = self.generate_expr(&s.obj);
                let start_code = s
                    .start
                    .as_ref()
                    .map(|e| self.generate_expr(e))
                    .unwrap_or_else(|| "0".to_string());
                let sentinel_code = s
                    .sentinel
                    .as_ref()
                    .map(|e| format!(" :{}", self.generate_expr(e)))
                    .unwrap_or_default();
                if let Some(end_expr) = &s.end {
                    let end_code = self.generate_expr(end_expr);
                    format!("{}[{}..{}{}]", obj_code, start_code, end_code, sentinel_code)
                } else {
                    format!("{}[{}..{}]", obj_code, start_code, sentinel_code)
                }
            },
            Expr::ReflectType { target_ty, .. } => {
                if let Type::Ref(id) = target_ty {
                    format!("_get_typeinfo_{}()", id.sym)
                } else {
                    "@panic(\"Cannot reflect non-class type\")".to_string()
                }
            }
            Expr::ReflectOf { expr, .. } => {
                let class_name = if let Expr::Ident(id) = &**expr {
                    self.resolve_var_type_name(&id.sym)
                } else if let Some(Type::Ref(id)) = self.infer_cp_type(expr) {
                    Some(id.sym.clone())
                } else {
                    None
                };
                if let Some(cname) = class_name {
                    if matches!(&**expr, Expr::Ident(_)) {
                        format!("_get_typeinfo_{}()", cname)
                    } else {
                        let e_code = self.generate_expr(expr);
                        format!("(blk: {{ _ = {}; break :blk _get_typeinfo_{}(); }})", e_code, cname)
                    }
                } else {
                    "@panic(\"Cannot determine type for reflect.of\")".to_string()
                }
            }
            Expr::Range { .. } => "0".to_string(),
            Expr::Cast {
                expr,
                target_ty,
                span,
            } => {
                if let Expr::Ident(id) = &**expr {
                    self.uninitialized_vars.remove(&id.sym);
                }
                let resolved_ty = if let Type::RawPtr { inner, .. } = target_ty {
                    if **inner == Type::Any {
                        self.infer_cp_type(&Expr::Cast {
                            span: *span,
                            expr: expr.clone(),
                            target_ty: target_ty.clone(),
                        })
                        .unwrap_or_else(|| target_ty.clone())
                    } else {
                        target_ty.clone()
                    }
                } else {
                    target_ty.clone()
                };
                let target_zig_ty = self.map_type(&resolved_ty);
                match &**expr {
                    Expr::Borrow(inner, _) => {
                        let inner_code = self.generate_expr(inner);
                        let is_class_ref = if let Expr::Ident(id) = &**inner {
                            if let Some(Type::Ref(cid)) = self.var_cp_types.get(&id.sym) {
                                self.class_defs.get(&cid.sym).map_or(false, |c| !c.is_ref)
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                        if is_class_ref && matches!(resolved_ty, Type::RawPtr { depth: 1, .. }) {
                            format!(
                                "@as({}, @ptrCast(@alignCast({})))",
                                target_zig_ty, inner_code
                            )
                        } else {
                            format!(
                                "@as({}, @ptrCast(@alignCast(&({}))))",
                                target_zig_ty, inner_code
                            )
                        }
                    }
                    _ => {
                        let inner_code = self.generate_expr(expr);
                        let src_ty = self.infer_cp_type(expr);
                        let is_src_float = matches!(src_ty, Some(Type::F32 | Type::F64));
                        let is_src_int = matches!(
                            src_ty,
                            Some(
                                Type::U8
                                    | Type::I8
                                    | Type::U16
                                    | Type::I16
                                    | Type::I32
                                    | Type::U32
                                    | Type::I64
                                    | Type::U64
                                    | Type::U128
                                    | Type::I128
                                    | Type::ISize
                                    | Type::USize
                            )
                        );
                        let is_src_ptr = matches!(
                            src_ty,
                            Some(Type::RawPtr { .. } | Type::ManyRawPtr(_) | Type::CStr | Type::BorrowType { .. })
                        );
                        let is_target_int = matches!(
                            resolved_ty,
                            Type::U8
                                | Type::I8
                                | Type::U16
                                | Type::I16
                                | Type::I32
                                | Type::U32
                                | Type::I64
                                | Type::U64
                                | Type::U128
                                | Type::I128
                                | Type::ISize
                                | Type::USize
                        );
                        let is_target_float = matches!(resolved_ty, Type::F32 | Type::F64);
                        let is_target_ptr = matches!(
                            resolved_ty,
                            Type::RawPtr { .. } | Type::ManyRawPtr(_) | Type::CStr | Type::BorrowType { .. }
                        );
                        let is_src_enum = if let Some(Type::Ref(id)) = &src_ty {
                            self.enum_defs.contains_key(&id.sym)
                        } else {
                            false
                        };
                        let is_target_enum = if let Type::Ref(id) = &resolved_ty {
                            self.enum_defs.contains_key(&id.sym)
                        } else {
                            false
                        };
                        let is_target_fn = matches!(resolved_ty, Type::Fn(_, _) | Type::CFn(_, _));

                        if is_target_int {
                            if is_src_float {
                                format!("@as({}, @intFromFloat({}))", target_zig_ty, inner_code)
                            } else if is_src_ptr {
                                if target_zig_ty == "usize" {
                                    format!("@intFromPtr({})", inner_code)
                                } else {
                                    format!(
                                        "cp_cast_int({}, @intFromPtr({}))",
                                        target_zig_ty, inner_code
                                    )
                                }
                            } else if is_src_enum {
                                format!(
                                    "cp_cast_int({}, @intFromEnum({}))",
                                    target_zig_ty, inner_code
                                )
                            } else if inner_code.starts_with("@intCast(")
                                || inner_code.starts_with("@truncate(")
                                || inner_code.starts_with("@bitCast(")
                                || inner_code.starts_with("@enumFromInt(")
                                || inner_code.starts_with("@ptrFromInt(")
                                || inner_code.starts_with("@ptrCast(")
                                || inner_code.starts_with("@floatCast(")
                                || inner_code.starts_with("@intFromFloat(")
                            {
                                format!("@as({}, {})", target_zig_ty, inner_code)
                            } else if self.is_comptime {
                                format!(
                                    "(if (@typeInfo(@TypeOf({0})) == .@\"enum\") cp_cast_int({1}, @intFromEnum({0})) else cp_cast_int({1}, {0}))",
                                    inner_code, target_zig_ty
                                )
                            } else {
                                format!(
                                    "(if (comptime @typeInfo(@TypeOf({0})) == .@\"enum\") cp_cast_int({1}, @intFromEnum({0})) else cp_cast_int({1}, {0}))",
                                    inner_code, target_zig_ty
                                )
                            }
                        } else if is_target_float {
                            if is_src_int {
                                format!("@as({}, @floatFromInt({}))", target_zig_ty, inner_code)
                            } else {
                                format!("@as({}, @floatCast({}))", target_zig_ty, inner_code)
                            }
                        } else if is_target_ptr {
                            if is_src_int {
                                format!(
                                    "@as({}, @ptrFromInt(@as(usize, @truncate({}))))",
                                    target_zig_ty, inner_code
                                )
                            } else if matches!(src_ty, Some(Type::Array(_)) | Some(Type::FixedArray(_, _)))
                                || matches!(src_ty, Some(Type::Ref(ref id)) if id.sym.starts_with('['))
                                || self.infer_zig_type(None, Some(expr)).starts_with('[')
                                || self.locals.get(&inner_code).map_or(false, |l| l.starts_with('['))
                            {
                                format!("@as({}, @ptrCast(&({})))", target_zig_ty, inner_code)
                            } else if matches!(src_ty, Some(Type::String)) {
                                format!("@as({}, @ptrCast({}.ptr))", target_zig_ty, inner_code)
                            } else {
                                format!(
                                    "@as({}, @ptrCast(@alignCast({})))",
                                    target_zig_ty, inner_code
                                )
                            }
                        } else if is_target_enum {
                            if inner_code.starts_with("@enumFromInt(") {
                                format!("@as({}, {})", target_zig_ty, inner_code)
                            } else {
                                format!("@as({}, @enumFromInt({}))", target_zig_ty, inner_code)
                            }
                        } else if is_target_fn {
                            format!("@as({}, @ptrCast({}))", target_zig_ty, inner_code)
                        } else if matches!(resolved_ty, Type::Any) {
                            format!(
                                "@as({}, @ptrCast(@alignCast({})))",
                                target_zig_ty, inner_code
                            )
                        } else if matches!(resolved_ty, Type::Ref(_)) && !matches!(resolved_ty, Type::Ref(ref id) if self.class_defs.contains_key(&id.sym) || self.trait_defs.contains_key(&id.sym)) {
                            if inner_code.starts_with("@enumFromInt(")
                                || inner_code.starts_with("@truncate(")
                                || inner_code.starts_with("@intCast(")
                                || inner_code.starts_with("@bitCast(")
                                || inner_code.starts_with("@ptrCast(")
                                || inner_code.starts_with("@ptrFromInt(")
                            {
                                format!("@as({}, {})", target_zig_ty, inner_code)
                            } else if self.is_comptime {
                                format!(
                                    "(if (@typeInfo({0}) == .@\"enum\") @as({0}, @enumFromInt({1})) else @as({0}, {1}))",
                                    target_zig_ty, inner_code
                                )
                            } else {
                                format!(
                                    "(if (comptime @typeInfo({0}) == .@\"enum\") @as({0}, @enumFromInt({1})) else @as({0}, {1}))",
                                    target_zig_ty, inner_code
                                )
                            }
                        } else if resolved_ty == Type::Bool {
                            if self.is_comptime {
                                format!("(if (@typeInfo(@TypeOf({0})) == .bool) {0} else ({0} != 0))", inner_code)
                            } else {
                                format!("(if (comptime @typeInfo(@TypeOf({0})) == .bool) {0} else ({0} != 0))", inner_code)
                            }
                        } else {
                            format!("@as({}, {})", target_zig_ty, inner_code)
                        }
                    }
                }
            }
            Expr::BitNot(inner) => {
                let inner_code = self.generate_expr(inner);
                format!("(~{})", inner_code)
            }
            Expr::Deref(inner) => {
                let inner_code = self.generate_expr(inner);
                if matches!(&**inner, Expr::Ident(_) | Expr::Member(_) | Expr::Deref(_)) {
                    format!("{}.*", inner_code)
                } else {
                    format!("({}).*", inner_code)
                }
            }
            Expr::Unwrap(inner) => {
                if let Expr::Ident(id) = &**inner {
                    if self.narrowed_null_vars.contains(&id.sym) {
                        let was = self.narrowed_null_vars.remove(&id.sym);
                        let inner_code = self.generate_expr(inner);
                        if was {
                            self.narrowed_null_vars.insert(id.sym.clone());
                        }
                        return if matches!(&**inner, Expr::Ident(_) | Expr::Member(_) | Expr::Deref(_)) {
                            format!("{}.?", inner_code)
                        } else {
                            format!("({}).?", inner_code)
                        };
                    }
                }
                let inner_code = self.generate_expr(inner);
                if matches!(&**inner, Expr::Ident(_) | Expr::Member(_) | Expr::Deref(_)) {
                    format!("{}.?", inner_code)
                } else {
                    format!("({}).?", inner_code)
                }
            }
            Expr::Unsafe(b) => {
                let saved = self.current_case_content.clone();
                self.current_case_content.clear();
                let len = b.body.len();
                for (idx, s) in b.body.iter().enumerate() {
                    if idx + 1 == len {
                        if let Stmt::Expr(e) = s {
                            let ec = self.generate_expr(e);
                            self.write_to_case(&format!("break :blk {};", ec));
                        } else {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(s);
                    }
                }
                let body_str = self.current_case_content.clone();
                self.current_case_content = saved;
                format!("(blk: {{\n{}\n}})", body_str)
            }
            Expr::Comptime(b) => {
                let saved = self.current_case_content.clone();
                let saved_sm = self.is_state_machine;
                let saved_comptime = self.is_comptime;
                self.is_state_machine = false;
                self.is_comptime = true;
                self.current_case_content.clear();
                let len = b.body.len();
                for (idx, s) in b.body.iter().enumerate() {
                    if idx + 1 == len {
                        if let Stmt::Expr(e) = s {
                            let ec = self.generate_expr(e);
                            self.write_to_case(&format!("break :blk {};", ec));
                        } else {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(s);
                    }
                }
                let body_str = self.current_case_content.clone();
                self.current_case_content = saved;
                self.is_state_machine = saved_sm;
                self.is_comptime = saved_comptime;
                format!("comptime (blk: {{\n{}\n}})", body_str)
            }
            Expr::Arena(b) => {
                let arena_id = self.tmp_counter;
                self.tmp_counter += 1;
                let blk_label = format!("blk_arena_{}", arena_id);
                let arena_var = format!("__arena_{}", arena_id);
                let alloc_var = format!("__arena_alloc_{}", arena_id);
                let saved = self.current_case_content.clone();
                self.current_case_content.clear();
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
                let len = b.body.len();
                if len == 0 {
                    self.write_to_case(&format!("break :{} {{}};", blk_label));
                } else {
                    for (idx, s) in b.body.iter().enumerate() {
                        if idx + 1 == len {
                            if let Stmt::Expr(e) = s {
                                let ec = self.generate_expr(e);
                                self.write_to_case(&format!("break :{} {};", blk_label, ec));
                            } else {
                                self.generate_stmt(s);
                                self.write_to_case(&format!("break :{} {{}};", blk_label));
                            }
                        } else {
                            self.generate_stmt(s);
                        }
                    }
                }
                self.arena_allocator_stack.pop();
                let body_str = self.current_case_content.clone();
                self.current_case_content = saved;
                format!("({}: {{\n{}\n}})", blk_label, body_str)
            }
            Expr::Template(t) => {
                if t.expressions.is_empty() {
                    if t.quasis.is_empty() {
                        return "\"\"".to_string();
                    } else {
                        return format!("\"{}\"", Self::escape_zig_string(&t.quasis[0]));
                    }
                }

                // Compile-time constant folding if all expressions are literals
                let all_lits = t.expressions.iter().all(|e| matches!(e, Expr::Lit(_)));
                if all_lits {
                    let mut folded = String::new();
                    for (idx, quasi) in t.quasis.iter().enumerate() {
                        folded.push_str(quasi);
                        if idx < t.expressions.len() {
                            if let Expr::Lit(lit) = &t.expressions[idx] {
                                match lit {
                                    Lit::Int(n) => folded.push_str(&n.to_string()),
                                    Lit::Float(f) => folded.push_str(&f.to_string()),
                                    Lit::Str(s) => folded.push_str(s),
                                    Lit::Bool(b) => {
                                        folded.push_str(if *b { "true" } else { "false" })
                                    }
                                    Lit::Null => folded.push_str("null"),
                                    Lit::Undefined => folded.push_str("undefined"),
                                }
                            }
                        }
                    }
                    return format!("\"{}\"", Self::escape_zig_string(&folded));
                }

                let mut fmt_pattern = String::new();
                let mut args_code = Vec::new();

                for (idx, quasi) in t.quasis.iter().enumerate() {
                    fmt_pattern.push_str(&Self::escape_zig_fmt_string(quasi));
                    if idx < t.expressions.len() {
                        fmt_pattern.push_str("{s}");
                        let expr_code = self.generate_expr(&t.expressions[idx]);
                        args_code.push(format!("cp_fmt({})", expr_code));
                    }
                }

                format!(
                    "(std.fmt.allocPrint({}, \"{}\", .{{ {} }}) catch unreachable)",
                    self.current_allocator(),
                    fmt_pattern,
                    args_code.join(", ")
                )
            }
            Expr::Continue(_) => "continue".to_string(),
            Expr::Break(_, val) => {
                let cur_blk = self.current_block_label.clone();
                if let Some(v) = val {
                    let vc = self.generate_expr(v);
                    if let Some(blk) = cur_blk {
                        format!("break :{} {}", blk, vc)
                    } else {
                        format!("break {}", vc)
                    }
                } else {
                    if let Some(blk) = cur_blk {
                        format!("break :{}", blk)
                    } else {
                        "break".to_string()
                    }
                }
            }
            Expr::LabeledBreak(_, label, val) => {
                if let Some(v) = val {
                    format!("break :{} {}", label.sym, self.generate_expr(v))
                } else {
                    format!("break :{}", label.sym)
                }
            }
            Expr::LabeledContinue(_, label) => {
                format!("continue :{}", label.sym)
            }
            Expr::LabeledBlock { label, block } => {
                let saved = self.current_case_content.clone();
                self.current_case_content.clear();
                let len = block.body.len();
                let blk_label = label
                    .as_ref()
                    .map(|l| l.sym.clone())
                    .unwrap_or_else(|| "blk".to_string());
                let old_blk = self.current_block_label.clone();
                self.current_block_label = Some(blk_label.clone());
                for (idx, s) in block.body.iter().enumerate() {
                    if idx + 1 == len {
                        if let Stmt::Expr(e) = s {
                            match e {
                                Expr::Break(_, _)
                                | Expr::LabeledBreak(_, _, _)
                                | Expr::Return(_, _) => {
                                    self.generate_stmt(s);
                                }
                                _ => {
                                    let ec = self.generate_expr(e);
                                    self.write_to_case(&format!("break :{} {};\n", blk_label, ec));
                                }
                            }
                        } else {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(s);
                    }
                }
                self.current_block_label = old_blk;
                let body_str = self.current_case_content.clone();
                self.current_case_content = saved;
                let uses_label = body_str.contains(&format!(":{}", blk_label));
                if label.is_some() || uses_label {
                    format!("{}: {{\n{}}}", blk_label, body_str)
                } else {
                    format!("({{\n{}}})", body_str)
                }
            }
            Expr::Return(_, val) => {
                if let Some(v) = val {
                    format!("return {}", self.generate_expr(v))
                } else {
                    "return".to_string()
                }
            }
            Expr::InlineAsm { is_volatile, content, .. } => {
                let vol = if *is_volatile { " volatile" } else { "" };
                format!("asm{} ({})", vol, content)
            }
            Expr::ComptimeMap { target_ty, entries, .. } => {
                let val_zig_ty = if let Some(ty) = target_ty {
                    self.map_type(ty)
                } else if let Some(first) = entries.first() {
                    if let Some(inferred) = self.infer_cp_type(&first.val) {
                        self.map_type(&inferred)
                    } else if let Expr::Member(m) = &first.val {
                        if let Expr::Ident(id) = &*m.obj {
                            id.sym.clone()
                        } else {
                            "anytype".to_string()
                        }
                    } else if let Expr::EnumInit(ei) = &first.val {
                        if !ei.enum_name.sym.is_empty() {
                            ei.enum_name.sym.clone()
                        } else {
                            "anytype".to_string()
                        }
                    } else {
                        "anytype".to_string()
                    }
                } else {
                    "anytype".to_string()
                };
                let mut entries_str = Vec::new();
                for f in entries {
                    let key_str = if let Some(k) = &f.key {
                        &k.sym
                    } else {
                        ""
                    };
                    let val_code = self.generate_expr(&f.val);
                    entries_str.push(format!(".{{ \"{}\", {} }}", key_str, val_code));
                }
                format!("std.StaticStringMap({}).initComptime(.{{ {} }})", val_zig_ty, entries_str.join(", "))
            }
            Expr::Zig(z) => lower_struct_body(&z.code),
            _ => "0".to_string(),
        }
    }

    pub(crate) fn generate_http_serve(&mut self, call_arg: &Expr) -> String {
        let mut port_expr = "3000".to_string();
        let mut fetch_fn_name = String::new();

        match call_arg {
            Expr::Object(fields) => {
                for f in fields {
                    if let Some(key) = &f.key {
                        if key.sym == "port" {
                            port_expr = self.generate_expr(&f.val);
                        } else if key.sym == "fetch" {
                            match &f.val {
                                Expr::Ident(id) => {
                                    fetch_fn_name = id.sym.clone();
                                }
                                Expr::Arrow(arrow) => {
                                    let handler_name =
                                        format!("http_handler_{}", self.lambda_counter);
                                    self.lambda_counter += 1;
                                    let mut handler_output = String::new();
                                    let mut h_gen = Codegen::new();
                                    h_gen.globals = self.globals.clone();
                                    h_gen.class_defs = self.class_defs.clone();
                                    h_gen.ref_classes = self.ref_classes.clone();
                                    h_gen.value_classes = self.value_classes.clone();
                                    h_gen.enum_defs = self.enum_defs.clone();
                                    h_gen.error_sets = self.error_sets.clone();
                                    h_gen.fn_sigs = self.fn_sigs.clone();
                                    h_gen.trait_defs = self.trait_defs.clone();
                                    h_gen.var_cp_types = self.var_cp_types.clone();
                                    h_gen.anon_obj_structs = self.anon_obj_structs.clone();
                                    h_gen.is_state_machine = false;
                                    h_gen.current_zig_fn_name = handler_name.clone();

                                    let param_name = if !arrow.params.is_empty() {
                                        if let crate::ast::Pattern::Ident(p_id) = &arrow.params[0] {
                                            p_id.sym.clone()
                                        } else {
                                            "req".to_string()
                                        }
                                    } else {
                                        "req".to_string()
                                    };
                                    h_gen
                                        .locals
                                        .insert(param_name.clone(), "*Request".to_string());

                                    writeln!(
                                        handler_output,
                                        "pub fn {}({}: *Request) *Response {{",
                                        handler_name, param_name
                                    )
                                    .unwrap();
                                    writeln!(handler_output, "    _ = &{};", param_name).unwrap();
                                    match &*arrow.body {
                                        crate::ast::ArrowBody::Expr(e) => {
                                            let ret = h_gen.generate_expr(e);
                                            writeln!(handler_output, "    return {};", ret)
                                                .unwrap();
                                        }
                                        crate::ast::ArrowBody::Block(b) => {
                                            for s in &b.body {
                                                h_gen.generate_stmt(s);
                                            }
                                            handler_output.push_str(&h_gen.current_case_content);
                                        }
                                    }
                                    writeln!(handler_output, "}}\n").unwrap();
                                    self.lambda_funcs.push(handler_output);
                                    fetch_fn_name = handler_name;
                                }
                                _ => {
                                    fetch_fn_name = self.generate_expr(&f.val);
                                }
                            }
                        }
                    }
                }
            }
            _ => match call_arg {
                Expr::Ident(id) => fetch_fn_name = id.sym.clone(),
                _ => fetch_fn_name = self.generate_expr(call_arg),
            },
        }

        format!(
            r#"(blk: {{
    const _port: u16 = @intCast({});
    const _addr = @import("std").Io.net.IpAddress.parse("0.0.0.0", _port) catch unreachable;
    const _listener = _addr.listen(global_io, .{{ .reuse_address = true }}) catch unreachable;
    const _s_fd: i32 = @intCast(_listener.socket.handle);
    const _HttpState = struct {{
        server_fd: i32,
        pub fn on_accept(s_ptr: *anyopaque) anyerror!void {{
            const self_state: *@This() = @ptrCast(@alignCast(s_ptr));
            var client_addr: std.os.linux.sockaddr = undefined;
            var client_addr_len: std.os.linux.socklen_t = @sizeOf(std.os.linux.sockaddr);
            const flags = std.os.linux.SOCK.CLOEXEC;
            const rc = std.os.linux.accept4(self_state.server_fd, &client_addr, &client_addr_len, flags);
            register_fd(self_state.server_fd, &on_accept, s_ptr, std.posix.POLL.IN);
            if (rc > 65535) return;
            const client_fd: i32 = @intCast(rc);
            defer _ = std.os.linux.close(client_fd);
            var buf: [4096]u8 = undefined;
            const n = std.os.linux.read(client_fd, &buf, buf.len);
            if (n == 0 or n > buf.len) return;
            const raw_req = buf[0..n];
            var method: []const u8 = "GET";
            var path: []const u8 = "/";
            var body: []const u8 = "";
            var iter = std.mem.splitScalar(u8, raw_req, '\r');
            if (iter.next()) |first_line| {{
                var parts = std.mem.splitScalar(u8, first_line, ' ');
                if (parts.next()) |m| {{ method = m; }}
                if (parts.next()) |p| {{ path = p; }}
            }}
            if (std.mem.indexOf(u8, raw_req, "\r\n\r\n")) |body_idx| {{
                body = raw_req[body_idx + 4..];
            }}
            const req_obj = try Request._create(std.heap.page_allocator);
            req_obj.method = method;
            req_obj.url = path;
            req_obj.body = body;
            const res_obj: *Response = {}(req_obj);
            const status_text = switch (res_obj.status) {{
                200 => "OK",
                201 => "Created",
                204 => "No Content",
                400 => "Bad Request",
                401 => "Unauthorized",
                403 => "Forbidden",
                404 => "Not Found",
                500 => "Internal Server Error",
                else => "OK",
            }};
            const response_str = try std.fmt.allocPrint(std.heap.page_allocator,
                "HTTP/1.1 {{d}} {{s}}\r\nContent-Type: {{s}}\r\nContent-Length: {{d}}\r\nConnection: close\r\n\r\n{{s}}",
                .{{ res_obj.status, status_text, res_obj.contentType, res_obj.body.len, res_obj.body }}
            );
            defer std.heap.page_allocator.free(response_str);
            _ = std.os.linux.write(client_fd, response_str.ptr, response_str.len);
        }}
    }};
    const state_ptr = std.heap.page_allocator.create(_HttpState) catch unreachable;
    state_ptr.* = .{{ .server_fd = _s_fd }};
    register_fd(_s_fd, &_HttpState.on_accept, state_ptr, std.posix.POLL.IN);
    const s_obj = Server._create(std.heap.page_allocator) catch unreachable;
    s_obj.port = _port;
    s_obj.fd = _s_fd;
    break :blk s_obj;
}})"#,
            port_expr, fetch_fn_name
        )
    }

    pub(crate) fn expr_to_zig_pure(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(id) => id.sym.clone(),
            Expr::Lit(lit) => match lit {
                crate::ast::Lit::Int(n) => n.to_string(),
                crate::ast::Lit::Float(f) => f.to_string(),
                crate::ast::Lit::Str(s) => format!("\"{}\"", s),
                crate::ast::Lit::Bool(b) => b.to_string(),
                crate::ast::Lit::Null => "null".to_string(),
                crate::ast::Lit::Undefined => "undefined".to_string(),
            },
            Expr::Member(mem) => {
                format!("{}.{}", self.expr_to_zig_pure(&mem.obj), mem.prop.sym)
            }
            Expr::Bin(bin) => {
                let op_str = match bin.op {
                    BinaryOp::Eq => "==",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::LogicalAnd => "and",
                    BinaryOp::LogicalOr => "or",
                    _ => "==",
                };
                format!(
                    "({} {} {})",
                    self.expr_to_zig_pure(&bin.left),
                    op_str,
                    self.expr_to_zig_pure(&bin.right)
                )
            }
            Expr::Unwrap(inner) => {
                format!("{}.?", self.expr_to_zig_pure(inner))
            }
            _ => "true".to_string(),
        }
    }
}
