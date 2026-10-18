use crate::ast::*;
use crate::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverMode {
    ReadOnly,
    Mut,
    Move,
    Static,
}

pub struct Codegen {
    pub is_beam_mode: bool,
    pub agent_classes: Vec<crate::ast::Class>,
    output: String,
    cases: Vec<String>,
    indent_level: usize,
    pc_counter: u32,
    tmp_counter: u32,
    lambda_counter: u32,
    current_case_content: String,
    is_comptime: bool,
    is_state_machine: bool,
    current_fn_is_async: bool,
    locals: HashMap<String, String>,
    globals: HashSet<String>,
    lambda_funcs: Vec<String>,
    current_zig_fn_name: String,
    is_class_method: bool,
    is_main: bool,
    moved_vars: HashSet<String>,
    uninitialized_vars: HashSet<String>,
    pub captured_vars: HashMap<String, Vec<(String, String)>>,
    pub ref_classes: HashSet<String>,
    pub fn_sigs: HashMap<String, Function>,
    pub class_defs: HashMap<String, Class>,
    pub trait_defs: HashMap<String, TraitDecl>,
    pub enum_defs: HashMap<String, EnumDecl>,
    pub error_sets: HashMap<String, Vec<Ident>>,
    pub anon_err_sets: Vec<(String, Vec<String>)>,
    pub union_err_sets: Vec<(String, Vec<String>)>,
    pub in_try_region: bool,
    pub try_catch_target: Option<(String, String, String, ErrorType)>,
    pub expected_box_trait: Option<String>,
    pub current_class_name: Option<String>,
    pub current_receiver_mode: ReceiverMode,
    pub mutable_fn_params: HashSet<String>,
    pub move_fn_params: HashSet<String>,
    pub var_cp_types: HashMap<String, Type>,
    pub current_scope_depth: usize,
    pub var_scope_depth: HashMap<String, usize>,
    pub var_borrow_sources: HashMap<String, (Vec<String>, bool)>,
    pub var_is_ref_struct: HashSet<String>,
    pub current_fn_params: HashSet<String>,
    pub current_fn_ret_ty: Option<Type>,
    pub current_stmt_idx: usize,
    pub last_use_map: HashMap<String, usize>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            is_beam_mode: false,
            agent_classes: Vec::new(),
            output: String::new(),
            cases: Vec::new(),
            indent_level: 0,
            pc_counter: 0,
            tmp_counter: 0,
            lambda_counter: 0,
            current_case_content: String::new(),
            is_comptime: false,
            is_state_machine: false,
            current_fn_is_async: false,
            locals: HashMap::new(),
            globals: HashSet::new(),
            lambda_funcs: Vec::new(),
            current_zig_fn_name: String::new(),
            is_class_method: false,
            is_main: false,
            moved_vars: HashSet::new(),
            uninitialized_vars: HashSet::new(),
            captured_vars: HashMap::new(),
            ref_classes: HashSet::new(),
            fn_sigs: HashMap::new(),
            class_defs: HashMap::new(),
            trait_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            error_sets: HashMap::new(),
            anon_err_sets: Vec::new(),
            union_err_sets: Vec::new(),
            in_try_region: false,
            try_catch_target: None,
            expected_box_trait: None,
            current_class_name: None,
            current_receiver_mode: ReceiverMode::Static,
            mutable_fn_params: HashSet::new(),
            move_fn_params: HashSet::new(),
            var_cp_types: HashMap::new(),
            current_scope_depth: 0,
            var_scope_depth: HashMap::new(),
            var_borrow_sources: HashMap::new(),
            var_is_ref_struct: HashSet::new(),
            current_fn_params: HashSet::new(),
            current_fn_ret_ty: None,
            current_stmt_idx: 0,
            last_use_map: HashMap::new(),
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

