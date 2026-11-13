use super::*;
use crate::ast::*;
use crate::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

impl Codegen {
    pub fn is_simple_literal_switch(m: &MatchExpr) -> bool {
        if m.arms.is_empty() {
            return false;
        }
        for arm in &m.arms {
            if arm.guard.is_some() {
                return false;
            }
            match &arm.pat {
                Pattern::Lit(_) | Pattern::Wildcard | Pattern::Range { .. } | Pattern::Expr(_) => {}
                Pattern::Ident(_) => {}
                Pattern::Enum(ep)
                    if ep.fields.is_none()
                        && (ep.tuple_pats.is_none()
                            || ep.tuple_pats.as_ref().map_or(false, |p| {
                                p.len() == 1 && matches!(&p[0], Pattern::Ident(_))
                            })) => {}
                Pattern::Or(pats) => {
                    if !pats.iter().all(|p| {
                        matches!(
                            p,
                            Pattern::Lit(_)
                                | Pattern::Range { .. }
                                | Pattern::Expr(_)
                                | Pattern::Enum(crate::ast::EnumPattern {
                                    fields: None,
                                    tuple_pats: None,
                                    ..
                                })
                        )
                    }) {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }

    pub(crate) fn collect_err_variants_for_type(&self, err_ty: &ErrorType) -> Vec<String> {
        let mut out = Vec::new();
        match err_ty {
            ErrorType::Named(id) => {
                if let Some(en) = self.enum_defs.get(&id.sym) {
                    for v in &en.variants {
                        out.push(v.ident.sym.clone());
                    }
                } else if let Some(es) = self.error_sets.get(&id.sym) {
                    for v in es {
                        out.push(v.sym.clone());
                    }
                }
            }
            ErrorType::AnonSet(ids) => {
                for id in ids {
                    out.push(id.sym.clone());
                }
            }
            ErrorType::Union(ids) => {
                for id in ids {
                    if id.sym.starts_with("error{") && id.sym.ends_with('}') {
                        let inner = &id.sym[6..id.sym.len() - 1];
                        for item in inner.split(',') {
                            let item = item.trim().to_string();
                            if !item.is_empty() && !out.contains(&item) {
                                out.push(item);
                            }
                        }
                    } else {
                        let sub = self.collect_err_variants_for_type(&ErrorType::Named(id.clone()));
                        for s in sub {
                            if !out.contains(&s) {
                                out.push(s);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        out
    }

    pub(crate) fn collect_covered_err_variants(
        pat: &Pattern,
        known_variants: &[String],
        covered: &mut HashSet<String>,
        has_err_catchall: &mut bool,
    ) {
        match pat {
            Pattern::Wildcard => {
                *has_err_catchall = true;
            }
            Pattern::Ident(id) => {
                if known_variants.contains(&id.sym) {
                    covered.insert(id.sym.clone());
                } else {
                    *has_err_catchall = true;
                }
            }
            Pattern::Enum(ep) => {
                if ep.variant_name.sym.is_empty() {
                    *has_err_catchall = true;
                } else {
                    covered.insert(ep.variant_name.sym.clone());
                }
            }
            Pattern::Tuple(_) | Pattern::Object(_) => {
                *has_err_catchall = true;
            }
            Pattern::Or(pats) => {
                for p in pats {
                    Self::collect_covered_err_variants(
                        p,
                        known_variants,
                        covered,
                        has_err_catchall,
                    );
                }
            }
            Pattern::Binding { subpat, .. } => {
                Self::collect_covered_err_variants(
                    subpat,
                    known_variants,
                    covered,
                    has_err_catchall,
                );
            }
            Pattern::TypeMatch { .. } => {
                *has_err_catchall = true;
            }
            _ => {}
        }
    }

    pub(crate) fn check_switch_exhaustiveness(&self, m: &MatchExpr, disc_cp_ty: Option<&Type>) {
        let has_global_wildcard = m
            .arms
            .iter()
            .any(|a| a.guard.is_none() && matches!(a.pat, Pattern::Wildcard));
        if has_global_wildcard {
            return;
        }
        if let Some(ty) = disc_cp_ty {
            match ty {
                Type::ResultType { ok, err } => {
                    let is_opt = matches!(&**ok, Type::Optional(_));
                    let mut has_ok_catchall = false;
                    let mut has_null = false;
                    let mut has_err_catchall = false;
                    let required_err_variants = self.collect_err_variants_for_type(err);
                    let mut covered_err_variants = HashSet::new();

                    for arm in &m.arms {
                        if arm.guard.is_some() {
                            continue;
                        }
                        match &arm.pat {
                            Pattern::Wildcard => {
                                return;
                            }
                            Pattern::Null => {
                                has_null = true;
                            }
                            Pattern::Err(inner_pat) => {
                                Self::collect_covered_err_variants(
                                    inner_pat,
                                    &required_err_variants,
                                    &mut covered_err_variants,
                                    &mut has_err_catchall,
                                );
                            }
                            Pattern::Enum(ep)
                                if required_err_variants.contains(&ep.variant_name.sym)
                                    || ep.enum_name.sym == "err"
                                    || ep.enum_name.sym == "error" =>
                            {
                                covered_err_variants.insert(ep.variant_name.sym.clone());
                            }
                            Pattern::Ident(_) => {
                                has_ok_catchall = true;
                            }
                            Pattern::Or(pats) => {
                                for p in pats {
                                    match p {
                                        Pattern::Null => has_null = true,
                                        Pattern::Err(ip) => Self::collect_covered_err_variants(
                                            ip,
                                            &required_err_variants,
                                            &mut covered_err_variants,
                                            &mut has_err_catchall,
                                        ),
                                        Pattern::Enum(ep)
                                            if required_err_variants
                                                .contains(&ep.variant_name.sym)
                                                || ep.enum_name.sym == "err"
                                                || ep.enum_name.sym == "error" =>
                                        {
                                            covered_err_variants
                                                .insert(ep.variant_name.sym.clone());
                                        }
                                        Pattern::Ident(_) => has_ok_catchall = true,
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if is_opt && !has_null {
                        panic!(
                            "CP Compile Error: Non-exhaustive switch on '?T ! E': missing 'null' branch (or '*' wildcard)"
                        );
                    }
                    if !has_err_catchall {
                        if required_err_variants.is_empty() {
                            panic!(
                                "CP Compile Error: Non-exhaustive switch on 'T ! E': missing 'Err(e)' branch (or '*' wildcard)"
                            );
                        }
                        let missing: Vec<String> = required_err_variants
                            .into_iter()
                            .filter(|v| !covered_err_variants.contains(v))
                            .collect();
                        if !missing.is_empty() {
                            panic!(
                                "CP Compile Error: Non-exhaustive switch: missing error variant(s): {}",
                                missing.join(", ")
                            );
                        }
                    }
                    if !has_ok_catchall && **ok != Type::Void {
                        // Also allow exhaustive enum/bool on ok if covered, otherwise require ok branch
                        let has_any_ok_arm = m.arms.iter().any(|a| {
                            !matches!(a.pat, Pattern::Null | Pattern::Err(_) | Pattern::Wildcard)
                        });
                        if !has_any_ok_arm {
                            panic!(
                                "CP Compile Error: Non-exhaustive switch on 'T ! E': missing success branch"
                            );
                        }
                    }
                }
                Type::Optional(_) => {
                    let has_null = m
                        .arms
                        .iter()
                        .any(|a| a.guard.is_none() && matches!(a.pat, Pattern::Null));
                    if !has_null {
                        panic!(
                            "CP Compile Error: Non-exhaustive switch on optional '?T': missing 'null' branch (or '*' wildcard)"
                        );
                    }
                }
                Type::Ref(ename) => {
                    let mut req_variants = Vec::new();
                    if let Some(en) = self.enum_defs.get(&ename.sym) {
                        for v in &en.variants {
                            req_variants.push(v.ident.sym.clone());
                        }
                    } else if let Some(es) = self.error_sets.get(&ename.sym) {
                        for v in es {
                            req_variants.push(v.sym.clone());
                        }
                    }
                    if !req_variants.is_empty() {
                        let mut covered = HashSet::new();
                        let mut has_catchall = false;
                        for arm in &m.arms {
                            if arm.guard.is_some() {
                                continue;
                            }
                            Self::collect_covered_err_variants(
                                &arm.pat,
                                &req_variants,
                                &mut covered,
                                &mut has_catchall,
                            );
                        }
                        if !has_catchall {
                            let missing: Vec<String> = req_variants
                                .into_iter()
                                .filter(|v| !covered.contains(v))
                                .collect();
                            if !missing.is_empty() {
                                panic!(
                                    "CP Compile Error: Non-exhaustive switch on enum '{}': missing variant(s): {}",
                                    ename.sym,
                                    missing.join(", ")
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn escape_zig_string(s: &str) -> String {
        let mut out = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '\"' {
                out.push('\\');
                out.push('\"');
                i += 2;
            } else if chars[i] == '\"' {
                out.push('\\');
                out.push('\"');
                i += 1;
            } else {
                out.push(chars[i]);
                i += 1;
            }
        }
        out
    }

    pub fn escape_zig_fmt_string(s: &str) -> String {
        let mut out = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '{' {
                out.push_str("{{");
                i += 1;
            } else if chars[i] == '}' {
                out.push_str("}}");
                i += 1;
            } else if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '\"' {
                out.push('\\');
                out.push('\"');
                i += 2;
            } else if chars[i] == '\"' {
                out.push('\\');
                out.push('\"');
                i += 1;
            } else if chars[i] == '\n' {
                out.push_str("\\n");
                i += 1;
            } else if chars[i] == '\r' {
                out.push_str("\\r");
                i += 1;
            } else {
                out.push(chars[i]);
                i += 1;
            }
        }
        out
    }

    pub(crate) fn generate_pipeline(&mut self, left: &Expr, right: &Expr) -> String {
        let pipe_id = self.lambda_counter;
        self.lambda_counter += 1;
        let pipe_var = format!("__pipe_{}", pipe_id);
        let l_code = self.generate_expr(left);
        let left_ty = self.infer_cp_type(left);
        if let Some(ref ty) = left_ty {
            self.var_cp_types.insert(pipe_var.clone(), ty.clone());
        }
        let transformed_rhs = transform_pipeline_rhs(right, &pipe_var);
        let r_code = self.generate_expr(&transformed_rhs);
        if left_ty.is_some() {
            self.var_cp_types.remove(&pipe_var);
        }
        format!(
            "(blk_p_{id}: {{ const {var} = {l_code}; break :blk_p_{id} {r_code}; }})",
            id = pipe_id,
            var = pipe_var,
            l_code = l_code,
            r_code = r_code
        )
    }

    pub(crate) fn generate_pattern_match(
        &self,
        val_expr: &str,
        known_variants: &[String],
        pat: &Pattern,
        bound_types: &mut Vec<(String, Type)>,
        val_cp_ty: Option<&Type>,
    ) -> (String, String) {
        match pat {
            Pattern::Wildcard => ("true".to_string(), String::new()),
            Pattern::Null => (format!("cp_is_null_val({})", val_expr), String::new()),
            Pattern::Lit(Lit::Int(v)) => {
                if let Some(Type::Ref(id)) = val_cp_ty {
                    if id.sym == "Err" || id.sym == "Error" {
                        return (format!("({}.code == {})", val_expr, v), String::new());
                    }
                }
                (format!("cp_eq({}, {})", val_expr, v), String::new())
            }
            Pattern::Lit(Lit::Float(f)) => (format!("cp_eq({}, {})", val_expr, f), String::new()),
            Pattern::Lit(Lit::Bool(b)) => (format!("cp_eq({}, {})", val_expr, b), String::new()),
            Pattern::Lit(Lit::Str(s)) => {
                if let Some(Type::Ref(id)) = val_cp_ty {
                    if id.sym == "Err" || id.sym == "Error" {
                        return (
                            format!(
                                "cp_eq({}.msg, \"{}\")",
                                val_expr,
                                Self::escape_zig_string(s)
                            ),
                            String::new(),
                        );
                    }
                }
                (
                    format!("cp_eq({}, \"{}\")", val_expr, Self::escape_zig_string(s)),
                    String::new(),
                )
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start_str = match start {
                    crate::ast::RangeBound::Int(v) => format!("{}", v),
                    crate::ast::RangeBound::Ident(id) => id.sym.clone(),
                    crate::ast::RangeBound::NegIdent(id) => format!("-{}", id.sym),
                };
                let end_str = match end {
                    crate::ast::RangeBound::Int(v) => format!("{}", v),
                    crate::ast::RangeBound::Ident(id) => id.sym.clone(),
                    crate::ast::RangeBound::NegIdent(id) => format!("-{}", id.sym),
                };
                if *inclusive {
                    (
                        format!("({0} >= {1} and {0} <= {2})", val_expr, start_str, end_str),
                        String::new(),
                    )
                } else {
                    (
                        format!("({0} >= {1} and {0} < {2})", val_expr, start_str, end_str),
                        String::new(),
                    )
                }
            }
            Pattern::Ident(id) => {
                if known_variants.contains(&id.sym) {
                    (
                        format!("cp_match_tag({}, .{})", val_expr, id.sym),
                        String::new(),
                    )
                } else {
                    if let Some(t) = val_cp_ty {
                        bound_types.push((id.sym.clone(), t.clone()));
                    }
                    let defer_drop = if matches!(val_cp_ty, Some(Type::BoxDyn(_))) {
                        format!("defer {}.release(); ", id.sym)
                    } else {
                        String::new()
                    };
                    (
                        "true".to_string(),
                        format!(
                            "const {0} = {1}; _ = &{0}; {2}",
                            id.sym, val_expr, defer_drop
                        ),
                    )
                }
            }
            Pattern::Tuple(pats) => {
                if let Some(Type::Ref(id)) = val_cp_ty {
                    if (id.sym == "Err" || id.sym == "Error") && pats.len() == 2 {
                        let code_expr = format!("{}.code", val_expr);
                        let msg_expr = format!("{}.msg", val_expr);
                        let (c0, b0) = self.generate_pattern_match(
                            &code_expr,
                            known_variants,
                            &pats[0],
                            bound_types,
                            Some(&Type::I32),
                        );
                        let (c1, b1) = self.generate_pattern_match(
                            &msg_expr,
                            known_variants,
                            &pats[1],
                            bound_types,
                            Some(&Type::String),
                        );
                        let mut conds = Vec::new();
                        if c0 != "true" {
                            conds.push(c0);
                        }
                        if c1 != "true" {
                            conds.push(c1);
                        }
                        let cond_str = if conds.is_empty() {
                            "true".to_string()
                        } else {
                            format!("({})", conds.join(" and "))
                        };
                        return (cond_str, format!("{}{}", b0, b1));
                    }
                }
                let mut conds = Vec::new();
                let mut binds = String::new();
                let rest_pos = pats.iter().position(|p| matches!(p, Pattern::Rest(_)));
                for (idx, sub_pat) in pats.iter().enumerate() {
                    if let Pattern::Rest(inner) = sub_pat {
                        if let Pattern::Ident(id) = &**inner {
                            let suffix_len = pats.len() - 1 - idx;
                            let slice_expr =
                                format!("cp_unpack_slice({}, {}, {})", val_expr, idx, suffix_len);
                            let read_only_slice = |elem: Type| Type::BorrowType {
                                inner: Box::new(Type::Array(Box::new(elem))),
                                is_mut: false,
                                lifetimes: vec![],
                            };
                            if let Some(Type::BorrowType {
                                inner,
                                is_mut: false,
                                ..
                            }) = val_cp_ty
                            {
                                if let Type::Array(elem_ty) = &**inner {
                                    bound_types.push((
                                        id.sym.clone(),
                                        read_only_slice((**elem_ty).clone()),
                                    ));
                                }
                            } else if let Some(Type::Array(elem_ty)) = val_cp_ty {
                                bound_types
                                    .push((id.sym.clone(), read_only_slice((**elem_ty).clone())));
                            } else if let Some(Type::Tuple(tys)) = val_cp_ty {
                                let elem_ty = tys
                                    .get(idx)
                                    .cloned()
                                    .or_else(|| tys.first().cloned())
                                    .unwrap_or(Type::I32);
                                bound_types.push((id.sym.clone(), read_only_slice(elem_ty)));
                            } else if let Some(Type::Object(fields)) = val_cp_ty {
                                let elem_ty = fields
                                    .get(idx)
                                    .map(|f| f.ty.clone())
                                    .or_else(|| fields.first().map(|f| f.ty.clone()))
                                    .unwrap_or(Type::I32);
                                bound_types.push((id.sym.clone(), read_only_slice(elem_ty)));
                            }
                            binds.push_str(&format!(
                                "const {0} = {1}; _ = &{0}; ",
                                id.sym, slice_expr
                            ));
                        }
                        continue;
                    }
                    let elem_expr = if let Some(r_idx) = rest_pos {
                        if idx > r_idx {
                            let offset_from_end = pats.len() - idx;
                            format!("cp_unpack_tail({}, {})", val_expr, offset_from_end)
                        } else {
                            format!("cp_unpack_elem({}, {})", val_expr, idx)
                        }
                    } else {
                        format!("cp_unpack_elem({}, {})", val_expr, idx)
                    };
                    let sub_ty = match val_cp_ty {
                        Some(Type::Tuple(tys)) => {
                            if let Some(r_idx) = rest_pos {
                                if idx > r_idx {
                                    let offset = pats.len() - idx;
                                    tys.len().checked_sub(offset).and_then(|ti| tys.get(ti))
                                } else {
                                    tys.get(idx)
                                }
                            } else {
                                tys.get(idx)
                            }
                        }
                        Some(Type::Object(fields)) => {
                            if let Some(r_idx) = rest_pos {
                                if idx > r_idx {
                                    let offset = pats.len() - idx;
                                    fields
                                        .len()
                                        .checked_sub(offset)
                                        .and_then(|ti| fields.get(ti))
                                        .map(|f| &f.ty)
                                } else {
                                    fields.get(idx).map(|f| &f.ty)
                                }
                            } else {
                                fields.get(idx).map(|f| &f.ty)
                            }
                        }
                        Some(Type::Array(elem_ty)) => Some(&**elem_ty),
                        _ => None,
                    };
                    let (c, b) = self.generate_pattern_match(
                        &elem_expr,
                        known_variants,
                        sub_pat,
                        bound_types,
                        sub_ty,
                    );
                    if c != "true" {
                        conds.push(c);
                    }
                    binds.push_str(&b);
                }
                let cond_str = if conds.is_empty() {
                    "true".to_string()
                } else {
                    format!("({})", conds.join(" and "))
                };
                (cond_str, binds)
            }
            Pattern::Object(props) => {
                let mut conds = Vec::new();
                let mut binds = String::new();
                let non_rest_keys: HashSet<String> = props
                    .iter()
                    .filter(|p| !p.is_rest)
                    .map(|p| p.key.sym.clone())
                    .collect();

                for prop in props {
                    if prop.is_rest {
                        if prop.key.sym == "*" || matches!(prop.val, Some(Pattern::Wildcard)) {
                            continue;
                        }
                        let remaining_fields = if let Some(Type::Object(fields)) = &val_cp_ty {
                            fields
                                .iter()
                                .filter(|f| !non_rest_keys.contains(&f.ident.sym))
                                .cloned()
                                .collect::<Vec<_>>()
                        } else {
                            vec![]
                        };
                        let rest_rhs = if remaining_fields.is_empty() {
                            ".{}".to_string()
                        } else {
                            let inits: Vec<String> = remaining_fields
                                .iter()
                                .map(|f| format!(".{} = {}.{}", f.ident.sym, val_expr, f.ident.sym))
                                .collect();
                            format!(".{{ {} }}", inits.join(", "))
                        };
                        let rest_ty = Type::Object(remaining_fields);
                        bound_types.push((prop.key.sym.clone(), rest_ty));
                        binds.push_str(&format!(
                            "const {0} = {1}; _ = &{0}; ",
                            prop.key.sym, rest_rhs
                        ));
                        continue;
                    }

                    let field_expr = format!("{}.{}", val_expr, prop.key.sym);
                    if let Some(sub_pat) = &prop.val {
                        let (c, b) = self.generate_pattern_match(
                            &field_expr,
                            known_variants,
                            sub_pat,
                            bound_types,
                            None,
                        );
                        if c != "true" {
                            conds.push(c);
                        }
                        binds.push_str(&b);
                    } else {
                        binds.push_str(&format!(
                            "const {0} = {1}; _ = &{0}; ",
                            prop.key.sym, field_expr
                        ));
                    }
                }
                let cond_str = if conds.is_empty() {
                    "true".to_string()
                } else {
                    format!("({})", conds.join(" and "))
                };
                (cond_str, binds)
            }
            Pattern::Enum(ep) => {
                let vname = &ep.variant_name.sym;
                let base_cond = format!("cp_match_tag({}, .{})", val_expr, vname);
                if let Some(tuple_pats) = &ep.tuple_pats {
                    let payload_expr = format!("@field({}, \"{}\")", val_expr, vname);
                    if tuple_pats.len() == 1 {
                        let mut sub_ty = None;
                        if !ep.enum_name.sym.is_empty() {
                            if let Some(en) = self.enum_defs.get(&ep.enum_name.sym) {
                                if let Some(v) = en.variants.iter().find(|v| &v.ident.sym == vname)
                                {
                                    sub_ty = v.tuple_type.as_ref();
                                }
                            }
                        }
                        let (sub_c, sub_b) = self.generate_pattern_match(
                            &payload_expr,
                            known_variants,
                            &tuple_pats[0],
                            bound_types,
                            sub_ty,
                        );
                        let full_cond = if sub_c == "true" {
                            base_cond
                        } else {
                            format!("({} and {})", base_cond, sub_c)
                        };
                        return (full_cond, sub_b);
                    } else {
                        let mut binds = String::new();
                        let mut conds = vec![base_cond.clone()];
                        for (idx, p) in tuple_pats.iter().enumerate() {
                            let elem_expr = format!("cp_unpack_elem({}, {})", payload_expr, idx);
                            let (c, b) = self.generate_pattern_match(
                                &elem_expr,
                                known_variants,
                                p,
                                bound_types,
                                None,
                            );
                            if c != "true" {
                                conds.push(c);
                            }
                            binds.push_str(&b);
                        }
                        let cond_str = if conds.is_empty() {
                            "true".to_string()
                        } else {
                            format!("({})", conds.join(" and "))
                        };
                        return (cond_str, binds);
                    }
                } else if let Some(fields) = &ep.fields {
                    let is_struct_pattern = ep.variant_name.sym.is_empty();
                    let payload_expr = if is_struct_pattern {
                        val_expr.to_string()
                    } else {
                        format!("@field({}, \"{}\")", val_expr, vname)
                    };
                    let mut binds = String::new();
                    let mut conds = if is_struct_pattern {
                        vec![]
                    } else {
                        vec![base_cond]
                    };
                    let is_tuple_variant = if !ep.enum_name.sym.is_empty() {
                        self.enum_defs
                            .get(&ep.enum_name.sym)
                            .and_then(|en| en.variants.iter().find(|v| &v.ident.sym == vname))
                            .map_or(false, |v| v.tuple_type.is_some())
                    } else {
                        false
                    };
                    for f in fields {
                        if f.is_rest {
                            if f.key.sym != "_" && f.key.sym != "*" {
                                binds.push_str(&format!(
                                    "const {0} = {1}; _ = &{0}; ",
                                    f.key.sym, payload_expr
                                ));
                            }
                            continue;
                        }
                        let f_expr = if is_tuple_variant {
                            payload_expr.clone()
                        } else {
                            format!("{}.{}", payload_expr, f.key.sym)
                        };
                        if let Some(sub_pat) = &f.val {
                            let (c, b) = self.generate_pattern_match(
                                &f_expr,
                                known_variants,
                                sub_pat,
                                bound_types,
                                None,
                            );
                            if c != "true" {
                                conds.push(c);
                            }
                            binds.push_str(&b);
                        } else {
                            binds.push_str(&format!(
                                "const {0} = {1}; _ = &{0}; ",
                                f.key.sym, f_expr
                            ));
                        }
                    }
                    let cond_str = if conds.is_empty() {
                        "true".to_string()
                    } else {
                        format!("({})", conds.join(" and "))
                    };
                    return (cond_str, binds);
                }
                (base_cond, String::new())
            }
            Pattern::Binding { ident, subpat } => {
                if let Pattern::Ident(type_id) = &**subpat {
                    if self.class_defs.contains_key(&type_id.sym)
                        || self.trait_defs.contains_key(&type_id.sym)
                        || type_id
                            .sym
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_uppercase())
                    {
                        if let Some(Type::Ref(src_id)) = val_cp_ty {
                            if src_id.sym == "Err" || src_id.sym == "Error" {
                                let cond =
                                    format!("({0}.downcast({1}) != null)", val_expr, type_id.sym);
                                let bind = format!(
                                    "const {0}: *const {1} = {2}.downcast({1}).?; _ = &{0}; ",
                                    ident.sym, type_id.sym, val_expr
                                );
                                return (cond, bind);
                            }
                        }
                        let cond = format!(
                            "({0}.vtable.__type_id == {1}.__TYPE_ID)",
                            val_expr, type_id.sym
                        );
                        let bind = format!(
                            "const {0}: *{1} = @ptrCast(@alignCast({2}.ptr)); _ = &{0}; ",
                            ident.sym, type_id.sym, val_expr
                        );
                        return (cond, bind);
                    }
                }
                let (sub_c, sub_b) = self.generate_pattern_match(
                    val_expr,
                    known_variants,
                    subpat,
                    bound_types,
                    val_cp_ty,
                );
                if let Some(t) = val_cp_ty {
                    bound_types.push((ident.sym.clone(), t.clone()));
                }
                let bind_self = format!("const {0} = {1}; _ = &{0}; ", ident.sym, val_expr);
                (sub_c, format!("{}{}", bind_self, sub_b))
            }
            Pattern::TypeMatch { ident, ty } => {
                bound_types.push((ident.sym.clone(), ty.clone()));
                match ty {
                    Type::String => {
                        let cond = format!("(std.meta.activeTag({}) == .string)", val_expr);
                        let bind =
                            format!("const {0} = {1}.string; _ = &{0}; ", ident.sym, val_expr);
                        (cond, bind)
                    }
                    Type::I32 => {
                        let cond = format!("(std.meta.activeTag({}) == .int)", val_expr);
                        let bind = format!("const {0} = {1}.int; _ = &{0}; ", ident.sym, val_expr);
                        (cond, bind)
                    }
                    Type::F32 | Type::F64 => {
                        let cond = format!("(std.meta.activeTag({}) == .float)", val_expr);
                        let bind =
                            format!("const {0} = {1}.float; _ = &{0}; ", ident.sym, val_expr);
                        (cond, bind)
                    }
                    Type::Bool => {
                        let cond = format!("(std.meta.activeTag({}) == .boolean)", val_expr);
                        let bind =
                            format!("const {0} = {1}.boolean; _ = &{0}; ", ident.sym, val_expr);
                        (cond, bind)
                    }
                    Type::Ref(ref_id) => {
                        if let Some(Type::Ref(src_id)) = val_cp_ty {
                            if src_id.sym == "Err" || src_id.sym == "Error" {
                                let cond =
                                    format!("({0}.downcast({1}) != null)", val_expr, ref_id.sym);
                                let bind = format!(
                                    "const {0}: *const {1} = {2}.downcast({1}).?; _ = &{0}; ",
                                    ident.sym, ref_id.sym, val_expr
                                );
                                return (cond, bind);
                            }
                        }
                        let cond = format!(
                            "({0}.vtable.__type_id == {1}.__TYPE_ID)",
                            val_expr, ref_id.sym
                        );
                        let bind = format!(
                            "const {0}: *{1} = @ptrCast(@alignCast({2}.ptr)); _ = &{0}; ",
                            ident.sym, ref_id.sym, val_expr
                        );
                        (cond, bind)
                    }
                    _ => {
                        let cond = "true".to_string();
                        let bind = format!("const {0} = {1}; _ = &{0}; ", ident.sym, val_expr);
                        (cond, bind)
                    }
                }
            }
            Pattern::Or(pats) => {
                let mut conds = Vec::new();
                for p in pats {
                    let (c, _) = self.generate_pattern_match(
                        val_expr,
                        known_variants,
                        p,
                        bound_types,
                        val_cp_ty,
                    );
                    conds.push(c);
                }
                (format!("({})", conds.join(" or ")), String::new())
            }
            Pattern::Expr(expr) => {
                let e_code = Self::format_simple_expr(expr);
                (format!("cp_eq({}, {})", val_expr, e_code), String::new())
            }
            _ => ("true".to_string(), String::new()),
        }
    }

    pub(crate) fn format_simple_expr(expr: &Expr) -> String {
        match expr {
            Expr::Ident(id) => id.sym.clone(),
            Expr::Lit(Lit::Str(s)) => format!("\"{}\"", Self::escape_zig_string(s)),
            Expr::Lit(Lit::Int(v)) => v.to_string(),
            Expr::Lit(Lit::Float(f)) => f.to_string(),
            Expr::Lit(Lit::Bool(b)) => b.to_string(),
            Expr::Member(m) => format!("{}.{}", Self::format_simple_expr(&m.obj), m.prop.sym),
            Expr::Builtin(b) => {
                let args: Vec<String> = b.args.iter().map(Self::format_simple_expr).collect();
                format!("{}({})", b.name, args.join(", "))
            }
            _ => String::new(),
        }
    }

    pub(crate) fn error_type_to_cp_type(&self, err_ty: &ErrorType) -> Type {
        match err_ty {
            ErrorType::Named(id) => Type::Ref(id.clone()),
            ErrorType::BoxDyn(id) => Type::BoxDyn(id.clone()),
            ErrorType::DefaultError => Type::Ref(Ident {
                span: Span { start: 0, end: 0 },
                sym: "Err".to_string(),
            }),
            ErrorType::AnyErr => Type::Ref(Ident {
                span: Span { start: 0, end: 0 },
                sym: "anyerror".to_string(),
            }),
            ErrorType::AnonSet(_) | ErrorType::Union(_) => Type::Ref(Ident {
                span: Span { start: 0, end: 0 },
                sym: self.map_error_type(err_ty),
            }),
        }
    }

    pub(crate) fn format_simple_pat(pat: &Pattern, has_else: &mut bool) -> String {
        match pat {
            Pattern::Expr(expr) => Self::format_simple_expr(expr),
            Pattern::Lit(Lit::Int(v)) => {
                if *v >= 32 && *v <= 126 {
                    let ch = *v as u8;
                    match ch {
                        b'\'' => "'\\''".to_string(),
                        b'\\' => "'\\\\'".to_string(),
                        _ => format!("'{}'", ch as char),
                    }
                } else {
                    v.to_string()
                }
            }
            Pattern::Lit(Lit::Float(f)) => f.to_string(),
            Pattern::Lit(Lit::Str(s)) => format!("\"{}\"", Self::escape_zig_string(s)),
            Pattern::Lit(Lit::Bool(b)) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start_str = match start {
                    crate::ast::RangeBound::Int(v) => format!("{}", v),
                    crate::ast::RangeBound::Ident(id) => id.sym.clone(),
                    crate::ast::RangeBound::NegIdent(id) => format!("-{}", id.sym),
                };
                let end_str = match end {
                    crate::ast::RangeBound::Int(v) => {
                        if *inclusive {
                            format!("{}", v)
                        } else {
                            format!("{}", v.saturating_sub(1))
                        }
                    }
                    crate::ast::RangeBound::Ident(id) => id.sym.clone(),
                    crate::ast::RangeBound::NegIdent(id) => format!("-{}", id.sym),
                };
                format!("{}...{}", start_str, end_str)
            }
            Pattern::Or(pats) => {
                let parts: Vec<String> = pats
                    .iter()
                    .map(|p| Self::format_simple_pat(p, has_else))
                    .collect();
                parts.join(", ")
            }
            Pattern::Wildcard => {
                *has_else = true;
                "else".to_string()
            }
            Pattern::Ident(id) if id.sym == "_" || id.sym == "else" => {
                *has_else = true;
                "else".to_string()
            }
            Pattern::Enum(ep) if ep.fields.is_none() && ep.tuple_pats.is_none() => {
                if ep.enum_name.sym.is_empty() {
                    format!(".{}", ep.variant_name.sym)
                } else {
                    format!("{}.{}", ep.enum_name.sym, ep.variant_name.sym)
                }
            }
            _ => {
                *has_else = true;
                "else".to_string()
            }
        }
    }

    pub(crate) fn collect_pat_variants(pat: &Pattern, set: &mut std::collections::HashSet<String>) {
        match pat {
            Pattern::Enum(ep) if ep.fields.is_none() => {
                set.insert(ep.variant_name.sym.clone());
            }
            Pattern::Or(pats) => {
                for p in pats {
                    Self::collect_pat_variants(p, set);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn generate_switch(&mut self, m: &MatchExpr, is_expr: bool) -> String {
        let disc_cp_ty = self.infer_cp_type(&m.disc);
        self.check_switch_exhaustiveness(m, disc_cp_ty.as_ref());

        if Self::is_simple_literal_switch(m) {
            let disc_code = self.generate_expr(&m.disc);
            let mut arms_code = String::new();
            let mut has_else = false;

            let is_exhaustive_enum = if let Some(Type::Ref(id)) = &disc_cp_ty {
                if let Some(en) = self.enum_defs.get(&id.sym) {
                    let is_pure = en
                        .variants
                        .iter()
                        .all(|v| v.fields.is_none() && v.from_err.is_none());
                    if is_pure {
                        let mut covered_variants = std::collections::HashSet::new();
                        for arm in &m.arms {
                            Self::collect_pat_variants(&arm.pat, &mut covered_variants);
                        }
                        en.variants
                            .iter()
                            .all(|v| covered_variants.contains(&v.ident.sym))
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            for arm in &m.arms {
                if is_exhaustive_enum
                    && (matches!(arm.pat, Pattern::Wildcard)
                        || matches!(&arm.pat, Pattern::Ident(id) if id.sym == "_" || id.sym == "else"))
                {
                    continue;
                }
                let mut bound_ident = None;
                let pat_arm_str = match &arm.pat {
                    Pattern::Enum(ep) if ep.fields.is_none() => {
                        let base = if ep.enum_name.sym.is_empty() {
                            format!(".{}", ep.variant_name.sym)
                        } else {
                            format!("{}.{}", ep.enum_name.sym, ep.variant_name.sym)
                        };
                        let inline_prefix = if arm.is_inline { "inline " } else { "" };
                        if let Some(tup) = &ep.tuple_pats {
                            if tup.len() == 1 {
                                if let Pattern::Ident(id) = &tup[0] {
                                    bound_ident = Some(id.sym.clone());
                                    format!("{}{} => |{}|", inline_prefix, base, id.sym)
                                } else {
                                    format!("{}{} =>", inline_prefix, base)
                                }
                            } else {
                                format!("{}{} =>", inline_prefix, base)
                            }
                        } else if let Some(cap) = &arm.capture {
                            bound_ident = Some(cap.sym.clone());
                            let ptr_prefix = if arm.is_capture_ptr { "*" } else { "" };
                            format!("{}{} => |{}{}|", inline_prefix, base, ptr_prefix, cap.sym)
                        } else {
                            format!("{}{} =>", inline_prefix, base)
                        }
                    }
                    Pattern::Ident(id) if id.sym != "_" && id.sym != "else" => {
                        has_else = true;
                        bound_ident = Some(id.sym.clone());
                        format!("else => |{}|", id.sym)
                    }
                    other => {
                        let s = Self::format_simple_pat(other, &mut has_else);
                        let inline_prefix = if arm.is_inline { "inline " } else { "" };
                        let cap_suffix = if let Some(cap) = &arm.capture {
                            bound_ident = Some(cap.sym.clone());
                            let ptr_prefix = if arm.is_capture_ptr { "*" } else { "" };
                            format!(" |{}{}|", ptr_prefix, cap.sym)
                        } else {
                            "".to_string()
                        };
                        format!("{}{} =>{}", inline_prefix, s, cap_suffix)
                    }
                };

                if let Some(b_id) = &bound_ident {
                    self.locals.insert(b_id.clone(), "anytype".to_string());
                    self.block_locals.insert(b_id.clone());
                }

                let body_str = if is_expr {
                    match &*arm.body {
                        Stmt::Expr(e) => self.generate_expr(e),
                        Stmt::Block(b) if b.body.len() == 1 => {
                            if let Stmt::Expr(e) = &b.body[0] {
                                self.generate_expr(e)
                            } else {
                                let saved = self.current_case_content.clone();
                                self.current_case_content.clear();
                                for s in &b.body {
                                    self.generate_stmt(s);
                                }
                                let s_str = self.current_case_content.trim().to_string();
                                self.current_case_content = saved;
                                format!("{{\n{}\n}}", s_str)
                            }
                        }
                        _ => {
                            let saved = self.current_case_content.clone();
                            self.current_case_content.clear();
                            self.generate_stmt(&arm.body);
                            let s_str = self.current_case_content.trim().to_string();
                            self.current_case_content = saved;
                            format!("{{\n{}\n}}", s_str)
                        }
                    }
                } else {
                    let saved = self.current_case_content.clone();
                    self.current_case_content.clear();
                    if let Stmt::Block(b) = &*arm.body {
                        for s in &b.body {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(&arm.body);
                    }
                    let s_str = self.current_case_content.trim().to_string();
                    self.current_case_content = saved;
                    format!("{{\n{}\n}}", s_str)
                };

                if let Some(b_id) = &bound_ident {
                    self.locals.remove(b_id);
                    self.block_locals.remove(b_id);
                }

                arms_code.push_str(&format!("{} {},\n", pat_arm_str, body_str));
            }
            let all_arms_are_enums = !m.arms.is_empty()
                && m.arms.iter().all(|a| {
                    matches!(a.pat, Pattern::Enum(_))
                        || matches!(&a.pat, Pattern::Or(pats) if pats.iter().all(|p| matches!(p, Pattern::Enum(_))))
                });
            if !has_else && !is_exhaustive_enum && !all_arms_are_enums {
                arms_code.push_str("else => unreachable,\n");
            }
            if is_expr {
                return format!("(switch ({}) {{\n{}}})", disc_code, arms_code);
            } else {
                return format!("switch ({}) {{\n{}}}", disc_code, arms_code);
            }
        }

        let sw_id = self.tmp_counter;
        self.tmp_counter += 1;
        let disc_code = self.generate_expr(&m.disc);
        let blk_label = format!("__sw_blk_{}", sw_id);
        let disc_var = format!("__disc_{}", sw_id);

        let is_result_disc = matches!(
            disc_cp_ty,
            Some(Type::ResultType { .. }) | Some(Type::ErrorUnion(_))
        ) || m.arms.iter().any(|a| matches!(a.pat, Pattern::Err(_)));

        let (ok_cp_ty, err_cp_ty, err_variants) = match &disc_cp_ty {
            Some(Type::ResultType { ok, err }) => {
                let unwrapped_ok = match &**ok {
                    Type::Optional(inner) => (**inner).clone(),
                    other => other.clone(),
                };
                (
                    Some(unwrapped_ok),
                    Some(self.error_type_to_cp_type(err)),
                    self.collect_err_variants_for_type(err),
                )
            }
            Some(Type::ErrorUnion(ok)) => {
                let unwrapped_ok = match &**ok {
                    Type::Optional(inner) => (**inner).clone(),
                    other => other.clone(),
                };
                (
                    Some(unwrapped_ok),
                    Some(Type::Ref(Ident {
                        sym: "anyerror".to_string(),
                        span: Span { start: 0, end: 0 },
                    })),
                    vec![],
                )
            }
            Some(Type::Optional(inner)) => (Some((**inner).clone()), None, vec![]),
            Some(Type::Ref(ename)) => {
                let mut vs = Vec::new();
                if let Some(en) = self.enum_defs.get(&ename.sym) {
                    for v in &en.variants {
                        vs.push(v.ident.sym.clone());
                    }
                } else if let Some(es) = self.error_sets.get(&ename.sym) {
                    for v in es {
                        vs.push(v.sym.clone());
                    }
                }
                (Some(Type::Ref(ename.clone())), None, vs)
            }
            other => (other.clone(), None, vec![]),
        };

        let mut arms_code = String::new();
        for arm in &m.arms {
            let mut bound_types = Vec::new();
            let is_err_pat = if !is_result_disc {
                false
            } else {
                match &arm.pat {
                    Pattern::Err(_) => true,
                    Pattern::Enum(ep) => {
                        ep.enum_name.sym == "err"
                            || ep.enum_name.sym == "error"
                            || self.error_sets.contains_key(&ep.enum_name.sym)
                            || (!ep.variant_name.sym.is_empty()
                                && err_variants.contains(&ep.variant_name.sym))
                    }
                    Pattern::Or(pats) => pats.iter().all(|p| match p {
                        Pattern::Err(_) => true,
                        Pattern::Enum(ep) => {
                            ep.enum_name.sym == "err"
                                || ep.enum_name.sym == "error"
                                || self.error_sets.contains_key(&ep.enum_name.sym)
                                || (!ep.variant_name.sym.is_empty()
                                    && err_variants.contains(&ep.variant_name.sym))
                        }
                        _ => false,
                    }),
                    _ => false,
                }
            };

            let (outer_cond, inner_setup, pat_cond, pat_binds) = match &arm.pat {
                Pattern::Wildcard => (
                    "true".to_string(),
                    String::new(),
                    "true".to_string(),
                    String::new(),
                ),
                Pattern::Null => (
                    format!("cp_is_null_val({})", disc_var),
                    String::new(),
                    "true".to_string(),
                    String::new(),
                ),
                _ if is_err_pat => {
                    let err_val_var = format!("__err_v_{}", sw_id);
                    let actual_pat = match &arm.pat {
                        Pattern::Err(inner) => &**inner,
                        other => other,
                    };
                    let (ec, eb) = self.generate_pattern_match(
                        &err_val_var,
                        &err_variants,
                        actual_pat,
                        &mut bound_types,
                        err_cp_ty.as_ref(),
                    );
                    (
                        format!("(!cp_has_ok_val({0}) and !cp_is_null_val({0}))", disc_var),
                        format!(
                            "const {0} = cp_unwrap_err_val({1}); _ = &{0}; ",
                            err_val_var, disc_var
                        ),
                        ec,
                        eb,
                    )
                }
                other_pat => {
                    if is_result_disc
                        || matches!(disc_cp_ty, Some(Type::Optional(_)))
                        || m.arms.iter().any(|a| matches!(a.pat, Pattern::Null))
                    {
                        let ok_val_var = format!("__ok_v_{}", sw_id);
                        let (oc, ob) = self.generate_pattern_match(
                            &ok_val_var,
                            &err_variants,
                            other_pat,
                            &mut bound_types,
                            ok_cp_ty.as_ref(),
                        );
                        (
                            format!("cp_has_ok_val({})", disc_var),
                            format!(
                                "const {0} = cp_unwrap_ok_val({1}); _ = &{0}; ",
                                ok_val_var, disc_var
                            ),
                            oc,
                            ob,
                        )
                    } else {
                        let (oc, ob) = self.generate_pattern_match(
                            &disc_var,
                            &err_variants,
                            other_pat,
                            &mut bound_types,
                            ok_cp_ty.as_ref(),
                        );
                        ("true".to_string(), String::new(), oc, ob)
                    }
                }
            };

            let old_var_cp_types = self.var_cp_types.clone();
            for (b_name, b_ty) in &bound_types {
                self.var_cp_types.insert(b_name.clone(), b_ty.clone());
                self.block_locals.insert(b_name.clone());
            }

            let guard_cond = if let Some(g) = &arm.guard {
                self.generate_expr(g)
            } else {
                "true".to_string()
            };

            let body_code = if is_expr {
                match &*arm.body {
                    Stmt::Expr(e) => {
                        let ec = self.generate_expr(e);
                        format!("break :{} cp_val({});", blk_label, ec)
                    }
                    Stmt::Return(_) => {
                        let saved = self.current_case_content.clone();
                        self.current_case_content.clear();
                        self.generate_stmt(&arm.body);
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
                                    let ec = self.generate_expr(e);
                                    self.write_to_case(&format!(
                                        "break :{} cp_val({});",
                                        blk_label, ec
                                    ));
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
                        self.generate_stmt(&arm.body);
                        let s_str = self.current_case_content.trim().to_string();
                        self.current_case_content = saved;
                        format!("{} break :{} {{}};", s_str, blk_label)
                    }
                }
            } else {
                let saved = self.current_case_content.clone();
                self.current_case_content.clear();
                if let Stmt::Block(b) = &*arm.body {
                    for s in &b.body {
                        self.generate_stmt(s);
                    }
                } else {
                    self.generate_stmt(&arm.body);
                }
                let s_str = self.current_case_content.trim().to_string();
                self.current_case_content = saved;
                if s_str.ends_with("return;")
                    || s_str.contains("\nreturn")
                    || s_str.starts_with("return ")
                {
                    s_str
                } else {
                    format!("{} break :{};", s_str, blk_label)
                }
            };

            for (b_name, _) in &bound_types {
                self.block_locals.remove(b_name);
            }
            self.var_cp_types = old_var_cp_types;

            let inner_check = if pat_cond == "true" && guard_cond == "true" {
                format!("{}{} {}", inner_setup, pat_binds, body_code)
            } else if guard_cond == "true" {
                format!(
                    "{}if ({}) {{ {} {} }}",
                    inner_setup, pat_cond, pat_binds, body_code
                )
            } else if pat_cond == "true" {
                format!(
                    "{}{} if ({}) {{ {} }}",
                    inner_setup, pat_binds, guard_cond, body_code
                )
            } else {
                format!(
                    "{}if ({}) {{ {} if ({}) {{ {} }} }}",
                    inner_setup, pat_cond, pat_binds, guard_cond, body_code
                )
            };

            if outer_cond == "true" {
                arms_code.push_str(&format!("{{ {} }} ", inner_check));
            } else {
                arms_code.push_str(&format!("if ({}) {{ {} }} ", outer_cond, inner_check));
            }
        }

        if is_expr {
            format!(
                "({}: {{ const {} = {}; _ = &{}; {} unreachable; }})",
                blk_label, disc_var, disc_code, disc_var, arms_code
            )
        } else {
            format!(
                "{}: {{ const {} = {}; _ = &{}; {} }}",
                blk_label, disc_var, disc_code, disc_var, arms_code
            )
        }
    }

    pub(crate) fn generate_err_value(
        &mut self,
        err_inner: &Expr,
        target_err: Option<&ErrorType>,
    ) -> String {
        match err_inner {
            Expr::Lit(Lit::Str(s)) => {
                format!("Err.init(0, \"{}\")", Self::escape_zig_string(s))
            }
            Expr::Lit(Lit::Int(c)) => {
                format!("Err.init({}, \"\")", c)
            }
            Expr::EnumInit(ei) if ei.enum_name.sym.is_empty() => {
                if let Some(dst_err) = target_err {
                    let dst_zig = self.map_error_type(dst_err);
                    if self.error_sets.contains_key(&dst_zig) || dst_zig == "anyerror" {
                        return format!("error.{}", ei.variant_name.sym);
                    }
                    if ei.fields.is_none() && ei.tuple_args.is_none() {
                        return format!("@as({}, .{})", dst_zig, ei.variant_name.sym);
                    }
                    let val_expr = self.generate_expr(err_inner);
                    if val_expr.starts_with(".{") {
                        return format!("{}{}", dst_zig, &val_expr[1..]);
                    }
                    return format!("cp_convert_err({}, {})", dst_zig, val_expr);
                }
                self.generate_expr(err_inner)
            }
            Expr::Ident(id)
                if !self.locals.contains_key(&id.sym) && !self.is_class_field(&id.sym) =>
            {
                if let Some(dst_err) = target_err {
                    let dst_zig = self.map_error_type(dst_err);
                    if self.error_sets.contains_key(&dst_zig) || dst_zig == "anyerror" {
                        return format!("error.{}", id.sym);
                    }
                    return format!("@as({}, .{})", dst_zig, id.sym);
                }
                format!(".{}", id.sym)
            }
            Expr::Member(mem) => {
                if let Expr::Ident(oid) = &*mem.obj {
                    if self.enum_defs.contains_key(&oid.sym)
                        || self.error_sets.contains_key(&oid.sym)
                    {
                        if let Some(dst_err) = target_err {
                            self.verify_error_type_compatibility(
                                &ErrorType::Named(oid.clone()),
                                dst_err,
                            );
                            let dst_zig = self.map_error_type(dst_err);
                            return format!(
                                "cp_convert_err({}, {}.{})",
                                dst_zig, oid.sym, mem.prop.sym
                            );
                        }
                        return format!("{}.{}", oid.sym, mem.prop.sym);
                    }
                }
                let raw = self.generate_expr(err_inner);
                if let Some(dst_err) = target_err {
                    let dst_zig = self.map_error_type(dst_err);
                    format!("cp_convert_err({}, {})", dst_zig, raw)
                } else {
                    raw
                }
            }
            Expr::Call(call) => {
                if let Expr::Member(mem) = &*call.callee {
                    if let Expr::Ident(oid) = &*mem.obj {
                        if self.enum_defs.contains_key(&oid.sym) {
                            if let Some(dst_err) = target_err {
                                self.verify_error_type_compatibility(
                                    &ErrorType::Named(oid.clone()),
                                    dst_err,
                                );
                                let dst_zig = self.map_error_type(dst_err);
                                let raw = self.generate_expr(err_inner);
                                return format!("cp_convert_err({}, {})", dst_zig, raw);
                            }
                        }
                    }
                }
                let raw = self.generate_expr(err_inner);
                if let Some(dst_err) = target_err {
                    let dst_zig = self.map_error_type(dst_err);
                    format!("cp_convert_err({}, {})", dst_zig, raw)
                } else {
                    raw
                }
            }
            Expr::BoxNew(_) => {
                let old_box = self.expected_box_trait.clone();
                if let Some(ErrorType::BoxDyn(tr_id)) = target_err {
                    self.expected_box_trait = Some(tr_id.sym.clone());
                }
                let raw = self.generate_expr(err_inner);
                self.expected_box_trait = old_box;
                raw
            }
            _ => {
                if let Expr::Ident(id) = err_inner {
                    if let Some(Type::Ref(src_id)) = self.var_cp_types.get(&id.sym).cloned() {
                        if let Some(dst_err) = target_err {
                            self.verify_error_type_compatibility(
                                &ErrorType::Named(src_id),
                                dst_err,
                            );
                        }
                    } else if let Some(Type::BoxDyn(tr_id)) =
                        self.var_cp_types.get(&id.sym).cloned()
                    {
                        if let Some(dst_err) = target_err {
                            self.verify_error_type_compatibility(
                                &ErrorType::BoxDyn(tr_id),
                                dst_err,
                            );
                        }
                        self.check_active_borrows_before_move(&id.sym);
                        let gen = self.generate_expr(err_inner);
                        self.moved_vars.insert(id.sym.clone());
                        if !self.is_state_machine {
                            return format!(
                                "(blk_emv: {{ __{}_moved = true; break :blk_emv {}; }})",
                                id.sym, gen
                            );
                        }
                        return gen;
                    }
                }
                let raw = self.generate_expr(err_inner);
                if let Some(dst_err) = target_err {
                    let dst_zig = self.map_error_type(dst_err);
                    format!("cp_convert_err({}, {})", dst_zig, raw)
                } else {
                    raw
                }
            }
        }
    }
}
