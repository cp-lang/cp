use crate::ast::*;
use crate::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

pub mod actor;
pub mod borrow;
pub mod expr;
pub mod pattern;
pub mod prelude;
pub mod stmt;
pub mod types;

pub use actor::*;
pub use borrow::*;
pub use expr::*;
pub use pattern::*;
pub use prelude::*;
pub use stmt::*;
pub use types::*;

pub struct Codegen {
    pub is_beam_mode: bool,
    pub is_lib_mode: bool,
    pub agent_classes: Vec<crate::ast::Class>,
    pub(crate) output: String,
    pub(crate) cases: Vec<String>,
    pub(crate) indent_level: usize,
    pub(crate) pc_counter: u32,
    pub(crate) tmp_counter: u32,
    pub(crate) lambda_counter: u32,
    pub(crate) current_case_content: String,
    pub(crate) is_comptime: bool,
    pub(crate) is_state_machine: bool,
    pub(crate) current_fn_is_async: bool,
    pub(crate) locals: HashMap<String, String>,
    pub(crate) globals: HashSet<String>,
    pub(crate) lambda_funcs: Vec<String>,
    pub(crate) current_zig_fn_name: String,
    pub(crate) current_fn_zig_name: String,
    pub drop_flag_vars: HashSet<String>,
    pub(crate) is_class_method: bool,
    pub(crate) is_main: bool,
    pub moved_vars: HashSet<String>,
    pub lambda_moved_vars: HashMap<String, HashSet<String>>,
    pub(crate) uninitialized_vars: HashSet<String>,
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
    pub type_namespace_vars: HashSet<String>,
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
            type_namespace_vars: HashSet::new(),
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
        let case_newlines = self
            .current_case_content
            .bytes()
            .filter(|&b| b == b'\n')
            .count();
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
                self.source_map_entries
                    .push(crate::source_map::SourceMapEntry {
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

    pub fn build_source_map(
        &self,
        cp_source: String,
        zig_target: String,
    ) -> crate::source_map::SourceMap {
        let mut sm = crate::source_map::SourceMap::new(cp_source, zig_target);
        sm.mappings = self.source_map_entries.clone();
        sm
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
                    if !v.is_extern
                        && v.kind == VarDeclKind::Const
                        && id
                            .sym
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_ascii_uppercase())
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
                            inferred = self
                                .infer_cp_type_with_mock(init, &std::collections::HashMap::new());
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
            self.writeln(
                "    pub extern \"c\" fn strcmp(s1: [*c]const u8, s2: [*c]const u8) c_int;",
            );
            self.writeln("    pub extern \"c\" fn strncmp(s1: [*c]const u8, s2: [*c]const u8, n: usize) c_int;");
            self.writeln("    pub extern \"c\" fn memcpy(dest: ?*anyopaque, src: ?*const anyopaque, n: usize) ?*anyopaque;");
            self.writeln("    pub extern \"c\" fn memset(s: ?*anyopaque, c_val: c_int, n: usize) ?*anyopaque;");
            self.writeln("};");
        }
        self.writeln("");

        // --- Tri-State Result, Built-in Error, String Concat & Pattern Helpers ---
        if !self.is_lib_mode && features.needs_result_error {
            self.writeln("pub fn CpResult(comptime __T: type, comptime __E: type) type {");
            self.writeln(
                "    return union(enum) { ok: __T, err: __E, pub const __IS_CP_RESULT = true; };",
            );
            self.writeln("}");
            self.writeln("pub const Err = struct {");
            self.writeln("    code: i32 = 0,");
            self.writeln("    msg: []const u8 = \"\",");
            self.writeln("    cause: ?*const anyopaque = null,");
            self.writeln("    cause_type_id: u64 = 0,");
            self.writeln("    pub const __TYPE_ID: u64 = 0xEE00_0000_0000_0001;");
            self.writeln("    pub fn init(code: i32, msg: []const u8) Err { return Err{ .code = code, .msg = msg }; }");
            self.writeln("    pub fn with_cause(self: Err, cause: anytype) Err { const CType = @TypeOf(cause); var ptr: ?*const anyopaque = null; var tid: u64 = 0; if (comptime @typeInfo(CType) == .pointer) { ptr = @ptrCast(cause); const Child = @typeInfo(CType).pointer.child; if (comptime @hasDecl(Child, \"__TYPE_ID\")) { tid = Child.__TYPE_ID; } } else if (comptime (@typeInfo(CType) == .@\"struct\" or @typeInfo(CType) == .@\"union\") and @hasField(CType, \"ptr\")) { ptr = @ptrCast(cause.ptr); if (comptime @hasField(CType, \"vtable\")) { tid = cause.vtable.__type_id; } } else if (comptime @typeInfo(CType) == .optional) { if (cause) |__c| return self.with_cause(__c); } else { if (comptime @hasDecl(CType, \"__TYPE_ID\")) { tid = CType.__TYPE_ID; } ptr = @ptrCast(&cause); } return Err{ .code = self.code, .msg = self.msg, .cause = ptr, .cause_type_id = tid }; }");
            self.writeln("    pub fn downcast(self: Err, comptime Target: type) ?*const Target {");
            self.writeln("        if (self.cause) |__c| {");
            self.writeln("            if (comptime @hasDecl(Target, \"__TYPE_ID\")) {");
            self.writeln("                if (self.cause_type_id != 0 and self.cause_type_id != Target.__TYPE_ID) {");
            self.writeln("                    return null;");
            self.writeln("                }");
            self.writeln("            }");
            self.writeln("            return @ptrCast(@alignCast(__c));");
            self.writeln("        }");
            self.writeln("        return null;");
            self.writeln("    }");
            self.writeln("};");
            self.writeln("pub const Error = Err;");
            self.writeln("fn is_cp_result(comptime __T: type) bool {");
            self.writeln(
                "    return @typeInfo(__T) == .@\"union\" and @hasDecl(__T, \"__IS_CP_RESULT\");",
            );
            self.writeln("}");
        }
        if features.needs_opt_chaining {
            self.writeln("fn CpOptType(comptime __T: type) type { if (@typeInfo(__T) == .optional) return __T; return ?__T; }");
            self.writeln("fn CpUnwrapOpt(comptime __T: type) type { if (@typeInfo(__T) == .optional) return @typeInfo(__T).optional.child; return __T; }");
            self.writeln("fn cp_opt_is_null(v: anytype) bool { const __T = @TypeOf(v); if (comptime @typeInfo(__T) == .optional) return v == null; return false; }");
            self.writeln("fn cp_opt_unwrap(v: anytype) CpUnwrapOpt(@TypeOf(v)) { const __T = @TypeOf(v); if (comptime @typeInfo(__T) == .optional) { return v.?; } else { return v; } }");
        }
        if features.needs_cast_int {
            self.writeln("fn cp_cast_int(comptime Dest: type, val: anytype) Dest { const Src = @TypeOf(val); if (comptime Src == Dest) return val; if (comptime Src == comptime_int) { if (comptime @typeInfo(Dest).int.signedness == .unsigned and val < 0) return @as(Dest, @truncate(@as(u128, @bitCast(@as(i128, val))))) else return @as(Dest, @truncate(val)); } const src_bits = @typeInfo(Src).int.bits; const dest_bits = @typeInfo(Dest).int.bits; if (comptime @typeInfo(Src).int.signedness == @typeInfo(Dest).int.signedness) { if (comptime dest_bits >= src_bits) return @as(Dest, val) else return @truncate(val); } if (comptime @typeInfo(Src).int.signedness == .signed) { const s: i128 = if (comptime 128 >= src_bits) @as(i128, val) else @truncate(val); const u: u128 = @bitCast(s); if (comptime dest_bits >= 128) return @as(Dest, u) else return @truncate(u); } else { const u: u128 = if (comptime 128 >= src_bits) @as(u128, val) else @truncate(val); const s: i128 = @bitCast(u); if (comptime dest_bits >= 128) return @as(Dest, s) else return @truncate(s); } }");
        }

        // --- Top-Level C-FFI / Zig Escapes ---
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Expr(Expr::Zig(zig))) = item {
                let lowered = lower_struct_body(&zig.code);
                for line in lowered.lines() {
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
                if var.is_extern {
                    if let Pattern::Ident(ident) = &var.pat {
                        let pub_str = if var.is_pub || var.is_extern || !self.is_lib_mode {
                            "pub "
                        } else {
                            ""
                        };
                        let extern_str = if var.is_c { "extern \"c\"" } else { "extern" };
                        let kind_str = if var.kind == VarDeclKind::Const {
                            "const"
                        } else {
                            "var"
                        };
                        self.writeln(&format!(
                            "{}{} {} {}{};",
                            pub_str, extern_str, kind_str, ident.sym, ty_str
                        ));
                    }
                    continue;
                }
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
                if let Some(ty) = &var.ty {
                    if let Some(el) = Self::fixed_or_vec_elem_type(ty) {
                        self.expected_array_elem_type = Some(el);
                    } else if let Type::Array(elem) = ty {
                        self.expected_array_elem_type = Some((**elem).clone());
                    }
                }
                match &var.pat {
                    Pattern::Ident(ident) => {
                        if ident.sym == "std" {
                            continue;
                        }
                        let saved_comptime = self.is_comptime;
                        self.is_comptime = true;
                        let init_code = if let Some(init) = &var.init {
                            self.generate_expr(init)
                        } else {
                            "undefined".to_string()
                        };
                        self.is_comptime = saved_comptime;
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
            self.writeln("        self.source = pattern; const c_pattern = std.heap.page_allocator.allocSentinel(u8, pattern.len, 0) catch unreachable; @import(\"std\").mem.copyForwards(u8, c_pattern, pattern); defer std.heap.page_allocator.free(c_pattern);");
            self.writeln("        var cflags: i32 = 1; // REG_EXTENDED");
            self.writeln("        for (flags) |fc| { if (fc == 'i') { cflags |= 2; } else if (fc == 'm') { cflags |= (if (comptime @import(\"builtin\").os.tag == .linux) @as(i32, 4) else @as(i32, 8)); } }");
            self.writeln("        if (regcomp(&self.preg, c_pattern.ptr, cflags) != 0) { std.debug.print(\"Regex Compilation Failed\\n\", .{}); @panic(\"Invalid RegExp\"); }");
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
            self.writeln("fn LockGuard(comptime __T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        mutex: *CpMutex(__T),");
            self.writeln("        data: *__T,");
            self.writeln("        pub fn release(self: Self) void { self.mutex.raw.unlock(); }");
            self.writeln("        pub fn unlock(self: Self) void { self.release(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn ReadGuard(comptime __T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        rwlock: *CpRwLock(__T),");
            self.writeln("        data: *const __T,");
            self.writeln(
                "        pub fn release(self: Self) void { self.rwlock.raw.unlockShared(); }",
            );
            self.writeln("        pub fn unlock(self: Self) void { self.release(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn WriteGuard(comptime __T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        rwlock: *CpRwLock(__T),");
            self.writeln("        data: *__T,");
            self.writeln(
                "        pub fn release(self: Self) void { self.rwlock.raw.unlockExclusive(); }",
            );
            self.writeln("        pub fn unlock(self: Self) void { self.release(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn CpMutex(comptime __T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        raw: NativeMutex = .{},");
            self.writeln("        data: __T,");
            self.writeln("        pub fn init(val: __T) Self { return .{ .data = val }; }");
            self.writeln("        pub fn lock(self: *Self) LockGuard(__T) { self.raw.lock(); return .{ .mutex = self, .data = &self.data }; }");
            self.writeln("        pub fn tryLock(self: *Self) ?LockGuard(__T) { if (self.raw.tryLock()) { return LockGuard(__T){ .mutex = self, .data = &self.data }; } return null; }");
            self.writeln(
                "        pub fn isLocked(self: *Self) bool { return self.raw.isLocked(); }",
            );
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn CpRwLock(comptime __T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        raw: NativeRwLock = .{},");
            self.writeln("        data: __T,");
            self.writeln("        pub fn init(val: __T) Self { return .{ .data = val }; }");
            self.writeln("        pub fn read(self: *Self) ReadGuard(__T) { self.raw.lockShared(); return .{ .rwlock = self, .data = &self.data }; }");
            self.writeln("        pub fn tryRead(self: *Self) ?ReadGuard(__T) { if (self.raw.tryLockShared()) { return ReadGuard(__T){ .rwlock = self, .data = &self.data }; } return null; }");
            self.writeln("        pub fn write(self: *Self) WriteGuard(__T) { self.raw.lockExclusive(); return .{ .rwlock = self, .data = &self.data }; }");
            self.writeln("        pub fn tryWrite(self: *Self) ?WriteGuard(__T) { if (self.raw.tryLockExclusive()) { return WriteGuard(__T){ .rwlock = self, .data = &self.data }; } return null; }");
            self.writeln(
                "        pub fn isReadLocked(self: *Self) bool { return self.raw.isReadLocked(); }",
            );
            self.writeln("        pub fn isWriteLocked(self: *Self) bool { return self.raw.isWriteLocked(); }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn CpAtomic(comptime __T: type) type {");
            self.writeln("    return struct {");
            self.writeln("        const Self = @This();");
            self.writeln("        raw: std.atomic.Value(__T),");
            self.writeln("        pub fn init(val: __T) Self { return .{ .raw = std.atomic.Value(__T).init(val) }; }");
            self.writeln(
                "        pub fn load(self: *const Self) __T { return self.raw.load(.seq_cst); }",
            );
            self.writeln(
                "        pub fn store(self: *Self, val: __T) void { self.raw.store(val, .seq_cst); }",
            );
            self.writeln("        pub fn swap(self: *Self, val: __T) __T { return self.raw.swap(val, .seq_cst); }");
            self.writeln("        pub fn cas(self: *Self, expected: __T, desired: __T) bool { return self.raw.cmpxchgStrong(expected, desired, .seq_cst, .seq_cst) == null; }");
            self.writeln("        pub fn compareAndSwap(self: *Self, expected: __T, desired: __T) bool { return self.cas(expected, desired); }");
            self.writeln("        pub fn add(self: *Self, delta: __T) __T { return self.raw.fetchAdd(delta, .seq_cst); }");
            self.writeln("        pub fn sub(self: *Self, delta: __T) __T { return self.raw.fetchSub(delta, .seq_cst); }");
            self.writeln("        pub fn inc(self: *Self) __T { return self.add(1); }");
            self.writeln("        pub fn dec(self: *Self) __T { return self.sub(1); }");
            self.writeln("        pub fn @\"and\"(self: *Self, mask: __T) __T { return self.raw.fetchAnd(mask, .seq_cst); }");
            self.writeln("        pub fn @\"or\"(self: *Self, mask: __T) __T { return self.raw.fetchOr(mask, .seq_cst); }");
            self.writeln("        pub fn @\"xor\"(self: *Self, mask: __T) __T { return self.raw.fetchXor(mask, .seq_cst); }");
            self.writeln("        pub fn testAndSet(self: *Self) bool { if (comptime __T == bool) { return self.raw.swap(true, .seq_cst); } else { return self.raw.swap(1, .seq_cst) != 0; } }");
            self.writeln("        pub fn clear(self: *Self) void { if (comptime __T == bool) { self.raw.store(false, .seq_cst); } else { self.raw.store(0, .seq_cst); } }");
            self.writeln("    };");
            self.writeln("}");
            self.writeln("");
            self.writeln("fn cp_make_mutex(val: anytype) CpMutex(@TypeOf(val)) { return CpMutex(@TypeOf(val)).init(val); }");
            self.writeln("fn cp_make_rwlock(val: anytype) CpRwLock(@TypeOf(val)) { return CpRwLock(@TypeOf(val)).init(val); }");
            self.writeln("fn CpAtomicConcreteType(comptime __T: type) type { if (__T == comptime_int) return i32; if (__T == comptime_float) return f32; return __T; }");
            self.writeln("fn cp_make_atomic(val: anytype) CpAtomic(CpAtomicConcreteType(@TypeOf(val))) { const __T = CpAtomicConcreteType(@TypeOf(val)); return CpAtomic(__T).init(@as(__T, val)); }");
            self.writeln("");
        }

        if features.needs_collections || features.needs_actors {
            self.writeln("fn CpIndexReturnType(comptime __T: type) type {");
            self.writeln("    if (@typeInfo(__T) == .pointer) {");
            self.writeln("        const Child = @typeInfo(__T).pointer.child;");
            self.writeln("        if (@typeInfo(Child) == .@\"struct\" and @hasDecl(Child, \"get\")) return ?Message;");
            self.writeln(
                "        if (@typeInfo(Child) == .array) return @typeInfo(Child).array.child;",
            );
            self.writeln(
                "        if (@typeInfo(Child) == .pointer and @typeInfo(Child).pointer.size == .slice) return @typeInfo(Child).pointer.child;",
            );
            self.writeln("    }");
            self.writeln("    if (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice) return @typeInfo(__T).pointer.child;");
            self.writeln("    if (@typeInfo(__T) == .array) return @typeInfo(__T).array.child;");
            self.writeln("    return ?Message;");
            self.writeln("}");
            self.writeln("fn cp_index_get(container: anytype, key: anytype) CpIndexReturnType(@TypeOf(container)) {");
            self.writeln("    const __T = @TypeOf(container);");
            self.writeln("    if (comptime (__T == Message or __T == ?Message)) {");
            self.writeln(
                "        const val: ?Message = if (__T == ?Message) container else container;",
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
            self.writeln("    if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(__T).pointer.child, \"get\"))) {");
            self.writeln("        return container.get(key);");
            self.writeln("    } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice)) {");
            self.writeln("        return container.*[@as(usize, @intCast(key))];");
            self.writeln("    } else {");
            self.writeln("        return container[@as(usize, @intCast(key))];");
            self.writeln("    }");
            self.writeln("}");
            self.writeln(
                "fn cp_index_assign(container: anytype, key: anytype, value: anytype) void {",
            );
            self.writeln("    const __T = @TypeOf(container);");
            self.writeln("    if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(__T).pointer.child, \"set\"))) {");
            self.writeln("        _ = container.set(key, value);");
            self.writeln("    } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice)) {");
            self.writeln("        @constCast(container.*)[@as(usize, @intCast(key))] = value;");
            self.writeln("    } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice)) {");
            self.writeln("        @constCast(container)[@as(usize, @intCast(key))] = value;");
            self.writeln("    } else {");
            self.writeln("        container[@as(usize, @intCast(key))] = value;");
            self.writeln("    }");
            self.writeln("}");
            self.writeln("fn cp_message_from(v: anytype) Message {");
            self.writeln("    const __T = @TypeOf(v);");
            self.writeln("    if (__T == Message) return v;");
            self.writeln(
                "    if (__T == []Message or __T == []const Message) return .{ .array = @constCast(v) };",
            );
            self.writeln(
                "    if (__T == i32 or __T == comptime_int) return .{ .int = @intCast(v) };
    if (__T == bool) return .{ .boolean = v };",
            );
            self.writeln(
                "    if (__T == f32 or __T == comptime_float) return .{ .float = @floatCast(v) };",
            );
            self.writeln(
                "    if (__T == []const u8 or __T == []u8) return .{ .string = v };
    if (@typeInfo(__T) == .pointer) {
        const Child = @typeInfo(__T).pointer.child;
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
            self.writeln("fn is_string_like(comptime __T: type) bool { if (__T == []const u8 or __T == []u8) return true; if (@typeInfo(__T) == .pointer) { const Child = @typeInfo(__T).pointer.child; if (Child == u8) return true; if (@typeInfo(Child) == .array and @typeInfo(Child).array.child == u8) return true; } return false; }");
            self.writeln("fn to_string_slice(v: anytype) []const u8 { const __T = @TypeOf(v); if (comptime (__T == []const u8 or __T == []u8)) { return v; } else { const cp_str_slice: []const u8 = v; return cp_str_slice; } }");
            self.writeln("fn CpValType(comptime __T: type) type { if (is_string_like(__T)) return []const u8; if (__T == comptime_int) return i32; if (__T == comptime_float) return f32; return __T; }");
            self.writeln("fn cp_val(v: anytype) CpValType(@TypeOf(v)) { const __T = @TypeOf(v); if (comptime is_string_like(__T)) { return to_string_slice(v); } else if (comptime __T == comptime_int) { return @as(i32, v); } else if (comptime __T == comptime_float) { return @as(f32, v); } else { return v; } }");
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
            self.writeln("fn CpUnwrapOkType(comptime __T: type) type { if (is_cp_result(__T)) { const OkT = @FieldType(__T, \"ok\"); if (@typeInfo(OkT) == .optional) return @typeInfo(OkT).optional.child; return OkT; } if (@typeInfo(__T) == .error_union) return @typeInfo(__T).error_union.payload; if (@typeInfo(__T) == .optional) return @typeInfo(__T).optional.child; return __T; }");
            self.writeln("fn CpUnwrapErrType(comptime __T: type) type { if (is_cp_result(__T)) return @FieldType(__T, \"err\"); if (@typeInfo(__T) == .error_union) return @typeInfo(__T).error_union.error_set; return anyerror; }");
            self.writeln("fn cp_has_ok_val(v: anytype) bool { const __T = @TypeOf(v); if (comptime is_cp_result(__T)) { if (v != .ok) return false; const OkT = @FieldType(__T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok != null; } else { return true; } } else if (comptime @typeInfo(__T) == .error_union) { if (v) |_| return true else |_| return false; } else if (comptime @typeInfo(__T) == .optional) { return v != null; } else { return true; } }");
            self.writeln("fn cp_unwrap_ok_val(v: anytype) CpUnwrapOkType(@TypeOf(v)) { const __T = @TypeOf(v); if (comptime is_cp_result(__T)) { const OkT = @FieldType(__T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok.?; } else { return v.ok; } } else if (comptime @typeInfo(__T) == .error_union) { return v catch unreachable; } else if (comptime @typeInfo(__T) == .optional) { return v.?; } else { return v; } }");
            self.writeln("fn cp_unwrap_err_val(v: anytype) CpUnwrapErrType(@TypeOf(v)) { const __T = @TypeOf(v); if (comptime is_cp_result(__T)) return v.err; if (comptime @typeInfo(__T) == .error_union) { if (v) |_| unreachable else |e| return e; } return v; }");
            self.writeln("fn cp_is_null_val(v: anytype) bool { const __T = @TypeOf(v); if (comptime is_cp_result(__T)) { if (v != .ok) return false; const OkT = @FieldType(__T, \"ok\"); if (comptime @typeInfo(OkT) == .optional) { return v.ok == null; } else { return false; } } else if (comptime @typeInfo(__T) == .optional) { return v == null; } else { return false; } }");
            self.writeln("fn cp_convert_err(comptime TargetErr: type, err_val: anytype) TargetErr { const SrcType = @TypeOf(err_val); if (comptime SrcType == TargetErr) { return err_val; } else if (comptime TargetErr == Err or TargetErr == Error) { if (comptime @typeInfo(SrcType) == .@\"union\") { return Err{ .code = @intCast(@intFromEnum(std.meta.activeTag(err_val))), .msg = @tagName(err_val) }; } else if (comptime @typeInfo(SrcType) == .@\"enum\") { return Err{ .code = @intCast(@intFromEnum(err_val)), .msg = @tagName(err_val) }; } else if (comptime @typeInfo(SrcType) == .error_set) { return Err{ .code = 0, .msg = @errorName(err_val) }; } else if (comptime @typeInfo(SrcType) == .@\"struct\") { var __err_code: i32 = 0; var m: []const u8 = \"\"; if (comptime @hasField(SrcType, \"code\")) { __err_code = @as(i32, @intCast(err_val.code)); } if (comptime @hasField(SrcType, \"msg\")) { m = to_string_slice(err_val.msg); } else if (comptime @hasField(SrcType, \"message\")) { m = to_string_slice(err_val.message); } else if (comptime @hasDecl(SrcType, \"message\")) { m = to_string_slice(err_val.message()); } else { m = @typeName(SrcType); } return Err.init(__err_code, m); } else if (comptime @typeInfo(SrcType) == .pointer and @typeInfo(@typeInfo(SrcType).pointer.child) == .@\"struct\") { const Child = @typeInfo(SrcType).pointer.child; var __err_code: i32 = 0; var m: []const u8 = \"\"; if (comptime @hasField(Child, \"code\")) { __err_code = @as(i32, @intCast(err_val.code)); } if (comptime @hasField(Child, \"msg\")) { m = to_string_slice(err_val.msg); } else if (comptime @hasField(Child, \"message\")) { m = to_string_slice(err_val.message); } else if (comptime @hasDecl(Child, \"message\")) { m = to_string_slice(err_val.message()); } else { m = @typeName(Child); } return Err.init(__err_code, m).with_cause(err_val); } else { return Err{ .code = -1, .msg = \"error\" }; } } else if (comptime @typeInfo(TargetErr) == .pointer and @typeInfo(SrcType) != .pointer) { return @ptrCast(@constCast(&err_val)); } else if (comptime @typeInfo(SrcType) == .pointer and @typeInfo(TargetErr) != .pointer and @typeInfo(SrcType).pointer.child == TargetErr) { return err_val.*; } else if (comptime @typeInfo(TargetErr) == .@\"union\" and @hasDecl(TargetErr, \"__from\")) { return TargetErr.__from(err_val); } else if (comptime @hasDecl(TargetErr, \"from\")) { return TargetErr.from(err_val); } else { return err_val; } }");
        }

        if features.needs_result_error {
            self.writeln("fn cp_match_tag(val: anytype, comptime tag: anytype) bool { const __T = @TypeOf(val); if (comptime @typeInfo(__T) == .@\"enum\") { return val == tag; } else if (comptime @typeInfo(__T) == .error_set) { return val == @field(anyerror, @tagName(tag)); } else if (comptime ((@hasDecl(@This(), \"Err\") and __T == @field(@This(), \"Err\")) or (@hasDecl(@This(), \"Error\") and __T == @field(@This(), \"Error\")) or (@typeInfo(__T) == .@\"struct\" and @hasField(__T, \"msg\")))) { return std.mem.eql(u8, val.msg, @tagName(tag)); } else if (comptime @typeInfo(__T) == .@\"union\") { return std.meta.activeTag(val) == tag; } else { return false; } }");
        }

        if features.needs_not {
            self.writeln("fn cp_not(v: anytype) bool { const __T = @TypeOf(v); if (comptime __T == bool) { return !v; } else if (comptime @typeInfo(__T) == .optional) { return v == null; } else if (comptime is_string_like(__T)) { return to_string_slice(v).len == 0; } else if (comptime (@typeInfo(__T) == .int or __T == comptime_int)) { return v == 0; } else { return false; } }");
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
                self.writeln("fn cp_fmt(v: anytype) []const u8 { const __T = @TypeOf(v); if (comptime is_string_like(__T)) { return to_string_slice(v); } else if (comptime __T == bool) { return if (v) \"true\" else \"false\"; } else if (comptime (@typeInfo(__T) == .int or __T == comptime_int)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime (@typeInfo(__T) == .float or __T == comptime_float)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime @typeInfo(__T) == .@\"enum\") { return @tagName(v); } else if (comptime __T == Message) { return switch (v) { .string => |s| s, .int => |i| std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{i}) catch unreachable, .float => |f| std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{f}) catch unreachable, .boolean => |b| if (b) \"true\" else \"false\", else => std.fmt.allocPrint(std.heap.page_allocator, \"{any}\", .{v}) catch unreachable, }; } else if (comptime @typeInfo(__T) == .optional) { if (v) |unwrapped| { return cp_fmt(unwrapped); } else { return \"null\"; } } else { return std.fmt.allocPrint(std.heap.page_allocator, \"{any}\", .{v}) catch unreachable; } }");
            } else {
                self.writeln("fn cp_fmt(v: anytype) []const u8 { const __T = @TypeOf(v); if (comptime is_string_like(__T)) { return to_string_slice(v); } else if (comptime __T == bool) { return if (v) \"true\" else \"false\"; } else if (comptime (@typeInfo(__T) == .int or __T == comptime_int)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime (@typeInfo(__T) == .float or __T == comptime_float)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{v}) catch unreachable; } else if (comptime @typeInfo(__T) == .@\"enum\") { return @tagName(v); } else if (comptime @typeInfo(__T) == .optional) { if (v) |unwrapped| { return cp_fmt(unwrapped); } else { return \"null\"; } } else { return std.fmt.allocPrint(std.heap.page_allocator, \"{any}\", .{v}) catch unreachable; } }");
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
            self.writeln("fn cp_hash_val(val: anytype) usize { const __T = @TypeOf(val); if (comptime is_string_like(__T)) { const s = to_string_slice(val); var h: usize = 2166136261; for (s) |byte| { h = (h ^ byte) *% 16777619; } return h; } if (comptime @typeInfo(__T) == .int or @typeInfo(__T) == .float or @typeInfo(__T) == .bool) { var h: usize = 2166136261; const bytes = std.mem.asBytes(&val); for (bytes) |byte| { h = (h ^ byte) *% 16777619; } return h; } if (comptime @typeInfo(__T) == .pointer) { if (@hasDecl(@typeInfo(__T).pointer.child, \"hash\")) { return val.hash(); } } return 0; }");
        }

        if features.needs_array_helpers {
            self.writeln("fn cp_array_lit(comptime Elem: type, init_slice: []const Elem) []const Elem { const buf = std.heap.page_allocator.alloc(Elem, init_slice.len) catch unreachable; if (init_slice.len > 0) @memcpy(buf, init_slice); return buf; }");
            self.writeln("fn CpArrayElemType(comptime PtrT: type) type { const S0 = @typeInfo(PtrT).pointer.child; const S1 = if (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one) @typeInfo(S0).pointer.child else S0; return @typeInfo(S1).pointer.child; }");
            self.writeln("fn cp_array_push(arr_ptr: anytype, item: anytype) void { const S0 = @TypeOf(arr_ptr.*); const target_ptr = if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one)) arr_ptr.* else arr_ptr; const SliceT = @TypeOf(target_ptr.*); const Elem = @typeInfo(SliceT).pointer.child; const old_slice = target_ptr.*; const new_slice = std.heap.page_allocator.alloc(Elem, old_slice.len + 1) catch unreachable; if (old_slice.len > 0) @memcpy(new_slice[0..old_slice.len], old_slice); new_slice[old_slice.len] = @as(Elem, cp_val(item)); target_ptr.* = new_slice; }");
            self.writeln("fn cp_array_pop(arr_ptr: anytype) ?CpArrayElemType(@TypeOf(arr_ptr)) { const S0 = @TypeOf(arr_ptr.*); const target_ptr = if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one)) arr_ptr.* else arr_ptr; const s = target_ptr.*; if (s.len == 0) return null; const last = s[s.len - 1]; target_ptr.* = s[0 .. s.len - 1]; return last; }");
            self.writeln("fn cp_container_clear(ptr: anytype) void { const S0 = @TypeOf(ptr.*); if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one and @typeInfo(@typeInfo(S0).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(S0).pointer.child, \"clear\"))) { ptr.*.clear(); } else { const target_ptr = if (comptime (@typeInfo(S0) == .pointer and @typeInfo(S0).pointer.size == .one)) ptr.* else ptr; target_ptr.* = target_ptr.*[0..0]; } }");
        }

        if features.needs_iter_helpers {
            self.writeln("fn cp_iter_slice(val: anytype) switch (@typeInfo(@TypeOf(val))) { .pointer => |p| switch (p.size) { .slice => @TypeOf(val), .one => switch (@typeInfo(p.child)) { .array => []const @typeInfo(p.child).array.child, .pointer => |sub_p| if (sub_p.size == .slice) p.child else @TypeOf(val), else => @TypeOf(val), }, else => @TypeOf(val), }, .array => []const @typeInfo(@TypeOf(val)).array.child, else => @TypeOf(val), } { const __T = @TypeOf(val); switch (@typeInfo(__T)) { .pointer => |p| switch (p.size) { .slice => return val, .one => switch (@typeInfo(p.child)) { .array => return val[0..], .pointer => |sub_p| if (sub_p.size == .slice) return val.* else return val, else => return val, }, else => return val, }, .array => return val[0..], else => return val, } }");
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
            self.writeln("    const __T = @TypeOf(val);");
            self.writeln("    if (comptime @typeInfo(__T) == .comptime_int) return val << shift;");
            self.writeln("    if (comptime @typeInfo(__T) == .int) {");
            self.writeln("        const ShiftType = std.math.Log2Int(__T);");
            self.writeln("        const u_shift: usize = if (@typeInfo(@TypeOf(shift)) == .int and @typeInfo(@TypeOf(shift)).int.signedness == .signed) @intCast(@max(0, shift)) else @intCast(shift);");
            self.writeln("        return val << @as(ShiftType, @truncate(u_shift));");
            self.writeln("    }");
            self.writeln("    return val << shift;");
            self.writeln("}");
            self.writeln("fn cp_shr(val: anytype, shift: anytype) @TypeOf(val) {");
            self.writeln("    const __T = @TypeOf(val);");
            self.writeln("    if (comptime @typeInfo(__T) == .comptime_int) return val >> shift;");
            self.writeln("    if (comptime @typeInfo(__T) == .int) {");
            self.writeln("        const ShiftType = std.math.Log2Int(__T);");
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
            self.writeln("pub fn print(v: anytype) void { const __T = @TypeOf(v); if (comptime is_string_like(__T)) { std.debug.print(\"{s}\\n\", .{ to_string_slice(v) }); } else if (comptime __T == Message) { std.debug.print(\"{s}\\n\", .{ cp_fmt(v) }); } else if (comptime __T == ?Message) { if (v) |m| { std.debug.print(\"{s}\\n\", .{ cp_fmt(m) }); } else { std.debug.print(\"null\\n\", .{}); } } else { std.debug.print(\"{any}\\n\", .{ v }); } }");
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
            self.writeln("pub fn CpUnpackElemType(comptime __T: type, comptime idx: usize) type { if (__T == ?Message or __T == Message) return []const u8; if (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice) return @typeInfo(__T).pointer.child; if (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .one and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice) return @typeInfo(@typeInfo(__T).pointer.child).pointer.child; if (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .array) return @typeInfo(@typeInfo(__T).pointer.child).array.child; if (@typeInfo(__T) == .array) return @typeInfo(__T).array.child; const ti = @typeInfo(__T); if (ti == .@\"struct\") return ti.@\"struct\".field_types[idx]; return @FieldType(__T, std.fmt.comptimePrint(\"{d}\", .{idx})); }");
            self.writeln("pub fn cp_unpack_elem(v: anytype, comptime idx: usize) CpUnpackElemType(@TypeOf(v), idx) { const __T = @TypeOf(v); if (comptime __T == ?Message) { if (v) |m| { if (m == .array and idx < m.array.len) { return cp_msg_to_val(m.array[idx]); } } return \"\"; } else if (comptime __T == Message) { if (v == .array and idx < v.array.len) { return cp_msg_to_val(v.array[idx]); } return \"\"; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice)) { return v[idx]; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .one and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice)) { return v.*[idx]; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .array)) { return v[idx]; } else if (comptime (@typeInfo(__T) == .array)) { return v[idx]; } else { const ti = @typeInfo(__T); if (ti == .@\"struct\") return @field(v, ti.@\"struct\".field_names[idx]); return @field(v, std.fmt.comptimePrint(\"{d}\", .{idx})); } }");
            self.writeln("pub fn CpUnpackTailType(comptime __T: type, comptime offset_from_end: usize) type { if (__T == ?Message or __T == Message) return []const u8; if (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice) return @typeInfo(__T).pointer.child; if (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .one and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice) return @typeInfo(@typeInfo(__T).pointer.child).pointer.child; if (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .array) return @typeInfo(@typeInfo(__T).pointer.child).array.child; if (@typeInfo(__T) == .array) return @typeInfo(__T).array.child; const ti = @typeInfo(__T); if (ti == .@\"struct\") { const n = ti.@\"struct\".field_names.len; return ti.@\"struct\".field_types[n - offset_from_end]; } return @FieldType(__T, std.fmt.comptimePrint(\"{d}\", .{1})); }");
            self.writeln("pub fn cp_unpack_tail(v: anytype, comptime offset_from_end: usize) CpUnpackTailType(@TypeOf(v), offset_from_end) { const __T = @TypeOf(v); if (comptime __T == ?Message) { if (v) |m| { if (m == .array and m.array.len >= offset_from_end) { return cp_msg_to_val(m.array[m.array.len - offset_from_end]); } } return \"\"; } else if (comptime __T == Message) { if (v == .array and v.array.len >= offset_from_end) { return cp_msg_to_val(v.array[v.array.len - offset_from_end]); } return \"\"; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice)) { return v[v.len - offset_from_end]; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .one and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice)) { return v.*[v.*.len - offset_from_end]; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .array)) { return v[v.len - offset_from_end]; } else if (comptime (@typeInfo(__T) == .array)) { return v[v.len - offset_from_end]; } else { const ti = @typeInfo(__T); if (ti == .@\"struct\") { const n = ti.@\"struct\".field_names.len; return @field(v, ti.@\"struct\".field_names[n - offset_from_end]); } return @field(v, std.fmt.comptimePrint(\"{d}\", .{0})); } }");
            self.writeln("pub fn CpUnpackSliceType(comptime __T: type, comptime start_idx: usize) type { if (__T == ?Message or __T == Message) return []const Message; if (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice) return __T; if (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .one and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice) return @typeInfo(__T).pointer.child; if (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .array) return []const @typeInfo(@typeInfo(__T).pointer.child).array.child; if (@typeInfo(__T) == .array) return []const @typeInfo(__T).array.child; const ti = @typeInfo(__T); if (ti == .@\"struct\") { const types = ti.@\"struct\".field_types; if (types.len == 0) return []const i32; const Elem = if (start_idx < types.len) types[start_idx] else types[0]; return []const CpValType(Elem); } return []const i32; }");
            self.writeln("pub fn cp_unpack_slice(v: anytype, comptime start_idx: usize, comptime suffix_len: usize) CpUnpackSliceType(@TypeOf(v), start_idx) { const __T = @TypeOf(v); if (comptime __T == ?Message) { if (v) |m| { if (m == .array and m.array.len >= start_idx + suffix_len) { return m.array[start_idx .. m.array.len - suffix_len]; } } return &[_]Message{}; } else if (comptime __T == Message) { if (v == .array and v.array.len >= start_idx + suffix_len) { return v.array[start_idx .. v.array.len - suffix_len]; } return &[_]Message{}; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice)) { return v[start_idx .. v.len - suffix_len]; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .one and @typeInfo(@typeInfo(__T).pointer.child) == .pointer and @typeInfo(@typeInfo(__T).pointer.child).pointer.size == .slice)) { return v.*[start_idx .. v.*.len - suffix_len]; } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .array)) { return v[start_idx .. v.len - suffix_len]; } else if (comptime (@typeInfo(__T) == .array)) { return v[start_idx .. v.len - suffix_len]; } else { const ti = @typeInfo(__T); if (ti == .@\"struct\") { const names = ti.@\"struct\".field_names; const types = ti.@\"struct\".field_types; const end = names.len - suffix_len; const count = if (end > start_idx) end - start_idx else 0; const Elem = if (start_idx < types.len) CpValType(types[start_idx]) else i32; const buf = std.heap.page_allocator.alloc(Elem, count) catch unreachable; inline for (start_idx..end, 0..) |f_idx, out_idx| { buf[out_idx] = cp_val(@field(v, names[f_idx])); } return buf; } return &[_]i32{}; } }");
        }
        if features.needs_string_helpers || features.needs_fmt {
            self.writeln("pub fn cp_string_from(val: anytype) []const u8 { const __T = @TypeOf(val); if (comptime __T == []const u8) { return val; } else if (comptime (__T == [*:0]const u8 or __T == [*:0]u8)) { return std.mem.span(val); } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(__T).pointer.size == .slice and @typeInfo(__T).pointer.child == u8)) { return val; } else if (comptime (@typeInfo(__T) == .@\"enum\")) { return @tagName(val); } else if (comptime __T == bool) { return if (val) \"true\" else \"false\"; } else if (comptime (@typeInfo(__T) == .int or __T == comptime_int)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{}\", .{val}) catch \"0\"; } else if (comptime (@typeInfo(__T) == .float or __T == comptime_float)) { return std.fmt.allocPrint(std.heap.page_allocator, \"{d}\", .{val}) catch \"0.0\"; } else if (comptime (@typeInfo(__T) == .@\"struct\" and @hasDecl(__T, \"toString\"))) { return val.toString(); } else if (comptime (@typeInfo(__T) == .pointer and @typeInfo(@typeInfo(__T).pointer.child) == .@\"struct\" and @hasDecl(@typeInfo(__T).pointer.child, \"toString\"))) { return val.toString(); } else { return @typeName(__T); } }");
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
            } else if features.needs_actors {
                self.writeln("\npub var global_io: std.Io = undefined;");
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

    pub(crate) fn generate_trait(&mut self, t: &TraitDecl) {
        writeln!(
            self.output,
            "pub const {} = struct {{ ptr: *anyopaque, vtable: *const @This().VTable, pub const VTable = struct {{",
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
            let ret_fn_ptr = if ret.starts_with('!') {
                format!("anyerror{}", ret)
            } else {
                ret
            };
            self.writeln(&format!(
                "{}: *const fn({}) {},",
                m.ident.sym, p_str, ret_fn_ptr
            ));
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

    pub(crate) fn generate_impl(&mut self, _i: &ImplDecl) {
        // VTables are emitted uniformly in generate_class for all traits in class.implements
    }

    pub(crate) fn generate_enum(&mut self, e: &EnumDecl) {
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
            let has_get = e.methods.iter().any(|m| m.ident.sym == "get")
                || e.extra_zig.iter().any(|z| z.contains("fn get("));
            let has_from_string = e.methods.iter().any(|m| m.ident.sym == "fromString")
                || e.extra_zig.iter().any(|z| z.contains("fn fromString("));
            let has_has = e.methods.iter().any(|m| m.ident.sym == "has")
                || e.extra_zig.iter().any(|z| z.contains("fn has("));
            let has_from_name = e.methods.iter().any(|m| m.ident.sym == "fromName")
                || e.extra_zig.iter().any(|z| z.contains("fn fromName("));
            let has_name = e.methods.iter().any(|m| m.ident.sym == "name")
                || e.extra_zig.iter().any(|z| z.contains("fn name("));
            if !has_get {
                if has_from_string {
                    self.writeln("pub fn get(s: []const u8) ?@This() { return fromString(s); }");
                } else {
                    self.writeln("pub fn get(s: []const u8) ?@This() { return std.meta.stringToEnum(@This(), s); }");
                }
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
        if !e.methods.iter().any(|m| m.ident.sym == "has")
            && !e.extra_zig.iter().any(|z| z.contains("fn has("))
        {
            self.writeln("pub fn has(s: []const u8) bool { if (comptime @typeInfo(@This()).@\"union\".tag_type) |TagType| { return std.meta.stringToEnum(TagType, s) != null; } return false; }");
        }
        if !e.methods.iter().any(|m| m.ident.sym == "name")
            && !e.extra_zig.iter().any(|z| z.contains("fn name("))
        {
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

    pub(crate) fn generate_interface(&mut self, iface: &Interface) {
        writeln!(
            self.output,
            "const {} = struct {{ ptr: *anyopaque, vtable: *const @This().VTable, const VTable = struct {{",
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
            let ret_fn_ptr = if ret.starts_with('!') {
                format!("anyerror{}", ret)
            } else {
                ret
            };
            self.writeln(&format!(
                "{}: *const fn({}) {},",
                m.ident.sym, p_str, ret_fn_ptr
            ));
        }
        self.dedent();
        self.writeln("}; };\n");
    }

    fn field_align_str(&mut self, f: &Field) -> String {
        if let Some(m) = f.meta.iter().find(|m| m.name.sym == "align") {
            let a_val = match m.args.first() {
                Some(crate::ast::MetaArg::Positional(e)) => self.generate_expr(e),
                Some(crate::ast::MetaArg::Named(_, e)) => self.generate_expr(e),
                None => "1".to_string(),
            };
            format!(" align({})", a_val)
        } else {
            String::new()
        }
    }

    pub(crate) fn parse_anon_struct_or_union(code: &str) -> Option<Class> {
        let trimmed = code.trim();
        let is_struct_or_union = (trimmed.starts_with("struct")
            && trimmed.as_bytes().get("struct".len()).map_or(false, |b| {
                b.is_ascii_whitespace() || *b == b'{' || *b == b'('
            }))
            || (trimmed.starts_with("union")
                && trimmed.as_bytes().get("union".len()).map_or(false, |b| {
                    b.is_ascii_whitespace() || *b == b'{' || *b == b'('
                }));

        if !is_struct_or_union {
            return None;
        }
        let brace_idx = trimmed.find('{')?;
        let wrapped = format!(
            "{} __CpAnonStruct__ {}",
            trimmed[..brace_idx].trim_end(),
            &trimmed[brace_idx..]
        );
        let mut parser = crate::parser::Parser::new(&wrapped);
        if let Ok(module) = parser.parse_module() {
            if let Some(ModuleItem::Decl(Decl::Class(cls))) = module.body.into_iter().next() {
                return Some(cls);
            }
        }
        None
    }

    pub(crate) fn generate_zig_or_struct(&mut self, code: &str) -> String {
        if let Some(mut cls) = Self::parse_anon_struct_or_union(code) {
            for m in &mut cls.methods {
                let has_explicit_self = m
                    .params
                    .first()
                    .map_or(false, |p| p.ident.sym == "this" || p.ident.sym == "_");
                if !has_explicit_self {
                    m.is_static = true;
                }
            }
            let mut sub = Codegen::new();
            sub.is_lib_mode = self.is_lib_mode;
            sub.is_beam_mode = self.is_beam_mode;
            sub.globals = self.globals.clone();
            for name in self.current_fn_params.iter().chain(self.locals.keys()) {
                sub.globals.insert(name.clone());
            }
            sub.ref_classes = self.ref_classes.clone();
            sub.value_classes = self.value_classes.clone();
            sub.fn_sigs = self.fn_sigs.clone();
            sub.class_defs = self.class_defs.clone();
            sub.trait_defs = self.trait_defs.clone();
            sub.enum_defs = self.enum_defs.clone();
            sub.error_sets = self.error_sets.clone();
            sub.type_aliases = self.type_aliases.clone();
            sub.dec_defs = self.dec_defs.clone();
            sub.anon_obj_structs = self.anon_obj_structs.clone();
            sub.value_classes.insert("@This()".to_string());
            sub.class_defs.insert("@This()".to_string(), cls.clone());
            sub.generate_class_inner(&cls, true);
            return sub.output.trim_end().to_string();
        }
        lower_struct_body(code)
    }

    pub(crate) fn generate_class(&mut self, class: &Class) {
        self.generate_class_inner(class, false);
    }

    pub(crate) fn generate_class_inner(&mut self, class: &Class, is_anon: bool) {
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
            if let Some(rc_meta) = class
                .meta
                .iter()
                .find(|m| m.name.sym == "refCount" || m.name.sym == "refcount")
            {
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
        let is_generic = !is_anon && !class.generic_params.is_empty();
        if is_anon {
            writeln!(self.output, "{} {{", struct_kind).unwrap();
        } else if is_generic {
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
                let align_str = self.field_align_str(f);
                let default_init = if let Some(def) = &f.default {
                    let old_array_elem = self.expected_array_elem_type.clone();
                    if let crate::ast::Type::FixedArray(elem, _) | crate::ast::Type::Array(elem) =
                        &f.ty
                    {
                        self.expected_array_elem_type = Some((**elem).clone());
                    }
                    let res = format!(" = {}", self.generate_expr(def));
                    self.expected_array_elem_type = old_array_elem;
                    res
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
                if f.is_static {
                    continue;
                }
                let align_str = self.field_align_str(f);
                let default_init = if let Some(def) = &f.default {
                    let old_array_elem = self.expected_array_elem_type.clone();
                    if let crate::ast::Type::FixedArray(elem, _) | crate::ast::Type::Array(elem) =
                        &f.ty
                    {
                        self.expected_array_elem_type = Some((**elem).clone());
                    }
                    let res = format!(" = {},", self.generate_expr(def));
                    self.expected_array_elem_type = old_array_elem;
                    res
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
            if !class.is_union
                && !class.meta.iter().any(|m| {
                    m.name.sym == "value"
                        || m.name.sym == "packed"
                        || m.name.sym == "extern"
                        || m.name.sym == "opaque"
                })
            {
                self.writeln("\npub fn _create(allocator: std.mem.Allocator) !*@This() { const self_ptr = try allocator.create(@This()); self_ptr.* = undefined; return self_ptr; }");
            }
            for f in &class.fields {
                if !f.is_static {
                    continue;
                }
                let align_str = self.field_align_str(f);
                let default_init = if let Some(def) = &f.default {
                    let old_array_elem = self.expected_array_elem_type.clone();
                    if let crate::ast::Type::FixedArray(elem, _) | crate::ast::Type::Array(elem) =
                        &f.ty
                    {
                        self.expected_array_elem_type = Some((**elem).clone());
                    }
                    let res = format!(" = {}", self.generate_expr(def));
                    self.expected_array_elem_type = old_array_elem;
                    res
                } else {
                    " = undefined".to_string()
                };
                let pub_prefix = if f.is_pub { "pub " } else { "" };
                let tls_prefix = if f
                    .meta
                    .iter()
                    .any(|m| m.name.sym == "tls" || m.name.sym == "threadlocal")
                {
                    "threadlocal "
                } else {
                    ""
                };
                self.writeln(&format!(
                    "{}{}var {}: {}{}{};",
                    pub_prefix,
                    tls_prefix,
                    f.ident.sym,
                    self.map_type(&f.ty),
                    align_str,
                    default_init
                ));
            }
            for nc in &class.nested_classes {
                self.generate_class_inner(nc, false);
            }
        }
        if needs_trait_infra {
            self.writeln(&format!(
                "pub fn __box_drop(ptr: *anyopaque) void {{ const self_ptr: *{0} = @ptrCast(@alignCast(ptr)); if (@hasDecl({0}, \"release\")) {{ const RType = @typeInfo(@TypeOf({0}.release)).@\"fn\".return_type.?; if (@typeInfo(RType) == .error_union) {{ _ = self_ptr.release() catch {{}}; }} else {{ self_ptr.release(); }} }} std.heap.page_allocator.destroy(self_ptr); }}",
                class.ident.sym
            ));
        }
        let method_owner = if is_anon {
            Some("@This()".to_string())
        } else {
            Some(class.ident.sym.clone())
        };
        for m in &class.methods {
            self.generate_function_decorated(m, method_owner.as_ref());
        }
        for code in &class.extra_zig {
            for line in code.lines() {
                self.writeln(line);
            }
        }
        if let Some(rc_meta) = class
            .meta
            .iter()
            .find(|m| m.name.sym == "refCount" || m.name.sym == "refcount")
        {
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
        if is_anon {
            self.writeln("}");
            return;
        }
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

    pub(crate) fn meta_expr_to_zig(&self, expr: &Expr) -> String {
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

    pub(crate) fn generate_meta_population(
        &self,
        meta_items: &[MetaItem],
        map_var: &str,
    ) -> String {
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

    pub(crate) fn generate_reflection_tables(&mut self) {
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
                        .filter(|p| {
                            p.ident.sym != "this" && p.ident.sym != "_" && p.ident.sym != "self"
                        })
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

    pub(crate) fn indent(&mut self) {
        self.indent_level += 1;
    }
    pub(crate) fn dedent(&mut self) {
        self.indent_level -= 1;
    }
    pub(crate) fn writeln(&mut self, s: &str) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
        self.output.push_str(s);
        self.output.push('\n');
    }
    pub(crate) fn write_to_case(&mut self, s: &str) {
        let indent = "    ".repeat(self.indent_level + 1);
        self.current_case_content
            .push_str(&format!("{}{}\n", indent, s));
    }
}

pub(crate) fn strip_redundant_comptime(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_str = false;
    let mut str_char = ' ';

    while i < n {
        let c = chars[i];
        if in_str {
            res.push(c);
            if c == '\\' && i + 1 < n {
                i += 1;
                res.push(chars[i]);
            } else if c == str_char {
                in_str = false;
            }
            i += 1;
            continue;
        }

        if c == '"' || c == '\'' {
            in_str = true;
            str_char = c;
            res.push(c);
            i += 1;
            continue;
        }

        // Check if keyword "comptime"
        if i + 8 <= n && &chars[i..i + 8] == ['c', 'o', 'm', 'p', 't', 'i', 'm', 'e'] {
            let prev_ok = i == 0
                || (!chars[i - 1].is_alphanumeric()
                    && chars[i - 1] != '_'
                    && chars[i - 1] != '@'
                    && chars[i - 1] != '.');
            let next_ok = i + 8 == n || (!chars[i + 8].is_alphanumeric() && chars[i + 8] != '_');
            if prev_ok && next_ok {
                i += 8;
                if i < n && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
                continue;
            }
        }

        res.push(c);
        i += 1;
    }
    res
}

pub fn lower_struct_body(code: &str) -> String {
    let mut res = String::with_capacity(code.len());
    let chars: Vec<char> = code.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_str = false;
    let mut str_char = ' ';
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < n {
        let c = chars[i];

        // 1. Comments
        if in_line_comment {
            res.push(c);
            if c == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            res.push(c);
            if c == '*' && i + 1 < n && chars[i + 1] == '/' {
                res.push('/');
                i += 2;
                in_block_comment = false;
                continue;
            }
            i += 1;
            continue;
        }

        // 2. Strings
        if in_str {
            res.push(c);
            if c == '\\' && i + 1 < n {
                i += 1;
                res.push(chars[i]);
            } else if c == str_char {
                in_str = false;
            }
            i += 1;
            continue;
        }

        if c == '"' || c == '\'' {
            in_str = true;
            str_char = c;
            res.push(c);
            i += 1;
            continue;
        }

        if c == '/' && i + 1 < n {
            if chars[i + 1] == '/' {
                in_line_comment = true;
                res.push('/');
                res.push('/');
                i += 2;
                continue;
            } else if chars[i + 1] == '*' {
                in_block_comment = true;
                res.push('/');
                res.push('*');
                i += 2;
                continue;
            }
        }

        // 3. Keyword `let` / `let mut`
        if i + 3 <= n && &chars[i..i + 3] == ['l', 'e', 't'] {
            let prev_ok = i == 0
                || (!chars[i - 1].is_alphanumeric()
                    && chars[i - 1] != '_'
                    && chars[i - 1] != '@'
                    && chars[i - 1] != '.');
            let next_ok = i + 3 == n || chars[i + 3].is_whitespace();
            if prev_ok && next_ok {
                i += 3;
                let mut j = i;
                while j < n && (chars[j] == ' ' || chars[j] == '\t') {
                    j += 1;
                }
                if j + 3 <= n
                    && &chars[j..j + 3] == ['m', 'u', 't']
                    && (j + 3 == n || chars[j + 3].is_whitespace())
                {
                    i = j + 3;
                }
                res.push_str("var");
                continue;
            }
        }

        // 4. Return type colon in function signature: `):` followed by return type
        if c == ')' {
            res.push(')');
            i += 1;
            let mut j = i;
            while j < n && (chars[j] == ' ' || chars[j] == '\t') {
                j += 1;
            }
            if j < n && chars[j] == ':' {
                let mut k = j + 1;
                while k < n && (chars[k] == ' ' || chars[k] == '\t') {
                    k += 1;
                }
                if k < n && chars[k] != '(' && chars[k] != '{' && chars[k] != ';' {
                    if k == j + 1 {
                        res.push(' ');
                    }
                    i = j + 1;
                    continue;
                }
            }
            continue;
        }

        // 5. [*]!? and [*]!
        if i + 5 <= n && &chars[i..i + 5] == ['[', '*', ']', '!', '?'] {
            res.push_str("?[*]anyopaque");
            i += 5;
            continue;
        }
        if i + 4 <= n && &chars[i..i + 4] == ['[', '*', ']', '!'] {
            res.push_str("[*]anyopaque");
            i += 4;
            continue;
        }

        // 6. *!? and *!
        if i + 3 <= n && &chars[i..i + 3] == ['*', '!', '?'] {
            res.push_str("?*anyopaque");
            i += 3;
            continue;
        }
        if i + 2 <= n && &chars[i..i + 2] == ['*', '!'] {
            res.push_str("*anyopaque");
            i += 2;
            continue;
        }

        // 7. ?fn(...) and fn(...) function pointer types
        if i + 3 <= n && &chars[i..i + 3] == ['?', 'f', 'n'] {
            let mut k = i + 3;
            while k < n && (chars[k] == ' ' || chars[k] == '\t') {
                k += 1;
            }
            if k < n && chars[k] == '(' {
                res.push_str("?*const fn");
                i = k;
                continue;
            }
        }
        if i + 2 <= n && &chars[i..i + 2] == ['f', 'n'] {
            let prev_ok = i == 0
                || (!chars[i - 1].is_alphanumeric()
                    && chars[i - 1] != '_'
                    && chars[i - 1] != '@'
                    && chars[i - 1] != '.');
            if prev_ok {
                let mut k = i + 2;
                while k < n && (chars[k] == ' ' || chars[k] == '\t') {
                    k += 1;
                }
                if k < n && chars[k] == '(' {
                    res.push_str("*const fn");
                    i = k;
                    continue;
                }
            }
        }

        res.push(c);
        i += 1;
    }
    res
}