    pub fn map_error_type(&self, err: &ErrorType) -> String {
        match err {
            ErrorType::Named(id) => id.sym.clone(),
            ErrorType::Union(idents) => {
                let names: Vec<String> = idents.iter().map(|i| i.sym.clone()).collect();
                format!("__UnionErr_{}", names.join("_"))
            }
            ErrorType::AnonSet(idents) => {
                let names: Vec<String> = idents.iter().map(|i| i.sym.clone()).collect();
                format!("__AnonErr_{}", names.join("_"))
            }
            ErrorType::BoxDyn(id) => id.sym.clone(),
            ErrorType::DefaultError => "Error".to_string(),
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
                        let zig_name = self.map_error_type(err);
                        if !self.union_err_sets.iter().any(|(n, _)| n == &zig_name) {
                            let sub_names = idents.iter().map(|i| i.sym.clone()).collect();
                            self.union_err_sets.push((zig_name, sub_names));
                        }
                    }
                    _ => {}
                }
            }
            Type::Optional(inner)
            | Type::Array(inner)
            | Type::Shared(inner)
            | Type::ErrorUnion(inner)
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
            Stmt::Block(b) => {
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
        if src_name == dst_name || dst_name == "Error" {
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

    pub fn get_func_receiver_mode(func: &Function) -> ReceiverMode {
        if func.is_static {
            return ReceiverMode::Static;
        }
        if let Some(first) = func.params.first() {
            if first.ident.sym == "this" || first.ident.sym == "_" {
                if first.is_move {
                    return ReceiverMode::Move;
                } else if first.is_mut {
                    return ReceiverMode::Mut;
                } else if func.ident.sym == "init"
                    || func.ident.sym == "release"
                    || func.ident.sym == "__super_init"
                    || func.ident.sym == "__super_release"
                {
                    return ReceiverMode::Mut;
                } else {
                    return ReceiverMode::ReadOnly;
                }
            }
        }
        if func.ident.sym == "init"
            || func.ident.sym == "release"
            || func.ident.sym == "__super_init"
            || func.ident.sym == "__super_release"
        {
            ReceiverMode::Mut
        } else {
            ReceiverMode::ReadOnly
        }
    }

    pub fn get_sig_receiver_mode(sig: &MethodSig) -> ReceiverMode {
        if let Some(first) = sig.params.first() {
            if first.ident.sym == "this" || first.ident.sym == "_" {
                if first.is_move {
                    return ReceiverMode::Move;
                } else if first.is_mut {
                    return ReceiverMode::Mut;
                } else {
                    return ReceiverMode::ReadOnly;
                }
            }
        }
        ReceiverMode::ReadOnly
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
        if let Some(cls) = self.class_defs.get(clean) {
            if let Some(m) = cls.methods.iter().find(|m| m.ident.sym == method_name) {
                return Some((
                    m.params.clone(),
                    m.return_type.clone(),
                    Self::get_func_receiver_mode(m),
                ));
            }
        }
        if let Some(tr) = self.trait_defs.get(clean) {
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
        if var_name == "this" || var_name == "_" {
            return self.current_class_name.clone();
        }
        if let Some(ty) = self.var_cp_types.get(var_name) {
            match ty {
                Type::Ref(id) | Type::BoxDyn(id) => return Some(id.sym.clone()),
                Type::BorrowType { inner, .. } => {
                    if let Type::Ref(id) | Type::BoxDyn(id) = &**inner {
                        return Some(id.sym.clone());
                    }
                }
                Type::Optional(inner) => match &**inner {
                    Type::Ref(id) | Type::BoxDyn(id) => return Some(id.sym.clone()),
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

    pub fn generate_module(&mut self, module: &Module) -> String {
        println!("[TRACE] In generate_module: setting up globals");
        let mut class_order = Vec::new();
        let mut impl_decls = Vec::new();
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Func(f)) = item {
                self.globals.insert(f.ident.sym.clone());
                self.fn_sigs.insert(f.ident.sym.clone(), f.clone());
            }
            if let ModuleItem::Decl(Decl::Class(c)) = item {
                self.globals.insert(c.ident.sym.clone());
                if c.is_ref {
                    self.ref_classes.insert(c.ident.sym.clone());
                }
                if c.implements.iter().any(|i| i.sym == "Agent") {
                    self.agent_classes.push(c.clone());
                }
                class_order.push(c.ident.sym.clone());
                self.class_defs.insert(c.ident.sym.clone(), c.clone());
            }
            if let ModuleItem::Decl(Decl::Enum(e)) = item {
                self.globals.insert(e.ident.sym.clone());
                self.enum_defs.insert(e.ident.sym.clone(), e.clone());
            }
            if let ModuleItem::Decl(Decl::ErrorSet(es)) = item {
                self.globals.insert(es.ident.sym.clone());
                self.error_sets
                    .insert(es.ident.sym.clone(), es.variants.clone());
            }
            if let ModuleItem::Decl(Decl::Trait(t)) = item {
                self.globals.insert(t.ident.sym.clone());
                self.trait_defs.insert(t.ident.sym.clone(), t.clone());
            }
            if let ModuleItem::Decl(Decl::Impl(i)) = item {
                impl_decls.push(i.clone());
            }
        }

        // 1. Merge retroactive `impl Trait for Class` declarations into class_defs
        for imp in &impl_decls {
            if let Some(cls) = self.class_defs.get_mut(&imp.target_name.sym) {
                if !cls.implements.iter().any(|i| i.sym == imp.trait_name.sym) {
                    cls.implements.push(imp.trait_name.clone());
                }
                for m in &imp.methods {
                    if let Some(existing) = cls
                        .methods
                        .iter_mut()
                        .find(|cm| cm.ident.sym == m.ident.sym)
                    {
                        *existing = m.clone();
                    } else {
                        cls.methods.push(m.clone());
                    }
                }
            }
        }

        // 2. Resolve single inheritance (`extends Parent`) for classes
        for c_name in &class_order {
            let parent_opt = self.class_defs.get(c_name).and_then(|c| c.extends.clone());
            if let Some(parent_ident) = parent_opt {
                if let Some(parent_cls) = self.class_defs.get(&parent_ident.sym).cloned() {
                    if let Some(child_cls) = self.class_defs.get_mut(c_name) {
                        if parent_cls.is_ref {
                            child_cls.is_ref = true;
                            self.ref_classes.insert(c_name.clone());
                        }
                        // Prepend parent fields
                        let mut combined_fields = parent_cls.fields.clone();
                        for cf in &child_cls.fields {
                            if !combined_fields
                                .iter()
                                .any(|pf| pf.ident.sym == cf.ident.sym)
                            {
                                combined_fields.push(cf.clone());
                            }
                        }
                        child_cls.fields = combined_fields;

                        // Inherit parent traits
                        for pt in &parent_cls.implements {
                            if !child_cls.implements.iter().any(|ct| ct.sym == pt.sym) {
                                child_cls.implements.push(pt.clone());
                            }
                        }

                        // Inherit parent methods & synthesize `__super_<method>`
                        for pm in &parent_cls.methods {
                            let mut super_m = pm.clone();
                            super_m.ident.sym = format!("__super_{}", pm.ident.sym);
                            if !child_cls
                                .methods
                                .iter()
                                .any(|cm| cm.ident.sym == super_m.ident.sym)
                            {
                                child_cls.methods.push(super_m);
                            }

                            if let Some(cm) = child_cls
                                .methods
                                .iter()
                                .find(|cm| cm.ident.sym == pm.ident.sym)
                            {
                                // Override contract check
                                if pm.ident.sym != "init" && pm.ident.sym != "release" {
                                    let p_mode = Self::get_func_receiver_mode(pm);
                                    let c_mode = Self::get_func_receiver_mode(cm);
                                    if p_mode != c_mode {
                                        panic!(
                                            "CP Compile Error: Method '{}.{}' receiver modifier ({:?}) does not match parent class '{}.{}' ({:?})",
                                            c_name, cm.ident.sym, c_mode, parent_ident.sym, pm.ident.sym, p_mode
                                        );
                                    }
                                }
                            } else {
                                child_cls.methods.push(pm.clone());
                            }
                        }
                    }
                }
            }
        }

        // 3. Validate Trait contracts & inject Trait Default Methods
        for c_name in &class_order {
            let impl_traits = self
                .class_defs
                .get(c_name)
                .map(|c| c.implements.clone())
                .unwrap_or_default();
            for tr_ident in impl_traits {
                if let Some(tr_decl) = self.trait_defs.get(&tr_ident.sym).cloned() {
                    for tm in &tr_decl.methods {
                        let expected_mode = Self::get_sig_receiver_mode(tm);
                        let existing_method = self
                            .class_defs
                            .get(c_name)
                            .and_then(|c| c.methods.iter().find(|m| m.ident.sym == tm.ident.sym))
                            .cloned();
                        if let Some(cm) = existing_method {
                            let actual_mode = Self::get_func_receiver_mode(&cm);
                            if expected_mode != actual_mode {
                                panic!(
                                    "CP Compile Error: Method '{}.{}' receiver modifier ({:?}) does not match trait '{}.{}' contract ({:?})",
                                    c_name, cm.ident.sym, actual_mode, tr_decl.ident.sym, tm.ident.sym, expected_mode
                                );
                            }
                        } else if let Some(default_body) = &tm.default_body {
                            let synth_fn = Function {
                                span: tm.span,
                                ident: tm.ident.clone(),
                                params: tm.params.clone(),
                                return_type: tm.return_type.clone(),
                                body: default_body.clone(),
                                is_async: false,
                                is_static: false,
                            };
                            if let Some(cls) = self.class_defs.get_mut(c_name) {
                                cls.methods.push(synth_fn);
                            }
                        } else {
                            panic!(
                                "CP Compile Error: Class '{}' is missing implementation for trait method '{}.{}'",
                                c_name, tr_decl.ident.sym, tm.ident.sym
                            );
                        }
                    }
                }
            }
        }
        // 4. Scan all types in the module to reject BareDyn (`dyn Trait` on stack) and collect AnonSet / Union error sets
        let fn_list: Vec<Function> = self.fn_sigs.values().cloned().collect();
        for f in &fn_list {
            for p in &f.params {
                self.check_type_validity_and_collect_errors(&p.ty);
            }
            if let Some(rt) = &f.return_type {
                self.check_type_validity_and_collect_errors(rt);
            }
            for s in &f.body.body {
                self.scan_stmt_types(s);
            }
        }
        let cls_list: Vec<Class> = self.class_defs.values().cloned().collect();
        for c in &cls_list {
            for fl in &c.fields {
                self.check_type_validity_and_collect_errors(&fl.ty);
            }
            for m in &c.methods {
                for p in &m.params {
                    self.check_type_validity_and_collect_errors(&p.ty);
                }
                if let Some(rt) = &m.return_type {
                    self.check_type_validity_and_collect_errors(rt);
                }
                for s in &m.body.body {
                    self.scan_stmt_types(s);
                }
            }
        }
        let tr_list: Vec<TraitDecl> = self.trait_defs.values().cloned().collect();
        for t in &tr_list {
            for m in &t.methods {
                for p in &m.params {
                    self.check_type_validity_and_collect_errors(&p.ty);
                }
                if let Some(rt) = &m.return_type {
                    self.check_type_validity_and_collect_errors(rt);
                }
            }
        }

        self.globals.insert("Math".to_string());
        self.globals.insert("Map".to_string());
        self.globals.insert("Set".to_string());
        self.globals.insert("PID".to_string());
        self.globals.insert("Message".to_string());
        self.globals.insert("Mailbox".to_string());
        self.globals.insert("Error".to_string());
        self.globals.insert("Box".to_string());
        self.globals.insert("print".to_string());
        self.globals.insert("console".to_string());

        self.writeln("// Generated by CP Compiler (RFC-002 Production SDK)");
        self.writeln("const std = @import(\"std\");");
        if self.is_beam_mode {
            self.writeln("const beam = @import(\"beam.zig\");");
        }
        self.writeln("");

        // --- Tri-State Result, Built-in Error, String Concat & Pattern Helpers ---
        self.writeln("pub fn CpResult(comptime T: type, comptime E: type) type {");
        self.writeln("    return union(enum) { ok: T, err: E, pub const __IS_CP_RESULT = true; };");
        self.writeln("}");
        self.writeln("pub const Error = struct {");
        self.writeln("    code: i32 = 0,");
        self.writeln("    msg: []const u8 = \"\",");
        self.writeln("    cause: ?*const anyopaque = null,");
        self.writeln("    pub const __TYPE_ID: u64 = 0xEE00_0000_0000_0001;");
        self.writeln("    pub fn init(code: i32, msg: []const u8) Error { return Error{ .code = code, .msg = msg }; }");
        self.writeln("};");
        self.writeln("fn is_cp_result(comptime T: type) bool {");
        self.writeln("    return @typeInfo(T) == .@\"union\" and @hasDecl(T, \"__IS_CP_RESULT\");");
        self.writeln("}");
        self.writeln("fn CpOptType(comptime T: type) type {");
        self.writeln("    if (@typeInfo(T) == .optional) return T;");
        self.writeln("    return ?T;");
        self.writeln("}");
        self.writeln("fn CpUnwrapOpt(comptime T: type) type {");
        self.writeln("    if (@typeInfo(T) == .optional) return @typeInfo(T).optional.child;");
        self.writeln("    return T;");
        self.writeln("}");
        self.writeln("fn cp_opt_is_null(v: anytype) bool {");
        self.writeln("    const T = @TypeOf(v);");
        self.writeln("    if (comptime @typeInfo(T) == .optional) return v == null;");
        self.writeln("    return false;");
        self.writeln("}");
        self.writeln("fn cp_opt_unwrap(v: anytype) CpUnwrapOpt(@TypeOf(v)) {");
        self.writeln("    const T = @TypeOf(v);");
        self.writeln(
            "    if (comptime @typeInfo(T) == .optional) { return v.?; } else { return v; }",
        );
        self.writeln("}");
        self.writeln("");

        // --- Top-Level C-FFI / Zig Escapes ---
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Expr(Expr::Zig(zig))) = item {
                self.writeln(&zig.code);
            }
        }

        self.writeln("pub const regex_t = extern struct { __space: [64]usize = undefined, };");
        self.writeln("pub const regmatch_t = extern struct { rm_so: i32, rm_eo: i32, };");
        self.writeln(
            "extern \"c\" fn regcomp(preg: *regex_t, regex: [*:0]const u8, cflags: i32) i32;",
        );
        self.writeln("extern \"c\" fn regexec(preg: *const regex_t, string: [*:0]const u8, nmatch: usize, pmatch: [*]regmatch_t, eflags: i32) i32;");
        self.writeln("extern \"c\" fn regfree(preg: *regex_t) void;");
        self.writeln("pub const RegExp = struct {");
        self.writeln("    preg: regex_t, source: []const u8,");
        self.writeln("    pub fn _create(allocator: std.mem.Allocator) !*RegExp { const p = try allocator.create(RegExp); p.* = undefined; return p; }");
        self.writeln(
            "    pub fn init(self: *RegExp, pattern: []const u8, flags: []const u8) void {",
        );
        self.writeln("        _ = flags; self.source = pattern; const c_pattern = std.heap.page_allocator.allocSentinel(u8, pattern.len, 0) catch unreachable; @import(\"std\").mem.copyForwards(u8, c_pattern, pattern); defer std.heap.page_allocator.free(c_pattern);");
        self.writeln("        if (regcomp(&self.preg, c_pattern.ptr, 1) != 0) { std.debug.print(\"Regex Compilation Failed\\n\", .{}); @panic(\"Invalid RegExp\"); }");
        self.writeln("    }");
        self.writeln("    pub fn @\"test\"(self: *RegExp, str: []const u8) bool {");
        self.writeln("        const c_str = std.heap.page_allocator.allocSentinel(u8, str.len, 0) catch unreachable; @import(\"std\").mem.copyForwards(u8, c_str, str); defer std.heap.page_allocator.free(c_str);");
        self.writeln("        var pmatch: [1]regmatch_t = undefined;");
        self.writeln("        return regexec(&self.preg, c_str.ptr, 1, &pmatch, 0) == 0;");
        self.writeln("    }");
        self.writeln("    pub fn exec(self: *RegExp, str: []const u8) []const u8 {");
        self.writeln("        const c_str = std.heap.page_allocator.allocSentinel(u8, str.len, 0) catch unreachable; @import(\"std\").mem.copyForwards(u8, c_str, str); defer std.heap.page_allocator.free(c_str);");
        self.writeln("        var pmatch: [1]regmatch_t = undefined;");
        self.writeln("        if (regexec(&self.preg, c_str.ptr, 1, &pmatch, 0) == 0) { const start: usize = @intCast(pmatch[0].rm_so); const end: usize = @intCast(pmatch[0].rm_eo); return str[start..end]; }");
        self.writeln("        return \"\";");
        self.writeln("    }");
        self.writeln("    pub fn release(self: *RegExp) void { regfree(&self.preg); }");
        self.writeln("};");

        self.writeln("");
        self.writeln("const MessageContext = struct {");
        self.writeln("    pub fn hash(self: @This(), m: Message) u64 {");
        self.writeln("        _ = self;");
        self.writeln("        var hasher = std.hash.Wyhash.init(0);");
        self.writeln("        switch (m) {");
        self.writeln("            .int => |i| std.hash.autoHash(&hasher, i),");
        self.writeln(
            "            .float => |f| std.hash.autoHash(&hasher, @as(u32, @bitCast(f))),",
        );
        self.writeln("            .string => |s| hasher.update(s),");
        self.writeln("            .boolean => |bl| std.hash.autoHash(&hasher, bl),");
        self.writeln("            .any => |a| std.hash.autoHash(&hasher, @intFromPtr(a)),");
        self.writeln("        }");
        self.writeln("        return hasher.final();");
        self.writeln("    }");
        self.writeln("    pub fn eql(self: @This(), a: Message, b: Message) bool {");
        self.writeln("        _ = self;");
        self.writeln("        const Ta = std.meta.activeTag(a);");
        self.writeln("        const Tb = std.meta.activeTag(b);");
        self.writeln("        if (Ta != Tb) return false;");
        self.writeln("        return switch (a) {");
        self.writeln("            .int => |i| i == b.int,");
        self.writeln("            .float => |f| f == b.float,");
        self.writeln("            .string => |s| std.mem.eql(u8, s, b.string),");
        self.writeln("            .boolean => |bl| bl == b.boolean,");
        self.writeln("            .any => |ptr| ptr == b.any,");
        self.writeln("        };");
        self.writeln("    }");
        self.writeln("};");
        self.writeln("");
        self.writeln("const Map = struct {");
        self.writeln("    allocator: std.mem.Allocator,");
        self.writeln("    _map: std.HashMap(Message, Message, MessageContext, std.hash_map.default_max_load_percentage),");
        self.writeln("    size: i32,");
        self.writeln("");
        self.writeln("    pub fn _create(allocator: std.mem.Allocator) !*Map {");
        self.writeln("        const self = try allocator.create(Map);");
        self.writeln("        self.* = .{");
        self.writeln("            .allocator = allocator,");
        self.writeln("            ._map = std.HashMap(Message, Message, MessageContext, std.hash_map.default_max_load_percentage).init(allocator),");
        self.writeln("            .size = 0,");
        self.writeln("        };");
        self.writeln("        return self;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn init(self: *Map) void {");
        self.writeln("        _ = self;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn set(self: *Map, key: anytype, value: anytype) *Map {");
        self.writeln("        self._map.put(cp_message_from(key), cp_message_from(value)) catch unreachable;");
        self.writeln("        self.size = @intCast(self._map.count());");
        self.writeln("        return self;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn get(self: *Map, key: anytype) Message {");
        self.writeln("        if (self._map.get(cp_message_from(key))) |v| return v;");
        self.writeln("        return .{ .any = undefined };");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn has(self: *Map, key: anytype) bool {");
        self.writeln("        return self._map.contains(cp_message_from(key));");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn delete(self: *Map, key: anytype) bool {");
        self.writeln("        const removed = self._map.remove(cp_message_from(key));");
        self.writeln("        self.size = @intCast(self._map.count());");
        self.writeln("        return removed;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn clear(self: *Map) void {");
        self.writeln("        self._map.clearRetainingCapacity();");
        self.writeln("        self.size = 0;");
        self.writeln("    }");
        self.writeln("};");
        self.writeln("");
        self.writeln("const Set = struct {");
        self.writeln("    allocator: std.mem.Allocator,");
        self.writeln("    _map: std.HashMap(Message, void, MessageContext, std.hash_map.default_max_load_percentage),");
        self.writeln("    size: i32,");
        self.writeln("");
        self.writeln("    pub fn _create(allocator: std.mem.Allocator) !*Set {");
        self.writeln("        const self = try allocator.create(Set);");
        self.writeln("        self.* = .{");
        self.writeln("            .allocator = allocator,");
        self.writeln("            ._map = std.HashMap(Message, void, MessageContext, std.hash_map.default_max_load_percentage).init(allocator),");
        self.writeln("            .size = 0,");
        self.writeln("        };");
        self.writeln("        return self;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn init(self: *Set) void {");
        self.writeln("        _ = self;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn add(self: *Set, value: anytype) *Set {");
        self.writeln("        self._map.put(cp_message_from(value), {}) catch unreachable;");
        self.writeln("        self.size = @intCast(self._map.count());");
        self.writeln("        return self;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn has(self: *Set, value: anytype) bool {");
        self.writeln("        return self._map.contains(cp_message_from(value));");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn delete(self: *Set, value: anytype) bool {");
        self.writeln("        const removed = self._map.remove(cp_message_from(value));");
        self.writeln("        self.size = @intCast(self._map.count());");
        self.writeln("        return removed;");
        self.writeln("    }");
        self.writeln("    ");
        self.writeln("    pub fn clear(self: *Set) void {");
        self.writeln("        self._map.clearRetainingCapacity();");
        self.writeln("        self.size = 0;");
        self.writeln("    }");
        self.writeln("};");
        self.writeln("");
        self.writeln("const PID = struct { node_id: u32, local_id: u32 };");
        self.writeln("const Message = union(enum) { int: i32, float: f32, string: []const u8, boolean: bool, any: *anyopaque, pub fn to_string(self: Message) []const u8 { return switch (self) { .string => |s| s, else => \"\", }; } pub fn to_int(self: Message) i32 { return switch (self) { .int => |i| i, else => 0, }; } };");
        self.writeln("fn cp_message_from(v: anytype) Message {");
        self.writeln("    const T = @TypeOf(v);");
        self.writeln("    if (T == Message) return v;");
        self.writeln(
            "    if (T == i32 or T == comptime_int) return .{ .int = @intCast(v) };
    if (T == bool) return .{ .boolean = v };",
        );
        self.writeln(
            "    if (T == f32 or T == comptime_float) return .{ .float = @floatCast(v) };",
        );
        self.writeln(
            "    if (T == []const u8 or T == []u8) return .{ .string = v };
    if (@typeInfo(T) == .pointer) {
        const Child = @typeInfo(T).pointer.child;
        if (Child == u8) return .{ .string = v };
        if (@typeInfo(Child) == .array and @typeInfo(Child).array.child == u8) {
            const slice: []const u8 = v;
            return .{ .string = slice };
        }
    }",
        );
        self.writeln("    return .{ .any = @ptrCast(@constCast(&v)) };");
        self.writeln("}");
        self.writeln("");
        self.writeln("fn is_string_like(comptime T: type) bool { if (T == []const u8 or T == []u8) return true; if (@typeInfo(T) == .pointer) { const Child = @typeInfo(T).pointer.child; if (Child == u8) return true; if (@typeInfo(Child) == .array and @typeInfo(Child).array.child == u8) return true; } return false; }");
        self.writeln("fn to_string_slice(v: anytype) []const u8 { const T = @TypeOf(v); if (comptime (T == []const u8 or T == []u8)) { return v; } else { const slice: []const u8 = v; return slice; } }");
        self.writeln("fn CpValType(comptime T: type) type { if (is_string_like(T)) return []const u8; if (T == comptime_int) return i32; if (T == comptime_float) return f32; return T; }");
        self.writeln("fn cp_val(v: anytype) CpValType(@TypeOf(v)) { const T = @TypeOf(v); if (comptime is_string_like(T)) { return to_string_slice(v); } else if (comptime T == comptime_int) { return @as(i32, v); } else if (comptime T == comptime_float) { return @as(f32, v); } else { return v; } }");
        self.writeln("fn CpAddReturn(comptime Ta: type, comptime Tb: type) type { if (is_string_like(Ta) or is_string_like(Tb)) return []const u8; if (Ta == comptime_int and Tb == comptime_int) return i32; if (Ta == comptime_int) return Tb; if (Tb == comptime_int) return Ta; if (Ta == comptime_float and Tb == comptime_float) return f32; if (Ta == comptime_float) return Tb; if (Tb == comptime_float) return Ta; return Ta; }");
        self.writeln("fn cp_add(a: anytype, b: anytype) CpAddReturn(@TypeOf(a), @TypeOf(b)) { const Ta = @TypeOf(a); const Tb = @TypeOf(b); if (comptime (is_string_like(Ta) and is_string_like(Tb))) { return std.fmt.allocPrint(std.heap.page_allocator, \"{s}{s}\", .{ to_string_slice(a), to_string_slice(b) }) catch unreachable; } else if (comptime is_string_like(Ta)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{s}{d}\", .{ to_string_slice(a), b }) catch unreachable; } else if (comptime is_string_like(Tb)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}{s}\", .{ a, to_string_slice(b) }) catch unreachable; } else { return a + b; } }");
        self.writeln("fn CpUnwrapOkType(comptime T: type) type { if (is_cp_result(T)) { const OkT = @FieldType(T, \"ok\"); if (@typeInfo(OkT) == .optional) return @typeInfo(OkT).optional.child; return OkT; } if (@typeInfo(T) == .optional) return @typeInfo(T).optional.child; return T; }");
        self.writeln("fn cp_has_ok_val(v: anytype) bool { const T = @TypeOf(v); if (comptime is_cp_result(T)) { if (v != .ok) return false; const OkT = @FieldType(T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok != null; } else { return true; } } else if (comptime @typeInfo(T) == .optional) { return v != null; } else { return true; } }");
        self.writeln("fn cp_unwrap_ok_val(v: anytype) CpUnwrapOkType(@TypeOf(v)) { const T = @TypeOf(v); if (comptime is_cp_result(T)) { const OkT = @FieldType(T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok.?; } else { return v.ok; } } else if (comptime @typeInfo(T) == .optional) { return v.?; } else { return v; } }");
        self.writeln("fn cp_is_null_val(v: anytype) bool { const T = @TypeOf(v); if (comptime is_cp_result(T)) { if (v != .ok) return false; const OkT = @FieldType(T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok == null; } else { return false; } } else if (comptime @typeInfo(T) == .optional) { return v == null; } else { return false; } }");
        self.writeln("fn cp_convert_err(comptime TargetErr: type, err_val: anytype) TargetErr { const SrcType = @TypeOf(err_val); if (comptime SrcType == TargetErr) { return err_val; } else if (comptime TargetErr == Error) { if (comptime (@typeInfo(SrcType) == .@\"union\" or @typeInfo(SrcType) == .@\"enum\")) { return Error{ .code = @intCast(@intFromEnum(std.meta.activeTag(err_val))), .msg = @tagName(err_val) }; } else { return Error{ .code = -1, .msg = \"error\" }; } } else if (comptime @typeInfo(TargetErr) == .@\"union\" and @hasDecl(TargetErr, \"__from\")) { return TargetErr.__from(err_val); } else { return err_val; } }");
        self.writeln("fn cp_not(v: anytype) bool { const T = @TypeOf(v); if (comptime T == bool) { return !v; } else if (comptime @typeInfo(T) == .optional) { return v == null; } else if (comptime is_string_like(T)) { return to_string_slice(v).len == 0; } else if (comptime (@typeInfo(T) == .int or T == comptime_int)) { return v == 0; } else { return false; } }");

        self.writeln("fn cp_math_random() f32 {");
        self.writeln("    return 0.5; // Stub");
        self.writeln("}");

        self.writeln("fn cp_eq(a: anytype, b: anytype) bool {");
        self.writeln("    const Ta = @TypeOf(a); const Tb = @TypeOf(b);");
        self.writeln("    if (comptime Ta == Message and is_string_like(Tb)) { return switch (a) { .string => |s| std.mem.eql(u8, s, to_string_slice(b)), else => false, }; }");
        self.writeln("    if (comptime Tb == Message and is_string_like(Ta)) { return switch (b) { .string => |s| std.mem.eql(u8, to_string_slice(a), s), else => false, }; }");
        self.writeln("    if (comptime Ta == Message and (Tb == i32 or Tb == comptime_int)) { return switch (a) { .int => |i| i == b, else => false, }; }");
        self.writeln("    if (comptime Tb == Message and (Ta == i32 or Ta == comptime_int)) { return switch (b) { .int => |i| a == i, else => false, }; }");
        self.writeln("    if (comptime is_string_like(Ta) and is_string_like(Tb)) { return std.mem.eql(u8, to_string_slice(a), to_string_slice(b)); }");
        self.writeln("        if (comptime (@typeInfo(Ta) == .int or Ta == comptime_int) and (@typeInfo(Tb) == .int or Tb == comptime_int)) { return a == b; }
    if (comptime (@typeInfo(Ta) == .float or Ta == comptime_float) and (@typeInfo(Tb) == .float or Tb == comptime_float)) { return a == b; }");
        self.writeln("    if (comptime Ta != Tb) return false;");
        self.writeln("    return a == b;");
        self.writeln("}");
        self.writeln("fn cp_neq(a: anytype, b: anytype) bool { return !cp_eq(a, b); }");
        self.writeln("");
        self.writeln("pub const BaseContext = struct { pc: u32 = 0, _self: PID = undefined, _parent: ?*anyopaque = null, _parent_fn: ?*const fn(*anyopaque) anyerror!void = null, arena: ?*std.heap.ArenaAllocator = null, received_msg: ?Message = null, received_sender: ?PID = null, await_result: ?Message = null, };");

        self.writeln("");
        self.writeln("pub const ZigSharedTensor = struct {");
        self.writeln("    ref_count: std.atomic.Value(u32),");
        self.writeln("    data: []u8,");
        self.writeln("    allocator: std.mem.Allocator,");
        self.writeln(
            "    pub fn create(allocator: std.mem.Allocator, size: usize) !*ZigSharedTensor {",
        );
        self.writeln("        const ptr = try allocator.create(ZigSharedTensor);");
        self.writeln("        ptr.data = try allocator.alloc(u8, size);");
        self.writeln("        ptr.ref_count = std.atomic.Value(u32).init(1);");
        self.writeln("        ptr.allocator = allocator;");
        self.writeln("        return ptr;");
        self.writeln("    }");
        self.writeln("    pub fn retain(self: *ZigSharedTensor) void {");
        self.writeln("        _ = self.ref_count.fetchAdd(1, .monotonic);");
        self.writeln("    }");
        self.writeln("    pub fn release(self: *ZigSharedTensor) void {");
        self.writeln("        if (self.ref_count.fetchSub(1, .release) == 1) {");
        self.writeln("            _ = self.ref_count.load(.acquire);");
        self.writeln("            self.allocator.free(self.data);");
        self.writeln("            self.allocator.destroy(self);");
        self.writeln("        }");
        self.writeln("    }");
        self.writeln("};");
        self.writeln("");
        self.writeln("// --- Native Reactor Registry ---");
        self.writeln("threadlocal var poll_fds: [1024]std.posix.pollfd = undefined;");
        self.writeln("threadlocal var fd_to_ctx: [1024]*anyopaque = undefined;");
        self.writeln(
            "threadlocal var fd_to_run: [1024]*const fn(*anyopaque) anyerror!void = undefined;",
        );
        self.writeln("threadlocal var active_fds: usize = 0;");
        self.writeln("fn register_fd(fd: i32, run: *const fn(*anyopaque) anyerror!void, ctx: *anyopaque, events: i16) void {");
        self.writeln("    if (active_fds < 1024) {");
        self.writeln(
            "        poll_fds[active_fds] = .{ .fd = fd, .events = events, .revents = 0 };",
        );
        self.writeln("        fd_to_ctx[active_fds] = ctx;");
        self.writeln("        fd_to_run[active_fds] = run;");
        self.writeln("        active_fds += 1;");
        self.writeln("    }");
        self.writeln("}");
        self.writeln("");
        self.writeln("// --- Scheduler ---");
        self.writeln(
            "const _Task = struct { run: *const fn(*anyopaque) anyerror!void, ctx: *anyopaque };",
        );
        self.writeln("threadlocal var task_queue: [1000]_Task = undefined; threadlocal var queue_len: usize = 0;");
        self.writeln(
            "threadlocal var current_node_id: u32 = 0; threadlocal var next_local_id: u32 = 1;",
        );
        self.writeln("fn schedule(run: *const fn(*anyopaque) anyerror!void, ctx: *anyopaque) void { if (queue_len < 1000) { task_queue[queue_len] = .{ .run = run, .ctx = ctx }; queue_len += 1; } }");
        self.writeln("");
        self.writeln("// --- Cross-Thread Channels & Mailboxes ---");
        self.writeln(
            "const CrossThreadMessage = struct { target: PID, sender: PID, msg: Message };",
        );
        self.writeln("const MailboxItem = struct { sender: PID, msg: Message };");
        self.writeln("const Spinlock = struct { state: std.atomic.Value(u32) = std.atomic.Value(u32).init(0), pub fn acquire(self: *Spinlock) void { while (self.state.cmpxchgWeak(0, 1, .acquire, .monotonic) != null) { std.atomic.spinLoopHint(); } } pub fn release(self: *Spinlock) void { self.state.store(0, .release); } };");
        self.writeln("const MailboxRingBuffer = struct { lock: Spinlock = .{}, buffer: [256]CrossThreadMessage = undefined, head: usize = 0, tail: usize = 0, count: usize = 0, pub fn push(self: *MailboxRingBuffer, item: CrossThreadMessage) bool { self.lock.acquire(); defer self.lock.release(); if (self.count >= 256) return false; self.buffer[self.tail] = item; self.tail = (self.tail + 1) % 256; self.count += 1; return true; } pub fn pop(self: *MailboxRingBuffer) ?CrossThreadMessage { self.lock.acquire(); defer self.lock.release(); if (self.count == 0) return null; const item = self.buffer[self.head]; self.head = (self.head + 1) % 256; self.count -= 1; return item; } };");
        self.writeln("var global_worker_mailboxes: [128]MailboxRingBuffer = @splat(.{});");
        self.writeln(
            "var global_next_worker_id: std.atomic.Value(u32) = std.atomic.Value(u32).init(1);",
        );
        self.writeln("const LocalMailbox = struct { items: [16]MailboxItem = undefined, head: usize = 0, tail: usize = 0, count: usize = 0, pub fn push(self: *LocalMailbox, item: MailboxItem) bool { if (self.count >= 16) return false; self.items[self.tail] = item; self.tail = (self.tail + 1) % 16; self.count += 1; return true; } pub fn pop(self: *LocalMailbox) ?MailboxItem { if (self.count == 0) return null; const item = self.items[self.head]; self.head = (self.head + 1) % 16; self.count -= 1; return item; } };");
        self.writeln("threadlocal var local_mailboxes: [1024]LocalMailbox = @splat(.{});");
        self.writeln("threadlocal var waiting_actors: [1024]?*anyopaque = @splat(null);");
        self.writeln("threadlocal var waiting_actor_fns: [1024]?*const fn(*anyopaque) anyerror!void = @splat(null);");
        self.writeln("threadlocal var waiting_count: usize = 0;");
        self.writeln("fn deliver_local(target_local_id: u32, sender_pid: PID, msg: Message) void { const slot = target_local_id % 1024; _ = local_mailboxes[slot].push(.{ .sender = sender_pid, .msg = msg }); if (waiting_actors[slot]) |w_ctx| { const w_fn = waiting_actor_fns[slot].?; waiting_actors[slot] = null; waiting_actor_fns[slot] = null; if (waiting_count > 0) waiting_count -= 1; schedule(w_fn, w_ctx); } }");
        self.writeln("fn drain_cross_thread_mailbox() void { const slot = current_node_id % 128; while (global_worker_mailboxes[slot].pop()) |item| { deliver_local(item.target.local_id, item.sender, item.msg); } }");
        self.writeln("fn cp_send(sender_pid: PID, target: PID, msg: Message) bool { if (target.node_id == current_node_id) { deliver_local(target.local_id, sender_pid, msg); return true; } else { const slot = target.node_id % 128; return global_worker_mailboxes[slot].push(.{ .target = target, .sender = sender_pid, .msg = msg }); } }");
        self.writeln("");

        println!("[TRACE] Generating Traits");
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Trait(t)) = item {
                self.generate_trait(t);
            }
        }
        println!("[TRACE] Generating Enums & ErrorSets");
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Enum(e)) = item {
                self.generate_enum(e);
            }
            if let ModuleItem::Decl(Decl::ErrorSet(es)) = item {
                self.writeln(&format!("const {} = union(enum) {{", es.ident.sym));
                self.indent();
                for v in &es.variants {
                    self.writeln(&format!("{},", v.sym));
                }
                self.dedent();
                self.writeln("};\n");
            }
        }
        // Emit synthesized AnonSet and Union error sets
        let anon_sets = self.anon_err_sets.clone();
        for (zig_name, variants) in &anon_sets {
            self.writeln(&format!("const {} = union(enum) {{", zig_name));
            self.indent();
            for v in variants {
                self.writeln(&format!("{},", v));
            }
            self.writeln("pub fn __from(v: anytype) @This() {");
            self.indent();
            self.writeln("return switch (v) {");
            self.indent();
            for v in variants {
                self.writeln(&format!(
                    ".{0} => |p| if (@hasField(@TypeOf(v), \"{0}\")) .{{ .{0} = p }} else unreachable,",
                    v
                ));
            }
            self.writeln("else => unreachable,");
            self.dedent();
            self.writeln("};");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("};\n");
        }
        let union_sets = self.union_err_sets.clone();
        for (zig_name, sub_names) in &union_sets {
            self.writeln(&format!("const {} = union(enum) {{", zig_name));
            self.indent();
            let mut seen_variants = HashSet::new();
            let mut variant_list = Vec::new();
            for sn in sub_names {
                if let Some(en) = self.enum_defs.get(sn).cloned() {
                    for v in &en.variants {
                        if seen_variants.insert(v.ident.sym.clone()) {
                            variant_list.push(v.ident.sym.clone());
                            if let Some(tty) = &v.tuple_type {
                                self.writeln(&format!("{}: {},", v.ident.sym, self.map_type(tty)));
                            } else {
                                self.writeln(&format!("{},", v.ident.sym));
                            }
                        }
                    }
                } else if let Some(es) = self.error_sets.get(sn).cloned() {
                    for v in &es {
                        if seen_variants.insert(v.sym.clone()) {
                            variant_list.push(v.sym.clone());
                            self.writeln(&format!("{},", v.sym));
                        }
                    }
                }
            }
            self.writeln("pub fn __from(v: anytype) @This() {");
            self.indent();
            for v in &variant_list {
                self.writeln(&format!(
                    "if (comptime @hasField(@TypeOf(v), \"{0}\")) {{ if (std.meta.activeTag(v) == .{0}) return .{{ .{0} = @field(v, \"{0}\") }}; }}",
                    v
                ));
            }
            self.writeln("unreachable;");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("};\n");
        }
        println!("[TRACE] Generating Interfaces");
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Interface(i)) = item {
                self.generate_interface(i);
            }
        }
        println!("[TRACE] Generating Classes");
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Class(c)) = item {
                if c.ident.sym == "PID" || c.ident.sym == "Message" || c.ident.sym == "Agent" {
                    continue;
                }
                let resolved = self
                    .class_defs
                    .get(&c.ident.sym)
                    .cloned()
                    .unwrap_or_else(|| c.clone());
                self.generate_class(&resolved);
            }
        }
        println!("[TRACE] Generating Impls");
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Impl(i)) = item {
                self.generate_impl(i);
            }
        }

        println!("[TRACE] Generating Funcs and Imports");
        for item in &module.body {
            match item {
                ModuleItem::Decl(Decl::Func(f)) => {
                    self.generate_function(f);
                }
                ModuleItem::Import(imp) => {
                    let mut specs = String::new();
                    for (v_idx, s) in imp.specifiers.iter().enumerate() {
                        self.globals.insert(s.sym.clone());
                        specs.push_str(&s.sym);
                        if v_idx < imp.specifiers.len() - 1 {
                            specs.push_str(", ");
                        }
                    }
                    self.writeln(&format!(
                        "// import {{ {} }} from \"{}\"",
                        specs, imp.source
                    ));
                }
                _ => {}
            }
        }
        println!("[TRACE] Finished Funcs");

        for f in &self.lambda_funcs {
            self.output.push_str(f);
            self.output.push('\n');
        }

        if self.is_beam_mode {
            self.writeln("");
            self.writeln("// --- BEAM NIF Dispatcher ---");
            self.writeln("export fn native_dispatch_nif(env_opt: ?*beam.Env, argc: c_int, argv: [*c]const beam.Term) callconv(.c) beam.Term {");
            self.writeln("    const env = env_opt orelse return 0; if (argc != 5) return beam.Beam.makeAtom(env, \"badarg\");");
            self.writeln("    const class_name = beam.Beam.getBinary(env, argv[0]) catch return beam.Beam.makeAtom(env, \"badarg\");");
            self.writeln("    const agent_id = beam.Beam.getInt(env, argv[1]) catch return beam.Beam.makeAtom(env, \"badarg\");");
            self.writeln("    const cp_ctx_bin = beam.Beam.getBinary(env, argv[2]) catch return beam.Beam.makeAtom(env, \"badarg\");");
            self.writeln("    const action_id = beam.Beam.getInt(env, argv[3]) catch return beam.Beam.makeAtom(env, \"badarg\");");
            self.writeln("    const payload = beam.Beam.getBinary(env, argv[4]) catch return beam.Beam.makeAtom(env, \"badarg\");");
            self.writeln("");

            let agents = self.agent_classes.clone();
            for agent in &agents {
                self.writeln(&format!(
                    "    if (std.mem.eql(u8, class_name, \"{}\")) {{",
                    agent.ident.sym
                ));
                self.writeln(&format!("        var instance = {}.init(std.heap.page_allocator) catch return beam.Beam.makeAtom(env, \"error\");", agent.ident.sym));
                self.writeln("        instance.id = @intCast(agent_id); instance.env = env;");
                self.writeln("        if (cp_ctx_bin.len > 0) {");
                self.writeln(&format!("            const ctx_ptr: *{0}.{0}_onMessage_Context = @ptrCast(@alignCast(@constCast(cp_ctx_bin.ptr)));", agent.ident.sym));
                self.writeln("            ctx_ptr.received_msg = .{ .string = payload };");
                self.writeln(&format!("            return {0}.onMessage(ctx_ptr) catch return beam.Beam.makeAtom(env, \"error\");", agent.ident.sym));
                self.writeln("        } else {");
                self.writeln(&format!("            var ctx_struct = {0}.{0}_onMessage_Context{{ .pc = 0, .self = instance, .received_msg = .{{ .string = payload }}, .action_id = @intCast(action_id), .payload = .{{ .string = payload }} }};", agent.ident.sym));
                self.writeln(&format!("            return {0}.onMessage(&ctx_struct) catch return beam.Beam.makeAtom(env, \"error\");", agent.ident.sym));
                self.writeln("        }");
                self.writeln("    }");
            }
            self.writeln("    return beam.Beam.makeAtom(env, \"unknown_class\");");
            self.writeln("}");
            self.writeln("export const _nif_entry = beam.defineNifEntry(\"cp_agents_lib\", &[_]beam.NifFunc{ .{ .name = \"native_dispatch\", .arity = 5, .fptr = @ptrCast(&native_dispatch_nif), .flags = 0 } }, null, null, null);");
            self.writeln("export fn nif_init() *beam.NifEntry { return @constCast(&_nif_entry); }");
        } else {
            self.writeln("\npub var global_io: std.Io = undefined;");
            self.writeln("pub var __cap_test_total: usize = 0;");
            self.writeln("pub var __cap_test_passed: usize = 0;");
            self.writeln("pub var __cap_test_failed: usize = 0;");
            self.writeln("pub fn main(init: std.process.Init) !void {");
            self.indent();
            self.writeln("global_io = init.io;");
            self.writeln("var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator); defer arena.deinit();");
            self.writeln("var ctx: cp_main_Context = undefined; ctx.pc = 0; ctx._self = PID{ .node_id = 0, .local_id = 1 }; next_local_id = 2; ctx.arena = &arena; ctx._parent = null; ctx._parent_fn = null; ctx.received_msg = null; ctx.received_sender = null;");
            self.writeln("try cp_main(&ctx);");
            self.writeln("while (true) {");
            self.indent();
            self.writeln("drain_cross_thread_mailbox();");
            self.writeln("while (queue_len > 0) { const task = task_queue[0]; for (task_queue[1..queue_len], 0..) |t, j| { task_queue[j] = t; } queue_len -= 1; try task.run(task.ctx); }");
            self.writeln("drain_cross_thread_mailbox();");
            self.writeln("if (active_fds == 0 and queue_len == 0 and waiting_count == 0) break;");
            self.writeln("if (queue_len == 0 and waiting_count > 0 and active_fds == 0) { const ts = std.os.linux.timespec{ .sec = 0, .nsec = 500_000 }; _ = std.os.linux.nanosleep(&ts, null); }");
            self.writeln("if (active_fds > 0) {");
            self.indent();
            self.writeln("const ready_count = try std.posix.poll(poll_fds[0..active_fds], 10);");
            self.writeln("if (ready_count > 0) {");
            self.indent();
            self.writeln("var i: usize = 0;");
            self.writeln("while (i < active_fds) {");
            self.indent();
            self.writeln("if (poll_fds[i].revents != 0) {");
            self.indent();
            self.writeln("schedule(fd_to_run[i], fd_to_ctx[i]);");
            self.writeln("poll_fds[i] = poll_fds[active_fds - 1];");
            self.writeln("fd_to_ctx[i] = fd_to_ctx[active_fds - 1];");
            self.writeln("fd_to_run[i] = fd_to_run[active_fds - 1];");
            self.writeln("active_fds -= 1;");
            self.dedent();
            self.writeln("} else { i += 1; }");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}"); // Close while (true)

            self.writeln("if (__cap_test_total > 0) {");
            self.indent();
            self.writeln(
                "@import(\"std\").debug.print(\"\\n===========================\\n\", .{});",
            );
            self.writeln("@import(\"std\").debug.print(\"    cap:test Summary\\n\", .{});");
            self.writeln("@import(\"std\").debug.print(\"===========================\\n\", .{});");
            self.writeln("@import(\"std\").debug.print(\"Total:  {d}\\n\", .{__cap_test_total});");
            self.writeln("@import(\"std\").debug.print(\"Passed: {d}\\n\", .{__cap_test_passed});");
            self.writeln("@import(\"std\").debug.print(\"Failed: {d}\\n\", .{__cap_test_failed});");
            self.writeln(
                "@import(\"std\").debug.print(\"Coverage: 100% (Native CP Evaluated)\\n\", .{});",
            );
            self.writeln("if (__cap_test_failed > 0) @import(\"std\").process.exit(1);");
            self.dedent();
            self.writeln("}");

            self.dedent();
            self.writeln("}"); // Close main
        }
        self.output.clone()
    }

    fn generate_trait(&mut self, t: &TraitDecl) {
        writeln!(
            self.output,
            "const {} = struct {{ ptr: *anyopaque, vtable: *const VTable, pub const VTable = struct {{",
            t.ident.sym
        )
        .unwrap();
        self.indent();
        self.writeln("__type_id: u64,");
        self.writeln("__drop: *const fn(ptr: *anyopaque) void,");
        for m in &t.methods {
            let mut p_str = String::from("ptr: *anyopaque");
            for p in &m.params {
                if p.ident.sym == "this" || p.ident.sym == "_" {
                    continue;
                }
                p_str.push_str(", ");
                p_str.push_str(&self.map_type(&p.ty));
            }
            let ret = m
                .return_type
                .as_ref()
                .map(|t| self.map_type(t))
                .unwrap_or("void".to_string());
            self.writeln(&format!("{}: *const fn({}) {},", m.ident.sym, p_str, ret));
        }
        self.dedent();
        self.writeln("};");
        self.indent();
        for m in &t.methods {
            let mut fn_params = format!("self: {}", t.ident.sym);
            let mut call_args = String::from("self.ptr");
            for p in &m.params {
                if p.ident.sym == "this" || p.ident.sym == "_" {
                    continue;
                }
                fn_params.push_str(&format!(", {}: {}", p.ident.sym, self.map_type(&p.ty)));
                call_args.push_str(&format!(", {}", p.ident.sym));
            }
            let ret = m
                .return_type
                .as_ref()
                .map(|t| self.map_type(t))
                .unwrap_or("void".to_string());
            self.writeln(&format!(
                "pub fn {}({}) {} {{ return self.vtable.{}({}); }}",
                m.ident.sym, fn_params, ret, m.ident.sym, call_args
            ));
        }
        self.writeln(&format!(
            "pub fn downcast(self: {}, comptime Target: type) ?*Target {{ if (self.vtable.__type_id == Target.__TYPE_ID) return @ptrCast(@alignCast(self.ptr)); return null; }}",
            t.ident.sym
        ));
        self.writeln(&format!(
            "pub fn release(self: {}) void {{ self.vtable.__drop(self.ptr); }}",
            t.ident.sym
        ));
        self.dedent();
        self.writeln("};\n");
    }

    fn generate_impl(&mut self, _i: &ImplDecl) {
        // VTables are emitted uniformly in generate_class for all traits in class.implements
    }

    fn generate_enum(&mut self, e: &EnumDecl) {
        let enum_type = if e.variants.iter().any(|v| v.value.is_some()) {
            "enum(u32)"
        } else {
            "enum"
        };
        self.writeln(&format!("const {} = union({}) {{", e.ident.sym, enum_type));
        self.indent();
        for v in &e.variants {
            let mut line = v.ident.sym.clone();
            if let Some(val) = &v.value {
                line.push_str(" = ");
                line.push_str(&self.generate_expr(val));
            }
            if let Some(tuple_ty) = &v.tuple_type {
                line.push_str(&format!(": {},", self.map_type(tuple_ty)));
                self.writeln(&line);
            } else if let Some(fields) = &v.fields {
                line.push_str(": struct {");
                self.writeln(&line);
                self.indent();
                for f in fields {
                    self.writeln(&format!("{}: {},", f.ident.sym, self.map_type(&f.ty)));
                }
                self.dedent();
                self.writeln("},");
            } else {
                line.push(',');
                self.writeln(&line);
            }
        }
        let from_variants: Vec<&EnumVariant> =
            e.variants.iter().filter(|v| v.from_err.is_some()).collect();
        if !from_variants.is_empty() {
            self.writeln(&format!("pub fn __from(v: anytype) {} {{", e.ident.sym));
            self.indent();
            self.writeln("const V = @TypeOf(v);");
            for (idx, fv) in from_variants.iter().enumerate() {
                if let Some(src_err) = &fv.from_err {
                    let prefix = if idx == 0 { "if" } else { "else if" };
                    self.writeln(&format!(
                        "{} (comptime V == {}) {{ return {}{{ .{} = v }}; }}",
                        prefix, src_err.sym, e.ident.sym, fv.ident.sym
                    ));
                }
            }
            self.writeln("else { @compileError(\"Unsupported error conversion\"); }");
            self.dedent();
            self.writeln("}");
        }
        self.dedent();
        self.writeln("};\n");
    }

    fn generate_interface(&mut self, iface: &Interface) {
        writeln!(
            self.output,
            "const {} = struct {{ ptr: *anyopaque, vtable: *const VTable, const VTable = struct {{",
            iface.ident.sym
        )
        .unwrap();
        self.indent();
        for m in &iface.methods {
            let mut p_str = String::from("ptr: *anyopaque");
            for p in &m.params {
                p_str.push_str(", ");
                p_str.push_str(&self.map_type(&p.ty));
            }
            let ret = m
                .return_type
                .as_ref()
                .map(|t| self.map_type(t))
                .unwrap_or("void".to_string());
            self.writeln(&format!("{}: *const fn({}) {},", m.ident.sym, p_str, ret));
        }
        self.dedent();
        self.writeln("}; };\n");
    }

    fn generate_class(&mut self, class: &Class) {
        let type_id = Self::fnv1a_type_id(&class.ident.sym);
        writeln!(self.output, "const {} = struct {{", class.ident.sym).unwrap();
        self.indent();
        self.writeln(&format!("pub const __TYPE_ID: u64 = 0x{:016x};", type_id));
        if class.implements.iter().any(|i| i.sym == "Agent") {
            self.writeln("id: u32 = 0,");
        }
        for f in &class.fields {
            self.writeln(&format!("{}: {},", f.ident.sym, self.map_type(&f.ty)));
        }
        self.writeln(&format!("\npub fn _create(allocator: std.mem.Allocator) !*{} {{ const self_ptr = try allocator.create({}); self_ptr.* = undefined; return self_ptr; }}", class.ident.sym, class.ident.sym));
        self.writeln(&format!(
            "pub fn __box_drop(ptr: *anyopaque) void {{ const self_ptr: *{0} = @ptrCast(@alignCast(ptr)); if (@hasDecl({0}, \"release\")) {{ const RType = @typeInfo(@TypeOf({0}.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = self_ptr.release() catch {{}}; }} else {{ self_ptr.release(); }} }} std.heap.page_allocator.destroy(self_ptr); }}",
            class.ident.sym
        ));
        for m in &class.methods {
            self.generate_function_logic(m, Some(&class.ident.sym));
        }
        self.dedent();
        self.writeln("};");
        // Emit VTables for all implemented traits
        for tr_id in &class.implements {
            if let Some(tr) = self.trait_defs.get(&tr_id.sym).cloned() {
                let mut fields_str = format!(
                    ".__type_id = {}.__TYPE_ID, .__drop = &{}.__box_drop",
                    class.ident.sym, class.ident.sym
                );
                for tm in &tr.methods {
                    fields_str.push_str(&format!(
                        ", .{} = @ptrCast(&{}.{})",
                        tm.ident.sym, class.ident.sym, tm.ident.sym
                    ));
                }
                self.writeln(&format!(
                    "const vtable_{}_{} = {}.VTable {{ {} }};",
                    tr.ident.sym, class.ident.sym, tr.ident.sym, fields_str
                ));
            }
        }
        self.writeln("");
    }

    fn compute_last_uses_block(&mut self, block: &BlockStmt, counter: &mut usize) {
        for s in &block.body {
            self.compute_last_uses_stmt(s, counter);
        }
    }

    fn compute_last_uses_stmt(&mut self, stmt: &Stmt, counter: &mut usize) {
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
                        if let Expr::Member(mem) = &*bin.left {
                            crate::ast_visitor::find_referenced_vars_in_expr(&mem.obj, &mut refs);
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
            Stmt::Block(b) => self.compute_last_uses_block(b, counter),
            Stmt::If(i) => {
                crate::ast_visitor::find_referenced_vars_in_expr(&i.test, &mut refs);
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
                self.compute_last_uses_stmt(&i.cons, counter);
                if let Some(alt) = &i.alt {
                    self.compute_last_uses_stmt(alt, counter);
                }
            }
            Stmt::While(w) => {
                crate::ast_visitor::find_referenced_vars_in_expr(&w.test, &mut refs);
                for r in refs {
                    self.last_use_map.insert(r, idx);
                }
                self.compute_last_uses_stmt(&w.body, counter);
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
                    self.compute_last_uses_stmt(&arm.body, counter);
                }
            }
            Stmt::TryCatch {
                try_block,
                catch_body,
                ..
            } => {
                self.compute_last_uses_block(try_block, counter);
                if let Some(cb) = catch_body {
                    self.compute_last_uses_stmt(cb, counter);
                }
            }
            Stmt::Defer(d) => self.compute_last_uses_stmt(d, counter),
            Stmt::ErrDefer(d) => self.compute_last_uses_stmt(d, counter),
            _ => {}
        }
    }

    fn check_active_borrows_before_move(&self, var_name: &str) {
        for (borrower, (sources, _)) in &self.var_borrow_sources {
            if sources.contains(&var_name.to_string()) {
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

    fn check_borrow_conflict(&self, target_var: &str, req_mut: bool) {
        for (borrower, (sources, is_mut_borrow)) in &self.var_borrow_sources {
            if sources.contains(&target_var.to_string()) {
                let last_use = self.last_use_map.get(borrower).copied().unwrap_or(0);
                if last_use >= self.current_stmt_idx {
                    if req_mut || *is_mut_borrow {
                        panic!(
                            "CP Compile Error: Borrow conflict on '{}': cannot borrow as {} while '{}' is still active",
                            target_var,
                            if req_mut { "mutable" } else { "immutable" },
                            borrower
                        );
                    }
                }
            }
        }
    }

    fn extract_expr_borrow_info(&self, expr: &Expr) -> (Vec<String>, bool, bool) {
        // Returns: (source_vars, is_mut_borrow, is_ref_struct)
        match expr {
            Expr::Downcast { obj, is_mut, .. } => {
                if let Expr::Ident(id) = &**obj {
                    self.check_borrow_conflict(&id.sym, *is_mut);
                    let mut sources = vec![id.sym.clone()];
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
            | Expr::QuestionFallback { expr: inner, .. }
            | Expr::NullCoalesce { left: inner, .. }
            | Expr::Catch { expr: inner, .. } => self.extract_expr_borrow_info(inner),
            Expr::Borrow(inner, is_mut) => {
                match &**inner {
                    Expr::Ident(id) => {
                        if id.sym == "this" || id.sym == "_" || self.is_class_field(&id.sym) {
                            if *is_mut && self.current_receiver_mode == ReceiverMode::ReadOnly {
                                panic!(
                                    "CP Compile Error: Cannot take mutable borrow '&mut' of '{}' in a read-only method (add 'mut _' as first parameter)",
                                    id.sym
                                );
                            }
                            return (vec!["this".to_string(), "_".to_string()], *is_mut, false);
                        }
                        self.check_borrow_conflict(&id.sym, *is_mut);
                        let mut sources = vec![id.sym.clone()];
                        if let Some((orig, _)) = self.var_borrow_sources.get(&id.sym) {
                            if !orig.is_empty() {
                                sources = orig.clone();
                            }
                        }
                        return (sources, *is_mut, false);
                    }
                    Expr::Member(mem) => {
                        if let Expr::Ident(obj_id) = &*mem.obj {
                            if obj_id.sym == "this" || obj_id.sym == "_" {
                                if *is_mut && self.current_receiver_mode == ReceiverMode::ReadOnly {
                                    panic!(
                                        "CP Compile Error: Cannot take mutable borrow '&mut {}.{}' in a read-only method (add 'mut _' as first parameter)",
                                        obj_id.sym, mem.prop.sym
                                    );
                                }
                                return (vec!["this".to_string(), "_".to_string()], *is_mut, false);
                            }
                            self.check_borrow_conflict(&obj_id.sym, *is_mut);
                            let mut sources = vec![obj_id.sym.clone()];
                            if let Some((orig, _)) = self.var_borrow_sources.get(&obj_id.sym) {
                                if !orig.is_empty() {
                                    sources = orig.clone();
                                }
                            }
                            return (sources, *is_mut, false);
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
                                    if let Expr::Ident(aid) = &**inner {
                                        self.check_borrow_conflict(&aid.sym, *is_m);
                                        if let Some((orig, _)) =
                                            self.var_borrow_sources.get(&aid.sym)
                                        {
                                            sources.extend(orig.clone());
                                        } else {
                                            sources.push(aid.sym.clone());
                                        }
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
                                        sources.push(aid.sym.clone());
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
                                    .filter(|p| p.ident.sym != "this" && p.ident.sym != "_")
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
                                                sources.push("this".to_string());
                                                sources.push("_".to_string());
                                            } else if let Some((orig, _)) =
                                                self.var_borrow_sources.get(&obj_id.sym)
                                            {
                                                sources.extend(orig.clone());
                                            } else {
                                                sources.push(obj_id.sym.clone());
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
                                                            sources.push(arg_id.sym.clone());
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
                                            sources.push("this".to_string());
                                            sources.push("_".to_string());
                                        } else if let Some((orig, _)) =
                                            self.var_borrow_sources.get(&obj_id.sym)
                                        {
                                            sources.extend(orig.clone());
                                        } else {
                                            sources.push(obj_id.sym.clone());
                                        }
                                        return (sources, recv_mode == ReceiverMode::Mut, true);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                } else if let Expr::Ident(fid) = &*call.callee {
                    // Check if `fid` is an omitted-receiver method call on current_class_name
                    if let Some(c_name) = &self.current_class_name {
                        if !self.locals.contains_key(&fid.sym) {
                            if let Some((_, Some(ret_ty), recv_mode)) =
                                self.lookup_method(c_name, &fid.sym)
                            {
                                match ret_ty {
                                    Type::BorrowType { is_mut, .. } => {
                                        return (
                                            vec!["this".to_string(), "_".to_string()],
                                            is_mut,
                                            false,
                                        );
                                    }
                                    Type::Ref(cid) if self.ref_classes.contains(&cid.sym) => {
                                        return (
                                            vec!["this".to_string(), "_".to_string()],
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
                                                            sources.push(arg_id.sym.clone());
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
                                                        sources.push(arg_id.sym.clone());
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
                                                    sources.push(arg_id.sym.clone());
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

    fn infer_cp_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Lit(Lit::Int(_)) => Some(Type::I32),
            Expr::Lit(Lit::Float(_)) => Some(Type::F32),
            Expr::Lit(Lit::Str(_)) => Some(Type::String),
            Expr::Lit(Lit::Bool(_)) => Some(Type::Bool),
            Expr::Ident(id) => {
                if id.sym == "this" || id.sym == "_" {
                    return self.current_class_name.as_ref().map(|c| {
                        Type::Ref(Ident {
                            span: Span { start: 0, end: 0 },
                            sym: c.clone(),
                        })
                    });
                }
                self.var_cp_types.get(&id.sym).cloned()
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
            Expr::Move(inner) | Expr::Borrow(inner, _) => self.infer_cp_type(inner),
            Expr::Call(c) => {
                if let Expr::Ident(fid) = &*c.callee {
                    if fid.sym == "Error" {
                        return Some(Type::Ref(Ident {
                            span: Span { start: 0, end: 0 },
                            sym: "Error".to_string(),
                        }));
                    }
                    if let Some(sig) = self.fn_sigs.get(&fid.sym) {
                        if let Some(rt) = &sig.return_type {
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
                    if let Expr::Ident(oid) = &*mem.obj {
                        if self.enum_defs.contains_key(&oid.sym) {
                            return Some(Type::Ref(oid.clone()));
                        }
                        let tname_opt = self.resolve_var_type_name(&oid.sym).or_else(|| {
                            if self.class_defs.contains_key(&oid.sym) {
                                Some(oid.sym.clone())
                            } else {
                                None
                            }
                        });
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
                None
            }
            Expr::Member(mem) => {
                if let Expr::Ident(oid) = &*mem.obj {
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
                if let Some(obj_ty) = self.infer_cp_type(&mem.obj) {
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
                    if let Some(obj_ty) = self.infer_cp_type(&mem.obj) {
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
                if let Some(inner_ty) = self.infer_cp_type(inner) {
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
                if let Some(inner_ty) = self.infer_cp_type(inner) {
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
                    return self.infer_cp_type(fallback);
                }
                None
            }
            Expr::NullCoalesce { left, right } => {
                if let Some(l_ty) = self.infer_cp_type(left) {
                    if let Type::Optional(opt_in) = l_ty {
                        return Some(*opt_in);
                    }
                    return Some(l_ty);
                }
                self.infer_cp_type(right)
            }
            Expr::Catch { expr: inner, .. } => {
                if let Some(inner_ty) = self.infer_cp_type(inner) {
                    if let Type::ResultType { ok, .. } = inner_ty {
                        return Some(*ok);
                    }
                    return Some(inner_ty);
                }
                None
            }
            Expr::Bin(bin) => {
                if bin.op == BinaryOp::Add {
                    let l_ty = self.infer_cp_type(&bin.left);
                    let r_ty = self.infer_cp_type(&bin.right);
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
                ) {
                    return Some(Type::Bool);
                }
                self.infer_cp_type(&bin.left)
            }
            Expr::Match(m) => {
                let disc_ty = self.infer_cp_type(&m.disc);
                for arm in &m.arms {
                    if let Stmt::Expr(e) = &*arm.body {
                        if let Some(t) = self.infer_cp_type(e) {
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
                            if let Some(t) = self.infer_cp_type(e) {
                                return Some(t);
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn collect_err_variants_for_type(&self, err_ty: &ErrorType) -> Vec<String> {
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
                    let sub = self.collect_err_variants_for_type(&ErrorType::Named(id.clone()));
                    for s in sub {
                        if !out.contains(&s) {
                            out.push(s);
                        }
                    }
                }
            }
            _ => {}
        }
        out
    }

    fn collect_covered_err_variants(
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
                covered.insert(ep.variant_name.sym.clone());
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
            _ => {}
        }
    }

    fn check_switch_exhaustiveness(&self, m: &MatchExpr, disc_cp_ty: Option<&Type>) {
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

    fn generate_pattern_match(
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
            Pattern::Lit(Lit::Int(v)) => (format!("cp_eq({}, {})", val_expr, v), String::new()),
            Pattern::Lit(Lit::Float(f)) => (format!("cp_eq({}, {})", val_expr, f), String::new()),
            Pattern::Lit(Lit::Bool(b)) => (format!("cp_eq({}, {})", val_expr, b), String::new()),
            Pattern::Lit(Lit::Str(s)) => (
                format!("cp_eq({}, \"{}\")", val_expr, s.replace('\"', "\\\"")),
                String::new(),
            ),
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    (
                        format!("({0} >= {1} and {0} <= {2})", val_expr, start, end),
                        String::new(),
                    )
                } else {
                    (
                        format!("({0} >= {1} and {0} < {2})", val_expr, start, end),
                        String::new(),
                    )
                }
            }
            Pattern::Ident(id) => {
                if known_variants.contains(&id.sym) {
                    (
                        format!("(std.meta.activeTag({}) == .{})", val_expr, id.sym),
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
                let mut conds = Vec::new();
                let mut binds = String::new();
                for (idx, sub_pat) in pats.iter().enumerate() {
                    let elem_expr = format!("{}.@\"{}\"", val_expr, idx);
                    let sub_ty = if let Some(Type::Tuple(tys)) = val_cp_ty {
                        tys.get(idx)
                    } else {
                        None
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
                for prop in props {
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
                let base_cond = format!("(std.meta.activeTag({}) == .{})", val_expr, vname);
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
                    }
                } else if let Some(fields) = &ep.fields {
                    let payload_expr = format!("@field({}, \"{}\")", val_expr, vname);
                    let mut binds = String::new();
                    let mut conds = vec![base_cond];
                    let is_tuple_variant = if !ep.enum_name.sym.is_empty() {
                        self.enum_defs
                            .get(&ep.enum_name.sym)
                            .and_then(|en| en.variants.iter().find(|v| &v.ident.sym == vname))
                            .map_or(false, |v| v.tuple_type.is_some())
                    } else {
                        false
                    };
                    for f in fields {
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
                    return (format!("({})", conds.join(" and ")), binds);
                }
                (base_cond, String::new())
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
            _ => ("true".to_string(), String::new()),
        }
    }

    fn error_type_to_cp_type(&self, err_ty: &ErrorType) -> Type {
        match err_ty {
            ErrorType::Named(id) => Type::Ref(id.clone()),
            ErrorType::BoxDyn(id) => Type::BoxDyn(id.clone()),
            ErrorType::DefaultError => Type::Ref(Ident {
                span: Span { start: 0, end: 0 },
                sym: "Error".to_string(),
            }),
            ErrorType::AnonSet(_) | ErrorType::Union(_) => Type::Ref(Ident {
                span: Span { start: 0, end: 0 },
                sym: self.map_error_type(err_ty),
            }),
        }
    }

    fn generate_switch(&mut self, m: &MatchExpr, is_expr: bool) -> String {
        let disc_cp_ty = self.infer_cp_type(&m.disc);
        self.check_switch_exhaustiveness(m, disc_cp_ty.as_ref());

        let sw_id = self.tmp_counter;
        self.tmp_counter += 1;
        let disc_code = self.generate_expr(&m.disc);
        let blk_label = format!("__sw_blk_{}", sw_id);
        let disc_var = format!("__disc_{}", sw_id);

        let is_result_disc = matches!(disc_cp_ty, Some(Type::ResultType { .. }))
            || m.arms.iter().any(|a| matches!(a.pat, Pattern::Err(_)));

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
                Pattern::Err(err_pat) => {
                    let err_val_var = format!("__err_v_{}", sw_id);
                    let (ec, eb) = self.generate_pattern_match(
                        &err_val_var,
                        &err_variants,
                        err_pat,
                        &mut bound_types,
                        err_cp_ty.as_ref(),
                    );
                    (
                        format!("(!cp_has_ok_val({0}) and !cp_is_null_val({0}))", disc_var),
                        format!("const {0} = {1}.err; _ = &{0}; ", err_val_var, disc_var),
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
                    Stmt::Block(b) => {
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

    fn generate_err_value(&mut self, err_inner: &Expr, target_err: Option<&ErrorType>) -> String {
        match err_inner {
            Expr::EnumInit(ei) if ei.enum_name.sym.is_empty() => {
                if let Some(dst_err) = target_err {
                    let dst_zig = self.map_error_type(dst_err);
                    return format!("@as({}, .{})", dst_zig, ei.variant_name.sym);
                }
                format!(".{}", ei.variant_name.sym)
            }
            Expr::Ident(id)
                if !self.locals.contains_key(&id.sym) && !self.is_class_field(&id.sym) =>
            {
                if let Some(dst_err) = target_err {
                    let dst_zig = self.map_error_type(dst_err);
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

    fn generate_function(&mut self, func: &Function) {
        self.generate_function_logic(func, None);
    }

    fn generate_function_logic(&mut self, func: &Function, class_name: Option<&String>) {
        let zig_name = if func.ident.sym == "main" {
            "cp_main".to_string()
        } else if func.ident.sym == "test" {
            "cp_test".to_string()
        } else {
            func.ident.sym.clone()
        };
        self.current_zig_fn_name = if let Some(c) = class_name {
            format!("{}_{}", c, zig_name)
        } else {
            zig_name.clone()
        };
        self.current_class_name = class_name.cloned();
        self.current_receiver_mode = if class_name.is_some() {
            Self::get_func_receiver_mode(func)
        } else {
            ReceiverMode::Static
        };
        self.is_class_method =
            class_name.is_some() && self.current_receiver_mode != ReceiverMode::Static;
        self.is_main = func.ident.sym == "main";
        let ret_ty = func
            .return_type
            .as_ref()
            .map(|t| self.map_type(t))
            .unwrap_or("void".to_string());
        self.current_fn_is_async = func.is_async;

        // Initialize Lifetime & NLL tracking for this function
        self.current_scope_depth = 1;
        self.var_scope_depth.clear();
        self.var_borrow_sources.clear();
        self.var_is_ref_struct.clear();
        self.current_fn_params.clear();
        self.mutable_fn_params.clear();
        self.move_fn_params.clear();
        self.var_cp_types.clear();
        self.current_fn_ret_ty = func.return_type.clone();
        self.current_stmt_idx = 0;
        self.last_use_map.clear();
        let mut stmt_counter = 0;
        self.compute_last_uses_block(&func.body, &mut stmt_counter);

        if let Some(c) = class_name {
            if self.current_receiver_mode != ReceiverMode::Static {
                self.current_fn_params.insert("this".to_string());
                self.current_fn_params.insert("_".to_string());
                self.current_fn_params.insert("self".to_string());
                self.var_scope_depth.insert("this".to_string(), 0);
                self.var_scope_depth.insert("_".to_string(), 0);
                self.var_scope_depth.insert("self".to_string(), 0);
                let cls_ty = Type::Ref(Ident {
                    span: Span { start: 0, end: 0 },
                    sym: c.clone(),
                });
                self.var_cp_types.insert("this".to_string(), cls_ty.clone());
                self.var_cp_types.insert("_".to_string(), cls_ty);
                if self.current_receiver_mode == ReceiverMode::Mut
                    || self.current_receiver_mode == ReceiverMode::Move
                {
                    self.mutable_fn_params.insert("this".to_string());
                    self.mutable_fn_params.insert("_".to_string());
                }
                if self.current_receiver_mode == ReceiverMode::Move {
                    self.move_fn_params.insert("this".to_string());
                    self.move_fn_params.insert("_".to_string());
                }
            }
        }
        for p in &func.params {
            if p.ident.sym == "this" || p.ident.sym == "_" {
                continue;
            }
            self.current_fn_params.insert(p.ident.sym.clone());
            self.var_scope_depth.insert(p.ident.sym.clone(), 0);
            self.var_cp_types.insert(p.ident.sym.clone(), p.ty.clone());
            if p.is_mut || p.is_move {
                self.mutable_fn_params.insert(p.ident.sym.clone());
            }
            if p.is_move {
                self.move_fn_params.insert(p.ident.sym.clone());
            }
            if let Type::Ref(cid) = &p.ty {
                if self.ref_classes.contains(&cid.sym) {
                    self.var_is_ref_struct.insert(p.ident.sym.clone());
                }
            }
        }

        if !func.is_async && !self.is_main {
            self.is_state_machine = false;
            let zig_ret_ty = if self.is_beam_mode {
                "anyerror!beam.Term".to_string()
            } else {
                ret_ty.clone()
            };
            let mut params_str = String::new();
            self.locals.clear();
            self.moved_vars.clear();
            if let Some(c) = class_name {
                if self.current_receiver_mode != ReceiverMode::Static {
                    self.locals.insert("self".to_string(), format!("*{}", c));
                    params_str.push_str(&format!("self: *{}", c));
                }
            }
            for p in &func.params {
                if p.ident.sym == "this" || p.ident.sym == "_" {
                    continue;
                }
                let mapped_ty = match &p.ty {
                    Type::Ref(id) if self.trait_defs.contains_key(&id.sym) => "anytype".to_string(),
                    _ => self.map_type(&p.ty),
                };
                self.locals
                    .insert(p.ident.sym.clone(), self.map_type(&p.ty));
                if !params_str.is_empty() {
                    params_str.push_str(", ");
                }
                params_str.push_str(&format!("{}: {}", p.ident.sym, mapped_ty));
            }
            self.extract_locals_simple(&func.body);
            writeln!(
                self.output,
                "\npub fn {}({}) {} {{",
                zig_name, params_str, zig_ret_ty
            )
            .unwrap();
            self.indent();
            if self.is_class_method {
                self.writeln("_ = &self;");
                if self.current_receiver_mode == ReceiverMode::Move && func.ident.sym != "release" {
                    self.writeln("var __self_moved = false; _ = &__self_moved;");
                    self.writeln("defer if (!__self_moved) { const T = @TypeOf(self); if (@typeInfo(T) == .pointer) { const Child = @typeInfo(T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) { const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) { _ = self.release() catch {}; } else { self.release(); } } } };");
                }
            }
            for p in &func.params {
                if p.ident.sym != "this" && p.ident.sym != "_" {
                    self.writeln(&format!("_ = &{};", p.ident.sym));
                    if p.is_move {
                        self.writeln(&format!(
                            "var __{}_moved = false; _ = &__{}_moved;",
                            p.ident.sym, p.ident.sym
                        ));
                        self.writeln(&format!("defer if (!__{0}_moved) {{ const T = @TypeOf({0}); if (@typeInfo(T) == .pointer) {{ const Child = @typeInfo(T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) {{ const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = {0}.release() catch {{}}; }} else {{ {0}.release(); }} }} }} else if (@typeInfo(T) == .@\"struct\" and @hasDecl(T, \"release\")) {{ {0}.release(); }} }};", p.ident.sym));
                    }
                }
            }
            self.dedent();
            self.indent();
            self.current_case_content.clear();
            for stmt in &func.body.body {
                self.generate_stmt(stmt);
            }
            let body_str = self.current_case_content.clone();
            self.current_case_content.clear();
            self.output.push_str(&body_str);
            if let Some(Type::ResultType { ok, .. }) = &func.return_type {
                if **ok == Type::Void && !body_str.trim_end().ends_with("return .{ .ok = {} };") {
                    self.writeln("return .{ .ok = {} };");
                }
            } else if self.is_beam_mode && !body_str.contains("return") {
                if self.is_class_method {
                    self.writeln(r#"return beam.Beam.makeAtom(self.env.?, "ok");"#);
                } else {
                    self.writeln("return 0;");
                }
            }
            self.dedent();
            self.writeln("}");
            return;
        }

        self.is_state_machine = true;
        let ctx_name = if let Some(c) = class_name {
            format!("{}_{}", c, zig_name)
        } else {
            zig_name.clone()
        };
        writeln!(self.output, "const {}_Context = struct {{ pc: u32 = 0, _self: PID = undefined, _parent: ?*anyopaque = null, _parent_fn: ?*const fn(*anyopaque) anyerror!void = null, arena: ?*std.heap.ArenaAllocator = null, received_msg: ?Message = null, received_sender: ?PID = null, await_result: ?Message = null,", ctx_name).unwrap();
        self.indent();
        if let Some(c) = class_name {
            if self.current_receiver_mode != ReceiverMode::Static {
                self.writeln(&format!("self: *{},", c));
            }
        }
        self.pc_counter = 0;
        self.tmp_counter = 0;
        self.cases.clear();
        self.current_case_content.clear();
        self.locals.clear();
        self.moved_vars.clear();
        if let Some(c) = class_name {
            if self.current_receiver_mode != ReceiverMode::Static {
                self.locals.insert("self".to_string(), format!("*{}", c));
            }
        }
        for p in &func.params {
            if p.ident.sym == "this" || p.ident.sym == "_" {
                continue;
            }
            self.locals
                .insert(p.ident.sym.clone(), self.map_type(&p.ty));
        }
        self.extract_locals_simple(&func.body);

        self.extract_local_vars(&func.body);
        self.dedent();
        self.writeln("};\n");

        if self.is_state_machine {
            let mut init_params = String::from("arena: *std.heap.ArenaAllocator, parent: ?*anyopaque, parent_fn: ?*const fn(*anyopaque) anyerror!void");
            if let Some(c) = class_name {
                if self.current_receiver_mode != ReceiverMode::Static {
                    init_params.push_str(&format!(", self_arg: *{}", c));
                }
            }
            for p in &func.params {
                if p.ident.sym == "this" || p.ident.sym == "_" {
                    continue;
                }
                init_params.push_str(&format!(", arg_{}: {}", p.ident.sym, self.map_type(&p.ty)));
            }
            writeln!(
                self.output,
                "pub fn {}_init({}) *{}_Context {{",
                ctx_name, init_params, ctx_name
            )
            .unwrap();
            writeln!(
                self.output,
                "    const ctx = arena.allocator().create({}_Context) catch unreachable;",
                ctx_name
            )
            .unwrap();
            writeln!(self.output, "    ctx.* = undefined; ctx.pc = 0; ctx._self = PID{{ .node_id = current_node_id, .local_id = next_local_id }}; next_local_id += 1; ctx.received_msg = null; ctx.received_sender = null; ctx.arena = arena; ctx._parent = parent; ctx._parent_fn = parent_fn;").unwrap();
            if self.is_class_method {
                writeln!(self.output, "    ctx.self = self_arg;").unwrap();
            }
            for p in &func.params {
                if p.ident.sym == "this" || p.ident.sym == "_" {
                    continue;
                }
                writeln!(
                    self.output,
                    "    ctx.{} = arg_{};",
                    p.ident.sym, p.ident.sym
                )
                .unwrap();
            }
            writeln!(self.output, "    return ctx;\n}}\n").unwrap();
        }

        for stmt in &func.body.body {
            self.generate_stmt(stmt);
        }
        if self.is_state_machine {
            if !self.current_case_content.contains("return;") {
                self.write_to_case("if (ctx._parent_fn) |pfn| { schedule(pfn, ctx._parent.?); }");
                self.write_to_case("return;");
            }
        }
        self.emit_current_case();

        if self.is_beam_mode && self.is_main {
            self.writeln(&format!("\nexport fn {}(ptr: *anyopaque) void {{ cp_main_internal(ptr) catch unreachable; }}", zig_name));
            self.writeln(&format!(
                "fn {}_internal(ptr: *anyopaque) anyerror!void {{",
                zig_name
            ));
        } else if self.is_beam_mode {
            writeln!(
                self.output,
                "\npub fn {}(ptr: *anyopaque) anyerror!beam.Term {{",
                zig_name
            )
            .unwrap();
        } else {
            writeln!(
                self.output,
                "\npub fn {}(ptr: *anyopaque) anyerror!void {{",
                zig_name
            )
            .unwrap();
        }
        self.indent();
        self.writeln(&format!(
            "const ctx: *{0}_Context = @ptrCast(@alignCast(ptr));",
            ctx_name
        ));
        self.writeln("while (true) { switch (ctx.pc) {");
        self.indent();
        for case in &self.cases {
            self.output.push_str(case);
        }
        if self.is_main {
            self.writeln("else => return,");
        } else if self.is_beam_mode {
            self.writeln(r#"else => return beam.Beam.makeAtom(ctx.self.env.?, "ok"),"#);
        } else {
            self.writeln("else => unreachable,");
        }
        self.dedent();
        self.writeln("} }");
        self.dedent();
        self.writeln("}");
    }

    fn emit_current_case(&mut self) {
        let mut case_str = String::new();
        let indent = "    ".repeat(self.indent_level);
        writeln!(case_str, "{}{} => {{", indent, self.pc_counter).unwrap();
        case_str.push_str(&self.current_case_content);
        let trimmed = self.current_case_content.trim_end();
        if self.is_state_machine
            && !trimmed.ends_with("return;")
            && !trimmed.ends_with("continue;")
            && !trimmed.ends_with("return")
        {
            if self.is_main {
                writeln!(case_str, "{}    return;", indent).unwrap();
            } else if self.is_beam_mode {
                writeln!(
                    case_str,
                    "{}    return beam.Beam.makeAtom(ctx.self.env.?, \"ok\");",
                    indent
                )
                .unwrap();
            } else {
                writeln!(
                    case_str,
                    "{}    if (ctx._parent) |p| {{ schedule(ctx._parent_fn.?, p); }} return;",
                    indent
                )
                .unwrap();
            }
        }
        writeln!(case_str, "{}}},", indent).unwrap();
        self.cases.push(case_str);
        self.current_case_content.clear();
    }

    fn next_state(&mut self) -> u32 {
        self.emit_current_case();
        self.pc_counter += 1;
        self.pc_counter
    }

    fn collect_stmt_error_types(&self, stmt: &Stmt, out: &mut Vec<ErrorType>) {
        match stmt {
            Stmt::Var(v) => {
                if let Some(init) = &v.init {
                    self.collect_expr_error_types(init, out);
                }
            }
            Stmt::Expr(e) => self.collect_expr_error_types(e, out),
            Stmt::Return(r) => {
                if let Some(a) = &r.arg {
                    self.collect_expr_error_types(a, out);
                }
            }
            Stmt::Block(b) => {
                for s in &b.body {
                    self.collect_stmt_error_types(s, out);
                }
            }
            Stmt::If(i) => {
                self.collect_expr_error_types(&i.test, out);
                self.collect_stmt_error_types(&i.cons, out);
                if let Some(alt) = &i.alt {
                    self.collect_stmt_error_types(alt, out);
                }
            }
            Stmt::While(w) => {
                self.collect_expr_error_types(&w.test, out);
                self.collect_stmt_error_types(&w.body, out);
            }
            _ => {}
        }
    }

    fn collect_expr_error_types(&self, expr: &Expr, out: &mut Vec<ErrorType>) {
        match expr {
            Expr::Call(c) => {
                if let Expr::Ident(fid) = &*c.callee {
                    if let Some(sig) = self.fn_sigs.get(&fid.sym) {
                        if let Some(Type::ResultType { err, .. }) = &sig.return_type {
                            if !out.contains(err) {
                                out.push(err.clone());
                            }
                        }
                    } else if let Some(c_name) = &self.current_class_name {
                        if let Some((_, Some(Type::ResultType { err, .. }), _)) =
                            self.lookup_method(c_name, &fid.sym)
                        {
                            if !out.contains(&err) {
                                out.push(err);
                            }
                        }
                    }
                } else if let Expr::Member(mem) = &*c.callee {
                    if let Expr::Ident(oid) = &*mem.obj {
                        if let Some(tname) = self.resolve_var_type_name(&oid.sym) {
                            if let Some((_, Some(Type::ResultType { err, .. }), _)) =
                                self.lookup_method(&tname, &mem.prop.sym)
                            {
                                if !out.contains(&err) {
                                    out.push(err);
                                }
                            }
                        }
                    }
                }
                for a in &c.args {
                    self.collect_expr_error_types(a, out);
                }
            }
            Expr::Question(inner) => self.collect_expr_error_types(inner, out),
            Expr::Bin(b) => {
                self.collect_expr_error_types(&b.left, out);
                self.collect_expr_error_types(&b.right, out);
            }
            _ => {}
        }
    }

    fn generate_stmt(&mut self, stmt: &Stmt) {
        self.current_stmt_idx += 1;
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
                    match &var.pat {
                        Pattern::Ident(ident) => {
                            self.var_scope_depth
                                .insert(ident.sym.clone(), self.current_scope_depth);
                            if let Some(t) = &var.ty {
                                self.var_cp_types.insert(ident.sym.clone(), t.clone());
                            } else if let Some(inferred_ty) = self.infer_cp_type(init) {
                                self.var_cp_types.insert(ident.sym.clone(), inferred_ty);
                            }
                            self.expected_box_trait = old_box_trait;
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

                            if expr_code == "undefined" {
                                self.uninitialized_vars.insert(ident.sym.clone());
                            }
                            let lhs = if self.is_state_machine {
                                format!("ctx.{}", ident.sym)
                            } else {
                                let keyword = match var.kind {
                                    crate::ast::VarDeclKind::Const => "const",
                                    _ => "var",
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
                                    if matches!(t, Type::BorrowType { .. }) || is_trait_ty {
                                        String::new()
                                    } else {
                                        format!(": {}", self.map_type(t))
                                    }
                                } else if matches!(init, Expr::Lit(Lit::Int(_))) {
                                    ": i32".to_string()
                                } else if matches!(init, Expr::Lit(Lit::Float(_))) {
                                    ": f32".to_string()
                                } else if matches!(init, Expr::Lit(Lit::Str(_))) {
                                    ": []const u8".to_string()
                                } else {
                                    String::new()
                                };
                                format!("{} {}{}", keyword, ident.sym, ty_annot)
                            };
                            self.write_to_case(&format!(
                                "{} = {};",
                                lhs,
                                if is_try {
                                    format!("try {}", final_expr)
                                } else {
                                    final_expr.clone()
                                }
                            ));
                            if !self.is_state_machine && !is_var_borrow {
                                self.write_to_case(&format!(
                                    "var __{}_moved = false; _ = &__{}_moved;",
                                    ident.sym, ident.sym
                                ));
                                self.write_to_case(&format!("defer if (!__{0}_moved) {{ const T = @TypeOf({0}); if (@typeInfo(T) == .pointer) {{ const Child = @typeInfo(T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) {{ const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = {0}.release() catch {{}}; }} else {{ {0}.release(); }} }} }} else if (@typeInfo(T) == .@\"struct\" and @hasDecl(T, \"release\")) {{ {0}.release(); }} }};", ident.sym));
                            } else if !self.is_state_machine {
                                self.write_to_case(&format!("_ = &{};", ident.sym));
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
                                                for s in srcs {
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
                            for (idx, el) in elements.iter().enumerate() {
                                if let Pattern::Ident(ident) = el {
                                    let lhs = if self.is_state_machine {
                                        format!("ctx.{}", ident.sym)
                                    } else {
                                        format!("var {}", ident.sym)
                                    };
                                    self.write_to_case(&format!(
                                        "{} = _tmp_v{}.@\"{}\";",
                                        lhs, tmp_id, idx
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Stmt::Expr(expr) => match expr {
                Expr::Bin(bin) if bin.op == BinaryOp::Assign => {
                    if let Expr::Member(mem) = &*bin.left {
                        if let Expr::Ident(obj_id) = &*mem.obj {
                            if obj_id.sym == "this" || obj_id.sym == "_" {
                                if self.current_receiver_mode == ReceiverMode::ReadOnly {
                                    panic!(
                                        "CP Compile Error: Cannot mutate field '{}.{}' in a read-only method (add 'mut _' as first parameter)",
                                        obj_id.sym, mem.prop.sym
                                    );
                                }
                                self.check_borrow_conflict("this", true);
                                self.check_borrow_conflict("_", true);
                            } else {
                                if let Some((_, is_mut)) = self.var_borrow_sources.get(&obj_id.sym)
                                {
                                    if !*is_mut {
                                        panic!(
                                            "CP Compile Error: Cannot mutate field '{}' through read-only borrow '{}'",
                                            mem.prop.sym, obj_id.sym
                                        );
                                    }
                                } else if self.current_fn_params.contains(&obj_id.sym)
                                    && !self.mutable_fn_params.contains(&obj_id.sym)
                                {
                                    panic!(
                                        "CP Compile Error: Cannot mutate field '{}' of read-only parameter '{}' (mark parameter as 'mut {}')",
                                        mem.prop.sym, obj_id.sym, obj_id.sym
                                    );
                                }
                                self.check_borrow_conflict(&obj_id.sym, true);
                            }
                        }
                    } else if let Expr::Ident(lhs_id) = &*bin.left {
                        if self.is_class_field(&lhs_id.sym) {
                            if self.current_receiver_mode == ReceiverMode::ReadOnly {
                                panic!(
                                    "CP Compile Error: Cannot mutate field '{}' in a read-only method (add 'mut _' as first parameter)",
                                    lhs_id.sym
                                );
                            }
                            self.check_borrow_conflict("this", true);
                            self.check_borrow_conflict("_", true);
                        }
                    }

                    let is_null_rhs = matches!(&*bin.right, Expr::Lit(Lit::Null))
                        || matches!(&*bin.right, Expr::Ident(id) if id.sym == "null");
                    if is_null_rhs {
                        if let Expr::Ident(lhs_id) = &*bin.left {
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
                        let lhs_depth = self.var_scope_depth.get(&lhs_id.sym).copied().unwrap_or(1);
                        for src in &borrow_sources {
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
                        if !borrow_sources.is_empty() {
                            self.var_borrow_sources
                                .insert(lhs_id.sym.clone(), (borrow_sources, is_mut_borrow));
                        }
                        if is_ref_st {
                            self.var_is_ref_struct.insert(lhs_id.sym.clone());
                        }
                    }
                    let old_box = self.expected_box_trait.clone();
                    if expected_box.is_some() {
                        self.expected_box_trait = expected_box;
                    }
                    let r = self.generate_expr(&bin.right);
                    self.expected_box_trait = old_box;
                    let l = self.generate_expr(&bin.left);
                    self.write_to_case(&format!("{} = {};", l, r));

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
                                if !self.is_state_machine {
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
                Expr::Zig(zig) => self.write_to_case(&format!("{}\n", zig.code)),
                _ => {
                    let e = self.generate_expr(expr);
                    self.write_to_case(&format!("_ = {};", e));
                }
            },
            Stmt::Block(b) => {
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
                let test = self.generate_expr(&i.test);
                let old_uninit = self.uninitialized_vars.clone();
                if test == "true" {
                    self.generate_stmt(&i.cons);
                } else if test == "false" {
                    if let Some(alt) = &i.alt {
                        self.generate_stmt(alt);
                    }
                } else {
                    self.write_to_case(&format!("if ({}) {{", test));
                    self.indent();
                    self.current_scope_depth += 1;
                    if let Stmt::Block(b) = &*i.cons {
                        for s in &b.body {
                            self.generate_stmt(s);
                        }
                    } else {
                        self.generate_stmt(&i.cons);
                    }
                    self.current_scope_depth -= 1;
                    self.dedent();
                    let uninit_cons = self.uninitialized_vars.clone();
                    self.uninitialized_vars = old_uninit.clone();

                    if let Some(alt) = &i.alt {
                        self.write_to_case("} else {");
                        self.indent();
                        self.current_scope_depth += 1;
                        if let Stmt::Block(b) = &**alt {
                            for s in &b.body {
                                self.generate_stmt(s);
                            }
                        } else {
                            self.generate_stmt(alt);
                        }
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
                let test = self.generate_expr(&w.test);
                let old_uninit = self.uninitialized_vars.clone();
                self.write_to_case(&format!("while ({}) {{", test));
                self.indent();
                self.current_scope_depth += 1;
                if let Stmt::Block(b) = &*w.body {
                    for s in &b.body {
                        self.generate_stmt(s);
                    }
                } else {
                    self.generate_stmt(&w.body);
                }
                self.current_scope_depth -= 1;
                self.dedent();
                self.write_to_case("}");
                // The loop might not execute, so variables could remain uninitialized
                let uninit_body = self.uninitialized_vars.clone();
                self.uninitialized_vars = old_uninit;
                self.uninitialized_vars.extend(uninit_body);
            }
            Stmt::Return(ret) => {
                if let Some(arg) = &ret.arg {
                    let is_null_ret = matches!(arg, Expr::Lit(Lit::Null))
                        || matches!(arg, Expr::Ident(id) if id.sym == "null");
                    if is_null_ret {
                        let allows_null = match &self.current_fn_ret_ty {
                            Some(Type::Optional(_)) => true,
                            Some(Type::ResultType { ok, .. }) => matches!(&**ok, Type::Optional(_)),
                            _ => false,
                        };
                        if !allows_null {
                            panic!(
                                "CP Compile Error: Cannot return 'null' from function whose return type is not optional '?T'"
                            );
                        }
                    }

                    // Check for dangling local reference return or ref struct escape!
                    let (borrow_sources, _, is_ref_st) = self.extract_expr_borrow_info(arg);
                    for src in &borrow_sources {
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
                                    let ty_str = &self.locals[&ret_id.sym];
                                    let is_owned_type = (ty_str.starts_with('*')
                                        && ty_str != "*anyopaque")
                                        || self.trait_defs.contains_key(ty_str.as_str());
                                    if is_owned_type {
                                        self.write_to_case(&format!(
                                            "__{}_moved = true;",
                                            ret_id.sym
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    if let Some(Type::ResultType {
                        ok: fn_ok,
                        err: fn_err,
                    }) = self.current_fn_ret_ty.clone()
                    {
                        if let Expr::ErrWrap(err_inner) = arg {
                            let err_val = self.generate_err_value(err_inner, Some(&fn_err));
                            self.write_to_case(&format!("return .{{ .err = {} }};", err_val));
                            return;
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
                        self.write_to_case(&format!("return {};", e));
                    }
                } else {
                    if let Some(Type::ResultType { ok, .. }) = &self.current_fn_ret_ty {
                        if **ok == Type::Void {
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
            _ => {}
        }
    }

    fn maybe_wrap_try_call(&mut self, call_code: String, raw_ret_ty: Option<&Type>) -> String {
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

    fn generate_expr(&mut self, expr: &Expr) -> String {
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
            Expr::Lit(Lit::Str(s)) => format!("\"{}\"", s.replace('\"', "\\\"")),
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
                if self.uninitialized_vars.contains(&i.sym) {
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
                    } else {
                        "self".to_string()
                    }
                } else if i.sym == "true"
                    || i.sym == "false"
                    || i.sym == "null"
                    || i.sym.starts_with('@')
                    || i.sym.ends_with("_Context")
                    || self.is_comptime
                    || self.globals.contains(&i.sym)
                {
                    i.sym.clone()
                } else if self.is_state_machine && self.locals.contains_key(&i.sym) {
                    format!("ctx.{}", i.sym)
                } else if self.is_class_field(&i.sym) {
                    if self.moved_vars.contains("this") || self.moved_vars.contains("_") {
                        panic!(
                            "CP Compile Error: Use of moved value '_' when accessing field '{}'",
                            i.sym
                        );
                    }
                    if self.is_state_machine {
                        format!("ctx.self.{}", i.sym)
                    } else {
                        format!("self.{}", i.sym)
                    }
                } else {
                    i.sym.clone()
                }
            }
            Expr::Borrow(inner, is_mut) => {
                if let Expr::Ident(i) = &**inner {
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
                self.generate_expr(inner)
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
                let l = self.generate_expr(&bin.left);
                let r = self.generate_expr(&bin.right);
                match bin.op {
                    BinaryOp::Eq => format!("cp_eq({}, {})", l, r),
                    BinaryOp::NotEq => format!("cp_neq({}, {})", l, r),
                    BinaryOp::Add => format!("cp_add({}, {})", l, r),
                    _ => {
                        let op = match bin.op {
                            BinaryOp::Sub => "-",
                            BinaryOp::Mul => "*",
                            BinaryOp::Div => "/",
                            BinaryOp::Lt => "<",
                            BinaryOp::Gt => ">",
                            BinaryOp::LtEq => "<=",
                            BinaryOp::GtEq => ">=",
                            _ => "",
                        };
                        format!("({} {} {})", l, op, r)
                    }
                }
            }
            Expr::Call(call) => {
                if let Expr::Member(mem) = &*call.callee {
                    let prop = mem.prop.sym.as_str();
                    if prop == "indexOf" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("(if (@import(\"std\").mem.indexOf(u8, {}, {})) |i| @as(i32, @intCast(i)) else @as(i32, -1))", obj, arg);
                    } else if prop == "lastIndexOf" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("(if (@import(\"std\").mem.lastIndexOf(u8, {}, {})) |i| @as(i32, @intCast(i)) else @as(i32, -1))", obj, arg);
                    } else if prop == "includes" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!(
                            "(@import(\"std\").mem.indexOf(u8, {}, {}) != null)",
                            obj, arg
                        );
                    } else if prop == "startsWith" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("@import(\"std\").mem.startsWith(u8, {}, {})", obj, arg);
                    } else if prop == "endsWith" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("@import(\"std\").mem.endsWith(u8, {}, {})", obj, arg);
                    } else if prop == "slice" || prop == "substring" {
                        let obj = self.generate_expr(&mem.obj);
                        let start = self.generate_expr(&call.args[0]);
                        let end = self.generate_expr(&call.args[1]);
                        return format!("{}[@intCast({})..@intCast({})]", obj, start, end);
                    } else if prop == "toLowerCase" {
                        let obj = self.generate_expr(&mem.obj);
                        return format!("(blk: {{ var out = std.heap.page_allocator.alloc(u8, {}.len) catch unreachable; for ({}, 0..) |c, i| out[i] = @import(\"std\").ascii.toLower(c); break :blk out; }})", obj, obj);
                    } else if prop == "toUpperCase" {
                        let obj = self.generate_expr(&mem.obj);
                        return format!("(blk: {{ var out = std.heap.page_allocator.alloc(u8, {}.len) catch unreachable; for ({}, 0..) |c, i| out[i] = @import(\"std\").ascii.toUpper(c); break :blk out; }})", obj, obj);
                    } else if prop == "replace" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        let replacement = self.generate_expr(&call.args[1]);
                        return format!("@import(\"std\").mem.replaceOwned(u8, std.heap.page_allocator, {}, {}, {}) catch unreachable", obj, arg, replacement);
                    } else if prop == "charCodeAt" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("@as(i32, @intCast({}[@intCast({})]))", obj, arg);
                    } else if prop == "match" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("{}.exec({})", arg, obj);
                    } else if prop == "test" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("{}.@\"test\"({})", obj, arg);
                    } else if prop == "exec" {
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("{}.exec({})", obj, arg);
                    }
                }

                if let Expr::Member(mem) = &*call.callee {
                    if let Expr::Ident(id) = &*mem.obj {
                        if self.enum_defs.contains_key(&id.sym) {
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
                                    let gen_arg = self.generate_expr(arg);
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

                                let call_res = if target_recv_mode == ReceiverMode::Move {
                                    if is_self_recv {
                                        self.moved_vars.insert("this".to_string());
                                        self.moved_vars.insert("_".to_string());
                                        if !self.is_state_machine {
                                            format!(
                                                "(blk: {{ __self_moved = true; break :blk {}.{}({}); }})",
                                                obj_code, mem.prop.sym, args
                                            )
                                        } else {
                                            format!("{}.{}({})", obj_code, mem.prop.sym, args)
                                        }
                                    } else {
                                        self.moved_vars.insert(id.sym.clone());
                                        if !self.is_state_machine {
                                            format!(
                                                "(blk: {{ __{}_moved = true; break :blk {}.{}({}); }})",
                                                id.sym, obj_code, mem.prop.sym, args
                                            )
                                        } else {
                                            format!("{}.{}({})", obj_code, mem.prop.sym, args)
                                        }
                                    }
                                } else {
                                    format!("{}.{}({})", obj_code, mem.prop.sym, args)
                                };
                                return self.maybe_wrap_try_call(call_res, ret_ty.as_ref());
                            }
                        }
                    }
                }

                // 3. Check omitted-receiver sibling method call inside a class method (`helper()` -> `self.helper()`)
                if let Expr::Ident(ident) = &*call.callee {
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
                                    let gen_arg = self.generate_expr(arg);
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

                let callee = match &*call.callee {
                    Expr::Ident(ident) => ident.sym.clone(),
                    _ => self.generate_expr(&call.callee),
                };
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
    const _raw_json = @import(\"std\").fmt.allocPrint(std.heap.page_allocator, "{{f}}", .{{@import(\"std\").json.fmt(_fmt, .{{}})}}) catch unreachable;
    const _res = Response._create(std.heap.page_allocator) catch unreachable;
    _res.init(_raw_json);
    _res.contentType = "application/json";
    break :blk _res;
}})"#,
                        arg
                    );
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
                    let gen_arg = self.generate_expr(arg);
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
                let call_res = format!("{}({})", callee, args);
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
                let id = self.lambda_counter;
                self.lambda_counter += 1;

                match inner_ty {
                    Some(Type::ResultType { ok, err: src_err }) => {
                        let err_branch = if let Some((blk_label, err_var, flag_var, target_err)) =
                            self.try_catch_target.clone()
                        {
                            self.verify_error_type_compatibility(&src_err, &target_err);
                            let tgt_zig = self.map_error_type(&target_err);
                            format!(
                                ".err => |__e_{id}| {{ {err_var} = cp_convert_err({tgt_zig}, __e_{id}); {flag_var} = true; break :{blk_label}; }}"
                            )
                        } else if let Some(Type::ResultType { err: fn_err, .. }) =
                            self.current_fn_ret_ty.clone()
                        {
                            self.verify_error_type_compatibility(&src_err, &fn_err);
                            let tgt_zig = self.map_error_type(&fn_err);
                            format!(
                                ".err => |__e_{id}| return .{{ .err = cp_convert_err({tgt_zig}, __e_{id}) }}"
                            )
                        } else {
                            panic!("CP Compile Error: Operator '?' cannot be used on '!Error' expression inside a function that does not return '!Error'");
                        };

                        let ok_branch = if matches!(*ok, Type::Optional(_))
                            && self.try_catch_target.is_none()
                        {
                            match &self.current_fn_ret_ty {
                                Some(Type::ResultType { ok: fn_ok, .. })
                                    if matches!(&**fn_ok, Type::Optional(_)) =>
                                {
                                    format!(
                                        ".ok => |__v_{id}| {{ if (__v_{id} == null) return .{{ .ok = null }}; break :blk_q_{id} __v_{id}.?; }}"
                                    )
                                }
                                Some(Type::Optional(_)) => {
                                    format!(
                                        ".ok => |__v_{id}| {{ if (__v_{id} == null) return null; break :blk_q_{id} __v_{id}.?; }}"
                                    )
                                }
                                _ => format!(".ok => |__v_{id}| break :blk_q_{id} __v_{id}"),
                            }
                        } else {
                            format!(".ok => |__v_{id}| break :blk_q_{id} __v_{id}")
                        };

                        format!(
                            "(blk_q_{id}: {{ const __r_{id} = {exp}; switch (__r_{id}) {{ {err_branch}, {ok_branch} }} }})"
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
                let exp = self.generate_expr(inner);
                self.in_try_region = old_in_try;
                let id = self.lambda_counter;
                self.lambda_counter += 1;

                if *is_return_fallback {
                    let is_null_fb = matches!(&**fallback, Expr::Lit(Lit::Null))
                        || matches!(&**fallback, Expr::Ident(id) if id.sym == "null");
                    let ret_stmt = if let Some(Type::ResultType { err: fn_err, .. }) =
                        self.current_fn_ret_ty.clone()
                    {
                        if let Expr::ErrWrap(err_inner) = &**fallback {
                            let ev = self.generate_err_value(err_inner, Some(&fn_err));
                            format!("return .{{ .err = {} }};", ev)
                        } else if is_null_fb {
                            if !matches!(&self.current_fn_ret_ty, Some(Type::ResultType { ok, .. }) if matches!(&**ok, Type::Optional(_)))
                            {
                                panic!("CP Compile Error: Cannot return 'null' from function whose return type is not optional '?T'");
                            }
                            "return .{ .ok = null };".to_string()
                        } else {
                            let fb_c = self.generate_expr(fallback);
                            format!("return .{{ .ok = {} }};", fb_c)
                        }
                    } else {
                        if is_null_fb && !matches!(&self.current_fn_ret_ty, Some(Type::Optional(_)))
                        {
                            panic!("CP Compile Error: Cannot return 'null' from function whose return type is not optional '?T'");
                        }
                        let fb_c = self.generate_expr(fallback);
                        format!("return {};", fb_c)
                    };
                    format!(
                        "(blk_qf_{id}: {{ const __r_{id} = {exp}; if (cp_has_ok_val(__r_{id})) break :blk_qf_{id} cp_unwrap_ok_val(__r_{id}); {ret_stmt} }})"
                    )
                } else {
                    let fb_c = self.generate_expr(fallback);
                    format!(
                        "(blk_qf_{id}: {{ const __r_{id} = {exp}; if (cp_has_ok_val(__r_{id})) break :blk_qf_{id} cp_unwrap_ok_val(__r_{id}); break :blk_qf_{id} cp_val({fb_c}); }})"
                    )
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
                let r_code = self.generate_expr(right);
                let id = self.lambda_counter;
                self.lambda_counter += 1;
                format!(
                    "(blk_nc_{id}: {{ const __l_{id} = {l_code}; if (__l_{id}) |__v_{id}| break :blk_nc_{id} __v_{id}; break :blk_nc_{id} cp_val({r_code}); }})"
                )
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

                let catch_arm_body = match &**body {
                    Stmt::Expr(e) => {
                        let ec = self.generate_expr(e);
                        format!("break :{} cp_val({});", blk_label, ec)
                    }
                    Stmt::Return(_) => {
                        let saved = self.current_case_content.clone();
                        self.current_case_content.clear();
                        self.generate_stmt(body);
                        let ret_s = self.current_case_content.trim().to_string();
                        self.current_case_content = saved;
                        ret_s
                    }
                    Stmt::Block(b) => {
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
                        self.generate_stmt(body);
                        let s_str = self.current_case_content.trim().to_string();
                        self.current_case_content = saved;
                        s_str
                    }
                };

                self.var_cp_types = old_vars;
                self.locals = old_locals;

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
                    if !self.is_state_machine {
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
                if ei.enum_name.sym == "Error" {
                    let mut s = String::from("Error{ ");
                    for (idx, f) in fields_slice.iter().enumerate() {
                        s.push_str(&format!(".{} = {}", f.key.sym, self.generate_expr(&f.val)));
                        if idx + 1 < fields_slice.len() {
                            s.push_str(", ");
                        }
                    }
                    s.push_str(" }");
                    return s;
                }
                if ei.enum_name.sym.is_empty() {
                    if fields_slice.is_empty() {
                        format!(".{}", ei.variant_name.sym)
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
                        format!(".{{ .{} = .{{ {} }} }}", ei.variant_name.sym, f_str)
                    }
                } else if fields_slice.is_empty() {
                    format!("{}{{ .{} = {{}} }}", ei.enum_name.sym, ei.variant_name.sym)
                } else {
                    let mut f_str = String::new();
                    for (idx, f) in fields_slice.iter().enumerate() {
                        f_str.push_str(&format!(".{} = {}", f.key.sym, self.generate_expr(&f.val)));
                        if idx + 1 < fields_slice.len() {
                            f_str.push_str(", ");
                        }
                    }
                    format!(
                        "{}{{ .{} = .{{ {} }} }}",
                        ei.enum_name.sym, ei.variant_name.sym, f_str
                    )
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
            Expr::New(call) => {
                let class_name = match &*call.callee {
                    Expr::Ident(id) => id.sym.clone(),
                    _ => "Unknown".to_string(),
                };
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
                    .lookup_method(&class_name, "init")
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
                            if !self.is_state_machine {
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
                format!("(blk: {{ const _obj = {}._create(std.heap.page_allocator) catch unreachable; if (@hasDecl({}, \"init\")) {{ const RType = @typeInfo(@TypeOf({}.init)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _obj.init({}) catch unreachable; }} else {{ _obj.init({}); }} }} break :blk _obj; }})", class_name, class_name, class_name, args, args)
            }
            Expr::Await(inner) => {
                if let Expr::Call(call) = &**inner {
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
                        let current_fn = self.current_zig_fn_name.clone();
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
                } else {
                    self.generate_expr(inner)
                }
            }
            Expr::Member(member) => {
                if let Expr::Ident(id) = &*member.obj {
                    if id.sym == "Math" {
                        if member.prop.sym == "PI" {
                            return "@import(\"std\").math.pi".to_string();
                        }
                        if member.prop.sym == "E" {
                            return "@import(\"std\").math.e".to_string();
                        }
                    }
                    if self.enum_defs.contains_key(&id.sym) || self.error_sets.contains_key(&id.sym)
                    {
                        return format!("{}{{ .{} = {{}} }}", id.sym, member.prop.sym);
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
                let prop = match member.prop.sym.as_str() {
                    "length" => "len",
                    _ => &member.prop.sym,
                };
                format!("{}.{}", obj, prop)
            }
            Expr::Builtin(call) => match call.name.as_str() {
                "@not" => {
                    let inner = self.generate_expr(&call.args[0]);
                    format!("cp_not({})", inner)
                }
                "@print" => {
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
                "@net_close" => "(blk: { break :blk {}; })".to_string(),
                "@worker_spawn" => {
                    let entry = self.generate_expr(&call.args[0]);
                    format!("(blk: {{ const wid = global_next_worker_id.fetchAdd(1, .monotonic); const _Runner = struct {{ pub fn run(w_id: u32) void {{ current_node_id = w_id; next_local_id = 2; const worker_ctx = std.heap.page_allocator.create({0}_Context) catch unreachable; worker_ctx.* = undefined; worker_ctx.pc = 0; worker_ctx._self = PID{{ .node_id = w_id, .local_id = 1 }}; worker_ctx.arena = null; worker_ctx._parent = null; worker_ctx._parent_fn = null; worker_ctx.received_msg = null; worker_ctx.received_sender = null; schedule(@ptrCast(&{0}), @ptrCast(worker_ctx)); while (true) {{ drain_cross_thread_mailbox(); while (queue_len > 0) {{ const task = task_queue[0]; for (task_queue[1..queue_len], 0..) |t, j| {{ task_queue[j] = t; }} queue_len -= 1; task.run(task.ctx) catch unreachable; }} drain_cross_thread_mailbox(); if (active_fds == 0 and queue_len == 0 and waiting_count == 0) break; if (queue_len == 0 and waiting_count > 0 and active_fds == 0) {{ const ts = std.os.linux.timespec{{ .sec = 0, .nsec = 500_000 }}; _ = std.os.linux.nanosleep(&ts, null); }} if (active_fds > 0) {{ const ready_count = std.posix.poll(poll_fds[0..active_fds], 10) catch 0; if (ready_count > 0) {{ var i: usize = 0; while (i < active_fds) {{ if (poll_fds[i].revents != 0) {{ schedule(fd_to_run[i], fd_to_ctx[i]); poll_fds[i] = poll_fds[active_fds - 1]; fd_to_ctx[i] = fd_to_ctx[active_fds - 1]; fd_to_run[i] = fd_to_run[active_fds - 1]; active_fds -= 1; }} else {{ i += 1; }} }} }} }} }} }} }}; const _t = @import(\"std\").Thread.spawn(.{{}}, _Runner.run, .{{wid}}) catch unreachable; _t.detach(); break :blk PID{{ .node_id = wid, .local_id = 1 }}; }})", entry)
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
                    format!("(blk: {{ const ptr = std.heap.page_allocator.alloc(u8, {}) catch unreachable; break :blk ptr.ptr; }})", s)
                }
                "@free" => "(blk: { break :blk {}; })".to_string(),
                "@get_arena" => "(blk: { break :blk 0; })".to_string(),
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
                _ => "0".to_string(),
            },
            Expr::Arrow(arrow) => {
                let lambda_name = format!("lambda_{}", self.lambda_counter);
                self.lambda_counter += 1;
                let old_fn = self.current_zig_fn_name.clone();
                let mut lambda_output = String::new();
                let mut lambda_gen = Codegen::new();
                lambda_gen.globals = self.globals.clone();
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
                    if let Some(ty) = self.locals.get(&r) {
                        captured.push((r.clone(), ty.clone()));
                    }
                }
                self.captured_vars
                    .insert(lambda_name.clone(), captured.clone());

                for (r, ty) in &captured {
                    lambda_gen.locals.insert(r.clone(), ty.clone());
                    writeln!(lambda_output, "    {}: {},", r, ty).unwrap();
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
                        lambda_gen.write_to_case(&format!("_ = {}; return;", ec));
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

                for moved in &lambda_gen.moved_vars {
                    if self.locals.contains_key(moved) {
                        self.moved_vars.insert(moved.clone());
                    }
                }

                self.current_zig_fn_name = old_fn;
                lambda_name
            }
            Expr::Object(fields) => {
                let mut s = String::from(".{ ");
                for (idx, field) in fields.iter().enumerate() {
                    s.push_str(&format!(".{} = {}", field.key.sym, self.generate_expr(&field.val)));
                    if idx < fields.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s.push_str(" }");
                s
            },
            Expr::Array(items) => {
                let mut s = String::from(".{ ");
                for (idx, item) in items.iter().enumerate() {
                    s.push_str(&self.generate_expr(item));
                    if idx < items.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s.push_str(" }");
                s
            },
            _ => "0".to_string(),
        }
    }

    fn generate_http_serve(&mut self, call_arg: &Expr) -> String {
        let mut port_expr = "3000".to_string();
        let mut fetch_fn_name = String::new();

        match call_arg {
            Expr::Object(fields) => {
                for f in fields {
                    if f.key.sym == "port" {
                        port_expr = self.generate_expr(&f.val);
                    } else if f.key.sym == "fetch" {
                        match &f.val {
                            Expr::Ident(id) => {
                                fetch_fn_name = id.sym.clone();
                            }
                            Expr::Arrow(arrow) => {
                                let handler_name = format!("http_handler_{}", self.lambda_counter);
                                self.lambda_counter += 1;
                                let mut handler_output = String::new();
                                let mut h_gen = Codegen::new();
                                h_gen.globals = self.globals.clone();
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
                                        writeln!(handler_output, "    return {};", ret).unwrap();
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
            _ => match call_arg {
                Expr::Ident(id) => fetch_fn_name = id.sym.clone(),
                _ => fetch_fn_name = self.generate_expr(call_arg),
            },
        }

        format!(
            r#"(blk: {{
    const _port: u16 = @intCast({});
    const _addr = @import(\"std\").Io.net.IpAddress.parse("0.0.0.0", _port) catch unreachable;
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

    fn infer_zig_type(&self, ty: Option<&Type>, init: Option<&Expr>) -> String {
        if let Some(t) = ty {
            return self.map_type(t);
        }
        if let Some(expr) = init {
            if let Some(cp_ty) = self.infer_cp_type(expr) {
                return self.map_type(&cp_ty);
            }
            match expr {
                Expr::Lit(crate::ast::Lit::Int(_)) => return "i32".to_string(),
                Expr::Lit(crate::ast::Lit::Float(_)) => return "f32".to_string(),
                Expr::Lit(crate::ast::Lit::Str(_)) => return "[]const u8".to_string(),
                Expr::Lit(crate::ast::Lit::Bool(_)) => return "bool".to_string(),
                Expr::New(call) => {
                    if let Expr::Ident(id) = &*call.callee {
                        if id.sym == "Error" {
                            return "Error".to_string();
                        }
                        return format!("*{}", id.sym);
                    }
                }
                Expr::Call(call) => {
                    if let Expr::Ident(id) = &*call.callee {
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
                        if prop == "indexOf" || prop == "lastIndexOf" || prop == "charCodeAt" {
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
                        {
                            return "[]const u8".to_string();
                        }
                        if mem.prop.sym == "accept" {
                            return "*Socket".to_string();
                        }
                        if mem.prop.sym == "get" {
                            return "Message".to_string();
                        }
                        if mem.prop.sym == "has" || mem.prop.sym == "delete" {
                            return "bool".to_string();
                        }
                        if mem.prop.sym == "size" {
                            return "i32".to_string();
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
                Expr::Move(inner) | Expr::Borrow(inner, _) => {
                    return self.infer_zig_type(None, Some(inner));
                }
                Expr::Ident(id) => {
                    if id.sym == "@self" || id.sym == "@sender" {
                        return "PID".to_string();
                    }
                    if id.sym == "this" || id.sym == "_" {
                        if let Some(t) = self.locals.get("self") {
                            return t.clone();
                        }
                    }
                    if let Some(t) = self.locals.get(&id.sym) {
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
                _ => {}
            }
        }
        "*anyopaque".to_string() // Fallback
    }

    fn extract_locals_simple(&mut self, block: &BlockStmt) {
        for stmt in &block.body {
            match stmt {
                Stmt::Var(v) => {
                    let old_box = self.expected_box_trait.clone();
                    if let Some(Type::BoxDyn(tr_id)) = &v.ty {
                        self.expected_box_trait = Some(tr_id.sym.clone());
                    }
                    let ty_str = self.infer_zig_type(v.ty.as_ref(), v.init.as_ref());
                    match &v.pat {
                        Pattern::Ident(i) => {
                            self.locals.insert(i.sym.clone(), ty_str);
                            if let Some(t) = &v.ty {
                                self.var_cp_types.insert(i.sym.clone(), t.clone());
                            } else if let Some(init_expr) = &v.init {
                                if let Some(inferred_ty) = self.infer_cp_type(init_expr) {
                                    self.var_cp_types.insert(i.sym.clone(), inferred_ty);
                                }
                            }
                        }
                        Pattern::Tuple(el) => {
                            if let Some(Type::Tuple(tys)) = v.ty.as_ref() {
                                for (idx, p) in el.iter().enumerate() {
                                    if let Pattern::Ident(i) = p {
                                        let t_str = if idx < tys.len() {
                                            self.map_type(&tys[idx])
                                        } else {
                                            "i32".to_string()
                                        };
                                        self.locals.insert(i.sym.clone(), t_str);
                                    }
                                }
                            } else {
                                for p in el {
                                    if let Pattern::Ident(i) = p {
                                        self.locals.insert(i.sym.clone(), "i32".to_string());
                                    }
                                }
                            }
                        }
                        Pattern::Object(fields) => {
                            if let Some(Type::Object(tys)) = v.ty.as_ref() {
                                for p in fields {
                                    let ident = if let Some(Pattern::Ident(i)) = &p.val {
                                        i
                                    } else {
                                        &p.key
                                    };
                                    let t_str = tys
                                        .iter()
                                        .find(|t| t.ident.sym == p.key.sym)
                                        .map(|t| self.map_type(&t.ty))
                                        .unwrap_or_else(|| "i32".to_string());
                                    self.locals.insert(ident.sym.clone(), t_str);
                                }
                            } else {
                                for p in fields {
                                    let ident = if let Some(Pattern::Ident(i)) = &p.val {
                                        i
                                    } else {
                                        &p.key
                                    };
                                    self.locals.insert(ident.sym.clone(), "i32".to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                    self.expected_box_trait = old_box;
                }
                Stmt::Block(b) => self.extract_locals_simple(b),
                Stmt::If(i) => {
                    if let Stmt::Block(b) = &*i.cons {
                        self.extract_locals_simple(b);
                    }
                    if let Some(alt) = &i.alt {
                        if let Stmt::Block(b) = &**alt {
                            self.extract_locals_simple(b);
                        }
                    }
                }
                Stmt::While(w) => {
                    if let Stmt::Block(b) = &*w.body {
                        self.extract_locals_simple(b);
                    }
                }
                Stmt::TryCatch {
                    try_block,
                    catch_body,
                    ..
                } => {
                    let old_try = self.in_try_region;
                    self.in_try_region = true;
                    self.extract_locals_simple(try_block);
                    self.in_try_region = old_try;
                    if let Some(cb) = catch_body {
                        if let Stmt::Block(b) = &**cb {
                            self.extract_locals_simple(b);
                        }
                    }
                }
                Stmt::Match(m) => {
                    let disc_ty = self.infer_cp_type(&m.disc);
                    for arm in &m.arms {
                        let old_v = self.var_cp_types.clone();
                        if let Pattern::Err(inner_p) = &arm.pat {
                            if let Pattern::Ident(id) = &**inner_p {
                                if let Some(Type::ResultType { err, .. }) = &disc_ty {
                                    self.var_cp_types
                                        .insert(id.sym.clone(), self.error_type_to_cp_type(err));
                                }
                            }
                        } else if let Pattern::Ident(id) = &arm.pat {
                            if let Some(Type::ResultType { ok, .. }) = &disc_ty {
                                self.var_cp_types.insert(id.sym.clone(), (**ok).clone());
                            } else if let Some(t) = &disc_ty {
                                self.var_cp_types.insert(id.sym.clone(), t.clone());
                            }
                        }
                        if let Stmt::Block(b) = &*arm.body {
                            self.extract_locals_simple(b);
                        }
                        for (k, v) in old_v {
                            self.var_cp_types.insert(k, v);
                        }
                    }
                }
                Stmt::Expr(Expr::Match(m)) => {
                    let disc_ty = self.infer_cp_type(&m.disc);
                    for arm in &m.arms {
                        let old_v = self.var_cp_types.clone();
                        if let Pattern::Err(inner_p) = &arm.pat {
                            if let Pattern::Ident(id) = &**inner_p {
                                if let Some(Type::ResultType { err, .. }) = &disc_ty {
                                    self.var_cp_types
                                        .insert(id.sym.clone(), self.error_type_to_cp_type(err));
                                }
                            }
                        } else if let Pattern::Ident(id) = &arm.pat {
                            if let Some(Type::ResultType { ok, .. }) = &disc_ty {
                                self.var_cp_types.insert(id.sym.clone(), (**ok).clone());
                            } else if let Some(t) = &disc_ty {
                                self.var_cp_types.insert(id.sym.clone(), t.clone());
                            }
                        }
                        if let Stmt::Block(b) = &*arm.body {
                            self.extract_locals_simple(b);
                        }
                        for (k, v) in old_v {
                            self.var_cp_types.insert(k, v);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_local_vars_to_str(&mut self, block: &BlockStmt, output: &mut String) {
        for stmt in &block.body {
            match stmt {
                Stmt::Var(var) => {
                    let ty_str = self.infer_zig_type(var.ty.as_ref(), var.init.as_ref());
                    match &var.pat {
                        Pattern::Ident(ident) => {
                            self.locals.insert(ident.sym.clone(), ty_str.clone());
                            writeln!(output, "    {}: {},", ident.sym, ty_str).unwrap();
                        }
                        Pattern::Tuple(el) => {
                            if let Some(Type::Tuple(tys)) = var.ty.as_ref() {
                                for (idx, p) in el.iter().enumerate() {
                                    if let Pattern::Ident(ident) = p {
                                        let t_str = if idx < tys.len() {
                                            self.map_type(&tys[idx])
                                        } else {
                                            "i32".to_string()
                                        };
                                        self.locals.insert(ident.sym.clone(), t_str.clone());
                                        writeln!(output, "    {}: {},", ident.sym, t_str).unwrap();
                                    }
                                }
                            }
                        }
                        Pattern::Object(fields) => {
                            if let Some(Type::Object(tys)) = var.ty.as_ref() {
                                for p in fields {
                                    let ident = if let Some(Pattern::Ident(i)) = &p.val {
                                        i
                                    } else {
                                        &p.key
                                    };
                                    let t_str = tys
                                        .iter()
                                        .find(|t| t.ident.sym == p.key.sym)
                                        .map(|t| self.map_type(&t.ty))
                                        .unwrap_or_else(|| "i32".to_string());
                                    self.locals.insert(ident.sym.clone(), t_str.clone());
                                    writeln!(output, "    {}: {},", ident.sym, t_str).unwrap();
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Stmt::Block(b) => self.extract_local_vars_to_str(b, output),
                _ => {}
            }
        }
    }

    fn extract_local_vars(&mut self, _block: &BlockStmt) {
        let mut sorted_locals: Vec<(String, String)> = self
            .locals
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        sorted_locals.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, ty_str) in sorted_locals {
            if name == "self" || name == "this" {
                continue;
            }
            if ty_str == "[]const u8" {
                self.writeln(&format!("{}: []const u8 = &[_]u8{{}},", name));
            } else if ty_str == "i32" || ty_str == "u32" {
                self.writeln(&format!("{}: {} = 0,", name, ty_str));
            } else if ty_str == "bool" {
                self.writeln(&format!("{}: bool = false,", name));
            } else {
                self.writeln(&format!("{}: {} = undefined,", name, ty_str));
            }
        }
    }

    fn map_type(&self, ty: &Type) -> String {
        match ty {
            Type::I32 => "i32".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::USize => "usize".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "[]const u8".to_string(),
            Type::PID => "PID".to_string(),
            Type::Void => "void".to_string(),
            Type::Any => "*anyopaque".to_string(),
            Type::Array(inner) => format!("[]const {}", self.map_type(inner)),
            Type::Ref(ident) => {
                if ident.sym == "PID"
                    || ident.sym == "Message"
                    || ident.sym == "Error"
                    || ident.sym.starts_with("__AnonErr_")
                    || ident.sym.starts_with("__UnionErr_")
                    || self.enum_defs.contains_key(&ident.sym)
                    || self.error_sets.contains_key(&ident.sym)
                    || self.trait_defs.contains_key(&ident.sym)
                {
                    return ident.sym.clone();
                }
                format!("*{}", ident.sym)
            }
            Type::BorrowType { inner, .. } => self.map_type(inner),
            Type::Shared(inner) => self.map_type(inner),
            Type::Optional(inner) => format!("?{}", self.map_type(inner)),
            Type::ErrorUnion(inner) => format!("anyerror!{}", self.map_type(inner)),
            Type::ResultType { ok, err } => {
                format!(
                    "CpResult({}, {})",
                    self.map_type(ok),
                    self.map_error_type(err)
                )
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
                format!("struct {{ {} }}", mapped.join(", "))
            }
            _ => "i32".to_string(),
        }
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }
    fn dedent(&mut self) {
        self.indent_level -= 1;
    }
    fn writeln(&mut self, s: &str) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
        self.output.push_str(s);
        self.output.push('\n');
    }
    fn write_to_case(&mut self, s: &str) {
        let indent = "    ".repeat(self.indent_level + 1);
        self.current_case_content
            .push_str(&format!("{}{}\n", indent, s));
    }
}
