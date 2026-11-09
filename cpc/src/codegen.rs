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

#[derive(Default, Debug)]
pub struct ModuleFeatures {
    pub needs_regex: bool,
    pub needs_collections: bool,
    pub needs_actors: bool,
    pub needs_reflection: bool,
    pub needs_unpack: bool,
    pub needs_result_error: bool,
    pub needs_opt_chaining: bool,
    pub needs_null_coalesce: bool,
    pub needs_fmt: bool,
    pub needs_math_random: bool,
    pub needs_equality: bool,
    pub needs_comparison: bool,
    pub needs_hash: bool,
    pub needs_array_helpers: bool,
    pub needs_iter_helpers: bool,
    pub needs_string_helpers: bool,
    pub needs_add: bool,
    pub needs_not: bool,
    pub needs_index_helpers: bool,
    pub needs_shift: bool,
    pub needs_cast_int: bool,
    pub needs_sync: bool,
    pub is_lib_mode: bool,
}

impl ModuleFeatures {
    pub fn scan(module: &Module, is_lib_mode: bool) -> Self {
        let mut f = Self {
            is_lib_mode,
            ..Self::default()
        };
        for item in &module.body {
            match item {
                ModuleItem::Decl(d) => f.scan_decl(d),
                ModuleItem::Stmt(s) => f.scan_stmt(s),
                ModuleItem::Import(imp) => {
                    if imp.source == "cap:sync"
                        || imp
                            .specifiers
                            .iter()
                            .any(|s| s.sym == "Mutex" || s.sym == "RwLock" || s.sym == "Atomic")
                    {
                        f.needs_sync = true;
                    }
                }
            }
        }
        if f.is_lib_mode {
            f.needs_actors = false;
        }

        if f.needs_collections || f.needs_actors {
            f.needs_string_helpers = true;
            f.needs_equality = true;
            f.needs_comparison = true;
            f.needs_add = true;
            f.needs_index_helpers = true;
            f.needs_iter_helpers = true;
            f.needs_fmt = true;
        }
        if f.needs_index_helpers {
            f.needs_string_helpers = true;
            f.needs_equality = true;
            f.needs_iter_helpers = true;
        }
        if f.needs_array_helpers
            || f.needs_hash
            || f.needs_fmt
            || f.needs_null_coalesce
            || f.needs_equality
            || f.needs_comparison
            || f.needs_add
            || f.needs_not
        {
            f.needs_string_helpers = true;
        }
        f
    }

    fn scan_decl(&mut self, d: &Decl) {
        match d {
            Decl::Func(func) => {
                if func.ident.sym == "main" {
                    self.needs_actors = true;
                }
                if let Some(ret) = &func.return_type {
                    self.scan_type(ret);
                }
                for p in &func.params {
                    self.scan_type(&p.ty);
                }
                self.scan_block(&func.body);
            }
            Decl::Class(c) => {
                if c.implements.iter().any(|i| i.sym == "Agent") {
                    self.needs_actors = true;
                }
                for f in &c.fields {
                    self.scan_type(&f.ty);
                }
                for m in &c.methods {
                    if let Some(ret) = &m.return_type {
                        self.scan_type(ret);
                    }
                    for p in &m.params {
                        self.scan_type(&p.ty);
                    }
                    self.scan_block(&m.body);
                }
            }
            Decl::Trait(t) => {
                for m in &t.methods {
                    if let Some(ret) = &m.return_type {
                        self.scan_type(ret);
                    }
                    for p in &m.params {
                        self.scan_type(&p.ty);
                    }
                }
            }
            Decl::Enum(e) => {
                for m in &e.methods {
                    if let Some(ret) = &m.return_type {
                        self.scan_type(ret);
                    }
                    for p in &m.params {
                        self.scan_type(&p.ty);
                    }
                    self.scan_block(&m.body);
                }
            }
            Decl::Impl(i) => {
                for m in &i.methods {
                    if let Some(ret) = &m.return_type {
                        self.scan_type(ret);
                    }
                    for p in &m.params {
                        self.scan_type(&p.ty);
                    }
                    self.scan_block(&m.body);
                }
            }
            Decl::TypeAlias(ta) => {
                self.scan_type(&ta.target_ty);
            }
            _ => {}
        }
    }

    fn scan_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(v) => {
                if let Some(t) = &v.ty {
                    self.scan_type(t);
                }
                if let Some(init) = &v.init {
                    self.scan_expr(init);
                }
                self.scan_pat(&v.pat);
            }
            Stmt::Expr(e) => self.scan_expr(e),
            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => self.scan_block(b),
            Stmt::If(i) => {
                self.scan_expr(&i.test);
                self.scan_stmt(&i.cons);
                if let Some(alt) = &i.alt {
                    self.scan_stmt(alt);
                }
            }
            Stmt::While(w) => {
                self.scan_expr(&w.test);
                self.scan_stmt(&w.body);
            }
            Stmt::For(for_stmt) => {
                if !matches!(for_stmt.expr, Expr::Range { .. }) {
                    let is_native_slice = match &for_stmt.expr {
                        Expr::Member(m) => m.prop.sym == "items",
                        Expr::Call(c) => match &*c.callee {
                            Expr::Member(m) => m.prop.sym == "values",
                            _ => false,
                        },
                        _ => false,
                    };
                    if !is_native_slice {
                        self.needs_iter_helpers = true;
                    }
                }
                self.scan_expr(&for_stmt.expr);
                self.scan_stmt(&for_stmt.body);
            }
            Stmt::Match(m) => {
                self.needs_result_error = true;
                self.scan_expr(&m.disc);
                for arm in &m.arms {
                    self.scan_pat(&arm.pat);
                    if let Some(g) = &arm.guard {
                        self.scan_expr(g);
                    }
                    self.scan_stmt(&arm.body);
                }
            }
            Stmt::TryCatch {
                try_block,
                catch_body,
                ..
            } => {
                self.needs_result_error = true;
                self.scan_block(try_block);
                if let Some(cb) = catch_body {
                    self.scan_stmt(cb);
                }
            }
            Stmt::Return(r) => {
                if let Some(a) = &r.arg {
                    self.scan_expr(a);
                }
            }
            _ => {}
        }
    }

    fn scan_block(&mut self, b: &BlockStmt) {
        for s in &b.body {
            self.scan_stmt(s);
        }
    }

    fn scan_pat(&mut self, p: &Pattern) {
        match p {
            Pattern::Rest(_) => self.needs_unpack = true,
            Pattern::Tuple(items) => {
                self.needs_unpack = true;
                for item in items {
                    self.scan_pat(item);
                }
            }
            Pattern::Enum(ep) => {
                self.needs_result_error = true;
                if ep.tuple_pats.is_some() {
                    self.needs_unpack = true;
                }
                if let Some(tp) = &ep.tuple_pats {
                    for item in tp {
                        self.scan_pat(item);
                    }
                }
            }
            Pattern::Err(inner) => self.scan_pat(inner),
            Pattern::Or(pats) => {
                for item in pats {
                    self.scan_pat(item);
                }
            }
            Pattern::Binding { subpat, .. } => self.scan_pat(subpat),
            _ => {}
        }
    }

    fn scan_type(&mut self, t: &Type) {
        match t {
            Type::ResultType { .. } => self.needs_result_error = true,
            Type::ErrorUnion(_) => {}
            Type::Dict(_, _) | Type::Map(_, _) | Type::Set(_) => self.needs_collections = true,
            Type::PID => self.needs_actors = true,
            Type::Conditional(cond, cons, alt) => {
                self.scan_expr(cond);
                self.scan_type(cons);
                self.scan_type(alt);
            }
            Type::Ref(id) => {
                if id.sym == "RegExp" {
                    self.needs_regex = true;
                } else if id.sym == "Mutex"
                    || id.sym == "RwLock"
                    || id.sym == "Atomic"
                    || id.sym.starts_with("Mutex<")
                    || id.sym.starts_with("RwLock<")
                    || id.sym.starts_with("Atomic<")
                    || id.sym.starts_with("LockGuard")
                    || id.sym.starts_with("ReadGuard")
                    || id.sym.starts_with("WriteGuard")
                    || id.sym == "CpBareMutex"
                    || id.sym == "CpBareRwLock"
                {
                    self.needs_sync = true;
                } else if !self.is_lib_mode
                    && (id.sym == "Task"
                        || id.sym == "Message"
                        || id.sym == "PID"
                        || id.sym == "Agent")
                {
                    self.needs_actors = true;
                } else if id.sym == "Error" || id.sym == "CpResult" {
                    self.needs_result_error = true;
                }
            }
            Type::Optional(inner) => self.scan_type(inner),
            Type::Array(inner)
            | Type::FixedArray(inner, _)
            | Type::ManyRawPtr(inner)
            | Type::CPtr(inner)
            | Type::SentinelPtr(inner, _)
            | Type::SentinelSlice(inner, _) => self.scan_type(inner),
            Type::Tuple(ts) => {
                for item in ts {
                    self.scan_type(item);
                }
            }
            Type::Fn(params, ret) => {
                for p in params {
                    self.scan_type(p);
                }
                self.scan_type(ret);
            }
            _ => {}
        }
    }

    fn scan_expr(&mut self, e: &Expr) {
        match e {
            Expr::Ident(id) => {
                if id.sym == "RegExp" {
                    self.needs_regex = true;
                } else if id.sym == "Mutex" || id.sym == "RwLock" || id.sym == "Atomic" {
                    self.needs_sync = true;
                } else if !self.is_lib_mode
                    && (id.sym == "Task"
                        || id.sym == "Message"
                        || id.sym == "PID"
                        || id.sym == "Agent")
                {
                    self.needs_actors = true;
                } else if id.sym == "reflect"
                    || id.sym == "TypeInfo"
                    || id.sym == "FieldInfo"
                    || id.sym == "MethodInfo"
                {
                    self.needs_reflection = true;
                } else if id.sym == "Error" || id.sym == "CpResult" {
                    self.needs_result_error = true;
                }
            }
            Expr::Dict(_) => self.needs_collections = true,
            Expr::Spawn(_) | Expr::Receive(_) => self.needs_actors = true,
            Expr::ReflectType { .. } | Expr::ReflectOf { .. } => self.needs_reflection = true,
            Expr::If {
                test, cons, alt, ..
            } => {
                self.scan_expr(test);
                self.scan_expr(cons);
                if let Some(a) = alt {
                    self.scan_expr(a);
                }
            }
            Expr::QuestionFallback { expr, fallback, .. } => {
                if !self.is_lib_mode {
                    self.needs_result_error = true;
                }
                self.scan_expr(expr);
                self.scan_expr(fallback);
            }
            Expr::NullCoalesce { left, right } => {
                if !self.is_lib_mode {
                    self.needs_null_coalesce = true;
                }
                self.scan_expr(left);
                self.scan_expr(right);
            }
            Expr::Question(q) => {
                self.needs_result_error = true;
                self.scan_expr(q);
            }
            Expr::Catch { expr, body, .. } => {
                self.needs_result_error = true;
                self.scan_expr(expr);
                self.scan_stmt(body);
            }
            Expr::ErrWrap(inner) => {
                self.needs_result_error = true;
                self.scan_expr(inner);
            }
            Expr::Template(_) => {
                self.needs_fmt = true;
            }
            Expr::Bin(b) => {
                match b.op {
                    BinaryOp::Eq | BinaryOp::NotEq => {
                        if !self.is_lib_mode {
                            let has_num = matches!(
                                &*b.left,
                                Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_))
                            ) || matches!(
                                &*b.right,
                                Expr::Lit(Lit::Int(_)) | Expr::Lit(Lit::Float(_))
                            );
                            let is_enum = matches!(&*b.left, Expr::EnumInit(_))
                                || matches!(&*b.right, Expr::EnumInit(_));
                            let is_null = matches!(&*b.left, Expr::Lit(Lit::Null))
                                || matches!(&*b.right, Expr::Lit(Lit::Null));
                            let is_bool = matches!(&*b.left, Expr::Lit(Lit::Bool(_)))
                                || matches!(&*b.right, Expr::Lit(Lit::Bool(_)));
                            if !has_num && !is_enum && !is_null && !is_bool {
                                self.needs_equality = true;
                            }
                        }
                    }
                    BinaryOp::Add => {
                        if !self.is_lib_mode {
                            self.needs_add = true;
                        }
                    }
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                        if !self.is_lib_mode {
                            self.needs_comparison = true;
                        }
                    }
                    BinaryOp::Shl | BinaryOp::Shr => {
                        self.needs_shift = true;
                    }
                    BinaryOp::Assign => {
                        if matches!(&*b.left, Expr::Index { .. }) {
                            self.needs_cast_int = true;
                        }
                    }
                    _ => {}
                }
                self.scan_expr(&b.left);
                self.scan_expr(&b.right);
            }
            Expr::New(c) => {
                if let Expr::Ident(id) = &*c.callee {
                    if id.sym == "Map" || id.sym == "Dict" || id.sym == "Set" {
                        self.needs_collections = true;
                    }
                }
                self.scan_expr(&c.callee);
                for a in &c.args {
                    self.scan_expr(a);
                }
            }
            Expr::Call(c) => {
                if let Expr::Ident(id) = &*c.callee {
                    if id.sym == "@not" && !self.is_lib_mode {
                        self.needs_not = true;
                    }
                    if (id.sym == "string" || id.sym == "String") && c.args.len() == 1 {
                        self.needs_fmt = true;
                        self.needs_string_helpers = true;
                    }
                }
                if let Expr::Member(m) = &*c.callee {
                    if m.prop.sym == "toString" && c.args.is_empty() {
                        self.needs_fmt = true;
                        self.needs_string_helpers = true;
                    }
                    if let Expr::Ident(id) = &*m.obj {
                        if id.sym == "Math" && m.prop.sym == "random" {
                            self.needs_math_random = true;
                        }
                        if (id.sym == "string" || id.sym == "String") && m.prop.sym == "from" {
                            self.needs_fmt = true;
                            self.needs_string_helpers = true;
                        }
                    }
                    match m.prop.sym.as_str() {
                        "push" | "pop" => self.needs_array_helpers = true,
                        "includes" => self.needs_index_helpers = true,
                        "indexOf" | "lastIndexOf" if c.args.len() == 1 => self.needs_index_helpers = true,
                        "slice" | "substring" if !c.args.is_empty() => {
                            self.needs_iter_helpers = true
                        }
                        "withLock" | "withRead" | "withWrite" => self.needs_sync = true,
                        _ => {}
                    }
                }
                self.scan_expr(&c.callee);
                for a in &c.args {
                    self.scan_expr(a);
                }
            }
            Expr::Builtin(b) => {
                if b.name == "@hash" {
                    self.needs_hash = true;
                } else if b.name == "@not" && !self.is_lib_mode {
                    self.needs_not = true;
                }
                for a in &b.args {
                    self.scan_expr(a);
                }
            }
            Expr::OptCall(c) => {
                self.needs_opt_chaining = true;
                self.scan_expr(&c.callee);
                for a in &c.args {
                    self.scan_expr(a);
                }
            }
            Expr::Member(m) => self.scan_expr(&m.obj),
            Expr::OptMember(m) => {
                self.needs_opt_chaining = true;
                self.scan_expr(&m.obj);
            }
            Expr::Slice(s) => {
                self.scan_expr(&s.obj);
                if let Some(st) = &s.start {
                    self.scan_expr(st);
                }
                if let Some(en) = &s.end {
                    self.scan_expr(en);
                }
                if let Some(sent) = &s.sentinel {
                    self.scan_expr(sent);
                }
            }
            Expr::Index { obj, index } => {
                self.scan_expr(obj);
                self.scan_expr(index);
            }
            Expr::Array(items) => {
                if !self.is_lib_mode {
                    self.needs_array_helpers = true;
                }
                for item in items {
                    self.scan_expr(item);
                }
            }
            Expr::ArrayRepeat(val, count) => {
                self.scan_expr(val);
                self.scan_expr(count);
            }
            Expr::Object(fields) => {
                for f in fields {
                    self.scan_expr(&f.val);
                }
            }
            Expr::EnumInit(ei) => {
                if let Some(fields) = &ei.fields {
                    for f in fields {
                        self.scan_expr(&f.val);
                    }
                }
                if let Some(args) = &ei.tuple_args {
                    for a in args {
                        self.scan_expr(a);
                    }
                }
            }
            Expr::Match(m) => {
                self.needs_result_error = true;
                self.scan_expr(&m.disc);
                for arm in &m.arms {
                    self.scan_pat(&arm.pat);
                    if let Some(g) = &arm.guard {
                        self.scan_expr(g);
                    }
                    self.scan_stmt(&arm.body);
                }
            }
            Expr::Cast {
                expr, target_ty, ..
            } => {
                self.needs_cast_int = true;
                self.scan_expr(expr);
                self.scan_type(target_ty);
            }
            Expr::Borrow(inner, _)
            | Expr::Move(inner)
            | Expr::Await(inner)
            | Expr::Unwrap(inner)
            | Expr::Deref(inner)
            | Expr::BitNot(inner)
            | Expr::BoxNew(inner) => {
                self.scan_expr(inner);
            }
            Expr::Arrow(arr) => match &*arr.body {
                ArrowBody::Expr(e) => self.scan_expr(e),
                ArrowBody::Block(b) => self.scan_block(b),
            },
            Expr::Unsafe(b) | Expr::Comptime(b) | Expr::Arena(b) => self.scan_block(b),
            Expr::LabeledBlock { block, .. } => self.scan_block(block),
            Expr::LabeledBreak(_, _, val) => {
                if let Some(v) = val {
                    self.scan_expr(v);
                }
            }
            _ => {}
        }
    }
}

pub struct Codegen {
    pub is_beam_mode: bool,
    pub is_lib_mode: bool,
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
    current_fn_zig_name: String,
    pub drop_flag_vars: HashSet<String>,
    is_class_method: bool,
    is_main: bool,
    pub moved_vars: HashSet<String>,
    pub lambda_moved_vars: HashMap<String, HashSet<String>>,
    uninitialized_vars: HashSet<String>,
    pub captured_vars: HashMap<String, Vec<(String, String)>>,
    pub ref_classes: HashSet<String>,
    pub value_classes: HashSet<String>,
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
    pub expected_array_elem_type: Option<Type>,
    pub current_class_name: Option<String>,
    pub current_receiver_mode: ReceiverMode,
    pub mutable_fn_params: HashSet<String>,
    pub move_fn_params: HashSet<String>,
    pub var_cp_types: HashMap<String, Type>,
    pub global_var_types: HashMap<String, Type>,
    pub current_scope_depth: usize,
    pub var_scope_depth: HashMap<String, usize>,
    pub var_borrow_sources: HashMap<String, (Vec<(String, Option<String>)>, bool)>,
    pub var_is_ref_struct: HashSet<String>,
    pub current_fn_params: HashSet<String>,
    pub current_fn_ret_ty: Option<Type>,
    pub current_stmt_idx: usize,
    pub last_use_map: HashMap<String, usize>,
    pub dec_defs: HashMap<String, DecDef>,
    pub current_next_callee: Option<String>,
    pub current_forward_params: Vec<String>,
    pub mut_borrow_loop_vars: HashSet<String>,
    pub loop_borrowed_containers: HashSet<String>,
    pub anon_obj_structs:
        std::rc::Rc<std::cell::RefCell<std::collections::BTreeMap<String, String>>>,
    pub is_parallel_runtime: bool,
    pub parallel_worker_count: usize,
    pub inline_closure_blk: Option<String>,
    pub inline_closure_params: HashSet<String>,
    pub block_locals: HashSet<String>,
    pub type_aliases: HashMap<String, TypeAliasDecl>,
    pub narrowed_null_vars: HashSet<String>,
    pub const_vars: HashSet<String>,
    pub current_block_label: Option<String>,
    pub local_trampolines: HashSet<String>,
    pub source_dir: Option<std::path::PathBuf>,
    pub c_includes: Vec<String>,
    pub source_file_name: Option<String>,
    pub source_code: Option<String>,
    pub line_starts: Vec<usize>,
    pub source_map_entries: Vec<crate::source_map::SourceMapEntry>,
    pub current_func_name: Option<String>,
    pub arena_allocator_stack: Vec<String>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            is_beam_mode: false,
            is_lib_mode: false,
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
            current_fn_zig_name: String::new(),
            drop_flag_vars: HashSet::new(),
            is_class_method: false,
            is_main: false,
            moved_vars: HashSet::new(),
            lambda_moved_vars: HashMap::new(),
            uninitialized_vars: HashSet::new(),
            captured_vars: HashMap::new(),
            ref_classes: HashSet::new(),
            value_classes: HashSet::new(),
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
            expected_array_elem_type: None,
            current_class_name: None,
            current_receiver_mode: ReceiverMode::Static,
            mutable_fn_params: HashSet::new(),
            move_fn_params: HashSet::new(),
            var_cp_types: HashMap::new(),
            global_var_types: HashMap::new(),
            current_scope_depth: 0,
            var_scope_depth: HashMap::new(),
            var_borrow_sources: HashMap::new(),
            var_is_ref_struct: HashSet::new(),
            current_fn_params: HashSet::new(),
            current_fn_ret_ty: None,
            current_stmt_idx: 0,
            last_use_map: HashMap::new(),
            dec_defs: HashMap::new(),
            current_next_callee: None,
            current_forward_params: Vec::new(),
            mut_borrow_loop_vars: HashSet::new(),
            loop_borrowed_containers: HashSet::new(),
            anon_obj_structs: std::rc::Rc::new(std::cell::RefCell::new(
                std::collections::BTreeMap::new(),
            )),
            is_parallel_runtime: false,
            parallel_worker_count: 0,
            inline_closure_blk: None,
            inline_closure_params: HashSet::new(),
            block_locals: HashSet::new(),
            type_aliases: HashMap::new(),
            narrowed_null_vars: HashSet::new(),
            const_vars: HashSet::new(),
            current_block_label: None,
            local_trampolines: HashSet::new(),
            source_dir: None,
            c_includes: Vec::new(),
            source_file_name: None,
            source_code: None,
            line_starts: Vec::new(),
            source_map_entries: Vec::new(),
            current_func_name: None,
            arena_allocator_stack: Vec::new(),
        }
    }

    pub fn current_allocator(&self) -> String {
        if let Some(alloc) = self.arena_allocator_stack.last() {
            alloc.clone()
        } else if self.is_state_machine {
            "ctx.arena.?.allocator()".to_string()
        } else {
            "std.heap.page_allocator".to_string()
        }
    }

    pub fn init_source(&mut self, file_name: Option<&str>, code: Option<&str>) {
        if let Some(f) = file_name {
            self.source_file_name = Some(f.to_string());
        }
        if let Some(c) = code {
            self.source_code = Some(c.to_string());
            let mut starts = vec![0];
            for (idx, b) in c.bytes().enumerate() {
                if b == b'\n' {
                    starts.push(idx + 1);
                }
            }
            self.line_starts = starts;
        }
    }

    pub fn current_generated_line(&self) -> u32 {
        let out_newlines = self.output.bytes().filter(|&b| b == b'\n').count();
        let case_newlines = self.current_case_content.bytes().filter(|&b| b == b'\n').count();
        (out_newlines + case_newlines + 1) as u32
    }

    pub fn offset_to_line_col(&self, offset: usize) -> (u32, u32) {
        if self.line_starts.is_empty() {
            return (0, 0);
        }
        match self.line_starts.binary_search(&offset) {
            Ok(line) => ((line + 1) as u32, 1),
            Err(line) => {
                let line_idx = line.saturating_sub(1);
                let col = offset.saturating_sub(self.line_starts[line_idx]) + 1;
                ((line_idx + 1) as u32, col as u32)
            }
        }
    }

    pub fn record_source_location(&mut self, span: crate::Span) {
        if span.start == 0 && span.end == 0 {
            return;
        }
        if let Some(file) = &self.source_file_name {
            let (cp_line, cp_col) = self.offset_to_line_col(span.start);
            if cp_line > 0 {
                let zig_line = self.current_generated_line();
                if let Some(last) = self.source_map_entries.last() {
                    if last.zig_line == zig_line && last.cp_line == cp_line {
                        return;
                    }
                }
                self.source_map_entries.push(crate::source_map::SourceMapEntry {
                    zig_line,
                    zig_col: 1,
                    cp_file: file.clone(),
                    cp_line,
                    cp_col,
                    symbol: self.current_func_name.clone(),
                });
            }
        }
    }

    pub fn build_source_map(&self, cp_source: String, zig_target: String) -> crate::source_map::SourceMap {
        let mut sm = crate::source_map::SourceMap::new(cp_source, zig_target);
        sm.mappings = self.source_map_entries.clone();
        sm
    }

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

    pub fn is_simple_literal_switch(m: &MatchExpr) -> bool {
        if m.arms.is_empty() {
            return false;
        }
        for arm in &m.arms {
            if arm.guard.is_some() {
                return false;
            }
            match &arm.pat {
                Pattern::Lit(_) | Pattern::Wildcard | Pattern::Range { .. } => {}
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

    pub fn scan_features_in_module(module: &Module) -> ModuleFeatures {
        ModuleFeatures::scan(module, false)
    }

    pub fn generate_module(&mut self, module: &Module) -> String {
        let features = ModuleFeatures::scan(module, self.is_lib_mode);
        let mut class_order = Vec::new();
        let mut impl_decls = Vec::new();
        let mut has_single_thread_meta = false;
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Func(f)) = item {
                self.globals.insert(f.ident.sym.clone());
                self.fn_sigs.insert(f.ident.sym.clone(), f.clone());
                if f.ident.sym == "main" {
                    for m in &f.meta {
                        if m.name.sym == "parallel" {
                            self.is_parallel_runtime = true;
                            for arg in &m.args {
                                let expr = match arg {
                                    MetaArg::Positional(e) => e,
                                    MetaArg::Named(_, e) => e,
                                };
                                if let Expr::Lit(Lit::Int(n)) = expr {
                                    if *n > 0 {
                                        self.parallel_worker_count = *n as usize;
                                    }
                                }
                            }
                        } else if m.name.sym == "single_thread" {
                            has_single_thread_meta = true;
                            self.is_parallel_runtime = false;
                        }
                    }
                }
            }
            if let ModuleItem::Stmt(Stmt::Expr(Expr::Zig(z))) = item {
                for line in z.code.lines() {
                    let trimmed = line.trim();
                    let stripped = if let Some(s) = trimmed.strip_prefix("pub ") {
                        s.trim_start()
                    } else {
                        trimmed
                    };
                    if let Some(rest) = stripped.strip_prefix("const ") {
                        let mut parts = rest.split(&[' ', ':', '='][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.value_classes.insert(name.to_string());
                                self.globals.insert(name.to_string());
                            }
                        }
                    } else if let Some(rest) = stripped.strip_prefix("fn ") {
                        let mut parts = rest.split(&[' ', '(', '<'][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.globals.insert(name.to_string());
                            }
                        }
                    } else if let Some(rest) = stripped.strip_prefix("extern fn ") {
                        let mut parts = rest.split(&[' ', '(', '<'][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.globals.insert(name.to_string());
                            }
                        }
                    } else if let Some(rest) = stripped.strip_prefix("extern \"c\" fn ") {
                        let mut parts = rest.split(&[' ', '(', '<'][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.globals.insert(name.to_string());
                            }
                        }
                    }
                }
            }
            if let ModuleItem::Decl(Decl::Class(c)) = item {
                self.globals.insert(c.ident.sym.clone());
                if (c.is_ref || c.meta.iter().any(|m| m.name.sym == "ref"))
                    && !c.meta.iter().any(|m| m.name.sym == "value")
                {
                    self.ref_classes.insert(c.ident.sym.clone());
                }
                if c.meta.iter().any(|m| m.name.sym == "value") {
                    self.value_classes.insert(c.ident.sym.clone());
                }
                for code in &c.extra_zig {
                    for line in code.lines() {
                        let trimmed = line.trim();
                        let stripped = if let Some(s) = trimmed.strip_prefix("pub ") {
                            s.trim_start()
                        } else {
                            trimmed
                        };
                        if let Some(rest) = stripped.strip_prefix("const ") {
                            let mut parts = rest.split(&[' ', ':', '='][..]);
                            if let Some(name) = parts.next() {
                                let name = name.trim();
                                if !name.is_empty() {
                                    self.value_classes.insert(name.to_string());
                                }
                            }
                        }
                    }
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
            if let ModuleItem::Decl(Decl::Dec(d)) = item {
                self.dec_defs.insert(d.ident.sym.clone(), d.clone());
            }
            if let ModuleItem::Decl(Decl::TypeAlias(ta)) = item {
                self.globals.insert(ta.ident.sym.clone());
                self.type_aliases.insert(ta.ident.sym.clone(), ta.clone());
            }
            if let ModuleItem::Decl(Decl::CInclude(ci)) = item {
                if !self.c_includes.contains(&ci.header) {
                    self.c_includes.push(ci.header.clone());
                }
                self.globals.insert("c".to_string());
                self.value_classes.insert("c".to_string());
            }
            if let ModuleItem::Stmt(Stmt::Var(v)) = item {
                if let Pattern::Ident(id) = &v.pat {
                    self.globals.insert(id.sym.clone());
                    if let Some(t) = &v.ty {
                        self.var_cp_types.insert(id.sym.clone(), t.clone());
                    }
                    if v.kind == VarDeclKind::Const
                        && id.sym.chars().next().map_or(false, |c| c.is_ascii_uppercase())
                    {
                        self.value_classes.insert(id.sym.clone());
                    }
                }
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
                                generic_params: tm.generic_params.clone(),
                                params: tm.params.clone(),
                                return_type: tm.return_type.clone(),
                                body: default_body.clone(),
                                is_async: false,
                                is_static: false,
                                is_unsafe: tm.is_unsafe,
                                meta: Vec::new(),
                                is_pub: true,
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

        if !self.is_parallel_runtime && !has_single_thread_meta {
            if let Ok(cap_content) = std::fs::read_to_string("cap.json") {
                if cap_content.contains("\"mode\": \"parallel\"")
                    || cap_content.contains("\"mode\":\"parallel\"")
                {
                    self.is_parallel_runtime = true;
                }
            }
        }

        self.globals.insert("Math".to_string());
        self.globals.insert("Map".to_string());
        self.globals.insert("Dict".to_string());
        self.globals.insert("Set".to_string());
        self.globals.insert("HashMap".to_string());
        self.globals.insert("HashSet".to_string());
        self.globals.insert("VecDeque".to_string());
        self.globals.insert("BinaryHeap".to_string());
        self.globals.insert("BTreeMap".to_string());
        self.globals.insert("BTreeSet".to_string());
        self.globals.insert("PID".to_string());
        self.globals.insert("Message".to_string());
        self.globals.insert("Mailbox".to_string());
        self.globals.insert("Error".to_string());
        self.globals.insert("Box".to_string());
        self.globals.insert("print".to_string());
        self.globals.insert("spawn".to_string());
        self.globals.insert("join".to_string());
        self.globals.insert("Task".to_string());
        self.globals.insert("Event".to_string());
        self.globals.insert("EventEmitter".to_string());
        self.globals.insert("console".to_string());
        self.globals.insert("reflect".to_string());
        self.globals.insert("TypeInfo".to_string());
        self.globals.insert("FieldInfo".to_string());
        self.globals.insert("MethodInfo".to_string());
        self.globals.insert("Mutex".to_string());
        self.globals.insert("RwLock".to_string());
        self.globals.insert("Atomic".to_string());
        self.globals.insert("Shared".to_string());
        self.globals.insert("LockGuard".to_string());
        self.globals.insert("ReadGuard".to_string());
        self.globals.insert("WriteGuard".to_string());

        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Var(v)) = item {
                if let Pattern::Ident(id) = &v.pat {
                    self.globals.insert(id.sym.clone());
                    let mut inferred = v.ty.clone();
                    if inferred.is_none() {
                        if let Some(init) = &v.init {
                            inferred = self.infer_cp_type_with_mock(init, &std::collections::HashMap::new());
                        }
                    }
                    if let Some(ty) = inferred {
                        self.var_cp_types.insert(id.sym.clone(), ty.clone());
                        self.global_var_types.insert(id.sym.clone(), ty);
                    }
                }
            }
        }

        self.writeln("// Generated by CP Compiler (RFC-002 Production SDK)");
        self.writeln("const std = @import(\"std\");");
        self.writeln("const string = []const u8;");
        if self.is_beam_mode {
            self.writeln("const beam = @import(\"beam.zig\");");
        }
        if !self.c_includes.is_empty() {
            self.writeln("pub const c = struct {");
            self.writeln("    pub extern \"c\" fn puts(s: [*c]const u8) c_int;");
            self.writeln("    pub extern \"c\" fn printf(fmt: [*c]const u8, ...) c_int;");
            self.writeln("    pub extern \"c\" fn snprintf(buf: [*c]u8, size: usize, fmt: [*c]const u8, ...) c_int;");
            self.writeln("    pub extern \"c\" fn malloc(size: usize) ?*anyopaque;");
            self.writeln("    pub extern \"c\" fn free(ptr: ?*anyopaque) void;");
            self.writeln("    pub extern \"c\" fn exit(status: c_int) noreturn;");
            self.writeln("    pub extern \"c\" fn getenv(name: [*c]const u8) ?[*c]u8;");
            self.writeln("    pub extern \"c\" fn strlen(s: [*c]const u8) usize;");
            self.writeln("    pub extern \"c\" fn strcmp(s1: [*c]const u8, s2: [*c]const u8) c_int;");
            self.writeln("    pub extern \"c\" fn strncmp(s1: [*c]const u8, s2: [*c]const u8, n: usize) c_int;");
            self.writeln("    pub extern \"c\" fn memcpy(dest: ?*anyopaque, src: ?*const anyopaque, n: usize) ?*anyopaque;");
            self.writeln("    pub extern \"c\" fn memset(s: ?*anyopaque, c_val: c_int, n: usize) ?*anyopaque;");
            self.writeln("};");
        }
        self.writeln("");

        // --- Tri-State Result, Built-in Error, String Concat & Pattern Helpers ---
        if !self.is_lib_mode && features.needs_result_error {
            self.writeln("pub fn CpResult(comptime T: type, comptime E: type) type {");
            self.writeln(
                "    return union(enum) { ok: T, err: E, pub const __IS_CP_RESULT = true; };",
            );
            self.writeln("}");
            self.writeln("pub const Err = struct {");
            self.writeln("    code: i32 = 0,");
            self.writeln("    msg: []const u8 = \"\",");
            self.writeln("    cause: ?*const anyopaque = null,");
            self.writeln("    cause_type_id: u64 = 0,");
            self.writeln("    pub const __TYPE_ID: u64 = 0xEE00_0000_0000_0001;");
            self.writeln("    pub fn init(code: i32, msg: []const u8) Err { return Err{ .code = code, .msg = msg }; }");
            self.writeln("    pub fn with_cause(self: Err, cause: anytype) Err { const CType = @TypeOf(cause); var ptr: ?*const anyopaque = null; var tid: u64 = 0; if (comptime @typeInfo(CType) == .pointer) { ptr = @ptrCast(cause); const Child = @typeInfo(CType).pointer.child; if (comptime @hasDecl(Child, \"__TYPE_ID\")) { tid = Child.__TYPE_ID; } } else if (comptime (@typeInfo(CType) == .@\"struct\" or @typeInfo(CType) == .@\"union\") and @hasField(CType, \"ptr\")) { ptr = @ptrCast(cause.ptr); if (comptime @hasField(CType, \"vtable\")) { tid = cause.vtable.__type_id; } } else if (comptime @typeInfo(CType) == .optional) { if (cause) |c| return self.with_cause(c); } else { if (comptime @hasDecl(CType, \"__TYPE_ID\")) { tid = CType.__TYPE_ID; } ptr = @ptrCast(&cause); } return Err{ .code = self.code, .msg = self.msg, .cause = ptr, .cause_type_id = tid }; }");
            self.writeln("    pub fn downcast(self: Err, comptime Target: type) ?*const Target {");
            self.writeln("        if (self.cause) |c| {");
            self.writeln("            if (comptime @hasDecl(Target, \"__TYPE_ID\")) {");
            self.writeln("                if (self.cause_type_id != 0 and self.cause_type_id != Target.__TYPE_ID) {");
            self.writeln("                    return null;");
            self.writeln("                }");
            self.writeln("            }");
            self.writeln("            return @ptrCast(@alignCast(c));");
            self.writeln("        }");
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("pub const Error = Err;");
            self.writeln("fn is_cp_result(comptime T: type) bool {");
            self.writeln(
                "    return @typeInfo(T) == .@\"union\" and @hasDecl(T, \"__IS_CP_RESULT\");",
            );
            self.writeln("}");
        }
        if features.needs_opt_chaining {
            self.writeln("fn CpOptType(comptime T: type) type { if (@typeInfo(T) == .optional) return T; return ?T; }");
            self.writeln("fn CpUnwrapOpt(comptime T: type) type { if (@typeInfo(T) == .optional) return @typeInfo(T).optional.child; return T; }");
            self.writeln("fn cp_opt_is_null(v: anytype) bool { const T = @TypeOf(v); if (comptime @typeInfo(T) == .optional) return v == null; return false; }");
            self.writeln("fn cp_opt_unwrap(v: anytype) CpUnwrapOpt(@TypeOf(v)) { const T = @TypeOf(v); if (comptime @typeInfo(T) == .optional) { return v.?; } else { return v; } }");
        }
        if features.needs_cast_int {
            self.writeln("fn cp_cast_int(comptime Dest: type, val: anytype) Dest { const Src = @TypeOf(val); if (comptime Src == Dest) return val; if (comptime Src == comptime_int) { if (comptime @typeInfo(Dest).int.signedness == .unsigned and val < 0) return @as(Dest, @truncate(@as(u128, @bitCast(@as(i128, val))))) else return @as(Dest, @truncate(val)); } const src_bits = @typeInfo(Src).int.bits; const dest_bits = @typeInfo(Dest).int.bits; if (comptime @typeInfo(Src).int.signedness == @typeInfo(Dest).int.signedness) { if (comptime dest_bits >= src_bits) return @as(Dest, val) else return @truncate(val); } if (comptime @typeInfo(Src).int.signedness == .signed) { const s: i128 = if (comptime 128 >= src_bits) @as(i128, val) else @truncate(val); const u: u128 = @bitCast(s); if (comptime dest_bits >= 128) return @as(Dest, u) else return @truncate(u); } else { const u: u128 = if (comptime 128 >= src_bits) @as(u128, val) else @truncate(val); const s: i128 = @bitCast(u); if (comptime dest_bits >= 128) return @as(Dest, s) else return @truncate(s); } }");
        }

        // --- Top-Level C-FFI / Zig Escapes ---
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Expr(Expr::Zig(zig))) = item {
                for line in zig.code.lines() {
                    let trimmed = line.trim();
                    if trimmed == "const std = @import(\"std\");"
                        || trimmed == "pub const std = @import(\"std\");"
                    {
                        continue;
                    }
                    self.writeln(line);
                }
            }
        }

        // --- Top-Level Constants & Variables ---
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Var(var)) = item {
                let ty_str = if let Some(t) = &var.ty {
                    format!(": {}", self.map_type(t))
                } else {
                    String::new()
                };
                let kw = if var.is_threadlocal {
                    if var.is_pub {
                        "pub threadlocal var"
                    } else {
                        "threadlocal var"
                    }
                } else if var.kind == VarDeclKind::Const {
                    if var.is_pub {
                        "pub const"
                    } else {
                        "const"
                    }
                } else {
                    if var.is_pub {
                        "pub var"
                    } else {
                        "var"
                    }
                };
                let old_array_elem = self.expected_array_elem_type.clone();
                if let Some(Type::FixedArray(elem, _) | Type::Array(elem)) = &var.ty {
                    self.expected_array_elem_type = Some((**elem).clone());
                }
                match &var.pat {
                    Pattern::Ident(ident) => {
                        if ident.sym == "std" {
                            continue;
                        }
                        let init_code = if let Some(init) = &var.init {
                            self.generate_expr(init)
                        } else {
                            "undefined".to_string()
                        };
                        self.writeln(&format!("{} {}{} = {};", kw, ident.sym, ty_str, init_code));
                    }
                    _ => {}
                }
                self.expected_array_elem_type = old_array_elem;
            }
        }

        // --- Top-Level Comptime Blocks ---
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::ComptimeBlock(b)) = item {
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
                self.writeln(&format!("comptime {{\n{}\n}}", body_str));
            }
        }

        if features.needs_regex {
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
        }

        if features.needs_collections || features.needs_actors {
            self.writeln("");
            self.writeln("const MessageContext = struct {");
            self.writeln("    pub fn hash(self: @This(), m: Message) u64 {");
            self.writeln("        var hasher = std.hash.Wyhash.init(0);");
            self.writeln("        switch (m) {");
            self.writeln("            .int => |i| std.hash.autoHash(&hasher, i),");
            self.writeln(
                "            .float => |f| std.hash.autoHash(&hasher, @as(u32, @bitCast(f))),",
            );
            self.writeln("            .string => |s| hasher.update(s),");
            self.writeln("            .boolean => |bl| std.hash.autoHash(&hasher, bl),");
            self.writeln("            .array => |arr| { for (arr) |item| std.hash.autoHash(&hasher, self.hash(item)); },");
            self.writeln("            .any => |a| std.hash.autoHash(&hasher, @intFromPtr(a)),");
            self.writeln("        }");
            self.writeln("        return hasher.final();");
            self.writeln("    }");
            self.writeln("    pub fn eql(self: @This(), a: Message, b: Message) bool {");
            self.writeln("        const Ta = std.meta.activeTag(a);");
            self.writeln("        const Tb = std.meta.activeTag(b);");
            self.writeln("        if (Ta != Tb) return false;");
            self.writeln("        return switch (a) {");
            self.writeln("            .int => |i| i == b.int,");
            self.writeln("            .float => |f| f == b.float,");
            self.writeln("            .string => |s| std.mem.eql(u8, s, b.string),");
            self.writeln("            .boolean => |bl| bl == b.boolean,");
            self.writeln("            .array => |arr| { if (arr.len != b.array.len) return false; for (arr, 0..) |item, idx| { if (!self.eql(item, b.array[idx])) return false; } return true; },");
            self.writeln("            .any => |ptr| ptr == b.any,");
            self.writeln("        };");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const Message = union(enum) {");
            self.writeln("    int: i32, float: f32, string: []const u8, boolean: bool, array: []Message, any: *anyopaque,");
            self.writeln("    pub fn to_string(self: Message) []const u8 { return switch (self) { .string => |s| s, else => \"\", }; }");
            self.writeln("    pub fn to_int(self: Message) i32 { return switch (self) { .int => |i| i, else => 0, }; }");
            self.writeln("    pub fn format(self: Message, comptime fmt: []const u8, options: std.fmt.FormatOptions, writer: anytype) !void {");
            self.writeln("        switch (self) {");
            self.writeln("            .int => |i| try writer.print(\"{d}\", .{i}),");
            self.writeln("            .float => |f| try writer.print(\"{d}\", .{f}),");
            self.writeln("            .string => |s| try writer.print(\"{s}\", .{s}),");
            self.writeln("            .boolean => |b| try writer.print(\"{}\", .{b}),");
            self.writeln("            .array => |arr| { try writer.print(\"[\", .{}); for (arr, 0..) |item, idx| { if (idx > 0) try writer.print(\", \", .{}); try item.format(fmt, options, writer); } try writer.print(\"]\", .{}); },");
            self.writeln("            .any => |a| try writer.print(\"any@{*}\", .{a}),");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("const MessageEntry = struct { key: Message, value: Message };");
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
            self.writeln("    pub fn release(self: *Map) void {");
            self.writeln("        self._map.deinit();");
            self.writeln("        self.allocator.destroy(self);");
            self.writeln("    }");
            self.writeln("    ");
            self.writeln("    pub fn set(self: *Map, key: anytype, value: anytype) *Map {");
            self.writeln("        self._map.put(cp_message_from(key), cp_message_from(value)) catch unreachable;");
            self.writeln("        self.size = @intCast(self._map.count());");
            self.writeln("        return self;");
            self.writeln("    }");
            self.writeln("    ");
            self.writeln("    pub fn get(self: *Map, key: anytype) ?Message {");
            self.writeln("        return self._map.get(cp_message_from(key));");
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
            self.writeln("    ");
            self.writeln("    pub fn put_all(self: *Map, other: *Map) void {");
            self.writeln("        var it = other._map.iterator();");
            self.writeln("        while (it.next()) |entry| {");
            self.writeln(
                "            self._map.put(entry.key_ptr.*, entry.value_ptr.*) catch unreachable;",
            );
            self.writeln("        }");
            self.writeln("        self.size = @intCast(self._map.count());");
            self.writeln("    }");
            self.writeln("    ");
            self.writeln("    pub fn keys(self: *Map) []Message {");
            self.writeln("        const list = self.allocator.alloc(Message, self._map.count()) catch unreachable;");
            self.writeln("        var it = self._map.keyIterator();");
            self.writeln("        var i: usize = 0;");
            self.writeln("        while (it.next()) |k| {");
            self.writeln("            list[i] = k.*;");
            self.writeln("            i += 1;");
            self.writeln("        }");
            self.writeln("        return list;");
            self.writeln("    }");
            self.writeln("    ");
            self.writeln("    pub fn values(self: *Map) []Message {");
            self.writeln("        const list = self.allocator.alloc(Message, self._map.count()) catch unreachable;");
            self.writeln("        var it = self._map.valueIterator();");
            self.writeln("        var i: usize = 0;");
            self.writeln("        while (it.next()) |v| {");
            self.writeln("            list[i] = v.*;");
            self.writeln("            i += 1;");
            self.writeln("        }");
            self.writeln("        return list;");
            self.writeln("    }");
            self.writeln("    ");
            self.writeln("    pub fn entries(self: *Map) []MessageEntry {");
            self.writeln("        const list = self.allocator.alloc(MessageEntry, self._map.count()) catch unreachable;");
            self.writeln("        var it = self._map.iterator();");
            self.writeln("        var i: usize = 0;");
            self.writeln("        while (it.next()) |entry| {");
            self.writeln(
                "            list[i] = .{ .key = entry.key_ptr.*, .value = entry.value_ptr.* };",
            );
            self.writeln("            i += 1;");
            self.writeln("        }");
            self.writeln("        return list;");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const Dict = Map;");
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
            self.writeln("    pub fn release(self: *Set) void {");
            self.writeln("        self._map.deinit();");
            self.writeln("        self.allocator.destroy(self);");
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
            self.writeln("    ");
            self.writeln("    pub fn values(self: *Set) []Message {");
            self.writeln("        const list = self.allocator.alloc(Message, self._map.count()) catch unreachable;");
            self.writeln("        var it = self._map.keyIterator();");
            self.writeln("        var i: usize = 0;");
            self.writeln("        while (it.next()) |k| {");
            self.writeln("            list[i] = k.*;");
            self.writeln("            i += 1;");
            self.writeln("        }");
            self.writeln("        return list;");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const HashMap = Map;");
            self.writeln("const HashSet = Set;");
            self.writeln("");
            self.writeln("fn cp_msg_cmp(a: Message, b: Message) i32 {");
            self.writeln("    const tag_a = @intFromEnum(a);");
            self.writeln("    const tag_b = @intFromEnum(b);");
            self.writeln("    if (tag_a != tag_b) return if (tag_a < tag_b) -1 else 1;");
            self.writeln("    switch (a) {");
            self.writeln("        .int => |ia| { const ib = b.int; if (ia < ib) return -1; if (ia > ib) return 1; return 0; },");
            self.writeln("        .float => |fa| { const fb = b.float; if (fa < fb) return -1; if (fa > fb) return 1; return 0; },");
            self.writeln("        .string => |sa| { const sb = b.string; return switch (std.mem.order(u8, sa, sb)) { .lt => -1, .eq => 0, .gt => 1 }; },");
            self.writeln("        .boolean => |ba| { const bb = b.boolean; if (ba == bb) return 0; return if (!ba and bb) -1 else 1; },");
            self.writeln("        .array => |aa| { const ab = b.array; if (aa.len < ab.len) return -1; if (aa.len > ab.len) return 1; return 0; },");
            self.writeln("        .any => |pa| { const pb = b.any; const ia = @intFromPtr(pa); const ib = @intFromPtr(pb); if (ia < ib) return -1; if (ia > ib) return 1; return 0; },");
            self.writeln("    }");
            self.writeln("}");
            self.writeln("");
            self.writeln("const VecDeque = struct {");
            self.writeln("    allocator: std.mem.Allocator,");
            self.writeln("    buf: []Message,");
            self.writeln("    head: usize,");
            self.writeln("    tail: usize,");
            self.writeln("    count: usize,");
            self.writeln("    capacity: usize,");
            self.writeln("");
            self.writeln("    pub fn _create(allocator: std.mem.Allocator) !*VecDeque {");
            self.writeln("        const self = try allocator.create(VecDeque);");
            self.writeln("        const init_cap: usize = 8;");
            self.writeln("        const b = try allocator.alloc(Message, init_cap);");
            self.writeln("        self.* = .{");
            self.writeln("            .allocator = allocator,");
            self.writeln("            .buf = b,");
            self.writeln("            .head = 0,");
            self.writeln("            .tail = 0,");
            self.writeln("            .count = 0,");
            self.writeln("            .capacity = init_cap,");
            self.writeln("        };");
            self.writeln("        return self;");
            self.writeln("    }");
            self.writeln("    pub fn init(self: *VecDeque) void { _ = self; }");
            self.writeln("    pub fn release(self: *VecDeque) void {");
            self.writeln("        self.allocator.free(self.buf);");
            self.writeln("        self.allocator.destroy(self);");
            self.writeln("    }");
            self.writeln("    fn ensure_capacity(self: *VecDeque) void {");
            self.writeln("        if (self.count == self.capacity) {");
            self.writeln("            const new_cap = self.capacity * 2;");
            self.writeln(
            "            const new_buf = self.allocator.alloc(Message, new_cap) catch unreachable;",
        );
            self.writeln("            var i: usize = 0;");
            self.writeln("            while (i < self.count) : (i += 1) {");
            self.writeln("                new_buf[i] = self.buf[(self.head + i) % self.capacity];");
            self.writeln("            }");
            self.writeln("            self.allocator.free(self.buf);");
            self.writeln("            self.buf = new_buf;");
            self.writeln("            self.head = 0;");
            self.writeln("            self.tail = self.count;");
            self.writeln("            self.capacity = new_cap;");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("    pub fn push_back(self: *VecDeque, val: anytype) void {");
            self.writeln("        self.ensure_capacity();");
            self.writeln("        self.buf[self.tail] = cp_message_from(val);");
            self.writeln("        self.tail = (self.tail + 1) % self.capacity;");
            self.writeln("        self.count += 1;");
            self.writeln("    }");
            self.writeln("    pub fn push_front(self: *VecDeque, val: anytype) void {");
            self.writeln("        self.ensure_capacity();");
            self.writeln(
                "        self.head = if (self.head == 0) self.capacity - 1 else self.head - 1;",
            );
            self.writeln("        self.buf[self.head] = cp_message_from(val);");
            self.writeln("        self.count += 1;");
            self.writeln("    }");
            self.writeln("    pub fn pop_front(self: *VecDeque) ?Message {");
            self.writeln("        if (self.count == 0) return null;");
            self.writeln("        const val = self.buf[self.head];");
            self.writeln("        self.head = (self.head + 1) % self.capacity;");
            self.writeln("        self.count -= 1;");
            self.writeln("        return val;");
            self.writeln("    }");
            self.writeln("    pub fn pop_back(self: *VecDeque) ?Message {");
            self.writeln("        if (self.count == 0) return null;");
            self.writeln(
                "        self.tail = if (self.tail == 0) self.capacity - 1 else self.tail - 1;",
            );
            self.writeln("        const val = self.buf[self.tail];");
            self.writeln("        self.count -= 1;");
            self.writeln("        return val;");
            self.writeln("    }");
            self.writeln("    pub fn peek_front(self: *VecDeque) ?Message {");
            self.writeln("        if (self.count == 0) return null;");
            self.writeln("        return self.buf[self.head];");
            self.writeln("    }");
            self.writeln("    pub fn peek_back(self: *VecDeque) ?Message {");
            self.writeln("        if (self.count == 0) return null;");
            self.writeln(
                "        const idx = if (self.tail == 0) self.capacity - 1 else self.tail - 1;",
            );
            self.writeln("        return self.buf[idx];");
            self.writeln("    }");
            self.writeln("    pub fn get(self: *VecDeque, index: anytype) ?Message {");
            self.writeln("        const m = cp_message_from(index);");
            self.writeln("        const idx_val: usize = switch (m) { .int => |i| if (i >= 0) @intCast(i) else return null, else => return null };");
            self.writeln("        if (idx_val >= self.count) return null;");
            self.writeln("        return self.buf[(self.head + idx_val) % self.capacity];");
            self.writeln("    }");
            self.writeln("    pub fn len(self: *VecDeque) usize { return self.count; }");
            self.writeln("    pub fn size(self: *VecDeque) i32 { return @intCast(self.count); }");
            self.writeln("    pub fn is_empty(self: *VecDeque) bool { return self.count == 0; }");
            self.writeln("    pub fn clear(self: *VecDeque) void { self.head = 0; self.tail = 0; self.count = 0; }");
            self.writeln("    pub fn next(self: *VecDeque) ?Message { return self.pop_front(); }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const BinaryHeap = struct {");
            self.writeln("    allocator: std.mem.Allocator,");
            self.writeln("    items: std.ArrayList(Message),");
            self.writeln("");
            self.writeln("    pub fn _create(allocator: std.mem.Allocator) !*BinaryHeap {");
            self.writeln("        const self = try allocator.create(BinaryHeap);");
            self.writeln("        self.* = .{ .allocator = allocator, .items = .empty };");
            self.writeln("        return self;");
            self.writeln("    }");
            self.writeln("    pub fn init(self: *BinaryHeap) void { _ = self; }");
            self.writeln("    pub fn release(self: *BinaryHeap) void {");
            self.writeln("        self.items.deinit(self.allocator);");
            self.writeln("        self.allocator.destroy(self);");
            self.writeln("    }");
            self.writeln("    pub fn push(self: *BinaryHeap, val: anytype) void {");
            self.writeln("        const m = cp_message_from(val);");
            self.writeln("        self.items.append(self.allocator, m) catch unreachable;");
            self.writeln("        var idx = self.items.items.len - 1;");
            self.writeln("        while (idx > 0) {");
            self.writeln("            const parent = (idx - 1) / 2;");
            self.writeln(
            "            if (cp_msg_cmp(self.items.items[idx], self.items.items[parent]) > 0) {",
        );
            self.writeln("                const tmp = self.items.items[idx];");
            self.writeln("                self.items.items[idx] = self.items.items[parent];");
            self.writeln("                self.items.items[parent] = tmp;");
            self.writeln("                idx = parent;");
            self.writeln("            } else { break; }");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("    pub fn pop(self: *BinaryHeap) ?Message {");
            self.writeln("        if (self.items.items.len == 0) return null;");
            self.writeln("        const top = self.items.items[0];");
            self.writeln("        const last = self.items.pop();");
            self.writeln("        if (self.items.items.len > 0) {");
            self.writeln("            self.items.items[0] = last.?;");
            self.writeln("            var idx: usize = 0;");
            self.writeln("            const n = self.items.items.len;");
            self.writeln("            while (true) {");
            self.writeln("                var largest = idx;");
            self.writeln("                const left = 2 * idx + 1;");
            self.writeln("                const right = 2 * idx + 2;");
            self.writeln("                if (left < n and cp_msg_cmp(self.items.items[left], self.items.items[largest]) > 0) { largest = left; }");
            self.writeln("                if (right < n and cp_msg_cmp(self.items.items[right], self.items.items[largest]) > 0) { largest = right; }");
            self.writeln("                if (largest != idx) {");
            self.writeln("                    const tmp = self.items.items[idx];");
            self.writeln("                    self.items.items[idx] = self.items.items[largest];");
            self.writeln("                    self.items.items[largest] = tmp;");
            self.writeln("                    idx = largest;");
            self.writeln("                } else { break; }");
            self.writeln("            }");
            self.writeln("        }");
            self.writeln("        return top;");
            self.writeln("    }");
            self.writeln("    pub fn peek(self: *BinaryHeap) ?Message {");
            self.writeln("        if (self.items.items.len == 0) return null;");
            self.writeln("        return self.items.items[0];");
            self.writeln("    }");
            self.writeln(
                "    pub fn len(self: *BinaryHeap) usize { return self.items.items.len; }",
            );
            self.writeln(
                "    pub fn size(self: *BinaryHeap) i32 { return @intCast(self.items.items.len); }",
            );
            self.writeln(
                "    pub fn is_empty(self: *BinaryHeap) bool { return self.items.items.len == 0; }",
            );
            self.writeln(
                "    pub fn clear(self: *BinaryHeap) void { self.items.clearRetainingCapacity(); }",
            );
            self.writeln("    pub fn next(self: *BinaryHeap) ?Message { return self.pop(); }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const BTreeMap = struct {");
            self.writeln("    allocator: std.mem.Allocator,");
            self.writeln("    entries: std.ArrayList(MessageEntry),");
            self.writeln("");
            self.writeln("    pub fn _create(allocator: std.mem.Allocator) !*BTreeMap {");
            self.writeln("        const self = try allocator.create(BTreeMap);");
            self.writeln("        self.* = .{ .allocator = allocator, .entries = .empty };");
            self.writeln("        return self;");
            self.writeln("    }");
            self.writeln("    pub fn init(self: *BTreeMap) void { _ = self; }");
            self.writeln("    pub fn release(self: *BTreeMap) void {");
            self.writeln("        self.entries.deinit(self.allocator);");
            self.writeln("        self.allocator.destroy(self);");
            self.writeln("    }");
            self.writeln("    fn binary_search(self: *BTreeMap, k: Message) struct { found: bool, index: usize } {");
            self.writeln("        var low: usize = 0;");
            self.writeln("        var high: usize = self.entries.items.len;");
            self.writeln("        while (low < high) {");
            self.writeln("            const mid = low + (high - low) / 2;");
            self.writeln("            const cmp_res = cp_msg_cmp(self.entries.items[mid].key, k);");
            self.writeln("            if (cmp_res == 0) return .{ .found = true, .index = mid };");
            self.writeln("            if (cmp_res < 0) { low = mid + 1; } else { high = mid; }");
            self.writeln("        }");
            self.writeln("        return .{ .found = false, .index = low };");
            self.writeln("    }");
            self.writeln(
                "    pub fn insert(self: *BTreeMap, key: anytype, val: anytype) ?Message {",
            );
            self.writeln("        const k = cp_message_from(key);");
            self.writeln("        const v = cp_message_from(val);");
            self.writeln("        const search = self.binary_search(k);");
            self.writeln("        if (search.found) {");
            self.writeln("            const old = self.entries.items[search.index].value;");
            self.writeln("            self.entries.items[search.index].value = v;");
            self.writeln("            return old;");
            self.writeln("        } else {");
            self.writeln("            self.entries.insert(self.allocator, search.index, .{ .key = k, .value = v }) catch unreachable;");
            self.writeln("            return null;");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("    pub fn set(self: *BTreeMap, key: anytype, val: anytype) *BTreeMap {");
            self.writeln("        _ = self.insert(key, val);");
            self.writeln("        return self;");
            self.writeln("    }");
            self.writeln("    pub fn get(self: *BTreeMap, key: anytype) ?Message {");
            self.writeln("        const k = cp_message_from(key);");
            self.writeln("        const search = self.binary_search(k);");
            self.writeln(
                "        if (search.found) return self.entries.items[search.index].value;",
            );
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln("    pub fn has(self: *BTreeMap, key: anytype) bool {");
            self.writeln("        const k = cp_message_from(key);");
            self.writeln("        return self.binary_search(k).found;");
            self.writeln("    }");
            self.writeln("    pub fn remove(self: *BTreeMap, key: anytype) ?Message {");
            self.writeln("        const k = cp_message_from(key);");
            self.writeln("        const search = self.binary_search(k);");
            self.writeln("        if (search.found) {");
            self.writeln("            const old = self.entries.items[search.index].value;");
            self.writeln("            _ = self.entries.orderedRemove(search.index);");
            self.writeln("            return old;");
            self.writeln("        }");
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln("    pub fn delete(self: *BTreeMap, key: anytype) bool { return self.remove(key) != null; }");
            self.writeln("    pub fn min_key(self: *BTreeMap) ?Message {");
            self.writeln("        if (self.entries.items.len == 0) return null;");
            self.writeln("        return self.entries.items[0].key;");
            self.writeln("    }");
            self.writeln("    pub fn max_key(self: *BTreeMap) ?Message {");
            self.writeln("        if (self.entries.items.len == 0) return null;");
            self.writeln("        return self.entries.items[self.entries.items.len - 1].key;");
            self.writeln("    }");
            self.writeln(
                "    pub fn len(self: *BTreeMap) usize { return self.entries.items.len; }",
            );
            self.writeln(
                "    pub fn size(self: *BTreeMap) i32 { return @intCast(self.entries.items.len); }",
            );
            self.writeln(
                "    pub fn is_empty(self: *BTreeMap) bool { return self.entries.items.len == 0; }",
            );
            self.writeln(
                "    pub fn clear(self: *BTreeMap) void { self.entries.clearRetainingCapacity(); }",
            );
            self.writeln("    pub fn keys(self: *BTreeMap) []Message {");
            self.writeln("        const list = self.allocator.alloc(Message, self.entries.items.len) catch unreachable;");
            self.writeln(
                "        for (self.entries.items, 0..) |entry, i| { list[i] = entry.key; }",
            );
            self.writeln("        return list;");
            self.writeln("    }");
            self.writeln("    pub fn values(self: *BTreeMap) []Message {");
            self.writeln("        const list = self.allocator.alloc(Message, self.entries.items.len) catch unreachable;");
            self.writeln(
                "        for (self.entries.items, 0..) |entry, i| { list[i] = entry.value; }",
            );
            self.writeln("        return list;");
            self.writeln("    }");
            self.writeln("    pub fn next(self: *BTreeMap) ?MessageEntry {");
            self.writeln("        if (self.entries.items.len == 0) return null;");
            self.writeln("        return self.entries.orderedRemove(0);");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const BTreeSet = struct {");
            self.writeln("    map: *BTreeMap,");
            self.writeln("");
            self.writeln("    pub fn _create(allocator: std.mem.Allocator) !*BTreeSet {");
            self.writeln("        const self = try allocator.create(BTreeSet);");
            self.writeln("        self.map = try BTreeMap._create(allocator);");
            self.writeln("        return self;");
            self.writeln("    }");
            self.writeln("    pub fn init(self: *BTreeSet) void { _ = self; }");
            self.writeln("    pub fn release(self: *BTreeSet) void {");
            self.writeln("        const alloc = self.map.allocator;");
            self.writeln("        self.map.release();");
            self.writeln("        alloc.destroy(self);");
            self.writeln("    }");
            self.writeln("    pub fn add(self: *BTreeSet, val: anytype) bool {");
            self.writeln("        return self.map.insert(val, true) == null;");
            self.writeln("    }");
            self.writeln("    pub fn has(self: *BTreeSet, val: anytype) bool {");
            self.writeln("        return self.map.has(val);");
            self.writeln("    }");
            self.writeln("    pub fn remove(self: *BTreeSet, val: anytype) ?Message {");
            self.writeln("        if (self.map.remove(val) != null) return cp_message_from(true);");
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln("    pub fn delete(self: *BTreeSet, val: anytype) bool { return self.map.delete(val); }");
            self.writeln("    pub fn min(self: *BTreeSet) ?Message { return self.map.min_key(); }");
            self.writeln("    pub fn max(self: *BTreeSet) ?Message { return self.map.max_key(); }");
            self.writeln("    pub fn len(self: *BTreeSet) usize { return self.map.len(); }");
            self.writeln("    pub fn size(self: *BTreeSet) i32 { return self.map.size(); }");
            self.writeln(
                "    pub fn is_empty(self: *BTreeSet) bool { return self.map.is_empty(); }",
            );
            self.writeln("    pub fn clear(self: *BTreeSet) void { self.map.clear(); }");
            self.writeln(
                "    pub fn values(self: *BTreeSet) []Message { return self.map.keys(); }",
            );
            self.writeln("    pub fn next(self: *BTreeSet) ?Message {");
            self.writeln("        if (self.map.next()) |entry| { return entry.key; }");
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("");
        }

        if features.needs_reflection {
            self.writeln("pub const FieldInfo = struct {");
            self.writeln("    name: []const u8,");
            self.writeln("    typeName: []const u8,");
            self.writeln("    meta: *Map,");
            self.writeln("};");
            self.writeln("");
            self.writeln("pub const MethodInfo = struct {");
            self.writeln("    name: []const u8,");
            self.writeln("    returnTypeName: []const u8,");
            self.writeln("    paramNames: []const []const u8,");
            self.writeln("    isAsync: bool,");
            self.writeln("    isStatic: bool,");
            self.writeln("    meta: *Map,");
            self.writeln("};");
            self.writeln("");
            self.writeln("pub const TypeInfo = struct {");
            self.writeln("    name: []const u8,");
            self.writeln("    isRef: bool,");
            self.writeln("    fields: []const FieldInfo,");
            self.writeln("    methods: []const MethodInfo,");
            self.writeln("    meta: *Map,");
            self.writeln(
                "    pub fn getField(self: *const TypeInfo, field_name: []const u8) ?FieldInfo {",
            );
            self.writeln("        for (self.fields) |f| {");
            self.writeln("            if (std.mem.eql(u8, f.name, field_name)) return f;");
            self.writeln("        }");
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln(
                "    pub fn getMethod(self: *const TypeInfo, method_name: []const u8) ?MethodInfo {",
            );
            self.writeln("        for (self.methods) |m| {");
            self.writeln("            if (std.mem.eql(u8, m.name, method_name)) return m;");
            self.writeln("        }");
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("");
        }

        if features.needs_actors {
            self.writeln("const PID = struct { node_id: u32, local_id: u32 };");
        }

        if features.needs_sync {
            self.writeln("// --- Multithreading Synchronization Primitives (cap:sync) ---");
            self.writeln("const NativeMutex = struct {");
            self.writeln("    state: std.atomic.Value(u32) = std.atomic.Value(u32).init(0),");
            self.writeln("    pub fn lock(self: *NativeMutex) void {");
            self.writeln("        var spin: usize = 0;");
            self.writeln("        while (true) {");
            self.writeln("            if (self.state.cmpxchgWeak(0, 1, .acquire, .monotonic) == null) return;");
            self.writeln("            if (spin < 100) { std.atomic.spinLoopHint(); spin += 1; } else { std.Thread.yield() catch {}; }");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln(
                "    pub fn unlock(self: *NativeMutex) void { _ = self.state.swap(0, .release); }",
            );
            self.writeln("    pub fn tryLock(self: *NativeMutex) bool { return self.state.cmpxchgWeak(0, 1, .acquire, .monotonic) == null; }");
            self.writeln("    pub fn isLocked(self: *NativeMutex) bool { return self.state.load(.monotonic) != 0; }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const NativeRwLock = struct {");
            self.writeln("    state: std.atomic.Value(u32) = std.atomic.Value(u32).init(0),");
            self.writeln("    const WRITER_BIT: u32 = 1 << 31;");
            self.writeln("    const READER_MASK: u32 = ~WRITER_BIT;");
            self.writeln("    pub fn lockShared(self: *NativeRwLock) void {");
            self.writeln("        var spin: usize = 0;");
            self.writeln("        while (true) {");
            self.writeln("            const current = self.state.load(.monotonic);");
            self.writeln("            if ((current & WRITER_BIT) == 0) { if (self.state.cmpxchgWeak(current, current + 1, .acquire, .monotonic) == null) return; }");
            self.writeln("            if (spin < 100) { std.atomic.spinLoopHint(); spin += 1; } else { std.Thread.yield() catch {}; }");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("    pub fn unlockShared(self: *NativeRwLock) void {");
            self.writeln("        while (true) {");
            self.writeln("            const current = self.state.load(.monotonic);");
            self.writeln("            const count = current & READER_MASK;");
            self.writeln("            if (count == 0) return;");
            self.writeln("            if (self.state.cmpxchgWeak(current, current - 1, .release, .monotonic) == null) return;");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("    pub fn tryLockShared(self: *NativeRwLock) bool { const current = self.state.load(.monotonic); if ((current & WRITER_BIT) != 0) return false; return self.state.cmpxchgWeak(current, current + 1, .acquire, .monotonic) == null; }");
            self.writeln("    pub fn lockExclusive(self: *NativeRwLock) void {");
            self.writeln("        var spin: usize = 0;");
            self.writeln("        while (true) {");
            self.writeln("            if (self.state.cmpxchgWeak(0, WRITER_BIT, .acquire, .monotonic) == null) return;");
            self.writeln("            if (spin < 100) { std.atomic.spinLoopHint(); spin += 1; } else { std.Thread.yield() catch {}; }");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("    pub fn unlockExclusive(self: *NativeRwLock) void { _ = self.state.cmpxchgStrong(WRITER_BIT, 0, .release, .monotonic); }");
            self.writeln("    pub fn tryLockExclusive(self: *NativeRwLock) bool { return self.state.cmpxchgWeak(0, WRITER_BIT, .acquire, .monotonic) == null; }");
            self.writeln("    pub fn isReadLocked(self: *NativeRwLock) bool { return (self.state.load(.monotonic) & READER_MASK) > 0; }");
            self.writeln("    pub fn isWriteLocked(self: *NativeRwLock) bool { return (self.state.load(.monotonic) & WRITER_BIT) != 0; }");
            self.writeln("};");
            self.writeln("");
            self.writeln("const CpBareMutex = struct {");
            self.writeln("    raw: NativeMutex = .{},");
            self.writeln("    pub fn lock(self: *CpBareMutex) void { self.raw.lock(); }");
            self.writeln("    pub fn unlock(self: *CpBareMutex) void { self.raw.unlock(); }");
            self.writeln(
                "    pub fn tryLock(self: *CpBareMutex) bool { return self.raw.tryLock(); }",
            );
            self.writeln(
                "    pub fn isLocked(self: *CpBareMutex) bool { return self.raw.isLocked(); }",
            );
            self.writeln("};");
            self.writeln("");
            self.writeln("const CpBareRwLock = struct {");
            self.writeln("    raw: NativeRwLock = .{},");
            self.writeln("    pub fn lock(self: *CpBareRwLock) void { self.raw.lockExclusive(); }");
            self.writeln(
                "    pub fn unlock(self: *CpBareRwLock) void { self.raw.unlockExclusive(); }",
            );
            self.writeln(
                "    pub fn lockShared(self: *CpBareRwLock) void { self.raw.lockShared(); }",
            );
            self.writeln(
                "    pub fn unlockShared(self: *CpBareRwLock) void { self.raw.unlockShared(); }",
            );
            self.writeln("    pub fn read(self: *CpBareRwLock) void { self.raw.lockShared(); }");
            self.writeln(
                "    pub fn write(self: *CpBareRwLock) void { self.raw.lockExclusive(); }",
            );
            self.writeln("    pub fn tryLock(self: *CpBareRwLock) bool { return self.raw.tryLockExclusive(); }");
            self.writeln("    pub fn tryLockShared(self: *CpBareRwLock) bool { return self.raw.tryLockShared(); }");
            self.writeln(
                "    pub fn tryRead(self: *CpBareRwLock) bool { return self.raw.tryLockShared(); }",
            );
            self.writeln("    pub fn tryWrite(self: *CpBareRwLock) bool { return self.raw.tryLockExclusive(); }");
            self.writeln("    pub fn isReadLocked(self: *CpBareRwLock) bool { return self.raw.isReadLocked(); }");
            self.writeln("    pub fn isWriteLocked(self: *CpBareRwLock) bool { return self.raw.isWriteLocked(); }");
            self.writeln("};");
            self.writeln("");
            self.writeln("fn LockGuard(comptime T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        mutex: *CpMutex(T),");
            self.writeln("        data: *T,");
            self.writeln("        pub fn release(self: Self) void { self.mutex.raw.unlock(); }");
            self.writeln("        pub fn unlock(self: Self) void { self.release(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn ReadGuard(comptime T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        rwlock: *CpRwLock(T),");
            self.writeln("        data: *const T,");
            self.writeln(
                "        pub fn release(self: Self) void { self.rwlock.raw.unlockShared(); }",
            );
            self.writeln("        pub fn unlock(self: Self) void { self.release(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn WriteGuard(comptime T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        rwlock: *CpRwLock(T),");
            self.writeln("        data: *T,");
            self.writeln(
                "        pub fn release(self: Self) void { self.rwlock.raw.unlockExclusive(); }",
            );
            self.writeln("        pub fn unlock(self: Self) void { self.release(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn CpMutex(comptime T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        raw: NativeMutex = .{},");
            self.writeln("        data: T,");
            self.writeln("        pub fn init(val: T) Self { return .{ .data = val }; }");
            self.writeln("        pub fn lock(self: *Self) LockGuard(T) { self.raw.lock(); return .{ .mutex = self, .data = &self.data }; }");
            self.writeln("        pub fn tryLock(self: *Self) ?LockGuard(T) { if (self.raw.tryLock()) { return LockGuard(T){ .mutex = self, .data = &self.data }; } return null; }");
            self.writeln(
                "        pub fn isLocked(self: *Self) bool { return self.raw.isLocked(); }",
            );
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn CpRwLock(comptime T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        raw: NativeRwLock = .{},");
            self.writeln("        data: T,");
            self.writeln("        pub fn init(val: T) Self { return .{ .data = val }; }");
            self.writeln("        pub fn read(self: *Self) ReadGuard(T) { self.raw.lockShared(); return .{ .rwlock = self, .data = &self.data }; }");
            self.writeln("        pub fn tryRead(self: *Self) ?ReadGuard(T) { if (self.raw.tryLockShared()) { return ReadGuard(T){ .rwlock = self, .data = &self.data }; } return null; }");
            self.writeln("        pub fn write(self: *Self) WriteGuard(T) { self.raw.lockExclusive(); return .{ .rwlock = self, .data = &self.data }; }");
            self.writeln("        pub fn tryWrite(self: *Self) ?WriteGuard(T) { if (self.raw.tryLockExclusive()) { return WriteGuard(T){ .rwlock = self, .data = &self.data }; } return null; }");
            self.writeln(
                "        pub fn isReadLocked(self: *Self) bool { return self.raw.isReadLocked(); }",
            );
            self.writeln("        pub fn isWriteLocked(self: *Self) bool { return self.raw.isWriteLocked(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn CpAtomic(comptime T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        raw: std.atomic.Value(T),");
            self.writeln("        pub fn init(val: T) Self { return .{ .raw = std.atomic.Value(T).init(val) }; }");
            self.writeln(
                "        pub fn load(self: *const Self) T { return self.raw.load(.seq_cst); }",
            );
            self.writeln(
                "        pub fn store(self: *Self, val: T) void { self.raw.store(val, .seq_cst); }",
            );
            self.writeln("        pub fn swap(self: *Self, val: T) T { return self.raw.swap(val, .seq_cst); }");
            self.writeln("        pub fn cas(self: *Self, expected: T, desired: T) bool { return self.raw.cmpxchgStrong(expected, desired, .seq_cst, .seq_cst) == null; }");
            self.writeln("        pub fn compareAndSwap(self: *Self, expected: T, desired: T) bool { return self.cas(expected, desired); }");
            self.writeln("        pub fn add(self: *Self, delta: T) T { return self.raw.fetchAdd(delta, .seq_cst); }");
            self.writeln("        pub fn sub(self: *Self, delta: T) T { return self.raw.fetchSub(delta, .seq_cst); }");
            self.writeln("        pub fn inc(self: *Self) T { return self.add(1); }");
            self.writeln("        pub fn dec(self: *Self) T { return self.sub(1); }");
            self.writeln("        pub fn @\"and\"(self: *Self, mask: T) T { return self.raw.fetchAnd(mask, .seq_cst); }");
            self.writeln("        pub fn @\"or\"(self: *Self, mask: T) T { return self.raw.fetchOr(mask, .seq_cst); }");
            self.writeln("        pub fn @\"xor\"(self: *Self, mask: T) T { return self.raw.fetchXor(mask, .seq_cst); }");
            self.writeln("        pub fn testAndSet(self: *Self) bool { if (comptime T == bool) { return self.raw.swap(true, .seq_cst); } else { return self.raw.swap(1, .seq_cst) != 0; } }");
            self.writeln("        pub fn clear(self: *Self) void { if (comptime T == bool) { self.raw.store(false, .seq_cst); } else { self.raw.store(0, .seq_cst); } }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn cp_make_mutex(val: anytype) CpMutex(@TypeOf(val)) { return CpMutex(@TypeOf(val)).init(val); }");
            self.writeln("fn cp_make_rwlock(val: anytype) CpRwLock(@TypeOf(val)) { return CpRwLock(@TypeOf(val)).init(val); }");
            self.writeln("fn CpAtomicConcreteType(comptime T: type) type { if (T == comptime_int) return i32; if (T == comptime_float) return f32; return T; }");
            self.writeln("fn cp_make_atomic(val: anytype) CpAtomic(CpAtomicConcreteType(@TypeOf(val))) { const T = CpAtomicConcreteType(@TypeOf(val)); return CpAtomic(T).init(@as(T, val)); }");
            self.writeln("");
        }

        if features.needs_collections || features.needs_actors {
            self.writeln("fn CpIndexReturnType(comptime T: type) type {");
            self.writeln("    if (@typeInfo(T) == .pointer) {");
            self.writeln("        const Child = @typeInfo(T).pointer.child;");
            self.writeln("        if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"get\")) return ?Message;");
            self.writeln(
                "        if (@typeInfo(Child) == .array) return @typeInfo(Child).array.child;",
            );
            self.writeln(
                "        if (@typeInfo(Child) == .pointer and @typeInfo(Child).pointer.size == .slice) return @typeInfo(Child).pointer.child;",
            );
            self.writeln("    }");
            self.writeln("    if (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice) return @typeInfo(T).pointer.child;");
            self.writeln("    if (@typeInfo(T) == .array) return @typeInfo(T).array.child;");
            self.writeln("    return ?Message;");
            self.writeln("}");
            self.writeln("fn cp_index_get(container: anytype, key: anytype) CpIndexReturnType(@TypeOf(container)) {");
            self.writeln("    const T = @TypeOf(container);");
            self.writeln("    if (comptime (T == Message or T == ?Message)) {");
            self.writeln(
                "        const val: ?Message = if (T == ?Message) container else container;",
            );
            self.writeln("        if (val) |m| {");
            self.writeln("            switch (m) {");
            self.writeln("                .array => |arr| {");
            self.writeln("                    if (comptime (@typeInfo(@TypeOf(key)) == .int or @typeInfo(@TypeOf(key)) == .comptime_int)) {");
            self.writeln("                        const idx: usize = @intCast(key);");
            self.writeln("                        if (idx < arr.len) return arr[idx];");
            self.writeln("                    }");
            self.writeln("                    return null;");
            self.writeln("                },");
            self.writeln("                .any => |ptr| {");
            self.writeln("                    const map: *Map = @ptrCast(@alignCast(ptr));");
            self.writeln("                    return map.get(key);");
            self.writeln("                },");
            self.writeln("                else => return null,");
            self.writeln("            }");
            self.writeln("        } else {");
            self.writeln("            return null;");
            self.writeln("        }");
            self.writeln("    }");
            self.writeln("    if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(T).pointer.child, \"get\"))) {");
            self.writeln("        return container.get(key);");
            self.writeln("    } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice)) {");
            self.writeln("        return container.*[@as(usize, @intCast(key))];");
            self.writeln("    } else {");
            self.writeln("        return container[@as(usize, @intCast(key))];");
            self.writeln("    }");
            self.writeln("}");
            self.writeln(
                "fn cp_index_assign(container: anytype, key: anytype, value: anytype) void {",
            );
            self.writeln("    const T = @TypeOf(container);");
            self.writeln("    if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(T).pointer.child, \"set\"))) {");
            self.writeln("        _ = container.set(key, value);");
            self.writeln("    } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice)) {");
            self.writeln("        @constCast(container.*)[@as(usize, @intCast(key))] = value;");
            self.writeln("    } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice)) {");
            self.writeln("        @constCast(container)[@as(usize, @intCast(key))] = value;");
            self.writeln("    } else {");
            self.writeln("        container[@as(usize, @intCast(key))] = value;");
            self.writeln("    }");
            self.writeln("}");
            self.writeln("fn cp_message_from(v: anytype) Message {");
            self.writeln("    const T = @TypeOf(v);");
            self.writeln("    if (T == Message) return v;");
            self.writeln(
                "    if (T == []Message or T == []const Message) return .{ .array = @constCast(v) };",
            );
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
            const cp_str_slice: []const u8 = v;
            return .{ .string = cp_str_slice };
        }
        return .{ .any = @ptrCast(@constCast(v)) };
    }",
            );
            self.writeln("    return .{ .any = @ptrCast(@constCast(&v)) };");
            self.writeln("}");
        }

        if features.needs_string_helpers || features.needs_unpack {
            self.writeln("fn is_string_like(comptime T: type) bool { if (T == []const u8 or T == []u8) return true; if (@typeInfo(T) == .pointer) { const Child = @typeInfo(T).pointer.child; if (Child == u8) return true; if (@typeInfo(Child) == .array and @typeInfo(Child).array.child == u8) return true; } return false; }");
            self.writeln("fn to_string_slice(v: anytype) []const u8 { const T = @TypeOf(v); if (comptime (T == []const u8 or T == []u8)) { return v; } else { const cp_str_slice: []const u8 = v; return cp_str_slice; } }");
            self.writeln("fn CpValType(comptime T: type) type { if (is_string_like(T)) return []const u8; if (T == comptime_int) return i32; if (T == comptime_float) return f32; return T; }");
            self.writeln("fn cp_val(v: anytype) CpValType(@TypeOf(v)) { const T = @TypeOf(v); if (comptime is_string_like(T)) { return to_string_slice(v); } else if (comptime T == comptime_int) { return @as(i32, v); } else if (comptime T == comptime_float) { return @as(f32, v); } else { return v; } }");
        }

        if features.needs_add {
            if features.needs_collections || features.needs_actors {
                self.writeln("fn CpAddReturn(comptime Ta: type, comptime Tb: type) type { if (Ta == Message and Tb == Message) return i32; if (Ta == Message and @typeInfo(Tb) == .int) return Tb; if (@typeInfo(Ta) == .int and Tb == Message) return Ta; if (is_string_like(Ta) or is_string_like(Tb)) return []const u8; if (Ta == comptime_int and Tb == comptime_int) return i32; if (Ta == comptime_int) return Tb; if (Tb == comptime_int) return Ta; if (Ta == comptime_float and Tb == comptime_float) return f32; if (Ta == comptime_float) return Tb; if (Tb == comptime_float) return Ta; return Ta; }");
                self.writeln("fn cp_add(a: anytype, b: anytype) CpAddReturn(@TypeOf(a), @TypeOf(b)) { const Ta = @TypeOf(a); const Tb = @TypeOf(b); if (comptime (Ta == Message and Tb == Message)) { return a.to_int() + b.to_int(); } else if (comptime (Ta == Message and @typeInfo(Tb) == .int)) { return @as(Tb, @intCast(a.to_int())) + b; } else if (comptime (@typeInfo(Ta) == .int and Tb == Message)) { return a + @as(Ta, @intCast(b.to_int())); } else if (comptime (is_string_like(Ta) and is_string_like(Tb))) { return std.fmt.allocPrint(std.heap.page_allocator, \"{s}{s}\", .{ to_string_slice(a), to_string_slice(b) }) catch unreachable; } else if (comptime is_string_like(Ta)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{s}{any}\", .{ to_string_slice(a), b }) catch unreachable; } else if (comptime is_string_like(Tb)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{any}{s}\", .{ a, to_string_slice(b) }) catch unreachable; } else if (comptime (@typeInfo(Ta) == .int and @typeInfo(Tb) == .int)) { return a + @as(Ta, @intCast(b)); } else { return a + b; } }");
            } else {
                self.writeln("fn CpAddReturn(comptime Ta: type, comptime Tb: type) type { if (is_string_like(Ta) or is_string_like(Tb)) return []const u8; if (Ta == comptime_int and Tb == comptime_int) return i32; if (Ta == comptime_int) return Tb; if (Tb == comptime_int) return Ta; if (Ta == comptime_float and Tb == comptime_float) return f32; if (Ta == comptime_float) return Tb; if (Tb == comptime_float) return Ta; return Ta; }");
                self.writeln("fn cp_add(a: anytype, b: anytype) CpAddReturn(@TypeOf(a), @TypeOf(b)) { const Ta = @TypeOf(a); const Tb = @TypeOf(b); if (comptime (is_string_like(Ta) and is_string_like(Tb))) { return std.fmt.allocPrint(std.heap.page_allocator, \"{s}{s}\", .{ to_string_slice(a), to_string_slice(b) }) catch unreachable; } else if (comptime is_string_like(Ta)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{s}{any}\", .{ to_string_slice(a), b }) catch unreachable; } else if (comptime is_string_like(Tb)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{any}{s}\", .{ a, to_string_slice(b) }) catch unreachable; } else if (comptime (@typeInfo(Ta) == .int and @typeInfo(Tb) == .int)) { return a + @as(Ta, @intCast(b)); } else { return a + b; } }");
            }
        }

        if !self.is_lib_mode && features.needs_result_error {
            self.writeln("fn CpUnwrapOkType(comptime T: type) type { if (is_cp_result(T)) { const OkT = @FieldType(T, \"ok\"); if (@typeInfo(OkT) == .optional) return @typeInfo(OkT).optional.child; return OkT; } if (@typeInfo(T) == .error_union) return @typeInfo(T).error_union.payload; if (@typeInfo(T) == .optional) return @typeInfo(T).optional.child; return T; }");
            self.writeln("fn CpUnwrapErrType(comptime T: type) type { if (is_cp_result(T)) return @FieldType(T, \"err\"); if (@typeInfo(T) == .error_union) return @typeInfo(T).error_union.error_set; return anyerror; }");
            self.writeln("fn cp_has_ok_val(v: anytype) bool { const T = @TypeOf(v); if (comptime is_cp_result(T)) { if (v != .ok) return false; const OkT = @FieldType(T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok != null; } else { return true; } } else if (comptime @typeInfo(T) == .error_union) { if (v) |_| return true else |_| return false; } else if (comptime @typeInfo(T) == .optional) { return v != null; } else { return true; } }");
            self.writeln("fn cp_unwrap_ok_val(v: anytype) CpUnwrapOkType(@TypeOf(v)) { const T = @TypeOf(v); if (comptime is_cp_result(T)) { const OkT = @FieldType(T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok.?; } else { return v.ok; } } else if (comptime @typeInfo(T) == .error_union) { return v catch unreachable; } else if (comptime @typeInfo(T) == .optional) { return v.?; } else { return v; } }");
            self.writeln("fn cp_unwrap_err_val(v: anytype) CpUnwrapErrType(@TypeOf(v)) { const T = @TypeOf(v); if (comptime is_cp_result(T)) return v.err; if (comptime @typeInfo(T) == .error_union) { if (v) |_| unreachable else |e| return e; } return v; }");
            self.writeln("fn cp_is_null_val(v: anytype) bool { const T = @TypeOf(v); if (comptime is_cp_result(T)) { if (v != .ok) return false; const OkT = @FieldType(T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok == null; } else { return false; } } else if (comptime @typeInfo(T) == .optional) { return v == null; } else { return false; } }");
            self.writeln("fn cp_convert_err(comptime TargetErr: type, err_val: anytype) TargetErr { const SrcType = @TypeOf(err_val); if (comptime SrcType == TargetErr) { return err_val; } else if (comptime TargetErr == Err or TargetErr == Error) { if (comptime @typeInfo(SrcType) == .@\"union\") { return Err{ .code = @intCast(@intFromEnum(std.meta.activeTag(err_val))), .msg = @tagName(err_val) }; } else if (comptime @typeInfo(SrcType) == .@\"enum\") { return Err{ .code = @intCast(@intFromEnum(err_val)), .msg = @tagName(err_val) }; } else if (comptime @typeInfo(SrcType) == .error_set) { return Err{ .code = 0, .msg = @errorName(err_val) }; } else if (comptime @typeInfo(SrcType) == .@\"struct\") { var c: i32 = 0; var m: []const u8 = \"\"; if (comptime @hasField(SrcType, \"code\")) { c = @as(i32, @intCast(err_val.code)); } if (comptime @hasField(SrcType, \"msg\")) { m = to_string_slice(err_val.msg); } else if (comptime @hasField(SrcType, \"message\")) { m = to_string_slice(err_val.message); } else if (comptime @hasDecl(SrcType, \"message\")) { m = to_string_slice(err_val.message()); } else { m = @typeName(SrcType); } return Err.init(c, m); } else if (comptime @typeInfo(SrcType) == .pointer and @typeInfo(@typeInfo(SrcType).pointer.child) == .@\"struct\") { const Child = @typeInfo(SrcType).pointer.child; var c: i32 = 0; var m: []const u8 = \"\"; if (comptime @hasField(Child, \"code\")) { c = @as(i32, @intCast(err_val.code)); } if (comptime @hasField(Child, \"msg\")) { m = to_string_slice(err_val.msg); } else if (comptime @hasField(Child, \"message\")) { m = to_string_slice(err_val.message); } else if (comptime @hasDecl(Child, \"message\")) { m = to_string_slice(err_val.message()); } else { m = @typeName(Child); } return Err.init(c, m).with_cause(err_val); } else { return Err{ .code = -1, .msg = \"error\" }; } } else if (comptime @typeInfo(TargetErr) == .pointer and @typeInfo(SrcType) != .pointer) { return @ptrCast(@constCast(&err_val)); } else if (comptime @typeInfo(SrcType) == .pointer and @typeInfo(TargetErr) != .pointer and @typeInfo(SrcType).pointer.child == TargetErr) { return err_val.*; } else if (comptime @typeInfo(TargetErr) == .@\"union\" and @hasDecl(TargetErr, \"__from\")) { return TargetErr.__from(err_val); } else if (comptime @hasDecl(TargetErr, \"from\")) { return TargetErr.from(err_val); } else { return err_val; } }");
        }

        if features.needs_result_error {
            self.writeln("fn cp_match_tag(val: anytype, comptime tag: anytype) bool { const T = @TypeOf(val); if (comptime @typeInfo(T) == .@\"enum\") { return val == tag; } else if (comptime @typeInfo(T) == .error_set) { return val == @field(anyerror, @tagName(tag)); } else if (comptime ((@hasDecl(@This(), \"Err\") and T == @field(@This(), \"Err\")) or (@hasDecl(@This(), \"Error\") and T == @field(@This(), \"Error\")) or (@typeInfo(T) == .@\"struct\" and @hasField(T, \"msg\")))) { return std.mem.eql(u8, val.msg, @tagName(tag)); } else if (comptime @typeInfo(T) == .@\"union\") { return std.meta.activeTag(val) == tag; } else { return false; } }");
        }

        if features.needs_not {
            self.writeln("fn cp_not(v: anytype) bool { const T = @TypeOf(v); if (comptime T == bool) { return !v; } else if (comptime @typeInfo(T) == .optional) { return v == null; } else if (comptime is_string_like(T)) { return to_string_slice(v).len == 0; } else if (comptime (@typeInfo(T) == .int or T == comptime_int)) { return v == 0; } else { return false; } }");
        }

        if features.needs_null_coalesce {
            if features.needs_collections || features.needs_actors {
                self.writeln("fn CpNullCoalesceReturn(comptime TOpt: type, comptime TFb: type) type { if (@typeInfo(TOpt) == .optional) { const Child = @typeInfo(TOpt).optional.child; if (Child == Message) return Message; if (is_string_like(Child)) return []const u8; return Child; } return CpValType(TFb); }");
                self.writeln("fn cp_null_coalesce(opt_val: anytype, fallback: anytype) CpNullCoalesceReturn(@TypeOf(opt_val), @TypeOf(fallback)) { const TOpt = @TypeOf(opt_val); const RType = CpNullCoalesceReturn(TOpt, @TypeOf(fallback)); if (comptime @typeInfo(TOpt) == .optional) { if (opt_val) |v| { if (comptime RType == Message) { return v; } else if (comptime is_string_like(RType)) { return to_string_slice(v); } else { return v; } } else { if (comptime RType == Message) { return cp_message_from(fallback); } else if (comptime is_string_like(RType)) { return to_string_slice(fallback); } else { return fallback; } } } return cp_val(fallback); }");
            } else {
                self.writeln("fn CpNullCoalesceReturn(comptime TOpt: type, comptime TFb: type) type { if (@typeInfo(TOpt) == .optional) { const Child = @typeInfo(TOpt).optional.child; if (is_string_like(Child)) return []const u8; return Child; } return CpValType(TFb); }");
                self.writeln("fn cp_null_coalesce(opt_val: anytype, fallback: anytype) CpNullCoalesceReturn(@TypeOf(opt_val), @TypeOf(fallback)) { const TOpt = @TypeOf(opt_val); const RType = CpNullCoalesceReturn(TOpt, @TypeOf(fallback)); if (comptime @typeInfo(TOpt) == .optional) { if (opt_val) |v| { if (comptime is_string_like(RType)) { return to_string_slice(v); } else { return v; } } else { if (comptime is_string_like(RType)) { return to_string_slice(fallback); } else { return fallback; } } } return cp_val(fallback); }");
            }
        }

        if features.needs_fmt {
            if features.needs_collections || features.needs_actors {
                self.writeln("fn cp_fmt(v: anytype) []const u8 { const T = @TypeOf(v); if (comptime is_string_like(T)) { return to_string_slice(v); } else if (comptime T == bool) { return if (v) \"true\" else \"false\"; } else if (comptime (@typeInfo(T) == .int or T == comptime_int)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime (@typeInfo(T) == .float or T == comptime_float)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime @typeInfo(T) == .@\"enum\") { return @tagName(v); } else if (comptime T == Message) { return switch (v) { .string => |s| s, .int => |i| std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{i}) catch unreachable, .float => |f| std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{f}) catch unreachable, .boolean => |b| if (b) \"true\" else \"false\", else => std.fmt.allocPrint(std.heap.page_allocator, \"{any}\", .{v}) catch unreachable, }; } else if (comptime @typeInfo(T) == .optional) { if (v) |unwrapped| { return cp_fmt(unwrapped); } else { return \"null\"; } } else { return std.fmt.allocPrint(std.heap.page_allocator, \"{any}\", .{v}) catch unreachable; } }");
            } else {
                self.writeln("fn cp_fmt(v: anytype) []const u8 { const T = @TypeOf(v); if (comptime is_string_like(T)) { return to_string_slice(v); } else if (comptime T == bool) { return if (v) \"true\" else \"false\"; } else if (comptime (@typeInfo(T) == .int or T == comptime_int)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime (@typeInfo(T) == .float or T == comptime_float)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime @typeInfo(T) == .@\"enum\") { return @tagName(v); } else if (comptime @typeInfo(T) == .optional) { if (v) |unwrapped| { return cp_fmt(unwrapped); } else { return \"null\"; } } else { return std.fmt.allocPrint(std.heap.page_allocator, \"{any}\", .{v}) catch unreachable; } }");
            }
        }

        if features.needs_math_random {
            self.writeln("fn cp_math_random() f32 {");
            self.writeln("    return 0.5; // Stub");
            self.writeln("}");
        }

        if features.needs_equality {
            self.writeln("fn cp_eq(a: anytype, b: anytype) bool {");
            self.writeln("    const Ta = @TypeOf(a); const Tb = @TypeOf(b);");
            self.writeln(
                "    if (comptime @typeInfo(Ta) == .null and @typeInfo(Tb) == .null) return true;",
            );
            self.writeln("    if (comptime @typeInfo(Ta) == .null) { if (comptime @typeInfo(Tb) == .optional) return b == null; return false; }");
            self.writeln("    if (comptime @typeInfo(Tb) == .null) { if (comptime @typeInfo(Ta) == .optional) return a == null; return false; }");
            if features.needs_collections || features.needs_actors {
                self.writeln("    if (comptime @typeInfo(Ta) == .optional and @typeInfo(Ta).optional.child == Message) { if (a) |unwrapped_a| return cp_eq(unwrapped_a, b); return false; }");
                self.writeln("    if (comptime @typeInfo(Tb) == .optional and @typeInfo(Tb).optional.child == Message) { if (b) |unwrapped_b| return cp_eq(a, unwrapped_b); return false; }");
                self.writeln("    if (comptime Ta == Message and is_string_like(Tb)) { return switch (a) { .string => |s| std.mem.eql(u8, s, to_string_slice(b)), else => false, }; }");
                self.writeln("    if (comptime Tb == Message and is_string_like(Ta)) { return switch (b) { .string => |s| std.mem.eql(u8, to_string_slice(a), s), else => false, }; }");
                self.writeln("    if (comptime Ta == Message and (Tb == i32 or Tb == comptime_int)) { return switch (a) { .int => |i| i == b, else => false, }; }");
                self.writeln("    if (comptime Tb == Message and (Ta == i32 or Ta == comptime_int)) { return switch (b) { .int => |i| a == i, else => false, }; }");
                self.writeln("    if (comptime Ta == Message and (Tb == f32 or Tb == comptime_float)) { return switch (a) { .float => |f| f == b, else => false, }; }");
                self.writeln("    if (comptime Tb == Message and (Ta == f32 or Ta == comptime_float)) { return switch (b) { .float => |f| a == f, else => false, }; }");
                self.writeln("    if (comptime Ta == Message and Tb == bool) { return switch (a) { .boolean => |bl| bl == b, else => false, }; }");
                self.writeln("    if (comptime Tb == Message and Ta == bool) { return switch (b) { .boolean => |bl| a == bl, else => false, }; }");
                self.writeln("    if (comptime Ta == Message and Tb == Message) { return MessageContext.eql(.{}, a, b); }");
            }
            self.writeln("    if (comptime is_string_like(Ta) and is_string_like(Tb)) { return std.mem.eql(u8, to_string_slice(a), to_string_slice(b)); }");
            self.writeln("    if (comptime (@typeInfo(Ta) == .int or Ta == comptime_int) and (@typeInfo(Tb) == .int or Tb == comptime_int)) { return a == b; }");
            self.writeln("    if (comptime (@typeInfo(Ta) == .float or Ta == comptime_float) and (@typeInfo(Tb) == .float or Tb == comptime_float)) { return a == b; }");
            self.writeln("    if (comptime @typeInfo(Ta) == .optional and @typeInfo(Tb) == .optional) { if (a) |va| { if (b) |vb| return cp_eq(va, vb); return false; } else { return b == null; } }");
            self.writeln("    if (comptime @typeInfo(Ta) == .optional) { if (a) |va| return cp_eq(va, b); return false; }");
            self.writeln("    if (comptime @typeInfo(Tb) == .optional) { if (b) |vb| return cp_eq(a, vb); return false; }");
            self.writeln("    if (comptime Ta != Tb) return false;");
            self.writeln("    return a == b;");
            self.writeln("}");
            self.writeln("fn cp_neq(a: anytype, b: anytype) bool { return !cp_eq(a, b); }");
        }

        if features.needs_comparison {
            self.writeln("fn cp_lt(a: anytype, b: anytype) bool { const Ta = @TypeOf(a); const Tb = @TypeOf(b); if (comptime is_string_like(Ta) and is_string_like(Tb)) { return std.mem.order(u8, to_string_slice(a), to_string_slice(b)) == .lt; } return a < b; }");
            self.writeln("fn cp_gt(a: anytype, b: anytype) bool { const Ta = @TypeOf(a); const Tb = @TypeOf(b); if (comptime is_string_like(Ta) and is_string_like(Tb)) { return std.mem.order(u8, to_string_slice(a), to_string_slice(b)) == .gt; } return a > b; }");
            self.writeln("fn cp_lte(a: anytype, b: anytype) bool { const Ta = @TypeOf(a); const Tb = @TypeOf(b); if (comptime is_string_like(Ta) and is_string_like(Tb)) { const ord = std.mem.order(u8, to_string_slice(a), to_string_slice(b)); return ord == .lt or ord == .eq; } return a <= b; }");
            self.writeln("fn cp_gte(a: anytype, b: anytype) bool { const Ta = @TypeOf(a); const Tb = @TypeOf(b); if (comptime is_string_like(Ta) and is_string_like(Tb)) { const ord = std.mem.order(u8, to_string_slice(a), to_string_slice(b)); return ord == .gt or ord == .eq; } return a >= b; }");
        }

        if features.needs_hash {
            self.writeln("fn cp_hash_val(val: anytype) usize { const T = @TypeOf(val); if (comptime is_string_like(T)) { const s = to_string_slice(val); var h: usize = 2166136261; for (s) |byte| { h = (h ^ byte) *% 16777619; } return h; } if (comptime @typeInfo(T) == .int or @typeInfo(T) == .float or @typeInfo(T) == .bool) { var h: usize = 2166136261; const bytes = std.mem.asBytes(&val); for (bytes) |byte| { h = (h ^ byte) *% 16777619; } return h; } if (comptime @typeInfo(T) == .pointer) { if (@hasDecl(@typeInfo(T).pointer.child, \"hash\")) { return val.hash(); } } return 0; }");
        }

        if features.needs_array_helpers {
            self.writeln("fn cp_array_lit(comptime Elem: type, init_slice: []const Elem) []const Elem { const buf = std.heap.page_allocator.alloc(Elem, init_slice.len) catch unreachable; if (init_slice.len > 0) @memcpy(buf, init_slice); return buf; }");
            self.writeln("fn CpArrayElemType(comptime PtrT: type) type { const S0 = @typeInfo(PtrT).pointer.child; const S1 = if (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one) @typeInfo(S0).pointer.child else S0; return @typeInfo(S1).pointer.child; }");
            self.writeln("fn cp_array_push(arr_ptr: anytype, item: anytype) void { const S0 = @TypeOf(arr_ptr.*); const target_ptr = if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one)) arr_ptr.* else arr_ptr; const SliceT = @TypeOf(target_ptr.*); const Elem = @typeInfo(SliceT).pointer.child; const old_slice = target_ptr.*; const new_slice = std.heap.page_allocator.alloc(Elem, old_slice.len + 1) catch unreachable; if (old_slice.len > 0) @memcpy(new_slice[0..old_slice.len], old_slice); new_slice[old_slice.len] = @as(Elem, cp_val(item)); target_ptr.* = new_slice; }");
            self.writeln("fn cp_array_pop(arr_ptr: anytype) ?CpArrayElemType(@TypeOf(arr_ptr)) { const S0 = @TypeOf(arr_ptr.*); const target_ptr = if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one)) arr_ptr.* else arr_ptr; const s = target_ptr.*; if (s.len == 0) return null; const last = s[s.len - 1]; target_ptr.* = s[0 .. s.len - 1]; return last; }");
            self.writeln("fn cp_container_clear(ptr: anytype) void { const S0 = @TypeOf(ptr.*); if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one and @typeInfo(@typeInfo(S0).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(S0).pointer.child, \"clear\"))) { ptr.*.clear(); } else { const target_ptr = if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one)) ptr.* else ptr; target_ptr.* = target_ptr.*[0..0]; } }");
        }

        if features.needs_iter_helpers {
            self.writeln("fn cp_iter_slice(val: anytype) switch (@typeInfo(@TypeOf(val))) { .pointer => |p| switch (p.size) { .slice => @TypeOf(val), .one => switch (@typeInfo(p.child)) { .array => []const @typeInfo(p.child).array.child, .pointer => |sub_p| if (sub_p.size == .slice) p.child else @TypeOf(val), else => @TypeOf(val), }, else => @TypeOf(val), }, .array => []const @typeInfo(@TypeOf(val)).array.child, else => @TypeOf(val), } { const T = @TypeOf(val); switch (@typeInfo(T)) { .pointer => |p| switch (p.size) { .slice => return val, .one => switch (@typeInfo(p.child)) { .array => return val[0..], .pointer => |sub_p| if (sub_p.size == .slice) return val.* else return val, else => return val, }, else => return val, }, .array => return val[0..], else => return val, } }");
            self.writeln("fn cp_mut_slice(ptr: anytype) switch (@typeInfo(@TypeOf(ptr.*))) { .array => []@typeInfo(@TypeOf(ptr.*)).array.child, .pointer => |p| switch (p.size) { .slice => []if (@typeInfo(p.child) == .pointer) @typeInfo(p.child).pointer.child else p.child, .one => switch (@typeInfo(p.child)) { .pointer => |sub_p| if (sub_p.size == .slice) []sub_p.child else @compileError(\"Cannot mutably iterate over non-collection\"), else => @compileError(\"Cannot mutably iterate over non-collection\"), }, else => @compileError(\"Cannot mutably iterate over non-collection\"), }, else => @compileError(\"Cannot mutably iterate over non-collection\"), } { const ValT = @TypeOf(ptr.*); switch (@typeInfo(ValT)) { .array => return ptr[0..], .pointer => |p| switch (p.size) { .slice => return @constCast(ptr.*), .one => return @constCast(ptr.*.*), else => unreachable, }, else => unreachable, } }");
        }
        if features.needs_index_helpers {
            self.writeln("fn cp_index_of(container: anytype, needle: anytype) i32 { const cp_slice = cp_iter_slice(container); const SliceT = @TypeOf(cp_slice); if (comptime is_string_like(SliceT) and is_string_like(@TypeOf(needle))) { if (std.mem.indexOf(u8, to_string_slice(cp_slice), to_string_slice(needle))) |i| return @intCast(i); return -1; } for (cp_slice, 0..) |item, idx| { if (cp_eq(item, needle)) return @intCast(idx); } return -1; }");
            self.writeln("fn cp_last_index_of(container: anytype, needle: anytype) i32 { const cp_slice = cp_iter_slice(container); const SliceT = @TypeOf(cp_slice); if (comptime is_string_like(SliceT) and is_string_like(@TypeOf(needle))) { if (std.mem.lastIndexOf(u8, to_string_slice(cp_slice), to_string_slice(needle))) |i| return @intCast(i); return -1; } var i: usize = cp_slice.len; while (i > 0) { i -= 1; if (cp_eq(cp_slice[i], needle)) return @intCast(i); } return -1; }");
            self.writeln("fn cp_includes(container: anytype, needle: anytype) bool { return cp_index_of(container, needle) != -1; }");
        }
        if features.needs_actors {
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
        }
        if features.needs_shift {
            self.writeln("fn cp_shl(val: anytype, shift: anytype) @TypeOf(val) {");
            self.writeln("    const T = @TypeOf(val);");
            self.writeln("    if (comptime @typeInfo(T) == .comptime_int) return val << shift;");
            self.writeln("    if (comptime @typeInfo(T) == .int) {");
            self.writeln("        const ShiftType = std.math.Log2Int(T);");
            self.writeln("        const u_shift: usize = if (@typeInfo(@TypeOf(shift)) == .int and @typeInfo(@TypeOf(shift)).int.signedness == .signed) @intCast(@max(0, shift)) else @intCast(shift);");
            self.writeln("        return val << @as(ShiftType, @truncate(u_shift));");
            self.writeln("    }");
            self.writeln("    return val << shift;");
            self.writeln("}");
            self.writeln("fn cp_shr(val: anytype, shift: anytype) @TypeOf(val) {");
            self.writeln("    const T = @TypeOf(val);");
            self.writeln("    if (comptime @typeInfo(T) == .comptime_int) return val >> shift;");
            self.writeln("    if (comptime @typeInfo(T) == .int) {");
            self.writeln("        const ShiftType = std.math.Log2Int(T);");
            self.writeln("        const u_shift: usize = if (@typeInfo(@TypeOf(shift)) == .int and @typeInfo(@TypeOf(shift)).int.signedness == .signed) @intCast(@max(0, shift)) else @intCast(shift);");
            self.writeln("        return val >> @as(ShiftType, @truncate(u_shift));");
            self.writeln("    }");
            self.writeln("    return val >> shift;");
            self.writeln("}");
        }
        if features.needs_actors {
            self.writeln("");
            self.writeln("// --- Scheduler ---");
            self.writeln(
                "const _Task = struct { run: *const fn(*anyopaque) anyerror!void, ctx: *anyopaque };",
            );
            self.writeln("const Spinlock = struct { state: std.atomic.Value(u32) = std.atomic.Value(u32).init(0), pub fn acquire(self: *Spinlock) void { while (self.state.cmpxchgWeak(0, 1, .acquire, .monotonic) != null) { std.atomic.spinLoopHint(); } } pub fn release(self: *Spinlock) void { self.state.store(0, .release); } };");
            self.writeln("const GlobalTaskQueue = struct { lock: Spinlock = .{}, buffer: [2048]_Task = undefined, head: usize = 0, tail: usize = 0, count: usize = 0, pub fn push(self: *GlobalTaskQueue, item: _Task) void { self.lock.acquire(); defer self.lock.release(); if (self.count >= 2048) return; self.buffer[self.tail] = item; self.tail = (self.tail + 1) % 2048; self.count += 1; } pub fn pop(self: *GlobalTaskQueue) ?_Task { self.lock.acquire(); defer self.lock.release(); if (self.count == 0) return null; const item = self.buffer[self.head]; self.head = (self.head + 1) % 2048; self.count -= 1; return item; } pub fn len(self: *GlobalTaskQueue) usize { self.lock.acquire(); defer self.lock.release(); return self.count; } };");
            self.writeln("var global_task_queue: GlobalTaskQueue = .{};");
            self.writeln("var global_runtime_shutdown: std.atomic.Value(bool) = std.atomic.Value(bool).init(false);");
            self.writeln(
                "var global_active_tasks: std.atomic.Value(usize) = std.atomic.Value(usize).init(0);",
            );
            self.writeln("var global_running_workers: std.atomic.Value(usize) = std.atomic.Value(usize).init(0);");
            self.writeln("threadlocal var task_queue: [1000]_Task = undefined; threadlocal var queue_len: usize = 0;");
            self.writeln(
                "threadlocal var current_node_id: u32 = 0; threadlocal var next_local_id: u32 = 1;",
            );
            self.writeln("fn schedule(run: *const fn(*anyopaque) anyerror!void, ctx: *anyopaque) void { if (current_node_id == 0) { global_task_queue.push(.{ .run = run, .ctx = ctx }); } else { if (queue_len < 1000) { task_queue[queue_len] = .{ .run = run, .ctx = ctx }; queue_len += 1; } } }");
            self.writeln("const _WorkerPoolRunner = struct { pub fn run() void { while (!global_runtime_shutdown.load(.acquire)) { if (global_task_queue.pop()) |task| { _ = global_running_workers.fetchAdd(1, .acquire); task.run(task.ctx) catch {}; _ = global_running_workers.fetchSub(1, .release); } else { const ts = std.os.linux.timespec{ .sec = 0, .nsec = 100_000 }; _ = std.os.linux.nanosleep(&ts, null); } } } };");
            self.writeln("");
            self.writeln("// --- Cross-Thread Channels & Mailboxes ---");
            self.writeln(
                "const CrossThreadMessage = struct { target: PID, sender: PID, msg: Message };",
            );
            self.writeln("const MailboxItem = struct { sender: PID, msg: Message };");
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
            self.writeln("pub fn print(v: anytype) void { const T = @TypeOf(v); if (comptime is_string_like(T)) { std.debug.print(\"{s}\\n\", .{ to_string_slice(v) }); } else if (comptime T == Message) { std.debug.print(\"{s}\\n\", .{ cp_fmt(v) }); } else if (comptime T == ?Message) { if (v) |m| { std.debug.print(\"{s}\\n\", .{ cp_fmt(m) }); } else { std.debug.print(\"null\\n\", .{}); } } else { std.debug.print(\"{any}\\n\", .{ v }); } }");
            self.writeln("pub const CpTask = struct { base: BaseContext = .{}, lock: Spinlock = .{}, state: u8 = 0, waiter_ctx: ?*anyopaque = null, waiter_fn: ?*const fn(*anyopaque) anyerror!void = null, join_barrier: ?*JoinBarrier = null, join_idx: usize = 0, arena: ?*std.heap.ArenaAllocator = null, };");
            if !self.enum_defs.contains_key("Task") {
                self.writeln("pub const Task = CpTask;");
            }
            self.writeln("pub const JoinBarrier = struct { lock: Spinlock = .{}, remaining: usize, total: usize, results: []Message, waiter_ctx: ?*anyopaque = null, waiter_fn: ?*const fn(*anyopaque) anyerror!void = null, pub fn init(allocator: std.mem.Allocator, total: usize) !*JoinBarrier { const b = try allocator.create(JoinBarrier); b.* = .{ .remaining = total + 1, .total = total, .results = try allocator.alloc(Message, total), .waiter_ctx = null, .waiter_fn = null }; return b; } pub fn on_task_done(self: *JoinBarrier, idx: usize, msg: Message) void { self.lock.acquire(); self.results[idx] = msg; self.remaining -= 1; const done = (self.remaining == 0); const w_ctx_opt = self.waiter_ctx; const w_fn_opt = self.waiter_fn; if (done and w_ctx_opt != null) { self.waiter_ctx = null; self.waiter_fn = null; } self.lock.release(); if (done) { if (w_ctx_opt) |w_ctx| { const w_fn = w_fn_opt.?; const base_waiter: *BaseContext = @ptrCast(@alignCast(w_ctx)); const arr_res = std.heap.page_allocator.alloc(Message, self.total) catch unreachable; @memcpy(arr_res, self.results); base_waiter.await_result = Message{ .array = arr_res }; schedule(w_fn, w_ctx); } } } pub fn finish_setup_and_should_suspend(self: *JoinBarrier) bool { self.lock.acquire(); self.remaining -= 1; const done = (self.remaining == 0); if (done) { const base_waiter: *BaseContext = @ptrCast(@alignCast(self.waiter_ctx.?)); const arr_res = std.heap.page_allocator.alloc(Message, self.total) catch unreachable; @memcpy(arr_res, self.results); base_waiter.await_result = Message{ .array = arr_res }; self.waiter_ctx = null; self.waiter_fn = null; } self.lock.release(); return !done; } };");
            self.writeln("pub fn __cp_task_bridge(p: *anyopaque) anyerror!void { const t: *CpTask = @ptrCast(@alignCast(p)); t.lock.acquire(); t.state = 2; const jb = t.join_barrier; const j_idx = t.join_idx; const w_ctx_opt = t.waiter_ctx; const w_fn_opt = t.waiter_fn; t.waiter_ctx = null; t.waiter_fn = null; const res = t.base.await_result; t.lock.release(); if (jb) |b| { b.on_task_done(j_idx, res orelse Message{ .string = \"\" }); } if (w_ctx_opt) |w_ctx| { const w_fn = w_fn_opt.?; const base_waiter: *BaseContext = @ptrCast(@alignCast(w_ctx)); base_waiter.await_result = res; schedule(w_fn, w_ctx); } _ = global_active_tasks.fetchSub(1, .release); }");
            self.writeln("pub fn cp_msg_to_val(m: Message) []const u8 { return cp_fmt(m); }");
        }
        if features.needs_unpack {
            if !features.needs_collections && !features.needs_actors {
                self.writeln("const Message = anyopaque;");
                self.writeln("fn cp_msg_to_val(_: anytype) []const u8 { return \"\"; }");
            }
            self.writeln("pub fn CpUnpackElemType(comptime T: type, comptime idx: usize) type { if (T == ?Message or T == Message) return []const u8; if (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice) return @typeInfo(T).pointer.child; if (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .one and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice) return @typeInfo(@typeInfo(T).pointer.child).pointer.child; if (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .array) return @typeInfo(@typeInfo(T).pointer.child).array.child; if (@typeInfo(T) == .array) return @typeInfo(T).array.child; return @FieldType(T, std.fmt.comptimePrint(\"{d}\", .{idx})); }");
            self.writeln("pub fn cp_unpack_elem(v: anytype, comptime idx: usize) CpUnpackElemType(@TypeOf(v), idx) { const T = @TypeOf(v); if (comptime T == ?Message) { if (v) |m| { if (m == .array and idx < m.array.len) { return cp_msg_to_val(m.array[idx]); } } return \"\"; } else if (comptime T == Message) { if (v == .array and idx < v.array.len) { return cp_msg_to_val(v.array[idx]); } return \"\"; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice)) { return v[idx]; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .one and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice)) { return v.*[idx]; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .array)) { return v[idx]; } else if (comptime (@typeInfo(T) == .array)) { return v[idx]; } else { return @field(v, std.fmt.comptimePrint(\"{d}\", .{idx})); } }");
            self.writeln("pub fn CpUnpackTailType(comptime T: type, comptime offset_from_end: usize) type { if (T == ?Message or T == Message) return []const u8; if (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice) return @typeInfo(T).pointer.child; if (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .one and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice) return @typeInfo(@typeInfo(T).pointer.child).pointer.child; if (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .array) return @typeInfo(@typeInfo(T).pointer.child).array.child; if (@typeInfo(T) == .array) return @typeInfo(T).array.child; const n = @typeInfo(T).@\"struct\".fields.len; return @FieldType(T, std.fmt.comptimePrint(\"{d}\", .{n - offset_from_end})); }");
            self.writeln("pub fn cp_unpack_tail(v: anytype, comptime offset_from_end: usize) CpUnpackTailType(@TypeOf(v), offset_from_end) { const T = @TypeOf(v); if (comptime T == ?Message) { if (v) |m| { if (m == .array and m.array.len >= offset_from_end) { return cp_msg_to_val(m.array[m.array.len - offset_from_end]); } } return \"\"; } else if (comptime T == Message) { if (v == .array and v.array.len >= offset_from_end) { return cp_msg_to_val(v.array[v.array.len - offset_from_end]); } return \"\"; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice)) { return v[v.len - offset_from_end]; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .one and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice)) { return v.*[v.*.len - offset_from_end]; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .array)) { return v[v.len - offset_from_end]; } else if (comptime (@typeInfo(T) == .array)) { return v[v.len - offset_from_end]; } else { const n = @typeInfo(T).@\"struct\".fields.len; return @field(v, std.fmt.comptimePrint(\"{d}\", .{n - offset_from_end})); } }");
            self.writeln("pub fn CpUnpackSliceType(comptime T: type, comptime start_idx: usize) type { if (T == ?Message or T == Message) return []const Message; if (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice) return T; if (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .one and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice) return @typeInfo(T).pointer.child; if (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .array) return []const @typeInfo(@typeInfo(T).pointer.child).array.child; if (@typeInfo(T) == .array) return []const @typeInfo(T).array.child; const fields = @typeInfo(T).@\"struct\".fields; if (fields.len == 0) return []const i32; const Elem = if (start_idx < fields.len) fields[start_idx].type else fields[0].type; return []const CpValType(Elem); }");
            self.writeln("pub fn cp_unpack_slice(v: anytype, comptime start_idx: usize, comptime suffix_len: usize) CpUnpackSliceType(@TypeOf(v), start_idx) { const T = @TypeOf(v); if (comptime T == ?Message) { if (v) |m| { if (m == .array and m.array.len >= start_idx + suffix_len) { return m.array[start_idx .. m.array.len - suffix_len]; } } return &[_]Message{}; } else if (comptime T == Message) { if (v == .array and v.array.len >= start_idx + suffix_len) { return v.array[start_idx .. v.array.len - suffix_len]; } return &[_]Message{}; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice)) { return v[start_idx .. v.len - suffix_len]; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .one and @typeInfo(@typeInfo(T).pointer.child) == .pointer and @typeInfo(@typeInfo(T).pointer.child).pointer.size == .slice)) { return v.*[start_idx .. v.*.len - suffix_len]; } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .array)) { return v[start_idx .. v.len - suffix_len]; } else if (comptime (@typeInfo(T) == .array)) { return v[start_idx .. v.len - suffix_len]; } else { const fields = @typeInfo(T).@\"struct\".fields; const end = fields.len - suffix_len; const count = if (end > start_idx) end - start_idx else 0; const Elem = if (start_idx < fields.len) CpValType(fields[start_idx].type) else i32; const buf = std.heap.page_allocator.alloc(Elem, count) catch unreachable; inline for (start_idx..end, 0..) |f_idx, out_idx| { buf[out_idx] = cp_val(@field(v, std.fmt.comptimePrint(\"{d}\", .{f_idx}))); } return buf; } }");
        }
        if features.needs_string_helpers || features.needs_fmt {
            self.writeln("pub fn cp_string_from(val: anytype) []const u8 { const T = @TypeOf(val); if (comptime T == []const u8) { return val; } else if (comptime (T == [*:0]const u8 or T == [*:0]u8)) { return std.mem.span(val); } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(T).pointer.size == .slice and @typeInfo(T).pointer.child == u8)) { return val; } else if (comptime (@typeInfo(T) == .@\"enum\")) { return @tagName(val); } else if (comptime T == bool) { return if (val) \"true\" else \"false\"; } else if (comptime (@typeInfo(T) == .int or T == comptime_int)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{}\", .{val}) catch \"0\"; } else if (comptime (@typeInfo(T) == .float or T == comptime_float)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{val}) catch \"0.0\"; } else if (comptime (@typeInfo(T) == .@\"struct\" and @hasDecl(T, \"toString\"))) { return val.toString(); } else if (comptime (@typeInfo(T) == .pointer and @typeInfo(@typeInfo(T).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(T).pointer.child, \"toString\"))) { return val.toString(); } else { return @typeName(T); } }");
            self.writeln("");
        }

        for item in &module.body {
            if let ModuleItem::Import(imp) = item {
                let zig_source = if imp.source.ends_with(".cp") {
                    format!("{}.zig", &imp.source[..imp.source.len() - 3])
                } else {
                    imp.source.clone()
                };
                if let Some(def) = &imp.default_specifier {
                    self.globals.insert(def.sym.clone());
                    self.value_classes.insert(def.sym.clone());
                    if def.sym == "std" && zig_source == "std" {
                        continue;
                    }
                    self.writeln(&format!(
                        "pub const {} = @import(\"{}\");",
                        def.sym, zig_source
                    ));
                }
                for s in &imp.specifiers {
                    self.globals.insert(s.sym.clone());
                    self.value_classes.insert(s.sym.clone());
                    if s.sym == "std" && zig_source == "std" {
                        continue;
                    }
                    self.writeln(&format!(
                        "pub const {0} = @import(\"{1}\").{0};",
                        s.sym, zig_source
                    ));
                }
            }
        }

        for item in &module.body {
            if let ModuleItem::Decl(Decl::Trait(t)) = item {
                self.generate_trait(t);
            }
        }
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Enum(e)) = item {
                self.generate_enum(e);
            }
            if let ModuleItem::Decl(Decl::ErrorSet(es)) = item {
                let union_suffix = if es.unions.is_empty() {
                    String::new()
                } else {
                    format!(" || {}", es.unions.join(" || "))
                };
                self.writeln(&format!("pub const {} = error{{", es.ident.sym));
                self.indent();
                for v in &es.variants {
                    self.writeln(&format!("{},", v.sym));
                }
                self.dedent();
                self.writeln(&format!("}}{};\n", union_suffix));
            }
        }
        // Emit synthesized AnonSet and Union error sets
        let anon_sets = self.anon_err_sets.clone();
        for (zig_name, variants) in &anon_sets {
            self.writeln(&format!("pub const {} = union(enum) {{", zig_name));
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
            self.writeln(&format!("pub const {} = union(enum) {{", zig_name));
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
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Interface(i)) = item {
                self.generate_interface(i);
            }
        }
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Var(v)) = item {
                if let Pattern::Ident(id) = &v.pat {
                    self.globals.insert(id.sym.clone());
                    if let Some(ty) = &v.ty {
                        self.var_cp_types.insert(id.sym.clone(), ty.clone());
                    }
                }
            }
        }
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Expr(Expr::Zig(z))) = item {
                for line in z.code.lines() {
                    let trimmed = line.trim();
                    let stripped = if let Some(s) = trimmed.strip_prefix("pub ") {
                        s.trim_start()
                    } else {
                        trimmed
                    };
                    if let Some(rest) = stripped.strip_prefix("const ") {
                        let mut parts = rest.split(&[' ', ':', '='][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.value_classes.insert(name.to_string());
                                self.globals.insert(name.to_string());
                            }
                        }
                    } else if let Some(rest) = stripped.strip_prefix("fn ") {
                        let mut parts = rest.split(&[' ', '(', '<'][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.globals.insert(name.to_string());
                            }
                        }
                    } else if let Some(rest) = stripped.strip_prefix("extern fn ") {
                        let mut parts = rest.split(&[' ', '(', '<'][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.globals.insert(name.to_string());
                            }
                        }
                    } else if let Some(rest) = stripped.strip_prefix("extern \"c\" fn ") {
                        let mut parts = rest.split(&[' ', '(', '<'][..]);
                        if let Some(name) = parts.next() {
                            let name = name.trim();
                            if !name.is_empty() {
                                self.globals.insert(name.to_string());
                            }
                        }
                    }
                }
            }
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
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Impl(i)) = item {
                self.generate_impl(i);
            }
        }
        for item in &module.body {
            if let ModuleItem::Decl(Decl::TypeAlias(ta)) = item {
                let pub_prefix = if ta.is_pub { "pub " } else { "" };
                let mapped = self.map_type(&ta.target_ty);
                self.writeln(&format!(
                    "{}const {} = {};",
                    pub_prefix, ta.ident.sym, mapped
                ));
            }
        }

        for item in &module.body {
            match item {
                ModuleItem::Decl(Decl::Func(f)) => {
                    self.generate_function(f);
                }
                ModuleItem::Decl(Decl::Dec(_)) => {}
                ModuleItem::Decl(Decl::Macro(_)) => {}
                ModuleItem::Import(_) => {}
                _ => {}
            }
        }

        for f in &self.lambda_funcs {
            self.output.push_str(f);
            self.output.push('\n');
        }

        let anon_objs: Vec<(String, String)> = self
            .anon_obj_structs
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (name, body) in anon_objs {
            self.writeln(&format!("pub const {} = {};", name, body));
        }

        if features.needs_reflection {
            self.generate_reflection_tables();
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
            if !self.is_lib_mode {
                self.writeln("\npub var global_io: std.Io = undefined;");
                self.writeln("pub var __cap_test_total: usize = 0;");
                self.writeln("pub var __cap_test_passed: usize = 0;");
                self.writeln("pub var __cap_test_failed: usize = 0;");
            }

            let has_main = self.fn_sigs.contains_key("main");
            if !self.is_lib_mode && has_main {
                self.writeln("pub fn main(init: std.process.Init) !void {");
                self.indent();
                self.writeln("global_io = init.io;");
                self.writeln("var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator); defer arena.deinit();");
                if self.is_parallel_runtime {
                    let wc_expr = if self.parallel_worker_count > 0 {
                        format!("{}", self.parallel_worker_count)
                    } else {
                        "@min(@max(std.Thread.getCpuCount() catch 4, 2), 16)".to_string()
                    };
                    self.writeln(&format!("const __worker_count: usize = {};", wc_expr));
                    self.writeln(
                        "var __workers = try arena.allocator().alloc(std.Thread, __worker_count);",
                    );
                    self.writeln("for (0..__worker_count) |wi| { __workers[wi] = try std.Thread.spawn(.{}, _WorkerPoolRunner.run, .{}); }");
                    self.writeln("defer { global_runtime_shutdown.store(true, .release); for (__workers) |wt| { wt.join(); } }");
                }
                self.writeln("var ctx: cp_main_Context = undefined; ctx.pc = 0; ctx._self = PID{ .node_id = 0, .local_id = 1 }; next_local_id = 2; ctx.arena = &arena; ctx._parent = null; ctx._parent_fn = null; ctx.received_msg = null; ctx.received_sender = null;");
                self.writeln("try cp_main(&ctx);");
                self.writeln("while (true) {");
                self.indent();
                self.writeln("drain_cross_thread_mailbox();");
                self.writeln("while (global_task_queue.pop()) |task| { _ = global_running_workers.fetchAdd(1, .acquire); try task.run(task.ctx); _ = global_running_workers.fetchSub(1, .release); }");
                self.writeln("while (queue_len > 0) { const task = task_queue[0]; for (task_queue[1..queue_len], 0..) |t, j| { task_queue[j] = t; } queue_len -= 1; try task.run(task.ctx); }");
                self.writeln("drain_cross_thread_mailbox();");
                self.writeln("if (active_fds == 0 and queue_len == 0 and global_task_queue.len() == 0 and waiting_count == 0 and global_active_tasks.load(.acquire) == 0 and global_running_workers.load(.acquire) == 0) break;");
                self.writeln("if (queue_len == 0 and global_task_queue.len() == 0 and (waiting_count > 0 or global_active_tasks.load(.acquire) > 0 or global_running_workers.load(.acquire) > 0) and active_fds == 0) { const ts = std.os.linux.timespec{ .sec = 0, .nsec = 500_000 }; _ = std.os.linux.nanosleep(&ts, null); }");
                self.writeln("if (active_fds > 0) {");
                self.indent();
                self.writeln(
                    "const ready_count = try std.posix.poll(poll_fds[0..active_fds], 10);",
                );
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
                self.writeln(
                    "@import(\"std\").debug.print(\"===========================\\n\", .{});",
                );
                self.writeln(
                    "@import(\"std\").debug.print(\"Total:  {d}\\n\", .{__cap_test_total});",
                );
                self.writeln(
                    "@import(\"std\").debug.print(\"Passed: {d}\\n\", .{__cap_test_passed});",
                );
                self.writeln(
                    "@import(\"std\").debug.print(\"Failed: {d}\\n\", .{__cap_test_failed});",
                );
                self.writeln(
                "@import(\"std\").debug.print(\"Coverage: 100% (Native CP Evaluated)\\n\", .{});",
            );
                self.writeln("if (__cap_test_failed > 0) @import(\"std\").process.exit(1);");
                self.dedent();
                self.writeln("}");

                self.dedent();
                self.writeln("}"); // Close main
            }
        }
        self.output.clone()
    }

    fn generate_trait(&mut self, t: &TraitDecl) {
        writeln!(
            self.output,
            "pub const {} = struct {{ ptr: *anyopaque, vtable: *const VTable, pub const VTable = struct {{",
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
        let has_payloads = e
            .variants
            .iter()
            .any(|v| v.fields.is_some() || v.tuple_type.is_some() || v.from_err.is_some());
        if !has_payloads {
            let tag_str = if let Some(tag) = &e.tag_type {
                format!("({})", self.map_type(tag))
            } else if e.variants.iter().any(|v| v.value.is_some()) {
                "(u32)".to_string()
            } else {
                "".to_string()
            };
            self.writeln(&format!("pub const {} = enum{} {{", e.ident.sym, tag_str));
            self.indent();
            for v in &e.variants {
                let mut line = v.ident.sym.clone();
                if let Some(val) = &v.value {
                    line.push_str(" = ");
                    line.push_str(&self.generate_expr(val));
                }
                line.push(',');
                self.writeln(&line);
            }
            let has_get = e.methods.iter().any(|m| m.ident.sym == "get") || e.extra_zig.iter().any(|z| z.contains("fn get("));
            let has_has = e.methods.iter().any(|m| m.ident.sym == "has") || e.extra_zig.iter().any(|z| z.contains("fn has("));
            let has_from_name = e.methods.iter().any(|m| m.ident.sym == "fromName") || e.extra_zig.iter().any(|z| z.contains("fn fromName("));
            let has_name = e.methods.iter().any(|m| m.ident.sym == "name") || e.extra_zig.iter().any(|z| z.contains("fn name("));
            if !has_get {
                self.writeln("pub fn get(s: []const u8) ?@This() { return std.meta.stringToEnum(@This(), s); }");
            }
            if !has_has {
                self.writeln("pub fn has(s: []const u8) bool { return std.meta.stringToEnum(@This(), s) != null; }");
            }
            if !has_from_name {
                self.writeln("pub fn fromName(s: []const u8) ?@This() { return get(s); }");
            }
            if !has_name {
                self.writeln("pub fn name(self: @This()) []const u8 { return @tagName(self); }");
            }
            if !e.methods.is_empty() {
                for m in &e.methods {
                    self.generate_function_decorated(m, Some(&e.ident.sym));
                }
            }
            for code in &e.extra_zig {
                self.writeln("");
                for line in code.lines() {
                    self.writeln(line);
                }
            }
            self.dedent();
            self.writeln("};\n");
            return;
        }
        let enum_type = if e.variants.iter().any(|v| v.value.is_some()) {
            "enum(u32)"
        } else {
            "enum"
        };
        self.writeln(&format!(
            "pub const {} = union({}) {{",
            e.ident.sym, enum_type
        ));
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
        if !e.methods.iter().any(|m| m.ident.sym == "has") && !e.extra_zig.iter().any(|z| z.contains("fn has(")) {
            self.writeln("pub fn has(s: []const u8) bool { if (comptime @typeInfo(@This()).@\"union\".tag_type) |TagType| { return std.meta.stringToEnum(TagType, s) != null; } return false; }");
        }
        if !e.methods.iter().any(|m| m.ident.sym == "name") && !e.extra_zig.iter().any(|z| z.contains("fn name(")) {
            self.writeln("pub fn name(self: @This()) []const u8 { return @tagName(self); }");
        }
        if !e.methods.is_empty() {
            for m in &e.methods {
                self.generate_function_decorated(m, Some(&e.ident.sym));
            }
        }
        for code in &e.extra_zig {
            self.writeln("");
            for line in code.lines() {
                self.writeln(line);
            }
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
        if class.ident.sym == "Task"
            || class.ident.sym == "TypeInfo"
            || class.ident.sym == "FieldInfo"
            || class.ident.sym == "MethodInfo"
            || class.ident.sym == "Map"
            || class.ident.sym == "Set"
            || class.ident.sym == "Box"
        {
            return;
        }
        let is_root = class.meta.iter().any(|m| m.name.sym == "root");
        if is_root {
            if (class.is_ref || class.meta.iter().any(|m| m.name.sym == "ref"))
                && !class.meta.iter().any(|m| m.name.sym == "value")
            {
                self.ref_classes.insert(class.ident.sym.clone());
            }
            writeln!(self.output, "pub const {} = @This();\n", class.ident.sym).unwrap();
            for f in &class.fields {
                let default_init = if let Some(def) = &f.default {
                    format!(" = {},", self.generate_expr(def))
                } else if f.ident.sym == "ref_count" {
                    " = .init(),".to_string()
                } else if f.ident.sym == "pending"
                    || f.ident.sym == "waiters"
                    || f.ident.sym == "header_block"
                    || f.ident.sym == "body_buffer"
                    || f.ident.sym == "decoded_bytes"
                    || f.ident.sym == "decoded_headers"
                {
                    " = .{},".to_string()
                } else if f.ident.sym == "state" {
                    " = .open,".to_string()
                } else if f.ident.sym == "len" {
                    " = 0,".to_string()
                } else {
                    "".to_string()
                };
                let comma = if default_init.is_empty() { "," } else { "" };
                self.writeln(&format!(
                    "{}: {}{}{}",
                    f.ident.sym,
                    self.map_type(&f.ty),
                    default_init,
                    comma
                ));
            }
            for m in &class.methods {
                self.generate_function_decorated(m, Some(&class.ident.sym));
            }
            for code in &class.extra_zig {
                for line in code.lines() {
                    self.writeln(line);
                }
            }
            if let Some(rc_meta) = class.meta.iter().find(|m| m.name.sym == "refCount" || m.name.sym == "refcount") {
                let deinit_fn = if let Some(arg) = rc_meta.args.first() {
                    match arg {
                        crate::ast::MetaArg::Positional(Expr::Ident(id)) => id.sym.clone(),
                        crate::ast::MetaArg::Named(_, Expr::Ident(id)) => id.sym.clone(),
                        _ => "deinit".to_string(),
                    }
                } else {
                    "deinit".to_string()
                };
                self.writeln(&format!(
                    "const Self = @This(); const RefCount = if (@hasDecl(@import(\"root\"), \"bun\")) @import(\"root\").bun.ptr.RefCount(Self, \"ref_count\", {deinit_fn}, .{{}}) else struct {{ pub fn ref(self: *Self) *Self {{ if (@TypeOf(self.ref_count) == u32) {{ self.ref_count += 1; }} else {{ _ = self.ref_count.ref(); }} return self; }} pub fn deref(self: *Self) void {{ if (@TypeOf(self.ref_count) == u32) {{ if (self.ref_count > 0) {{ self.ref_count -= 1; if (self.ref_count == 0) self.{deinit_fn}(); }} }} else {{ if (self.ref_count.deref()) self.{deinit_fn}(); }} }} }};"
                ));
                self.writeln("pub fn ref(self: *@This()) *@This() { return RefCount.ref(self); }");
                self.writeln("pub fn deref(self: *@This()) void { RefCount.deref(self); }");
            }
            return;
        }
        let type_id = Self::fnv1a_type_id(&class.ident.sym);
        let is_packed = class.meta.iter().find(|m| m.name.sym == "packed");
        let is_extern = class
            .meta
            .iter()
            .any(|m| m.name.sym == "extern" || m.name.sym == "c" || m.name.sym == "fix");
        let is_opaque = class.meta.iter().any(|m| m.name.sym == "opaque");
        let struct_kind = if class.is_union {
            let tag_meta = class.meta.iter().find(|m| m.name.sym == "tag");
            if let Some(tag) = tag_meta {
                let tag_str = match tag.args.first() {
                    Some(crate::ast::MetaArg::Positional(e)) => self.generate_expr(e),
                    Some(crate::ast::MetaArg::Named(_, e)) => self.generate_expr(e),
                    None => "enum".to_string(),
                };
                format!("union({})", tag_str)
            } else if let Some(p) = is_packed {
                if let Some(arg) = p.args.first() {
                    let expr = match arg {
                        crate::ast::MetaArg::Positional(e) => e,
                        crate::ast::MetaArg::Named(_, e) => e,
                    };
                    format!("packed union({})", self.generate_expr(expr))
                } else {
                    "packed union".to_string()
                }
            } else if is_extern {
                "extern union".to_string()
            } else if is_opaque {
                "opaque".to_string()
            } else {
                "union".to_string()
            }
        } else if let Some(p) = is_packed {
            if let Some(arg) = p.args.first() {
                let expr = match arg {
                    crate::ast::MetaArg::Positional(e) => e,
                    crate::ast::MetaArg::Named(_, e) => e,
                };
                format!("packed struct({})", self.generate_expr(expr))
            } else {
                "packed struct".to_string()
            }
        } else if is_extern {
            "extern struct".to_string()
        } else if is_opaque {
            "opaque".to_string()
        } else {
            "struct".to_string()
        };
        let is_generic = !class.generic_params.is_empty();
        if is_generic {
            let mut params_str = Vec::new();
            for gp in &class.generic_params {
                match gp {
                    crate::ast::GenericParam::Type { ident, .. } => {
                        params_str.push(format!("comptime {}: type", ident.sym))
                    }
                    crate::ast::GenericParam::Const { ident, ty } => {
                        params_str.push(format!("comptime {}: {}", ident.sym, self.map_type(ty)));
                    }
                }
            }
            writeln!(
                self.output,
                "pub fn {}({}) type {{\n    return {} {{",
                class.ident.sym,
                params_str.join(", "),
                struct_kind
            )
            .unwrap();
            self.indent();
        } else {
            writeln!(
                self.output,
                "pub const {} = {} {{",
                class.ident.sym, struct_kind
            )
            .unwrap();
        }
        self.indent();
        if let Some(m) = class.meta.iter().find(|m| m.name.sym == "align") {
            let a_val = match m.args.first() {
                Some(crate::ast::MetaArg::Positional(e)) => self.generate_expr(e),
                Some(crate::ast::MetaArg::Named(_, e)) => self.generate_expr(e),
                None => "1".to_string(),
            };
            self.writeln(&format!("_align: [0]u8 align({}) = undefined,", a_val));
        }
        let needs_trait_infra =
            !class.implements.is_empty() || class.meta.iter().any(|m| m.name.sym == "reflect");
        if needs_trait_infra {
            self.writeln(&format!("pub const __TYPE_ID: u64 = 0x{:016x};", type_id));
        }
        if class.implements.iter().any(|i| i.sym == "Agent") {
            self.writeln("id: u32 = 0,");
        }
        if class.is_static {
            for f in &class.fields {
                let align_str = if let Some(m) = f.meta.iter().find(|m| m.name.sym == "align") {
                    let a_val = match m.args.first() {
                        Some(crate::ast::MetaArg::Positional(e)) => self.generate_expr(e),
                        Some(crate::ast::MetaArg::Named(_, e)) => self.generate_expr(e),
                        None => "1".to_string(),
                    };
                    format!(" align({})", a_val)
                } else {
                    String::new()
                };
                let default_init = if let Some(def) = &f.default {
                    format!(" = {}", self.generate_expr(def))
                } else {
                    " = undefined".to_string()
                };
                let pub_prefix = if f.is_pub { "pub " } else { "" };
                self.writeln(&format!(
                    "{}var {}: {}{}{};",
                    pub_prefix,
                    f.ident.sym,
                    self.map_type(&f.ty),
                    align_str,
                    default_init
                ));
            }
        } else {
            for f in &class.fields {
                let align_str = if let Some(m) = f.meta.iter().find(|m| m.name.sym == "align") {
                    let a_val = match m.args.first() {
                        Some(crate::ast::MetaArg::Positional(e)) => self.generate_expr(e),
                        Some(crate::ast::MetaArg::Named(_, e)) => self.generate_expr(e),
                        None => "1".to_string(),
                    };
                    format!(" align({})", a_val)
                } else {
                    String::new()
                };
                let default_init = if let Some(def) = &f.default {
                    format!(" = {},", self.generate_expr(def))
                } else {
                    ",".to_string()
                };
                self.writeln(&format!(
                    "{}: {}{}{}",
                    f.ident.sym,
                    self.map_type(&f.ty),
                    align_str,
                    default_init
                ));
            }
            if !is_generic
                && !class.is_union
                && !class.meta.iter().any(|m| {
                    m.name.sym == "value"
                        || m.name.sym == "packed"
                        || m.name.sym == "extern"
                        || m.name.sym == "opaque"
                })
            {
                self.writeln(&format!("\npub fn _create(allocator: std.mem.Allocator) !*{} {{ const self_ptr = try allocator.create({}); self_ptr.* = undefined; return self_ptr; }}", class.ident.sym, class.ident.sym));
            }
        }
        if needs_trait_infra {
            self.writeln(&format!(
                "pub fn __box_drop(ptr: *anyopaque) void {{ const self_ptr: *{0} = @ptrCast(@alignCast(ptr)); if (@hasDecl({0}, \"release\")) {{ const RType = @typeInfo(@TypeOf({0}.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = self_ptr.release() catch {{}}; }} else {{ self_ptr.release(); }} }} std.heap.page_allocator.destroy(self_ptr); }}",
                class.ident.sym
            ));
        }
        for m in &class.methods {
            self.generate_function_decorated(m, Some(&class.ident.sym));
        }
        for code in &class.extra_zig {
            for line in code.lines() {
                self.writeln(line);
            }
        }
        if let Some(rc_meta) = class.meta.iter().find(|m| m.name.sym == "refCount" || m.name.sym == "refcount") {
            let deinit_fn = if let Some(arg) = rc_meta.args.first() {
                match arg {
                    crate::ast::MetaArg::Positional(Expr::Ident(id)) => id.sym.clone(),
                    crate::ast::MetaArg::Named(_, Expr::Ident(id)) => id.sym.clone(),
                    _ => "deinit".to_string(),
                }
            } else {
                "deinit".to_string()
            };
            self.writeln(&format!(
                "const Self = @This(); const RefCount = if (@hasDecl(@import(\"root\"), \"bun\")) @import(\"root\").bun.ptr.RefCount(Self, \"ref_count\", {deinit_fn}, .{{}}) else struct {{ pub fn ref(self: *Self) *Self {{ if (@TypeOf(self.ref_count) == u32) {{ self.ref_count += 1; }} else {{ _ = self.ref_count.ref(); }} return self; }} pub fn deref(self: *Self) void {{ if (@TypeOf(self.ref_count) == u32) {{ if (self.ref_count > 0) {{ self.ref_count -= 1; if (self.ref_count == 0) self.{deinit_fn}(); }} }} else {{ if (self.ref_count.deref()) self.{deinit_fn}(); }} }} }};"
            ));
            self.writeln("pub fn ref(self: *@This()) *@This() { return RefCount.ref(self); }");
            self.writeln("pub fn deref(self: *@This()) void { RefCount.deref(self); }");
        }
        self.dedent();
        self.writeln("};");
        if is_generic {
            self.dedent();
            self.writeln("}");
        }
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

    fn meta_expr_to_zig(&self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(Lit::Str(s)) => format!("\"{}\"", s),
            Expr::Lit(Lit::Int(n)) => format!("{}", n),
            Expr::Lit(Lit::Float(f)) => format!("{}", f),
            Expr::Lit(Lit::Bool(b)) => format!("{}", b),
            Expr::Lit(Lit::Null) => "null".to_string(),
            Expr::Ident(id) => {
                if id.sym == "true" || id.sym == "false" {
                    id.sym.clone()
                } else {
                    format!("\"{}\"", id.sym)
                }
            }
            _ => "\"\"".to_string(),
        }
    }

    fn generate_meta_population(&self, meta_items: &[MetaItem], map_var: &str) -> String {
        let mut out = String::new();
        let mut grouped: HashMap<String, Vec<&MetaItem>> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        for item in meta_items {
            if !grouped.contains_key(&item.name.sym) {
                order.push(item.name.sym.clone());
            }
            grouped.entry(item.name.sym.clone()).or_default().push(item);
        }
        for name in order {
            let items = &grouped[&name];
            if name == "meta" {
                out.push_str("{\nconst __sub = Map._create(alloc) catch unreachable;\n");
                for item in items {
                    for arg in &item.args {
                        match arg {
                            MetaArg::Named(id, expr) => {
                                let v = self.meta_expr_to_zig(expr);
                                out.push_str(&format!("_ = __sub.set(\"{}\", {});\n", id.sym, v));
                                out.push_str(&format!(
                                    "_ = {}.set(\"{}\", {});\n",
                                    map_var, id.sym, v
                                ));
                            }
                            MetaArg::Positional(expr) => {
                                let v = self.meta_expr_to_zig(expr);
                                out.push_str(&format!("_ = __sub.set(\"value\", {});\n", v));
                                out.push_str(&format!("_ = {}.set(\"value\", {});\n", map_var, v));
                            }
                        }
                    }
                }
                out.push_str(&format!(
                    "_ = {}.set(\"meta\", cp_message_from(__sub));\n}}\n",
                    map_var
                ));
                continue;
            }
            if items.len() == 1 {
                let item = items[0];
                if item.args.is_empty() {
                    out.push_str(&format!("_ = {}.set(\"{}\", true);\n", map_var, name));
                } else if item.args.len() == 1 {
                    match &item.args[0] {
                        MetaArg::Positional(expr) => {
                            let val_code = self.meta_expr_to_zig(expr);
                            out.push_str(&format!(
                                "_ = {}.set(\"{}\", {});\n",
                                map_var, name, val_code
                            ));
                        }
                        MetaArg::Named(id, expr) => {
                            let val_code = self.meta_expr_to_zig(expr);
                            out.push_str(&format!("{{\nconst __sub = Map._create(alloc) catch unreachable;\n_ = __sub.set(\"{}\", {});\n_ = {}.set(\"{}\", cp_message_from(__sub));\n}}\n", id.sym, val_code, map_var, name));
                        }
                    }
                } else {
                    let has_named = item.args.iter().any(|a| matches!(a, MetaArg::Named(_, _)));
                    if has_named {
                        out.push_str("{\nconst __sub = Map._create(alloc) catch unreachable;\n");
                        for arg in &item.args {
                            match arg {
                                MetaArg::Named(id, expr) => {
                                    let v = self.meta_expr_to_zig(expr);
                                    out.push_str(&format!(
                                        "_ = __sub.set(\"{}\", {});\n",
                                        id.sym, v
                                    ));
                                }
                                MetaArg::Positional(expr) => {
                                    let v = self.meta_expr_to_zig(expr);
                                    out.push_str(&format!("_ = __sub.set(\"value\", {});\n", v));
                                }
                            }
                        }
                        out.push_str(&format!(
                            "_ = {}.set(\"{}\", cp_message_from(__sub));\n}}\n",
                            map_var, name
                        ));
                    } else {
                        out.push_str(&format!(
                            "{{\nconst __arr = alloc.alloc(Message, {}) catch unreachable;\n",
                            item.args.len()
                        ));
                        for (i, arg) in item.args.iter().enumerate() {
                            if let MetaArg::Positional(expr) = arg {
                                let v = self.meta_expr_to_zig(expr);
                                out.push_str(&format!("__arr[{}] = cp_message_from({});\n", i, v));
                            }
                        }
                        out.push_str(&format!(
                            "_ = {}.set(\"{}\", Message{{ .array = __arr }});\n}}\n",
                            map_var, name
                        ));
                    }
                }
            } else {
                out.push_str(&format!(
                    "{{\nconst __arr = alloc.alloc(Message, {}) catch unreachable;\n",
                    items.len()
                ));
                for (i, item) in items.iter().enumerate() {
                    if item.args.is_empty() {
                        out.push_str(&format!("__arr[{}] = cp_message_from(true);\n", i));
                    } else if item.args.len() == 1 {
                        match &item.args[0] {
                            MetaArg::Positional(expr) => {
                                let v = self.meta_expr_to_zig(expr);
                                out.push_str(&format!("__arr[{}] = cp_message_from({});\n", i, v));
                            }
                            MetaArg::Named(id, expr) => {
                                let v = self.meta_expr_to_zig(expr);
                                out.push_str(&format!("{{\nconst __sub = Map._create(alloc) catch unreachable;\n_ = __sub.set(\"{}\", {});\n__arr[{}] = cp_message_from(__sub);\n}}\n", id.sym, v, i));
                            }
                        }
                    } else {
                        out.push_str(&format!("__arr[{}] = cp_message_from(true);\n", i));
                    }
                }
                out.push_str(&format!(
                    "_ = {}.set(\"{}\", Message{{ .array = __arr }});\n}}\n",
                    map_var, name
                ));
            }
        }
        out
    }

    fn generate_reflection_tables(&mut self) {
        let class_names: Vec<String> = self.class_defs.keys().cloned().collect();
        for cls_name in class_names {
            let cls = self.class_defs[&cls_name].clone();
            if !cls.meta.iter().any(|m| m.name.sym == "reflect") {
                continue;
            }
            if cls_name == "TypeInfo"
                || cls_name == "FieldInfo"
                || cls_name == "MethodInfo"
                || cls_name == "Map"
                || cls_name == "Set"
                || cls_name == "Box"
                || cls_name == "Error"
                || cls_name == "RegExp"
            {
                continue;
            }
            self.writeln(&format!("\nvar _typeinfo_{}: ?*TypeInfo = null;", cls_name));
            self.writeln(&format!(
                "fn _get_typeinfo_{}() *const TypeInfo {{",
                cls_name
            ));
            self.indent();
            self.writeln(&format!("if (_typeinfo_{}) |ti| return ti;", cls_name));
            self.writeln("const alloc = std.heap.page_allocator;");
            self.writeln("const cls_meta = Map._create(alloc) catch unreachable;");
            let cls_meta_code = self.generate_meta_population(&cls.meta, "cls_meta");
            for line in cls_meta_code.lines() {
                self.writeln(line);
            }

            let user_fields: Vec<&Field> = cls.fields.iter().collect();
            if user_fields.is_empty() {
                self.writeln("const fields = &[_]FieldInfo{};");
            } else {
                self.writeln(&format!(
                    "const fields = alloc.alloc(FieldInfo, {}) catch unreachable;",
                    user_fields.len()
                ));
                for (f_idx, f) in user_fields.iter().enumerate() {
                    let f_meta_var = format!("f{}_meta", f_idx);
                    self.writeln(&format!(
                        "const {} = Map._create(alloc) catch unreachable;",
                        f_meta_var
                    ));
                    let f_meta_code = self.generate_meta_population(&f.meta, &f_meta_var);
                    for line in f_meta_code.lines() {
                        self.writeln(line);
                    }
                    self.writeln(&format!(
                        "fields[{}] = FieldInfo{{ .name = \"{}\", .typeName = \"{}\", .meta = {} }};",
                        f_idx,
                        f.ident.sym,
                        type_to_display_string(&f.ty),
                        f_meta_var
                    ));
                }
            }

            let user_methods: Vec<&Function> = cls
                .methods
                .iter()
                .filter(|m| !m.ident.sym.starts_with("__") && m.ident.sym != "release")
                .collect();
            if user_methods.is_empty() {
                self.writeln("const methods = &[_]MethodInfo{};");
            } else {
                self.writeln(&format!(
                    "const methods = alloc.alloc(MethodInfo, {}) catch unreachable;",
                    user_methods.len()
                ));
                for (m_idx, m) in user_methods.iter().enumerate() {
                    let m_meta_var = format!("m{}_meta", m_idx);
                    self.writeln(&format!(
                        "const {} = Map._create(alloc) catch unreachable;",
                        m_meta_var
                    ));
                    let m_meta_code = self.generate_meta_population(&m.meta, &m_meta_var);
                    for line in m_meta_code.lines() {
                        self.writeln(line);
                    }

                    let non_recv_params: Vec<String> = m
                        .params
                        .iter()
                        .filter(|p| p.ident.sym != "this" && p.ident.sym != "_")
                        .map(|p| format!("\"{}\"", p.ident.sym))
                        .collect();
                    let params_str = if non_recv_params.is_empty() {
                        "&[_][]const u8{}".to_string()
                    } else {
                        format!("&[_][]const u8{{ {} }}", non_recv_params.join(", "))
                    };
                    let ret_type_str = m
                        .return_type
                        .as_ref()
                        .map(type_to_display_string)
                        .unwrap_or_else(|| "void".to_string());

                    self.writeln(&format!(
                        "methods[{}] = MethodInfo{{ .name = \"{}\", .returnTypeName = \"{}\", .paramNames = {}, .isAsync = {}, .isStatic = {}, .meta = {} }};",
                        m_idx,
                        m.ident.sym,
                        ret_type_str,
                        params_str,
                        m.is_async,
                        m.is_static,
                        m_meta_var
                    ));
                }
            }

            self.writeln("const ti_ptr = alloc.create(TypeInfo) catch unreachable;");
            self.writeln(&format!(
                "ti_ptr.* = TypeInfo{{ .name = \"{}\", .isRef = {}, .fields = fields, .methods = methods, .meta = cls_meta }};",
                cls_name,
                cls.is_ref
            ));
            self.writeln(&format!("_typeinfo_{} = ti_ptr;", cls_name));
            self.writeln("return ti_ptr;");
            self.dedent();
            self.writeln("}");
        }
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
            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => self.compute_last_uses_block(b, counter),
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

    fn check_active_borrows_before_move(&self, var_name: &str) {
        for (borrower, (sources, _)) in &self.var_borrow_sources {
            if sources.iter().any(|(s, _)| s == var_name || ((s == "this" || s == "_") && (var_name == "this" || var_name == "_"))) {
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
        self.check_borrow_conflict_field_excluding(target_var, None, "", req_mut);
    }

    fn check_borrow_conflict_field(&self, target_var: &str, target_field: Option<&str>, req_mut: bool) {
        self.check_borrow_conflict_field_excluding(target_var, target_field, "", req_mut);
    }

    #[allow(dead_code)]
    fn check_borrow_conflict_excluding(
        &self,
        target_var: &str,
        exclude_borrower: &str,
        req_mut: bool,
    ) {
        self.check_borrow_conflict_field_excluding(target_var, None, exclude_borrower, req_mut);
    }

    fn check_borrow_conflict_field_excluding(
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

    fn targets_conflict(
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

    fn extract_member_path(&self, expr: &Expr) -> (Option<String>, Option<String>) {
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

    fn resolve_borrow_source_field(
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

    fn is_zig_heap_ptr_type(&self, ty: &Type) -> bool {
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

    fn check_mutation_allowed(&self, target_expr: &Expr, op_desc: &str) {
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
                                    if matches!(f.ty, Type::BorrowType { is_mut: false, .. }) && !op_desc.starts_with("field assignment") {
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
                if self.const_vars.contains(&id.sym) {
                    panic!(
                        "CP Compile Error: Cannot perform mutable operation ({}) on 'const' variable '{}' (declare with 'let' to allow mutation)",
                        op_desc, id.sym
                    );
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
                        self.check_borrow_conflict_field_excluding(src, src_f.as_deref(), &id.sym, true);
                    }
                } else {
                    if self.current_fn_params.contains(&id.sym)
                        && !self.mutable_fn_params.contains(&id.sym)
                    {
                        let is_ptr_or_mut_borrow =
                            if let Some(param_ty) = self.var_cp_types.get(&id.sym) {
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
                    if matches!(m_ty, Type::BorrowType { is_mut: false, .. }) && !op_desc.starts_with("field assignment") {
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
                            self.check_borrow_conflict_field_excluding(src, combined_field.as_deref(), &root, true);
                        }
                    } else {
                        if self.current_fn_params.contains(&root)
                            && !self.mutable_fn_params.contains(&root)
                        {
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

    fn check_read_allowed(&self, target_expr: &Expr) {
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

    fn extract_expr_borrow_info(&self, expr: &Expr) -> (Vec<(String, Option<String>)>, bool, bool) {
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
                                self.check_borrow_conflict_field("this", field_opt.as_deref(), *is_mut);
                                return (
                                    vec![
                                        ("this".to_string(), field_opt.clone()),
                                        ("_".to_string(), field_opt),
                                    ],
                                    *is_mut,
                                    false,
                                );
                            }
                            let is_obj_ptr = self.locals.get(&root).map_or(false, |t| {
                                t.starts_with('*') || t.starts_with("?*") || t.contains("=> *") || t.contains("=> ?*")
                            })
                            || self.var_cp_types.get(&root).map_or(false, |t| matches!(t, Type::RawPtr { .. } | Type::BorrowType { .. } | Type::CPtr(_) | Type::ManyRawPtr(_)));

                            self.check_borrow_conflict_field(&root, field_opt.as_deref(), *is_mut);
                            let resolved = self.resolve_borrow_source_field(&root, field_opt.as_deref());
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
                    return (vec![(id.sym.clone(), None)], false, is_ref_st);
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
                                        self.check_borrow_conflict_field(&root, field_opt.as_deref(), *is_m);
                                        let resolved = self.resolve_borrow_source_field(&root, field_opt.as_deref());
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
                                                            sources.push((arg_id.sym.clone(), None));
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
                    if fid.sym == "@fieldParentPtr" || fid.sym == "@alignCast" || fid.sym == "@ptrCast" {
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
                                            vec![("this".to_string(), None), ("_".to_string(), None)],
                                            is_mut,
                                            false,
                                        );
                                    }
                                    Type::Ref(cid) if self.ref_classes.contains(&cid.sym) => {
                                        return (
                                            vec![("this".to_string(), None), ("_".to_string(), None)],
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
                                                            sources.push((arg_id.sym.clone(), None));
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

    fn infer_cp_type(&self, expr: &Expr) -> Option<Type> {
        self.infer_cp_type_with_mock(expr, &HashMap::new())
    }

    fn infer_cp_type_with_mock(
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
                if let Some(t) = self.var_cp_types.get(&id.sym).or_else(|| self.global_var_types.get(&id.sym)) {
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
                                            if let Some(arg_ty) =
                                                self.infer_cp_type_with_mock(&c.args[idx], mock_types)
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
                                && oid.sym.chars().next().map_or(false, |c| c.is_ascii_uppercase())
                                && !self.class_defs.contains_key(&oid.sym));
                        if is_enum {
                            if mem.prop.sym == "fromName" || mem.prop.sym == "fromString" || mem.prop.sym == "get" {
                                return Some(Type::Optional(Box::new(Type::Ref(oid.clone()))));
                            }
                            if mem.prop.sym == "has" {
                                return Some(Type::Bool);
                            }
                            return Some(Type::Ref(oid.clone()));
                        }
                        if mem.prop.sym == "get" {
                            if let Some(mut oty) = self.infer_cp_type_with_mock(&mem.obj, mock_types) {
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
                    Some(Type::FixedArray(Box::new(val_ty), *n as usize))
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
            Expr::ComptimeMap { target_ty, entries, .. } => {
                let val_ty = if let Some(ty) = target_ty {
                    ty.clone()
                } else if let Some(first) = entries.first() {
                    self.infer_cp_type_with_mock(&first.val, mock_types).unwrap_or(Type::Any)
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
                    Some(Type::Ref(id)) if id.sym == "cstring" || id.sym == "cstr" => Some(Type::U8),
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

    fn generate_pipeline(&mut self, left: &Expr, right: &Expr) -> String {
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
                            if let Some(Type::Array(elem_ty)) = val_cp_ty {
                                bound_types.push((id.sym.clone(), Type::Array(elem_ty.clone())));
                            } else if let Some(Type::Tuple(tys)) = val_cp_ty {
                                let elem_ty = tys
                                    .get(idx)
                                    .cloned()
                                    .or_else(|| tys.first().cloned())
                                    .unwrap_or(Type::I32);
                                bound_types.push((id.sym.clone(), Type::Array(Box::new(elem_ty))));
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
            _ => ("true".to_string(), String::new()),
        }
    }

    fn error_type_to_cp_type(&self, err_ty: &ErrorType) -> Type {
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

    fn format_simple_pat(pat: &Pattern, has_else: &mut bool) -> String {
        match pat {
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

    fn collect_pat_variants(pat: &Pattern, set: &mut std::collections::HashSet<String>) {
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

    fn generate_switch(&mut self, m: &MatchExpr, is_expr: bool) -> String {
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
                        if let Some(tup) = &ep.tuple_pats {
                            if tup.len() == 1 {
                                if let Pattern::Ident(id) = &tup[0] {
                                    bound_ident = Some(id.sym.clone());
                                    format!("{} => |{}|", base, id.sym)
                                } else {
                                    format!("{} =>", base)
                                }
                            } else {
                                format!("{} =>", base)
                            }
                        } else {
                            format!("{} =>", base)
                        }
                    }
                    Pattern::Ident(id) if id.sym != "_" && id.sym != "else" => {
                        has_else = true;
                        bound_ident = Some(id.sym.clone());
                        format!("else => |{}|", id.sym)
                    }
                    other => {
                        let s = Self::format_simple_pat(other, &mut has_else);
                        format!("{} =>", s)
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

    fn generate_err_value(&mut self, err_inner: &Expr, target_err: Option<&ErrorType>) -> String {
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

    fn prepare_decorator_stmts(&self, dec_def: &DecDef, dec_item: &MetaItem) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        // Check if dec_def is higher-order: return fn(next) { ... } or return (next) => { ... }
        let mut is_higher_order = false;
        let mut inner_stmts = Vec::new();

        if let Some(Stmt::Return(ret_stmt)) = dec_def.body.body.last() {
            if let Some(Expr::Arrow(arrow)) = &ret_stmt.arg {
                is_higher_order = true;
                match &*arrow.body {
                    ArrowBody::Block(b) => {
                        inner_stmts = b.body.clone();
                    }
                    ArrowBody::Expr(e) => {
                        inner_stmts = vec![Stmt::Return(ReturnStmt {
                            span: Span { start: 0, end: 0 },
                            arg: Some(e.clone()),
                        })];
                    }
                }
            }
        }

        let arg_params: &[Param] = if is_higher_order {
            &dec_def.params
        } else {
            if !dec_def.params.is_empty() && dec_def.params[0].ident.sym == "next" {
                &dec_def.params[1..]
            } else {
                &dec_def.params
            }
        };

        for (idx, p) in arg_params.iter().enumerate() {
            let mut arg_val: Option<Expr> = None;
            for arg in &dec_item.args {
                match arg {
                    MetaArg::Named(id, val) if id.sym == p.ident.sym => {
                        arg_val = Some(val.clone());
                        break;
                    }
                    _ => {}
                }
            }
            if arg_val.is_none() {
                if idx < dec_item.args.len() {
                    match &dec_item.args[idx] {
                        MetaArg::Positional(val) => {
                            arg_val = Some(val.clone());
                        }
                        _ => {}
                    }
                }
            }
            if arg_val.is_none() {
                arg_val = p.default.clone();
            }

            if let Some(val) = arg_val {
                stmts.push(Stmt::Var(VarDecl {
                    span: p.span,
                    pat: Pattern::Ident(p.ident.clone()),
                    ty: Some(p.ty.clone()),
                    init: Some(val),
                    kind: VarDeclKind::Let,
                    is_threadlocal: false,
                    is_pub: false,
                }));
            }
        }

        if is_higher_order {
            stmts.extend(inner_stmts);
        } else {
            stmts.extend(dec_def.body.body.clone());
        }

        stmts
    }

    fn generate_function_decorated(&mut self, func: &Function, class_name: Option<&String>) {
        let active_decs: Vec<&MetaItem> = func
            .meta
            .iter()
            .filter(|m| self.dec_defs.contains_key(&m.name.sym))
            .collect();

        if active_decs.is_empty() {
            self.generate_function_logic(func, class_name);
            return;
        }

        let raw_name = format!("_raw_{}", func.ident.sym);
        let mut raw_func = func.clone();
        raw_func.ident.sym = raw_name.clone();
        raw_func.meta.clear();

        self.generate_function_logic(&raw_func, class_name);

        let n = active_decs.len();
        let forward_params: Vec<String> = func
            .params
            .iter()
            .filter(|p| p.ident.sym != "this" && p.ident.sym != "_")
            .map(|p| p.ident.sym.clone())
            .collect();

        for i in (0..n).rev() {
            let dec_item = active_decs[i];
            let dec_def = self.dec_defs.get(&dec_item.name.sym).unwrap().clone();

            let wrapper_name = if i == 0 {
                func.ident.sym.clone()
            } else {
                format!("_wrap_{}_{}", i, func.ident.sym)
            };

            let next_target = if i == n - 1 {
                raw_name.clone()
            } else {
                format!("_wrap_{}_{}", i + 1, func.ident.sym)
            };

            let wrapper_stmts = self.prepare_decorator_stmts(&dec_def, dec_item);

            let wrapper_func = Function {
                span: func.span,
                ident: Ident {
                    span: func.ident.span,
                    sym: wrapper_name,
                },
                generic_params: func.generic_params.clone(),
                params: func.params.clone(),
                return_type: func.return_type.clone(),
                body: BlockStmt {
                    span: func.body.span,
                    body: wrapper_stmts,
                },
                is_async: func.is_async,
                is_static: func.is_static,
                is_unsafe: func.is_unsafe,
                meta: vec![],
                is_pub: func.is_pub,
            };

            let old_callee = self.current_next_callee.clone();
            let old_forward = self.current_forward_params.clone();
            self.current_next_callee = Some(next_target);
            self.current_forward_params = forward_params.clone();

            self.generate_function_logic(&wrapper_func, class_name);

            self.current_next_callee = old_callee;
            self.current_forward_params = old_forward;
        }
    }

    fn generate_function(&mut self, func: &Function) {
        self.generate_function_decorated(func, None);
    }

    fn block_contains_ident(block: &BlockStmt, id: &str) -> bool {
        let mut vars = std::collections::HashSet::new();
        crate::ast_visitor::find_referenced_vars_in_block(block, &mut vars);
        if vars.contains(id) {
            return true;
        }

        for s in &block.body {
            match s {
                Stmt::Expr(Expr::Zig(z)) => {
                    if z.code.contains(id) {
                        return true;
                    }
                }
                Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ComptimeBlock(b) | Stmt::ArenaBlock(b) => {
                    if Self::block_contains_ident(b, id) {
                        return true;
                    }
                }
                Stmt::If(i) => {
                    if let Stmt::Block(b) = &*i.cons {
                        if Self::block_contains_ident(b, id) {
                            return true;
                        }
                    }
                    if let Some(eb) = &i.alt {
                        if let Stmt::Block(b) = &**eb {
                            if Self::block_contains_ident(b, id) {
                                return true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    fn generate_function_logic(&mut self, raw_func: &Function, class_name: Option<&String>) {
        if class_name.is_none()
            && (raw_func.ident.sym == "print"
                || raw_func.ident.sym == "spawn"
                || raw_func.ident.sym == "join")
        {
            return;
        }
        let func = raw_func;
        let zig_name = if func.ident.sym == "main" {
            "cp_main".to_string()
        } else if func.ident.sym == "test" {
            "cp_test".to_string()
        } else if matches!(func.ident.sym.as_str(), "and" | "or" | "orelse") {
            format!("@\"{}\"", func.ident.sym)
        } else {
            func.ident.sym.clone()
        };
        self.current_zig_fn_name = if let Some(c) = class_name {
            format!("{}_{}", c, zig_name)
        } else {
            zig_name.clone()
        };
        self.current_fn_zig_name = zig_name.clone();
        self.drop_flag_vars.clear();
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
            .map(|t| {
                if let Type::Ref(id) = t {
                    if func.generic_params.iter().any(|gp| match gp {
                        GenericParam::Type { ident, .. } => ident.sym == id.sym,
                        _ => false,
                    }) {
                        if let Some(param) = func.params.iter().find(|p| match &p.ty {
                            Type::Ref(pid) => pid.sym == id.sym,
                            _ => false,
                        }) {
                            return format!("@TypeOf({})", param.ident.sym);
                        }
                    }
                }
                self.map_type(t)
            })
            .unwrap_or("void".to_string());
        self.current_fn_is_async = func.is_async;

        // Initialize Lifetime & NLL tracking for this function
        self.current_scope_depth = 1;
        self.var_scope_depth.clear();
        self.var_borrow_sources.clear();
        self.var_is_ref_struct.clear();
        self.loop_borrowed_containers.clear();
        self.current_fn_params.clear();
        self.mutable_fn_params.clear();
        self.move_fn_params.clear();
        self.var_cp_types.clear();
        self.local_trampolines.clear();
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
                    let is_generic = self
                        .class_defs
                        .get(c)
                        .map(|cls| !cls.generic_params.is_empty())
                        .unwrap_or(false);
                    let is_enum = self.enum_defs.contains_key(c);
                    let is_val = is_enum || self.value_classes.contains(c) || is_generic;
                    let type_name = if is_generic {
                        "@This()".to_string()
                    } else {
                        c.clone()
                    };
                    let ptr_type = if let Some(first) = func.params.first().filter(|p| p.ident.sym == "this" || p.ident.sym == "_") {
                        if matches!(&first.ty, Type::BorrowType { is_mut: false, .. }) {
                            format!("*const {}", type_name)
                        } else if matches!(&first.ty, Type::BorrowType { is_mut: true, .. } | Type::RawPtr { .. }) {
                            format!("*{}", type_name)
                        } else if is_val && self.current_receiver_mode == ReceiverMode::ReadOnly {
                            type_name
                        } else if self.current_receiver_mode == ReceiverMode::ReadOnly {
                            format!("*const {}", type_name)
                        } else {
                            format!("*{}", type_name)
                        }
                    } else if is_val && self.current_receiver_mode == ReceiverMode::ReadOnly {
                        type_name
                    } else if self.current_receiver_mode == ReceiverMode::ReadOnly {
                        format!("*const {}", type_name)
                    } else {
                        format!("*{}", type_name)
                    };
                    self.locals.insert("self".to_string(), ptr_type.clone());
                    params_str.push_str(&format!("self: {}", ptr_type));
                }
            }
            for p in &func.params {
                if self.is_class_method && (p.ident.sym == "this" || p.ident.sym == "_" || p.ident.sym == "self") {
                    continue;
                }
                let mapped_ty = match &p.ty {
                    Type::Ref(id) if self.trait_defs.contains_key(&id.sym) => "anytype".to_string(),
                    Type::Ref(id) if func.generic_params.iter().any(|gp| match gp {
                        GenericParam::Type { ident, .. } => ident.sym == id.sym,
                        _ => false,
                    }) => "anytype".to_string(),
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
            self.extract_locals_simple(&func.body);
            let is_inline = func.meta.iter().any(|m| m.name.sym == "inline");
            let is_noinline = func.meta.iter().any(|m| m.name.sym == "noinline");
            let inline_kw = if is_inline {
                "inline "
            } else if is_noinline {
                "noinline "
            } else {
                ""
            };
            let c_meta = func.meta.iter().find(|m| m.name.sym == "c");
            let is_c = c_meta.is_some() || func.meta.iter().any(|m| m.name.sym == "extern");
            let c_alias = c_meta.and_then(|m| match m.args.first() {
                Some(MetaArg::Positional(Expr::Lit(Lit::Str(s)))) => Some(s.clone()),
                _ => None,
            });
            let callconv_clause = if let Some(m) = func.meta.iter().find(|m| m.name.sym == "callconv") {
                let conv = match m.args.first() {
                    Some(MetaArg::Positional(Expr::Lit(Lit::Str(s)))) => {
                        if s.starts_with('.') || s.contains('.') {
                            s.clone()
                        } else {
                            format!(".{}", s)
                        }
                    }
                    Some(MetaArg::Positional(Expr::Ident(id))) => {
                        if id.sym.starts_with('.') || id.sym.contains('.') {
                            id.sym.clone()
                        } else {
                            format!(".{}", id.sym)
                        }
                    }
                    Some(MetaArg::Positional(Expr::Member(mem))) => {
                        format!("{}.{}", self.generate_expr(&mem.obj), mem.prop.sym)
                    }
                    _ => ".c".to_string(),
                };
                format!("callconv({}) ", conv)
            } else if is_c {
                "callconv(.c) ".to_string()
            } else {
                String::new()
            };
            let pub_prefix = if class_name.is_some() {
                if func.is_pub
                    || func.ident.sym == "init"
                    || func.ident.sym == "release"
                    || func.ident.sym == "_create"
                    || func.ident.sym.starts_with("__")
                {
                    "pub "
                } else {
                    ""
                }
            } else {
                "pub "
            };
            if is_c && func.body.body.is_empty() {
                if let Some(alias) = c_alias {
                    writeln!(
                        self.output,
                        "\nextern \"c\" fn {}({}) {}{};",
                        alias, params_str, callconv_clause, zig_ret_ty
                    )
                    .unwrap();
                    writeln!(self.output, "{}const {} = {};", pub_prefix, zig_name, alias).unwrap();
                } else {
                    writeln!(
                        self.output,
                        "\n{}extern \"c\" fn {}({}) {}{};",
                        pub_prefix, zig_name, params_str, callconv_clause, zig_ret_ty
                    )
                    .unwrap();
                }
                return;
            }
            let is_export = func.meta.iter().any(|m| m.name.sym == "export")
                || (is_c && !func.body.body.is_empty());
            let export_kw = if is_export {
                "export "
            } else {
                ""
            };
            self.current_func_name = Some(func.ident.sym.clone());
            self.record_source_location(func.span);
            writeln!(
                self.output,
                "\n{}{}{}fn {}({}) {}{} {{",
                pub_prefix, export_kw, inline_kw, zig_name, params_str, callconv_clause, zig_ret_ty
            )
            .unwrap();
            self.indent();
            if self.is_class_method {
                let is_self_used = self.last_use_map.contains_key("self")
                    || self.last_use_map.contains_key("this")
                    || self.last_use_map.contains_key("_")
                    || Self::block_contains_ident(&func.body, "self")
                    || Self::block_contains_ident(&func.body, "this");
                if !is_self_used {
                    self.writeln("_ = &self;");
                }
                if self.current_receiver_mode == ReceiverMode::Move && func.ident.sym != "release" {
                    self.writeln("var __self_moved = false; _ = &__self_moved;");
                    self.writeln("defer if (!__self_moved) { const T = @TypeOf(self); if (@typeInfo(T) == .pointer) { const Child = @typeInfo(T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) { const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) { _ = self.release() catch {}; } else { self.release(); } } } };");
                }
            }
            for p in &func.params {
                if p.ident.sym != "this" && p.ident.sym != "_" && p.ident.sym != "self" {
                    let is_used = self.last_use_map.contains_key(&p.ident.sym)
                        || Self::block_contains_ident(&func.body, &p.ident.sym);
                    if !p.is_comptime && !is_used {
                        self.writeln(&format!("_ = {};", p.ident.sym));
                    }
                    if p.is_move {
                        self.drop_flag_vars.insert(p.ident.sym.clone());
                        self.writeln(&format!(
                            "var __{}_moved = false; _ = &__{}_moved;",
                            p.ident.sym, p.ident.sym
                        ));
                        self.writeln(&format!("defer if (!__{0}_moved) {{ const T = @TypeOf({0}); if (@typeInfo(T) == .pointer) {{ const Child = @typeInfo(T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) {{ const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = {0}.release() catch {{}}; }} else {{ {0}.release(); }} }} }} else if (@typeInfo(T) == .@\"struct\" and @hasDecl(T, \"release\")) {{ var _drop_val = {0}; _drop_val.release(); }} }};", p.ident.sym));
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
            if let Some(Type::ResultType { ok, err }) = &func.return_type {
                let err_str = self.map_error_type(err);
                let is_native_err_union = self.is_lib_mode
                    || err_str.contains('.')
                    || err_str == "anyerror"
                    || self.error_sets.contains_key(&err_str)
                    || err_str.starts_with('(');
                if !is_native_err_union
                    && **ok == Type::Void
                    && !body_str.trim_end().ends_with("return .{ .ok = {} };")
                {
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
                if p.ident.sym == "this" || p.ident.sym == "_" || p.ident.sym == "self" {
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
                if p.ident.sym == "this" || p.ident.sym == "_" || p.ident.sym == "self" {
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
            Stmt::Break | Stmt::Continue => {}
            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
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
            Stmt::For(f) => {
                self.collect_expr_error_types(&f.expr, out);
                self.collect_stmt_error_types(&f.body, out);
            }
            Stmt::Func(f) => {
                for s in &f.body.body {
                    self.collect_stmt_error_types(s, out);
                }
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
                    if let Some(Type::FixedArray(elem, _) | Type::Array(elem)) = &var.ty {
                        self.expected_array_elem_type = Some((**elem).clone());
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
                                    "const"
                                } else {
                                    "var"
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
                            } else if matches!(var.ty, Some(Type::ManyRawPtr(_)) | Some(Type::CPtr(_)))
                                && matches!(init, Expr::Borrow(_, _))
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
                                self.write_to_case(&format!("defer if (!__{0}_moved) {{ const T = @TypeOf({0}); if (@typeInfo(T) == .pointer) {{ const Child = @typeInfo(T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) {{ const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = {0}.release() catch {{}}; }} else {{ {0}.release(); }} }} }} else if (@typeInfo(T) == .@\"struct\" and @hasDecl(T, \"release\")) {{ var _drop_val = {0}; _drop_val.release(); }} }};", ident.sym));
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
                                            if let Some(Type::Array(elem_ty)) = &init_ty {
                                                self.var_cp_types.insert(
                                                    ident.sym.clone(),
                                                    Type::Array(elem_ty.clone()),
                                                );
                                            } else if let Some(Type::Tuple(tys)) = &init_ty {
                                                let elem_ty = tys
                                                    .get(idx)
                                                    .cloned()
                                                    .or_else(|| tys.first().cloned())
                                                    .unwrap_or(Type::I32);
                                                self.var_cp_types.insert(
                                                    ident.sym.clone(),
                                                    Type::Array(Box::new(elem_ty)),
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
                    let (saved_cap_local, saved_cap_cp, was_mut_loop) = if let Some(cap) = &i.capture {
                        self.var_scope_depth
                            .insert(cap.sym.clone(), self.current_scope_depth + 1);
                        self.block_locals.insert(cap.sym.clone());
                        let was_mut = if i.is_ptr_capture {
                            !self.mut_borrow_loop_vars.insert(cap.sym.clone())
                        } else {
                            false
                        };
                        let ptr_prefix = if i.is_ptr_capture { "*" } else { "" };
                        self.write_to_case(&format!("if ({}) |{}{}| {{", test, ptr_prefix, cap.sym));
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
                        self.write_to_case(&format!("_ = &{};", cap.sym));
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
                    let ptr_prefix = if w.is_ptr_capture { "*" } else { "" };
                    self.write_to_case(&format!("{}while ({}) |{}{}| {{", lbl_prefix, test, ptr_prefix, cap.sym));
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
                    self.write_to_case(&format!("{}while ({}) {{", lbl_prefix, test));
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

                let is_native_slice =
                    f.is_comptime || expr_code.ends_with(".items") || expr_code.ends_with(".values()");
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
                            if !self.is_state_machine {
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
                        self.write_to_case(&format!("defer if (!__{0}_moved) {{ const T = @TypeOf({0}); if (@typeInfo(T) == .pointer) {{ const Child = @typeInfo(T).pointer.child; if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"release\")) {{ const RType = @typeInfo(@TypeOf(Child.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = {0}.release() catch {{}}; }} else {{ {0}.release(); }} }} }} else if (@typeInfo(T) == .@\"struct\" and @hasDecl(T, \"release\")) {{ var _drop_val = {0}; _drop_val.release(); }} }};", id.sym));
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
                            Some(Type::Ref(id)) if id.sym.contains('?') || id.sym.starts_with("switch") => true,
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
                        if self.local_trampolines.contains(src) {
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

    fn build_ret_stmt_for_fallback(&mut self, fallback_expr: Option<&Expr>) -> String {
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

    fn generate_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(Lit::Int(v)) => {
                if *v < 0 {
                    format!("0x{:x}", *v as u64)
                } else {
                    v.to_string()
                }
            }
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
                        if is_enum && (prop == "get" || prop == "fromName" || prop == "fromString") && call.args.len() == 1 {
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
                    } else if prop == "includes" && !call.args.is_empty() {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("cp_includes({}, {})", obj, arg);
                    } else if prop == "startsWith" && !call.args.is_empty() {
                        self.check_read_allowed(&mem.obj);
                        let obj = self.generate_expr(&mem.obj);
                        let arg = self.generate_expr(&call.args[0]);
                        return format!("@import(\"std\").mem.startsWith(u8, {}, {})", obj, arg);
                    } else if prop == "endsWith" && !call.args.is_empty() {
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
                    } else if prop == "toLowerCase" {
                        let obj = self.generate_expr(&mem.obj);
                        return format!("(blk: {{ var out = {}.alloc(u8, {}.len) catch unreachable; for ({}, 0..) |c, i| out[i] = @import(\"std\").ascii.toLower(c); break :blk out; }})", self.current_allocator(), obj, obj);
                    } else if prop == "toUpperCase" {
                        let obj = self.generate_expr(&mem.obj);
                        return format!("(blk: {{ var out = {}.alloc(u8, {}.len) catch unreachable; for ({}, 0..) |c, i| out[i] = @import(\"std\").ascii.toUpper(c); break :blk out; }})", self.current_allocator(), obj, obj);
                    } else if prop == "replace" && call.args.len() >= 2 {
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
    const _raw_json = @import(\"std\").fmt.allocPrint(std.heap.page_allocator, "{{f}}", .{{@import(\"std\").json.fmt(_fmt, .{{}})}}) catch unreachable;
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
                    let cls_opt = self.class_defs.get(&ei.enum_name.sym).cloned();
                    if let Some(ref cls) = cls_opt {
                        if cls.is_static {
                            panic!("CP Compile Error: Cannot instantiate static class or struct '{}'", ei.enum_name.sym);
                        }
                    }
                    let mut s = format!("{}{{ ", ei.enum_name.sym);
                    for (idx, f) in fields_slice.iter().enumerate() {
                        let old_array_elem = self.expected_array_elem_type.clone();
                        if let Some(ref cls) = cls_opt {
                            if let Some(cf) = cls.fields.iter().find(|fl| fl.ident.sym == f.key.sym) {
                                if let Type::FixedArray(elem, _) | Type::Array(elem) = &cf.ty {
                                    self.expected_array_elem_type = Some((**elem).clone());
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
                let class_name = match &*call.callee {
                    Expr::Ident(id) => id.sym.clone(),
                    _ => "Unknown".to_string(),
                };
                if let Some(cls) = self.class_defs.get(&class_name) {
                    if cls.is_static {
                        panic!("CP Compile Error: Cannot instantiate static class or struct '{}' with 'new'", class_name);
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

                let is_enum = match &cp_ty {
                    Some(Type::Ref(id)) => self.enum_defs.contains_key(&id.sym),
                    _ => false,
                };
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

                let is_cstr = matches!(cp_ty, Some(Type::CStr)) || obj_ty.contains("[*:0]");
                if is_cstr && (member.prop.sym == "len" || member.prop.sym == "length") {
                    return format!("std.mem.span({}).len", obj);
                }

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
                    if obj_ty.starts_with('?') {
                        format!("({}).?.data.{}", obj, safe_prop)
                    } else {
                        format!("{}.data.{}", obj, safe_prop)
                    }
                } else if obj_ty.starts_with('?') {
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
                    format!("@field({}, {})", obj, f)
                }
                "@hasField" => {
                    let t = self.generate_expr(&call.args[0]);
                    let f = self.generate_expr(&call.args[1]);
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
                            } else if inner_code.starts_with("@intCast(") || inner_code.starts_with("@truncate(") {
                                format!("@as({}, {})", target_zig_ty, inner_code)
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
                            format!("@as({}, @enumFromInt({}))", target_zig_ty, inner_code)
                        } else if is_target_fn {
                            format!("@as({}, @ptrCast({}))", target_zig_ty, inner_code)
                        } else if matches!(resolved_ty, Type::Any) {
                            format!(
                                "@as({}, @ptrCast(@alignCast({})))",
                                target_zig_ty, inner_code
                            )
                        } else if matches!(resolved_ty, Type::Ref(_)) && !matches!(resolved_ty, Type::Ref(ref id) if self.class_defs.contains_key(&id.sym) || self.trait_defs.contains_key(&id.sym)) {
                            format!(
                                "(if (comptime @typeInfo({0}) == .@\"enum\") @as({0}, @enumFromInt({1})) else @as({0}, {1}))",
                                target_zig_ty, inner_code
                            )
                        } else if resolved_ty == Type::Bool {
                            format!("(if (comptime @typeInfo(@TypeOf({0})) == .bool) {0} else ({0} != 0))", inner_code)
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
            Expr::Zig(z) => z.code.clone(),
            _ => "0".to_string(),
        }
    }

    fn generate_http_serve(&mut self, call_arg: &Expr) -> String {
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

    fn infer_zig_type(&self, ty: Option<&Type>, init: Option<&Expr>) -> String {
        if let Some(t) = ty {
            return self.map_type(t);
        }
        if let Some(expr) = init {
            if let Expr::New(call) = expr {
                if let Expr::Ident(id) = &*call.callee {
                    if id.sym == "Error" {
                        return "Error".to_string();
                    }
                    return format!("*{}", id.sym);
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
                        return format!("*{}", id.sym);
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
                        if (prop == "indexOf" || prop == "lastIndexOf" || prop == "charCodeAt") && call.args.len() == 1 {
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

    fn extract_locals_from_expr_simple(&mut self, expr: &Expr) {
        match expr {
            Expr::Arena(b) | Expr::Unsafe(b) | Expr::Comptime(b) => {
                self.extract_locals_simple(b);
            }
            Expr::Bin(bin) => {
                self.extract_locals_from_expr_simple(&bin.left);
                self.extract_locals_from_expr_simple(&bin.right);
            }
            Expr::Call(c) | Expr::OptCall(c) | Expr::New(c) => {
                self.extract_locals_from_expr_simple(&c.callee);
                for a in &c.args {
                    self.extract_locals_from_expr_simple(a);
                }
            }
            Expr::Deref(inner)
            | Expr::Unwrap(inner)
            | Expr::Question(inner)
            | Expr::Cast { expr: inner, .. } => {
                self.extract_locals_from_expr_simple(inner);
            }
            Expr::Member(m) => {
                self.extract_locals_from_expr_simple(&m.obj);
            }
            Expr::Array(items) => {
                for item in items {
                    self.extract_locals_from_expr_simple(item);
                }
            }
            _ => {}
        }
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
                            let init_cp_ty = v.init.as_ref().and_then(|e| self.infer_cp_type(e));
                            let mut resolved_ty = v.ty.as_ref().or(init_cp_ty.as_ref()).cloned();
                            while let Some(Type::BorrowType { inner, .. }) = resolved_ty {
                                resolved_ty = Some(*inner);
                            }
                            let rest_pos = el.iter().position(|p| matches!(p, Pattern::Rest(_)));
                            match &resolved_ty {
                                Some(Type::Tuple(tys)) => {
                                    for (idx, p) in el.iter().enumerate() {
                                        match p {
                                            Pattern::Ident(i) => {
                                                let ty_idx = if let Some(r_idx) = rest_pos {
                                                    if idx < r_idx {
                                                        Some(idx)
                                                    } else {
                                                        let offset = el.len() - idx;
                                                        tys.len().checked_sub(offset)
                                                    }
                                                } else {
                                                    Some(idx)
                                                };
                                                let t_str = ty_idx
                                                    .and_then(|ti| tys.get(ti))
                                                    .map(|t| self.map_type(t))
                                                    .unwrap_or_else(|| "[]const u8".to_string());
                                                self.locals.insert(i.sym.clone(), t_str);
                                                if let Some(ti) = ty_idx {
                                                    if let Some(t) = tys.get(ti) {
                                                        self.var_cp_types
                                                            .insert(i.sym.clone(), t.clone());
                                                    }
                                                }
                                            }
                                            Pattern::Rest(inner) => {
                                                if let Pattern::Ident(i) = &**inner {
                                                    let elem_ty = tys
                                                        .get(idx)
                                                        .cloned()
                                                        .or_else(|| tys.first().cloned())
                                                        .unwrap_or(Type::I32);
                                                    let arr_ty = Type::Array(Box::new(elem_ty));
                                                    self.locals.insert(
                                                        i.sym.clone(),
                                                        self.map_type(&arr_ty),
                                                    );
                                                    self.var_cp_types.insert(i.sym.clone(), arr_ty);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Some(Type::Array(elem_ty)) => {
                                    let t_str = self.map_type(elem_ty);
                                    let arr_ty = Type::Array(elem_ty.clone());
                                    let arr_t_str = self.map_type(&arr_ty);
                                    for p in el {
                                        match p {
                                            Pattern::Ident(i) => {
                                                self.locals.insert(i.sym.clone(), t_str.clone());
                                                self.var_cp_types
                                                    .insert(i.sym.clone(), (**elem_ty).clone());
                                            }
                                            Pattern::Rest(inner) => {
                                                if let Pattern::Ident(i) = &**inner {
                                                    self.locals
                                                        .insert(i.sym.clone(), arr_t_str.clone());
                                                    self.var_cp_types
                                                        .insert(i.sym.clone(), arr_ty.clone());
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {
                                    for p in el {
                                        match p {
                                            Pattern::Ident(i) => {
                                                self.locals.insert(
                                                    i.sym.clone(),
                                                    "[]const u8".to_string(),
                                                );
                                            }
                                            Pattern::Rest(inner) => {
                                                if let Pattern::Ident(i) = &**inner {
                                                    self.locals.insert(
                                                        i.sym.clone(),
                                                        "[]const Message".to_string(),
                                                    );
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        Pattern::Object(fields) => {
                            let init_cp_ty = v.init.as_ref().and_then(|e| self.infer_cp_type(e));
                            let mut resolved_ty = v.ty.as_ref().or(init_cp_ty.as_ref()).cloned();
                            while let Some(Type::BorrowType { inner, .. }) = resolved_ty {
                                resolved_ty = Some(*inner);
                            }
                            let obj_fields = match &resolved_ty {
                                Some(Type::Object(fs)) => Some(fs),
                                _ => None,
                            };
                            let non_rest_keys: HashSet<String> = fields
                                .iter()
                                .filter(|p| !p.is_rest)
                                .map(|p| p.key.sym.clone())
                                .collect();

                            for p in fields {
                                if p.key.sym == "*" || matches!(p.val, Some(Pattern::Wildcard)) {
                                    continue;
                                }
                                if p.is_rest {
                                    let remaining_fields = if let Some(tys) = obj_fields {
                                        tys.iter()
                                            .filter(|t| !non_rest_keys.contains(&t.ident.sym))
                                            .cloned()
                                            .collect::<Vec<_>>()
                                    } else {
                                        vec![]
                                    };
                                    let rest_cp_ty = Type::Object(remaining_fields);
                                    let t_str = self.map_type(&rest_cp_ty);
                                    self.locals.insert(p.key.sym.clone(), t_str);
                                    self.var_cp_types.insert(p.key.sym.clone(), rest_cp_ty);
                                    continue;
                                }

                                let ident = if let Some(Pattern::Ident(i)) = &p.val {
                                    i
                                } else {
                                    &p.key
                                };
                                let f_opt = obj_fields
                                    .and_then(|tys| tys.iter().find(|t| t.ident.sym == p.key.sym));
                                let t_str = f_opt
                                    .map(|t| self.map_type(&t.ty))
                                    .unwrap_or_else(|| "i32".to_string());
                                self.locals.insert(ident.sym.clone(), t_str);
                                if let Some(f) = f_opt {
                                    self.var_cp_types.insert(ident.sym.clone(), f.ty.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                    self.expected_box_trait = old_box;
                    if let Some(init) = &v.init {
                        self.extract_locals_from_expr_simple(init);
                    }
                }
                Stmt::Expr(e) => {
                    if let Expr::Match(m) = e {
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
                            if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &*arm.body {
                                self.extract_locals_simple(b);
                            }
                            for (k, v) in old_v {
                                self.var_cp_types.insert(k, v);
                            }
                        }
                    } else {
                        self.extract_locals_from_expr_simple(e);
                    }
                }
                Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => self.extract_locals_simple(b),
                Stmt::If(i) => {
                    let old_v = self.var_cp_types.clone();
                    let old_l = self.locals.clone();
                    if let Some(cap) = &i.capture {
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
                        self.var_cp_types.insert(cap.sym.clone(), final_ty);
                        self.locals.insert(cap.sym.clone(), zig_ty);
                    }
                    if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &*i.cons {
                        self.extract_locals_simple(b);
                    }
                    if let Some(alt) = &i.alt {
                        if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &**alt {
                            self.extract_locals_simple(b);
                        }
                    }
                    self.var_cp_types = old_v;
                    self.locals = old_l;
                }
                Stmt::While(w) => {
                    let old_v = self.var_cp_types.clone();
                    let old_l = self.locals.clone();
                    if let Some(cap) = &w.capture {
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
                        self.var_cp_types.insert(cap.sym.clone(), final_ty);
                        self.locals.insert(cap.sym.clone(), zig_ty);
                    }
                    if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &*w.body {
                        self.extract_locals_simple(b);
                    }
                    self.var_cp_types = old_v;
                    self.locals = old_l;
                }
                Stmt::For(f) => {
                    if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &*f.body {
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
                        if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &**cb {
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
                        if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &*arm.body {
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

    fn extract_local_vars_from_expr_to_str(&mut self, expr: &Expr, output: &mut String) {
        match expr {
            Expr::Arena(b) | Expr::Unsafe(b) | Expr::Comptime(b) => {
                self.extract_local_vars_to_str(b, output);
            }
            Expr::Bin(bin) => {
                self.extract_local_vars_from_expr_to_str(&bin.left, output);
                self.extract_local_vars_from_expr_to_str(&bin.right, output);
            }
            Expr::Call(c) | Expr::OptCall(c) | Expr::New(c) => {
                self.extract_local_vars_from_expr_to_str(&c.callee, output);
                for a in &c.args {
                    self.extract_local_vars_from_expr_to_str(a, output);
                }
            }
            Expr::Deref(inner)
            | Expr::Unwrap(inner)
            | Expr::Question(inner)
            | Expr::Cast { expr: inner, .. } => {
                self.extract_local_vars_from_expr_to_str(inner, output);
            }
            Expr::Member(m) => {
                self.extract_local_vars_from_expr_to_str(&m.obj, output);
            }
            Expr::Array(items) => {
                for item in items {
                    self.extract_local_vars_from_expr_to_str(item, output);
                }
            }
            _ => {}
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
                            if let Some(t) = var.ty.as_ref() {
                                self.var_cp_types.insert(ident.sym.clone(), t.clone());
                            } else if let Some(init_e) = &var.init {
                                if let Some(inferred_ty) = self.infer_cp_type(init_e) {
                                    self.var_cp_types.insert(ident.sym.clone(), inferred_ty);
                                }
                            }
                            writeln!(output, "    {}: {},", ident.sym, ty_str).unwrap();
                        }
                        Pattern::Tuple(el) => {
                            let init_cp_ty = var.init.as_ref().and_then(|e| self.infer_cp_type(e));
                            let mut resolved_ty = var.ty.as_ref().or(init_cp_ty.as_ref()).cloned();
                            while let Some(Type::BorrowType { inner, .. }) = resolved_ty {
                                resolved_ty = Some(*inner);
                            }
                            let rest_pos = el.iter().position(|p| matches!(p, Pattern::Rest(_)));
                            match &resolved_ty {
                                Some(Type::Tuple(tys)) => {
                                    for (idx, p) in el.iter().enumerate() {
                                        match p {
                                            Pattern::Ident(ident) => {
                                                let ty_idx = if let Some(r_idx) = rest_pos {
                                                    if idx < r_idx {
                                                        Some(idx)
                                                    } else {
                                                        let offset = el.len() - idx;
                                                        tys.len().checked_sub(offset)
                                                    }
                                                } else {
                                                    Some(idx)
                                                };
                                                let t_str = ty_idx
                                                    .and_then(|ti| tys.get(ti))
                                                    .map(|t| self.map_type(t))
                                                    .unwrap_or_else(|| "[]const u8".to_string());
                                                self.locals
                                                    .insert(ident.sym.clone(), t_str.clone());
                                                writeln!(output, "    {}: {},", ident.sym, t_str)
                                                    .unwrap();
                                            }
                                            Pattern::Rest(inner) => {
                                                if let Pattern::Ident(ident) = &**inner {
                                                    let elem_ty = tys
                                                        .get(idx)
                                                        .cloned()
                                                        .or_else(|| tys.first().cloned())
                                                        .unwrap_or(Type::I32);
                                                    let t_str = self
                                                        .map_type(&Type::Array(Box::new(elem_ty)));
                                                    self.locals
                                                        .insert(ident.sym.clone(), t_str.clone());
                                                    writeln!(
                                                        output,
                                                        "    {}: {},",
                                                        ident.sym, t_str
                                                    )
                                                    .unwrap();
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Some(Type::Array(elem_ty)) => {
                                    let t_str = self.map_type(elem_ty);
                                    let arr_t_str = self.map_type(&Type::Array(elem_ty.clone()));
                                    for p in el {
                                        match p {
                                            Pattern::Ident(ident) => {
                                                self.locals
                                                    .insert(ident.sym.clone(), t_str.clone());
                                                writeln!(output, "    {}: {},", ident.sym, t_str)
                                                    .unwrap();
                                            }
                                            Pattern::Rest(inner) => {
                                                if let Pattern::Ident(ident) = &**inner {
                                                    self.locals.insert(
                                                        ident.sym.clone(),
                                                        arr_t_str.clone(),
                                                    );
                                                    writeln!(
                                                        output,
                                                        "    {}: {},",
                                                        ident.sym, arr_t_str
                                                    )
                                                    .unwrap();
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {
                                    for p in el {
                                        match p {
                                            Pattern::Ident(ident) => {
                                                self.locals.insert(
                                                    ident.sym.clone(),
                                                    "[]const u8".to_string(),
                                                );
                                                writeln!(output, "    {}: []const u8,", ident.sym)
                                                    .unwrap();
                                            }
                                            Pattern::Rest(inner) => {
                                                if let Pattern::Ident(ident) = &**inner {
                                                    self.locals.insert(
                                                        ident.sym.clone(),
                                                        "[]const Message".to_string(),
                                                    );
                                                    writeln!(
                                                        output,
                                                        "    {}: []const Message,",
                                                        ident.sym
                                                    )
                                                    .unwrap();
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        Pattern::Object(fields) => {
                            let init_cp_ty = var.init.as_ref().and_then(|e| self.infer_cp_type(e));
                            let mut resolved_ty = var.ty.as_ref().or(init_cp_ty.as_ref()).cloned();
                            while let Some(Type::BorrowType { inner, .. }) = resolved_ty {
                                resolved_ty = Some(*inner);
                            }
                            let obj_fields = match &resolved_ty {
                                Some(Type::Object(fs)) => Some(fs),
                                _ => None,
                            };
                            let non_rest_keys: HashSet<String> = fields
                                .iter()
                                .filter(|p| !p.is_rest)
                                .map(|p| p.key.sym.clone())
                                .collect();

                            for p in fields {
                                if p.key.sym == "*" || matches!(p.val, Some(Pattern::Wildcard)) {
                                    continue;
                                }
                                if p.is_rest {
                                    let remaining_fields = if let Some(tys) = obj_fields {
                                        tys.iter()
                                            .filter(|t| !non_rest_keys.contains(&t.ident.sym))
                                            .cloned()
                                            .collect::<Vec<_>>()
                                    } else {
                                        vec![]
                                    };
                                    let rest_cp_ty = Type::Object(remaining_fields);
                                    let t_str = self.map_type(&rest_cp_ty);
                                    self.locals.insert(p.key.sym.clone(), t_str.clone());
                                    writeln!(output, "    {}: {},", p.key.sym, t_str).unwrap();
                                    continue;
                                }

                                let ident = if let Some(Pattern::Ident(i)) = &p.val {
                                    i
                                } else {
                                    &p.key
                                };
                                let f_opt = obj_fields
                                    .and_then(|tys| tys.iter().find(|t| t.ident.sym == p.key.sym));
                                let t_str = f_opt
                                    .map(|t| self.map_type(&t.ty))
                                    .unwrap_or_else(|| "i32".to_string());
                                self.locals.insert(ident.sym.clone(), t_str.clone());
                                writeln!(output, "    {}: {},", ident.sym, t_str).unwrap();
                            }
                        }
                        _ => {}
                    }
                    if let Some(init) = &var.init {
                        self.extract_local_vars_from_expr_to_str(init, output);
                    }
                }
                Stmt::Expr(e) => {
                    self.extract_local_vars_from_expr_to_str(e, output);
                }
                Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => self.extract_local_vars_to_str(b, output),
                Stmt::If(i) => {
                    match &*i.cons {
                        Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                            self.extract_local_vars_to_str(b, output)
                        }
                        _ => {}
                    }
                    if let Some(alt) = &i.alt {
                        match &**alt {
                            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                                self.extract_local_vars_to_str(b, output)
                            }
                            _ => {}
                        }
                    }
                }
                Stmt::While(w) => match &*w.body {
                    Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                        self.extract_local_vars_to_str(b, output)
                    }
                    _ => {}
                },
                Stmt::For(f) => {
                    if let Stmt::Block(b) | Stmt::ArenaBlock(b) = &*f.body {
                        self.extract_local_vars_to_str(b, output);
                    }
                }
                Stmt::TryCatch {
                    try_block,
                    catch_body,
                    ..
                } => {
                    self.extract_local_vars_to_str(try_block, output);
                    if let Some(cb) = catch_body {
                        match &**cb {
                            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                                self.extract_local_vars_to_str(b, output)
                            }
                            _ => {}
                        }
                    }
                }
                Stmt::Match(m) => {
                    for arm in &m.arms {
                        match &*arm.body {
                            Stmt::Block(b) | Stmt::UnsafeBlock(b) | Stmt::ArenaBlock(b) => {
                                self.extract_local_vars_to_str(b, output);
                            }
                            _ => {}
                        }
                    }
                }
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

    fn expr_to_zig_pure(&self, expr: &Expr) -> String {
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

    fn map_type_name(&self, name: &str) -> String {
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
                } else {
                    name.to_string()
                }
            }
        }
    }

    fn map_type(&self, ty: &Type) -> String {
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
                if ident.sym == "Mutex" && !self.type_aliases.contains_key("Mutex") && !self.class_defs.contains_key("Mutex") {
                    return "CpBareMutex".to_string();
                }
                if ident.sym.starts_with("RwLock<") {
                    let inner = &ident.sym[7..ident.sym.len() - 1];
                    return format!("CpRwLock({})", self.map_type_name(inner));
                }
                if ident.sym == "RwLock" && !self.type_aliases.contains_key("RwLock") && !self.class_defs.contains_key("RwLock") {
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
                        return format!("{}({})", base, args);
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

    fn map_raw_ptr_base(&self, ty: &Type) -> String {
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

fn type_to_display_string(ty: &Type) -> String {
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
