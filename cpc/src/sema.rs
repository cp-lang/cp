use crate::ast::*;
use crate::diagnostic::{find_closest_match, Diagnostic, STANDARD_BUILTIN_IMPORTS};
use crate::Span;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemaError {
    #[error("Diagnostic: {0:?}")]
    Diagnostic(Diagnostic),
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
    #[error("Compile Error: Decorator '@{0}' can only be applied to {1}, but was applied to {2} at {3:?}")]
    DecInvalidTarget(String, String, String, Span),
    #[error(
        "Compile Error: Macro cycle detected: '@{0}' recursively invoked on class '{1}' at {2:?}"
    )]
    MacroCycleDetected(String, String, Span),
    #[error("Compile Error: Macro recursion limit (128) exceeded in '@{0}' at {1:?}")]
    MacroRecursionLimitExceeded(String, Span),
    #[error(
        "Compile Error: Macro '@{0}' can only be applied to {1}, but was applied to {2} at {3:?}"
    )]
    MacroInvalidTarget(String, String, String, Span),
    #[error("Compile Error: Type-directed macro '{0}' cannot create new top-level types (classes or structs) at {1:?}")]
    MacroProhibitedTypeCreation(String, Span),
    #[error("Compile Error: Macro hygiene violation: generated code from '@{0}' cannot carry macro annotations at {1:?}")]
    MacroHygieneViolation(String, Span),
    #[error("Compile Error: Pipeline placeholder '.' can only be used on the right-hand side of a pipeline ('|>') operator at {0:?}")]
    PipePlaceholderOutsidePipeline(Span),
    #[error(
        "Compile Error: Cannot mutate '{0}' while it is borrowed in 'for ... of' loop at {1:?}"
    )]
    MutateWhileBorrowedInFor(String, Span),
    #[error("Compile Error: {0} at {1:?}")]
    UnsafeRequired(String, Span),
    #[error("Compile Error: {0} at {1:?}")]
    InvalidCast(String, Span),
    #[error("Compile Error: Cannot redeclare block-scoped variable '{0}' at {1:?}")]
    DuplicateVarDecl(String, Span),
    #[error(
        "Compile Error: Destructuring pattern cannot contain multiple '...' rest elements at {0:?}"
    )]
    MultipleRestInPattern(Span),
    #[error("Compile Error: {0} at {1:?}")]
    PrivateAccess(String, Span),
    #[error("Compile Error: Cannot instantiate static class or struct '{0}' at {1:?}")]
    CannotInstantiateStatic(String, Span),
    #[error("Compile Error: {0} at {1:?}")]
    InvalidPointerArithmetic(String, Span),
    #[error(
        "Compile Error: Index {index} is out of bounds for buffer of length {len} at {span:?}"
    )]
    CompileTimeIndexOutOfBounds { index: i64, len: usize, span: Span },
    #[error("Compile Error: Negative index {index} at {span:?}")]
    NegativeIndex { index: i64, span: Span },
    #[error("Compile Error: Mutation via unbounded raw pointer or cstr '{0}' requires 'unsafe' block at {1:?}")]
    UnboundedRawPointerMutation(String, Span),
    #[error(
        "Compile Error: Type '{type_name}' does not satisfy trait bound '{trait_name}' at {span:?}"
    )]
    TraitBoundNotSatisfied {
        type_name: String,
        trait_name: String,
        missing_method: Option<String>,
        span: Span,
    },
    #[error(
        "Compile Error: Class '{class_name}' does not satisfy trait '{trait_name}' at {span:?}"
    )]
    TraitMissingMethod {
        class_name: String,
        trait_name: String,
        missing_method: String,
        span: Span,
    },
}

impl SemaError {
    pub fn to_diagnostic(&self, known_symbols: &[String]) -> Diagnostic {
        match self {
            SemaError::Diagnostic(d) => d.clone(),
            SemaError::TraitBoundNotSatisfied {
                type_name,
                trait_name,
                missing_method,
                span,
            } => {
                let (cause, help, suggestion) = if let Some(method) = missing_method {
                    (
                        format!(
                            "type '{}' is missing required method '{}' from trait '{}'.",
                            type_name, method, trait_name
                        ),
                        format!(
                            "trait '{}' requires method '{}'. Ensure '{}' implements it.",
                            trait_name, method, type_name
                        ),
                        format!(
                            "implement method '{}' on '{}':\n  fn {}(...): void {{ ... }}",
                            method, type_name, method
                        ),
                    )
                } else {
                    (
                        format!(
                            "type '{}' does not satisfy trait bound '{}'.",
                            type_name, trait_name
                        ),
                        format!(
                            "generic parameter requires a type satisfying trait '{}'.",
                            trait_name
                        ),
                        format!(
                            "pass an argument whose type satisfies trait '{}'.",
                            trait_name
                        ),
                    )
                };
                Diagnostic::new(
                    "E0301",
                    format!(
                        "type '{}' does not satisfy trait bound '{}'",
                        type_name, trait_name
                    ),
                )
                .with_span(*span)
                .with_cause(cause)
                .with_help(help)
                .with_suggestion(suggestion)
            }
            SemaError::TraitMissingMethod {
                class_name,
                trait_name,
                missing_method,
                span,
            } => Diagnostic::new(
                "E0301",
                format!("class '{}' does not satisfy trait '{}'", class_name, trait_name),
            )
            .with_span(*span)
            .with_cause(format!(
                "class '{}' declares 'implements {}' but is missing required method '{}'.",
                class_name, trait_name, missing_method
            ))
            .with_help(format!(
                "all methods declared in trait '{}' without default implementations must be implemented.",
                trait_name
            ))
            .with_suggestion(format!(
                "implement method '{}' in class '{}':\n  fn {}(...): void {{ ... }}",
                missing_method, class_name, missing_method
            )),
            SemaError::CannotMutateReadOnlyBorrow(name, span) => Diagnostic::new(
                "E0201",
                format!("cannot mutate read-only borrow '{}'", name),
            )
            .with_span(*span)
            .with_cause(format!(
                "parameter '{}' is a read-only borrow (immutable by default in CP).",
                name
            ))
            .with_help("in CP, function parameters are read-only borrows by default. To modify arguments in-place, declare them as exclusive mutable borrows using '*' prefix.")
            .with_suggestion(format!(
                "parameter '{}' is a read-only borrow. Add '*' to declare it as mutable borrow: '*{}: <Type>'",
                name, name
            )),

            SemaError::MutableBorrowConflict(name, span) => Diagnostic::new(
                "E0202",
                format!("cannot borrow '{}' as mutable because it is already borrowed as immutable", name),
            )
            .with_span(*span)
            .with_cause(format!(
                "variable '{}' is already borrowed as immutable in the current scope.",
                name
            ))
            .with_help("CP enforces aliasing XOR mutability at compile time (single writer OR multiple readers).")
            .with_suggestion(format!(
                "ensure immutable borrows of '{}' end before taking a mutable borrow, or clone the value.",
                name
            )),

            SemaError::UseOfMovedValue(name, span) => Diagnostic::new(
                "E0203",
                format!("use of moved value '{}'", name),
            )
            .with_span(*span)
            .with_cause(format!(
                "value of '{}' was moved and its ownership was transferred.",
                name
            ))
            .with_help("variables moved via 'own' or 'move' transfer ownership and cannot be accessed afterwards.")
            .with_suggestion(format!(
                "avoid moving '{}' if you need to use it later, or pass by borrow '&{}'",
                name, name
            )),

            SemaError::ConstAssignment(name, span) => Diagnostic::new(
                "E0204",
                format!("cannot assign to const variable '{}'", name),
            )
            .with_span(*span)
            .with_cause(format!(
                "variable '{}' is declared with 'const' and cannot be reassigned.",
                name
            ))
            .with_help("variables declared with 'const' are immutable bindings.")
            .with_suggestion(format!(
                "change 'const {}' to 'let {}' to allow reassignment",
                name, name
            )),

            SemaError::AwaitOutsideAsync(span) => Diagnostic::new(
                "E0205",
                "'await' can only be used in async functions",
            )
            .with_span(*span)
            .with_cause("'await' pauses execution and requires state-machine CPS transformation.")
            .with_help("in CP, 'await' can only be used inside functions marked with 'async fn'.")
            .with_suggestion("declare the enclosing function as async: 'async fn ...'"),

            SemaError::UndefinedSymbol(sym, span) => {
                let mut diag = Diagnostic::new("E0206", format!("undefined symbol '{}'", sym))
                    .with_span(*span)
                    .with_cause(format!("identifier '{}' is not declared in current scope or imported modules.", sym));

                if let Some(&(_, mod_path)) = STANDARD_BUILTIN_IMPORTS.iter().find(|(name, _)| *name == sym.as_str()) {
                    diag.help = Some(format!("'{}' is a standard library symbol in '{}'.", sym, mod_path));
                    diag.suggestion = Some(format!("import {{ {} }} from \"{}\";", sym, mod_path));
                } else {
                    let cand_refs: Vec<&str> = known_symbols.iter().map(|s| s.as_str()).collect();
                    if let Some(closest) = find_closest_match(sym, &cand_refs, 3) {
                        diag.help = Some(format!("a symbol with a similar name '{}' exists in scope.", closest));
                        diag.suggestion = Some(format!("did you mean '{}'?", closest));
                    } else {
                        diag.help = Some("declare the identifier before use with 'let' / 'const' or import it from a module.".to_string());
                        diag.suggestion = Some(format!("let {} = ...;", sym));
                    }
                }
                diag
            }

            SemaError::TypeMismatch(expected, found, span) => {
                let mut diag = Diagnostic::new(
                    "E0207",
                    format!("type mismatch: expected '{:?}', found '{:?}'", expected, found),
                )
                .with_span(*span)
                .with_cause(format!(
                    "expression has type '{:?}', but the target requires '{:?}'.",
                    found, expected
                ));

                match (expected, found) {
                    (target, Type::Optional(inner)) if target == inner.as_ref() => {
                        diag.help = Some("the value is an Optional ('T?'). It may be null.".to_string());
                        diag.suggestion = Some("unwrap the optional using '?? <default_value>' or check for null using 'if (val != null)'".to_string());
                    }
                    (target, Type::ErrorUnion(inner)) if target == inner.as_ref() => {
                        diag.help = Some("the value is an Error Union ('T ! Error'). It may contain an error.".to_string());
                        diag.suggestion = Some("propagate the error using '?' (e.g. 'expr ?') or handle it using 'catch'".to_string());
                    }
                    (Type::USize, Type::I32) => {
                        diag.help = Some("CP does not perform implicit integer widening conversions.".to_string());
                        diag.suggestion = Some("cast explicitly using '@as(usize, val)'".to_string());
                    }
                    (target, Type::Optional(inner)) if matches!(inner.as_ref(), Type::Unknown | Type::Any) => {
                        diag.help = Some(format!("'null' cannot be assigned to non-optional type '{:?}'.", target));
                        diag.suggestion = Some(format!("change the type annotation to '{:?}?' to allow null", target));
                    }
                    _ => {
                        diag.help = Some("types must match exactly in CP's strong static type system.".to_string());
                        diag.suggestion = Some(format!("ensure the expression evaluates to '{:?}'", expected));
                    }
                }
                diag
            }

            SemaError::ReflectNotEnabled(type_name, span) => Diagnostic::new(
                "E0208",
                format!("type '{}' cannot be reflected at runtime", type_name),
            )
            .with_span(*span)
            .with_cause(format!("reflection metadata for '{}' was omitted during compilation.", type_name))
            .with_help("CP strips runtime metadata by default for zero memory overhead. Runtime reflection requires the '@reflect' attribute.")
            .with_suggestion(format!("add '@reflect' decorator before the definition: '@reflect class {} {{ ... }}'", type_name)),

            SemaError::UnsafeRequired(msg, span) => Diagnostic::new(
                "E0209",
                msg.clone(),
            )
            .with_span(*span)
            .with_cause("memory-unsafe operation (e.g. raw pointer dereference or C-FFI call) detected outside unsafe block.")
            .with_help("all operations that may violate memory safety must be enclosed in 'unsafe { ... }' blocks or 'unsafe fn'.")
            .with_suggestion("wrap the operation in 'unsafe { ... }'"),

            SemaError::MutateWhileBorrowedInFor(name, span) => Diagnostic::new(
                "E0210",
                format!("cannot mutate '{}' while it is borrowed in 'for ... of' loop", name),
            )
            .with_span(*span)
            .with_cause(format!("collection '{}' is actively borrowed by the iterator.", name))
            .with_help("mutating a container during iteration causes iterator invalidation undefined behavior.")
            .with_suggestion("iterate over a cloned collection or use an index-based 'while' loop"),

            SemaError::PrivateAccess(msg, span) => Diagnostic::new(
                "E0211",
                msg.clone(),
            )
            .with_span(*span)
            .with_cause("attempted to access a private member outside of its defining class.")
            .with_help("members are private by default in CP. Only members with 'pub' are accessible externally.")
            .with_suggestion("add 'pub' modifier to the member declaration, or provide a public getter/method"),

            SemaError::CannotInstantiateStatic(cls_name, span) => Diagnostic::new(
                "E0212",
                format!("cannot instantiate static class '{}'", cls_name),
            )
            .with_span(*span)
            .with_cause(format!("class '{}' is marked with 'static' and cannot be instantiated with 'new'.", cls_name))
            .with_help("static classes only act as namespaces for static functions and constants.")
            .with_suggestion(format!("call static methods directly via '{}.method(...)' without 'new'", cls_name)),

            SemaError::InvalidCast(msg, span) => Diagnostic::new(
                "E0213",
                msg.clone(),
            )
            .with_span(*span)
            .with_cause("semantic type conversions are prohibited via raw 'as' cast.")
            .with_help("use dedicated parser methods for string-to-number parsing.")
            .with_suggestion("use '<Type>.parse(str) ?' instead of 'as'"),

            SemaError::DuplicateVarDecl(name, span) => Diagnostic::new(
                "E0214",
                format!("cannot redeclare variable '{}'", name),
            )
            .with_span(*span)
            .with_cause(format!("identifier '{}' is already bound in the current block scope.", name))
            .with_help("block-scoped variables cannot be redeclared with 'let' in the same scope.")
            .with_suggestion(format!("remove 'let' to reassign to '{}', or rename the new variable", name)),

            SemaError::MultipleRestInPattern(span) => Diagnostic::new(
                "E0215",
                "destructuring pattern cannot contain multiple '...' rest elements",
            )
            .with_span(*span)
            .with_cause("a pattern can only have at most one rest element '...' to avoid ambiguity in slice length.")
            .with_help("ensure only one rest element exists in array or object pattern.")
            .with_suggestion("remove additional '...' rest elements from the pattern"),

            SemaError::NotAFunction(span) => Diagnostic::new(
                "E0216",
                "expression is not a callable function",
            )
            .with_span(*span)
            .with_cause("attempted to invoke a non-function value with '(...)' call syntax.")
            .with_help("verify that the callee is a function, method, or callable object."),

            SemaError::CompileTimeIndexOutOfBounds { index, len, span } => Diagnostic::new(
                "E0217",
                format!("index out of bounds: index {} is out of bounds for buffer of length {}", index, len),
            )
            .with_span(*span)
            .with_cause(format!("index {} exceeds compile-time known length {} of the buffer.", index, len))
            .with_help("accessing a fixed-size array or known-length buffer beyond its bound is a compile-time fatal error.")
            .with_suggestion(format!("ensure index is within 0..{}, or use safe slicing: 'slice = s[0..max_len]'", len)),

            SemaError::NegativeIndex { index, span } => Diagnostic::new(
                "E0218",
                format!("negative index {} is invalid for array indexing", index),
            )
            .with_span(*span)
            .with_cause(format!("array index {} is negative.", index))
            .with_help("array and buffer indices must be non-negative unsigned integers.")
            .with_suggestion("use a non-negative index value: e.g. 0"),

            SemaError::UnboundedRawPointerMutation(name, span) => Diagnostic::new(
                "E0209",
                format!("mutation via unbounded raw pointer or cstr '{}' requires 'unsafe' block", name),
            )
            .with_span(*span)
            .with_cause(format!("modifying memory via unbounded pointer/cstr '{}' without statically known length is unsafe.", name))
            .with_help("raw pointers and unbounded cstr buffers do not carry runtime length metadata. Modifying memory through them can lead to buffer overflows.")
            .with_suggestion(format!("wrap the mutation in 'unsafe {{ {}[...] = ...; }}' or slice it first: 'let slice = {}[0..len]; slice[...] = ...;'", name, name)),

            _ => Diagnostic::new("E0290", format!("semantic error: {}", self))
                .with_help("check semantic rules and types according to CP specification."),
        }
    }
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
        is_unsafe: bool,
    },
    Class(Class),
    Trait(TraitDecl),
    Enum(EnumDecl),
}

pub struct Analyzer {
    scopes: Vec<HashMap<String, SymbolKind>>,
    classes: HashMap<String, Class>,
    pub traits: HashMap<String, TraitDecl>,
    pub impls: Vec<ImplDecl>,
    pub functions: HashMap<String, Function>,
    pub active_generic_bounds: HashMap<String, Vec<String>>,
    current_fn_is_async: bool,
    current_fn_is_unsafe: bool,
    unsafe_depth: usize,
    pub symbol_map: HashMap<Span, SymbolKind>,
    pub meta_defs: HashMap<String, MetaDefInfo>,
    pub dec_defs: HashMap<String, DecDef>,
    pub macro_defs: HashMap<String, MacroDef>,
    pub macro_call_stack: Vec<(String, String)>,
    pub loop_borrowed_vars: std::collections::HashSet<String>,
    pub moved_vars: std::collections::HashSet<String>,
    pub current_class: Option<String>,
    pub type_aliases: HashMap<String, Type>,
    pub known_lengths_stack: Vec<HashMap<String, usize>>,
    pub const_int_vars_stack: Vec<HashMap<String, i64>>,
}

impl Analyzer {
    pub fn get_known_symbols(&self) -> Vec<String> {
        let mut symbols = std::collections::HashSet::new();
        for scope in &self.scopes {
            for key in scope.keys() {
                symbols.insert(key.clone());
            }
        }
        for key in self.classes.keys() {
            symbols.insert(key.clone());
        }
        symbols.into_iter().collect()
    }

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
        let mut meta_targets = std::collections::HashSet::new();
        meta_targets.insert("class".to_string());
        meta_targets.insert("struct".to_string());
        meta_targets.insert("function".to_string());
        meta_targets.insert("method".to_string());
        meta_targets.insert("field".to_string());
        meta_targets.insert("param".to_string());
        meta_targets.insert("trait".to_string());
        meta_targets.insert("enum".to_string());
        meta_defs.insert(
            "meta".to_string(),
            MetaDefInfo {
                allowed_targets: meta_targets,
                is_once: false,
            },
        );

        let mut macro_defs = HashMap::new();
        let mock_span = Span { start: 0, end: 0 };
        for name in [
            "Json",
            "Table",
            "Clone",
            "ToString",
            "Comparable",
            "Hash",
            "Debug",
        ] {
            macro_defs.insert(
                name.to_string(),
                MacroDef {
                    span: mock_span.clone(),
                    ident: Ident {
                        span: mock_span.clone(),
                        sym: name.to_string(),
                    },
                    params: vec![],
                    return_type: None,
                    body: BlockStmt {
                        span: mock_span.clone(),
                        body: vec![],
                    },
                    methods: vec![],
                    meta: vec![],
                },
            );
        }

        Self {
            scopes: vec![HashMap::new()],
            classes: HashMap::new(),
            traits: HashMap::new(),
            impls: Vec::new(),
            functions: HashMap::new(),
            active_generic_bounds: HashMap::new(),
            current_fn_is_async: false,
            current_fn_is_unsafe: false,
            unsafe_depth: 0,
            symbol_map: HashMap::new(),
            meta_defs,
            dec_defs: HashMap::new(),
            macro_defs,
            macro_call_stack: Vec::new(),
            loop_borrowed_vars: std::collections::HashSet::new(),
            moved_vars: std::collections::HashSet::new(),
            current_class: None,
            type_aliases: HashMap::new(),
            known_lengths_stack: vec![HashMap::new()],
            const_int_vars_stack: vec![HashMap::new()],
        }
    }

    pub fn is_in_unsafe(&self) -> bool {
        self.current_fn_is_unsafe || self.unsafe_depth > 0
    }

    pub fn analyze_module(&mut self, module: &mut Module) -> SemaResult<()> {
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
            extra_zig: vec![],
            is_pub: true,
            is_static: false,
            is_union: false,
            generic_params: vec![],
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
            extra_zig: vec![],
            is_pub: true,
            is_static: false,
            is_union: false,
            generic_params: vec![],
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
            extra_zig: vec![],
            is_pub: true,
            is_static: false,
            is_union: false,
            generic_params: vec![],
        };
        self.classes.insert("Map".to_string(), map_class.clone());
        self.scopes[0].insert("Map".to_string(), SymbolKind::Class(map_class));

        let dict_class = crate::ast::Class {
            span: mock_span.clone(),
            ident: crate::ast::Ident {
                span: mock_span.clone(),
                sym: "Dict".to_string(),
            },
            fields: vec![],
            methods: vec![],
            implements: vec![],
            extends: None,
            is_ref: false,
            meta: Vec::new(),
            extra_zig: vec![],
            is_pub: true,
            is_static: false,
            is_union: false,
            generic_params: vec![],
        };
        self.classes.insert("Dict".to_string(), dict_class.clone());
        self.scopes[0].insert("Dict".to_string(), SymbolKind::Class(dict_class));

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
                    default: None,
                    is_pub: true,
                    is_static: false,
                },
                crate::ast::Field {
                    span: mock_span.clone(),
                    ident: crate::ast::Ident {
                        span: mock_span.clone(),
                        sym: "msg".to_string(),
                    },
                    ty: Type::String,
                    meta: Vec::new(),
                    default: None,
                    is_pub: true,
                    is_static: false,
                },
            ],
            methods: vec![],
            implements: vec![],
            extends: None,
            is_ref: false,
            meta: Vec::new(),
            extra_zig: vec![],
            is_pub: true,
            is_static: false,
            is_union: false,
            generic_params: vec![],
        };
        self.classes
            .insert("Error".to_string(), error_class.clone());
        self.scopes[0].insert("Error".to_string(), SymbolKind::Class(error_class.clone()));
        self.classes.insert("Err".to_string(), error_class.clone());
        self.scopes[0].insert("Err".to_string(), SymbolKind::Class(error_class));

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
            extra_zig: vec![],
            is_pub: true,
            is_static: false,
            is_union: false,
            generic_params: vec![],
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
                extra_zig: vec![],
                meta: vec![crate::ast::MetaItem {
                    span: mock_span.clone(),
                    name: crate::ast::Ident {
                        span: mock_span.clone(),
                        sym: "reflect".to_string(),
                    },
                    args: vec![],
                }],
                is_pub: true,
                is_static: false,
                is_union: false,
                generic_params: vec![],
            };
            self.classes.insert(reflect_name.to_string(), cls.clone());
            self.scopes[0].insert(reflect_name.to_string(), SymbolKind::Class(cls));
        }

        for name in &[
            "Task",
            "Event",
            "EventEmitter",
            "Set",
            "VecDeque",
            "BinaryHeap",
            "BTreeMap",
            "BTreeSet",
            "HashMap",
            "HashSet",
        ] {
            let cls = crate::ast::Class {
                span: mock_span.clone(),
                ident: crate::ast::Ident {
                    span: mock_span.clone(),
                    sym: name.to_string(),
                },
                fields: vec![],
                methods: vec![],
                implements: vec![],
                extends: None,
                is_ref: false,
                extra_zig: vec![],
                meta: Vec::new(),
                is_pub: true,
                is_static: false,
                is_union: false,
                generic_params: vec![],
            };
            self.classes.insert(name.to_string(), cls.clone());
            self.scopes[0].insert(name.to_string(), SymbolKind::Class(cls));
        }

        for (fn_name, is_async, ret_ty) in &[
            (
                "spawn",
                false,
                Some(Type::Ref(Ident {
                    span: mock_span.clone(),
                    sym: "Task".to_string(),
                })),
            ),
            (
                "@task_spawn",
                false,
                Some(Type::Ref(Ident {
                    span: mock_span.clone(),
                    sym: "Task".to_string(),
                })),
            ),
            (
                "@task_spawn_cpu",
                false,
                Some(Type::Ref(Ident {
                    span: mock_span.clone(),
                    sym: "Task".to_string(),
                })),
            ),
            ("join", true, Some(Type::Any)),
            ("@task_join", true, Some(Type::Any)),
            (
                "@create_pending_task",
                false,
                Some(Type::Ref(Ident {
                    span: mock_span.clone(),
                    sym: "Task".to_string(),
                })),
            ),
            ("@complete_task", false, Some(Type::Void)),
        ] {
            self.scopes[0].insert(
                fn_name.to_string(),
                SymbolKind::Func {
                    params: vec![],
                    ret: ret_ty.clone(),
                    is_async: *is_async,
                    is_unsafe: false,
                },
            );
        }

        for item in &module.body {
            if let ModuleItem::Stmt(stmt) = item {
                self.extract_zig_code_symbols(stmt);
            }
        }

        for item in &module.body {
            match item {
                ModuleItem::Decl(Decl::Meta(m)) => {
                    self.register_meta_def(m)?;
                }
                ModuleItem::Decl(Decl::Dec(d)) => {
                    self.dec_defs.insert(d.ident.sym.clone(), d.clone());
                    self.define(
                        d.ident.sym.clone(),
                        SymbolKind::Func {
                            params: d.params.iter().map(|p| p.ty.clone()).collect(),
                            ret: d.return_type.clone(),
                            is_async: false,
                            is_unsafe: false,
                        },
                        d.ident.span,
                    )?;
                }
                ModuleItem::Decl(Decl::Macro(m)) => {
                    self.macro_defs.insert(m.ident.sym.clone(), m.clone());
                    self.define(
                        m.ident.sym.clone(),
                        SymbolKind::Func {
                            params: m.params.iter().map(|p| p.ty.clone()).collect(),
                            ret: Some(crate::ast::Type::Void),
                            is_async: false,
                            is_unsafe: false,
                        },
                        m.ident.span,
                    )?;
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
                    self.traits.insert(t.ident.sym.clone(), t.clone());
                    self.define(
                        t.ident.sym.clone(),
                        SymbolKind::Trait(t.clone()),
                        t.ident.span,
                    )?;
                    self.symbol_map
                        .insert(t.ident.span, SymbolKind::Trait(t.clone()));
                }
                ModuleItem::Decl(Decl::Impl(i)) => {
                    self.impls.push(i.clone());
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
                ModuleItem::Decl(Decl::Func(f)) => {
                    self.functions.insert(f.ident.sym.clone(), f.clone());
                    let params = f.params.iter().map(|p| p.ty.clone()).collect();
                    let kind = SymbolKind::Func {
                        params,
                        ret: f.return_type.clone(),
                        is_async: f.is_async,
                        is_unsafe: f.is_unsafe,
                    };
                    self.define(f.ident.sym.clone(), kind.clone(), f.ident.span)?;
                    self.symbol_map.insert(f.ident.span, kind);
                }
                ModuleItem::Decl(Decl::TypeAlias(ta)) => {
                    self.type_aliases
                        .insert(ta.ident.sym.clone(), ta.target_ty.clone());
                    let _ = self.define(
                        ta.ident.sym.clone(),
                        SymbolKind::Var {
                            ty: Type::Any,
                            kind: VarDeclKind::Const,
                        },
                        ta.ident.span,
                    );
                }
                ModuleItem::Decl(Decl::CInclude(ci)) => {
                    let _ = self.define(
                        "c".to_string(),
                        SymbolKind::Var {
                            ty: Type::Any,
                            kind: VarDeclKind::Const,
                        },
                        ci.span,
                    );
                }
                ModuleItem::Import(imp) => {
                    if let Some(def_spec) = &imp.default_specifier {
                        let _ = self.define(
                            def_spec.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Const,
                            },
                            def_spec.span,
                        );
                    }
                    for spec in &imp.specifiers {
                        let _ = self.define(
                            spec.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Const,
                            },
                            spec.span,
                        );
                    }
                }
                ModuleItem::Stmt(Stmt::Var(v)) => {
                    if let Pattern::Ident(id) = &v.pat {
                        let _ = self.define(
                            id.sym.clone(),
                            SymbolKind::Var {
                                ty: v.ty.clone().unwrap_or(Type::Any),
                                kind: v.kind.clone(),
                            },
                            id.span,
                        );
                    }
                }
                _ => {}
            }
        }

        // Pass 2: Phase 2 Type-Directed Macro Expansion
        self.expand_macros(module)?;

        // Update self.classes, symbol_map, and scopes with classes that had methods added by macros
        for item in &module.body {
            if let ModuleItem::Decl(Decl::Class(c)) = item {
                self.classes.insert(c.ident.sym.clone(), c.clone());
                self.symbol_map
                    .insert(c.ident.span, SymbolKind::Class(c.clone()));
                if let Some(scope_entry) = self.scopes[0].get_mut(&c.ident.sym) {
                    *scope_entry = SymbolKind::Class(c.clone());
                }
            }
        }

        // Pass 3: Full Semantic Analysis & Type Checking
        for item in &module.body {
            if let ModuleItem::Decl(Decl::TypeAlias(ta)) = item {
                self.type_aliases
                    .insert(ta.ident.sym.clone(), ta.target_ty.clone());
            }
        }
        for item in &module.body {
            if let ModuleItem::Stmt(stmt) = item {
                self.extract_zig_code_symbols(stmt);
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

    fn expand_macros(&mut self, module: &mut Module) -> SemaResult<()> {
        let mut class_modifications: Vec<(String, Vec<Function>)> = Vec::new();

        for item in &module.body {
            if let ModuleItem::Decl(Decl::Class(cls)) = item {
                let mut derived_methods = Vec::new();
                for meta_item in &cls.meta {
                    let macro_name = &meta_item.name.sym;
                    if !self.macro_defs.contains_key(macro_name) {
                        continue;
                    }

                    // Anti-Cycle Guardrails:
                    let call_key = (macro_name.clone(), cls.ident.sym.clone());
                    if self.macro_call_stack.contains(&call_key) {
                        return Err(SemaError::MacroCycleDetected(
                            macro_name.clone(),
                            cls.ident.sym.clone(),
                            meta_item.span,
                        ));
                    }
                    if self.macro_call_stack.len() >= 128 {
                        return Err(SemaError::MacroRecursionLimitExceeded(
                            macro_name.clone(),
                            meta_item.span,
                        ));
                    }

                    self.macro_call_stack.push(call_key);

                    let derived_list: Vec<Function> =
                        self.expand_single_macro(macro_name, cls, meta_item)?;

                    self.macro_call_stack.pop();

                    for derived in derived_list {
                        // Macro Hygiene: generated AST nodes strictly cannot carry macro annotations!
                        if !derived.meta.is_empty() {
                            return Err(SemaError::MacroHygieneViolation(
                                macro_name.clone(),
                                meta_item.span,
                            ));
                        }
                        derived_methods.push(derived);
                    }
                }

                if !derived_methods.is_empty() {
                    class_modifications.push((cls.ident.sym.clone(), derived_methods));
                }
            }
        }

        for item in &mut module.body {
            if let ModuleItem::Decl(Decl::Class(cls)) = item {
                if let Some((_, methods)) = class_modifications
                    .iter()
                    .find(|(name, _)| name == &cls.ident.sym)
                {
                    for m in methods {
                        if !cls
                            .methods
                            .iter()
                            .any(|existing| existing.ident.sym == m.ident.sym)
                        {
                            cls.methods.push(m.clone());
                        }
                    }
                }
                // Auto-generate memberwise init if not present and fields are non-empty (exempting @root, @value, @packed, @extern structs)
                let is_root = cls.meta.iter().any(|m| m.name.sym == "root");
                let is_value = cls.meta.iter().any(|m| m.name.sym == "value");
                let is_packed = cls.meta.iter().any(|m| m.name.sym == "packed");
                let is_extern = cls.meta.iter().any(|m| m.name.sym == "extern");
                let is_generic = !cls.generic_params.is_empty();
                if !cls.is_static
                    && !cls.is_union
                    && !is_root
                    && !is_value
                    && !is_packed
                    && !is_extern
                    && !is_generic
                    && !cls.methods.iter().any(|m| m.ident.sym == "init")
                    && !cls.fields.is_empty()
                {
                    let mock_span = Span { start: 0, end: 0 };
                    let mut init_params = Vec::new();
                    let mut init_stmts = Vec::new();
                    for f in &cls.fields {
                        init_params.push(Param {
                            span: mock_span,
                            ident: f.ident.clone(),
                            ty: f.ty.clone(),
                            is_mut: false,
                            is_move: matches!(f.ty, Type::Ref(_) | Type::BoxDyn(_)),
                            is_comptime: false,
                            default: None,
                            meta: vec![],
                        });
                        init_stmts.push(Stmt::Expr(Expr::Bin(BinExpr {
                            span: mock_span,
                            op: BinaryOp::Assign,
                            left: Box::new(Expr::Member(MemberExpr {
                                span: mock_span,
                                obj: Box::new(Expr::Ident(Ident {
                                    span: mock_span,
                                    sym: "_".to_string(),
                                })),
                                prop: f.ident.clone(),
                            })),
                            right: Box::new(Expr::Ident(f.ident.clone())),
                        })));
                    }
                    cls.methods.insert(
                        0,
                        Function {
                            span: mock_span,
                            ident: Ident {
                                span: mock_span,
                                sym: "init".to_string(),
                            },
                            generic_params: vec![],
                            params: init_params,
                            return_type: Some(Type::Void),
                            body: BlockStmt {
                                span: mock_span,
                                body: init_stmts,
                            },
                            is_async: false,
                            is_static: false,
                            is_unsafe: false,
                            meta: vec![],
                            is_pub: true,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    fn expand_single_macro(
        &self,
        macro_name: &str,
        cls: &Class,
        meta_item: &MetaItem,
    ) -> SemaResult<Vec<Function>> {
        match macro_name {
            "Json" => Ok(vec![self.derive_json_method(cls)]),
            "Table" => Ok(vec![self.derive_table_method(cls, meta_item)]),
            "ToString" => Ok(vec![self.derive_tostring_method(cls)]),
            "Debug" => Ok(self.derive_debug_methods(cls)),
            "Clone" => Ok(vec![self.derive_clone_method(cls)]),
            "Comparable" => Ok(self.derive_comparable_methods(cls)),
            "Hash" => Ok(vec![self.derive_hash_method(cls)]),
            _ => self.derive_custom_macro_methods(macro_name, cls, meta_item),
        }
    }

    fn derive_debug_methods(&self, cls: &Class) -> Vec<Function> {
        let mock_span = Span { start: 0, end: 0 };
        let to_str = self.derive_tostring_method(cls);
        let mut fmt_debug = to_str.clone();
        fmt_debug.ident = Ident {
            span: mock_span,
            sym: "fmt_debug".to_string(),
        };
        vec![to_str, fmt_debug]
    }

    fn derive_clone_method(&self, cls: &Class) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let mut args = Vec::new();
        for f in &cls.fields {
            args.push(Expr::Member(MemberExpr {
                span: mock_span,
                obj: Box::new(Expr::Ident(Ident {
                    span: mock_span,
                    sym: "_".to_string(),
                })),
                prop: f.ident.clone(),
            }));
        }
        let ret_expr = Expr::New(CallExpr {
            span: mock_span,
            callee: Box::new(Expr::Ident(cls.ident.clone())),
            args,
            is_comptime: false,
        });
        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: "clone".to_string(),
            },
            generic_params: vec![],
            params: vec![],
            return_type: Some(Type::Ref(cls.ident.clone())),
            body: BlockStmt {
                span: mock_span,
                body: vec![Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(ret_expr),
                })],
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![],
            is_pub: true,
        }
    }

    fn derive_comparable_methods(&self, cls: &Class) -> Vec<Function> {
        vec![self.derive_eq_method(cls), self.derive_cmp_method(cls)]
    }

    fn derive_eq_method(&self, cls: &Class) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let mut body = Vec::new();
        for f in &cls.fields {
            let self_access = Expr::Member(MemberExpr {
                span: mock_span,
                obj: Box::new(Expr::Ident(Ident {
                    span: mock_span,
                    sym: "_".to_string(),
                })),
                prop: f.ident.clone(),
            });
            let other_access = Expr::Member(MemberExpr {
                span: mock_span,
                obj: Box::new(Expr::Ident(Ident {
                    span: mock_span,
                    sym: "other".to_string(),
                })),
                prop: f.ident.clone(),
            });
            let not_eq = Expr::Bin(BinExpr {
                span: mock_span,
                op: BinaryOp::NotEq,
                left: Box::new(self_access),
                right: Box::new(other_access),
            });
            body.push(Stmt::If(IfStmt {
                span: mock_span,
                test: not_eq,
                cons: Box::new(Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(Expr::Lit(Lit::Bool(false))),
                })),
                alt: None,
                is_comptime: false,
                capture: None,
                is_ptr_capture: false,
            }));
        }
        body.push(Stmt::Return(crate::ast::ReturnStmt {
            span: mock_span,
            arg: Some(Expr::Lit(Lit::Bool(true))),
        }));
        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: "eq".to_string(),
            },
            generic_params: vec![],
            params: vec![Param {
                span: mock_span,
                ident: Ident {
                    span: mock_span,
                    sym: "other".to_string(),
                },
                ty: Type::Ref(cls.ident.clone()),
                is_mut: false,
                is_move: false,
                is_comptime: false,
                default: None,
                meta: vec![],
            }],
            return_type: Some(Type::Bool),
            body: BlockStmt {
                span: mock_span,
                body,
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![],
            is_pub: true,
        }
    }

    fn derive_cmp_method(&self, cls: &Class) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let mut body = Vec::new();
        for f in &cls.fields {
            let self_access = Expr::Member(MemberExpr {
                span: mock_span,
                obj: Box::new(Expr::Ident(Ident {
                    span: mock_span,
                    sym: "_".to_string(),
                })),
                prop: f.ident.clone(),
            });
            let other_access = Expr::Member(MemberExpr {
                span: mock_span,
                obj: Box::new(Expr::Ident(Ident {
                    span: mock_span,
                    sym: "other".to_string(),
                })),
                prop: f.ident.clone(),
            });
            let lt = Expr::Bin(BinExpr {
                span: mock_span,
                op: BinaryOp::Lt,
                left: Box::new(self_access.clone()),
                right: Box::new(other_access.clone()),
            });
            body.push(Stmt::If(IfStmt {
                span: mock_span,
                test: lt,
                cons: Box::new(Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(Expr::Lit(Lit::Int(-1))),
                })),
                alt: None,
                is_comptime: false,
                capture: None,
                is_ptr_capture: false,
            }));
            let gt = Expr::Bin(BinExpr {
                span: mock_span,
                op: BinaryOp::Gt,
                left: Box::new(self_access),
                right: Box::new(other_access),
            });
            body.push(Stmt::If(IfStmt {
                span: mock_span,
                test: gt,
                cons: Box::new(Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(Expr::Lit(Lit::Int(1))),
                })),
                alt: None,
                is_comptime: false,
                capture: None,
                is_ptr_capture: false,
            }));
        }
        body.push(Stmt::Return(crate::ast::ReturnStmt {
            span: mock_span,
            arg: Some(Expr::Lit(Lit::Int(0))),
        }));
        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: "cmp".to_string(),
            },
            generic_params: vec![],
            params: vec![Param {
                span: mock_span,
                ident: Ident {
                    span: mock_span,
                    sym: "other".to_string(),
                },
                ty: Type::Ref(cls.ident.clone()),
                is_mut: false,
                is_move: false,
                is_comptime: false,
                default: None,
                meta: vec![],
            }],
            return_type: Some(Type::I32),
            body: BlockStmt {
                span: mock_span,
                body,
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![],
            is_pub: true,
        }
    }

    fn derive_hash_method(&self, cls: &Class) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let mut args = Vec::new();
        for f in &cls.fields {
            args.push(Expr::Member(MemberExpr {
                span: mock_span,
                obj: Box::new(Expr::Ident(Ident {
                    span: mock_span,
                    sym: "_".to_string(),
                })),
                prop: f.ident.clone(),
            }));
        }
        let ret_expr = Expr::Builtin(BuiltinCall {
            span: mock_span,
            name: "@hash".to_string(),
            args,
        });
        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: "hash".to_string(),
            },
            generic_params: vec![],
            params: vec![],
            return_type: Some(Type::USize),
            body: BlockStmt {
                span: mock_span,
                body: vec![Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(ret_expr),
                })],
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![],
            is_pub: true,
        }
    }

    fn derive_json_method(&self, cls: &Class) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let ret_expr = if cls.fields.is_empty() {
            Expr::Lit(Lit::Str("{}".to_string()))
        } else {
            let mut expr = Expr::Lit(Lit::Str("{".to_string()));
            for (idx, f) in cls.fields.iter().enumerate() {
                let prefix = if idx == 0 {
                    format!("\"{}\":", f.ident.sym)
                } else {
                    format!(",\"{}\":", f.ident.sym)
                };
                expr = Expr::Bin(BinExpr {
                    span: mock_span,
                    op: BinaryOp::Add,
                    left: Box::new(expr),
                    right: Box::new(Expr::Lit(Lit::Str(prefix))),
                });

                let field_access = Expr::Member(MemberExpr {
                    span: mock_span,
                    obj: Box::new(Expr::Ident(Ident {
                        span: mock_span,
                        sym: "_".to_string(),
                    })),
                    prop: f.ident.clone(),
                });

                let val_expr = match &f.ty {
                    Type::String => Expr::Bin(BinExpr {
                        span: mock_span,
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Lit(Lit::Str("\"".to_string()))),
                        right: Box::new(Expr::Bin(BinExpr {
                            span: mock_span,
                            op: BinaryOp::Add,
                            left: Box::new(field_access),
                            right: Box::new(Expr::Lit(Lit::Str("\"".to_string()))),
                        })),
                    }),
                    Type::Ref(id) if self.classes.contains_key(&id.sym) => {
                        let ref_cls = &self.classes[&id.sym];
                        let has_json_macro = ref_cls.meta.iter().any(|m| m.name.sym == "Json")
                            || ref_cls.methods.iter().any(|m| m.ident.sym == "toJson");
                        if has_json_macro {
                            Expr::Call(CallExpr {
                                span: mock_span,
                                callee: Box::new(Expr::Member(MemberExpr {
                                    span: mock_span,
                                    obj: Box::new(field_access),
                                    prop: Ident {
                                        span: mock_span,
                                        sym: "toJson".to_string(),
                                    },
                                })),
                                args: vec![],
                                is_comptime: false,
                            })
                        } else {
                            field_access
                        }
                    }
                    _ => field_access,
                };

                expr = Expr::Bin(BinExpr {
                    span: mock_span,
                    op: BinaryOp::Add,
                    left: Box::new(expr),
                    right: Box::new(val_expr),
                });
            }

            Expr::Bin(BinExpr {
                span: mock_span,
                op: BinaryOp::Add,
                left: Box::new(expr),
                right: Box::new(Expr::Lit(Lit::Str("}".to_string()))),
            })
        };

        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: "toJson".to_string(),
            },
            generic_params: vec![],
            params: vec![],
            return_type: Some(Type::String),
            body: BlockStmt {
                span: mock_span,
                body: vec![Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(ret_expr),
                })],
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![], // Macro Hygiene!
            is_pub: true,
        }
    }

    fn derive_table_method(&self, cls: &Class, meta_item: &MetaItem) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let table_name = if let Some(arg) = meta_item.args.first() {
            match arg {
                MetaArg::Positional(Expr::Lit(Lit::Str(s))) => s.clone(),
                MetaArg::Named(_, Expr::Lit(Lit::Str(s))) => s.clone(),
                _ => cls.ident.sym.to_lowercase(),
            }
        } else {
            cls.ident.sym.to_lowercase()
        };
        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: "getTableName".to_string(),
            },
            generic_params: vec![],
            params: vec![],
            return_type: Some(Type::String),
            body: BlockStmt {
                span: mock_span,
                body: vec![Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(Expr::Lit(Lit::Str(table_name))),
                })],
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![],
            is_pub: true,
        }
    }

    fn derive_tostring_method(&self, cls: &Class) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let mut expr = Expr::Lit(Lit::Str(format!("{}(", cls.ident.sym)));
        for (idx, f) in cls.fields.iter().enumerate() {
            let prefix = if idx == 0 {
                format!("{}=", f.ident.sym)
            } else {
                format!(", {}=", f.ident.sym)
            };
            expr = Expr::Bin(BinExpr {
                span: mock_span,
                op: BinaryOp::Add,
                left: Box::new(expr),
                right: Box::new(Expr::Lit(Lit::Str(prefix))),
            });
            let field_access = Expr::Member(MemberExpr {
                span: mock_span,
                obj: Box::new(Expr::Ident(Ident {
                    span: mock_span,
                    sym: "_".to_string(),
                })),
                prop: f.ident.clone(),
            });
            expr = Expr::Bin(BinExpr {
                span: mock_span,
                op: BinaryOp::Add,
                left: Box::new(expr),
                right: Box::new(field_access),
            });
        }
        expr = Expr::Bin(BinExpr {
            span: mock_span,
            op: BinaryOp::Add,
            left: Box::new(expr),
            right: Box::new(Expr::Lit(Lit::Str(")".to_string()))),
        });
        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: "toString".to_string(),
            },
            generic_params: vec![],
            params: vec![],
            return_type: Some(Type::String),
            body: BlockStmt {
                span: mock_span,
                body: vec![Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(expr),
                })],
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![],
            is_pub: true,
        }
    }

    fn derive_custom_macro_method(
        &self,
        macro_name: &str,
        _cls: &Class,
        meta_item: &MetaItem,
    ) -> Function {
        let mock_span = Span { start: 0, end: 0 };
        let method_name = format!("get{}", macro_name);
        let return_val = if let Some(arg) = meta_item.args.first() {
            match arg {
                MetaArg::Positional(Expr::Lit(Lit::Str(s))) => s.clone(),
                MetaArg::Named(_, Expr::Lit(Lit::Str(s))) => s.clone(),
                _ => macro_name.to_string(),
            }
        } else {
            macro_name.to_string()
        };
        Function {
            span: mock_span,
            ident: Ident {
                span: mock_span,
                sym: method_name,
            },
            generic_params: vec![],
            params: vec![],
            return_type: Some(Type::String),
            body: BlockStmt {
                span: mock_span,
                body: vec![Stmt::Return(crate::ast::ReturnStmt {
                    span: mock_span,
                    arg: Some(Expr::Lit(Lit::Str(return_val))),
                })],
            },
            is_async: false,
            is_static: false,
            is_unsafe: false,
            meta: vec![],
            is_pub: true,
        }
    }

    fn derive_custom_macro_methods(
        &self,
        macro_name: &str,
        cls: &Class,
        meta_item: &MetaItem,
    ) -> SemaResult<Vec<Function>> {
        if let Some(macro_def) = self.macro_defs.get(macro_name) {
            if !macro_def.methods.is_empty() {
                let mut subst: HashMap<String, String> = HashMap::new();
                subst.insert("${target}".to_string(), cls.ident.sym.clone());
                subst.insert("target".to_string(), cls.ident.sym.clone());

                let mut arg_idx = 0;
                for (p_idx, p) in macro_def.params.iter().enumerate() {
                    if p_idx == 0
                        && (p.ident.sym == "target" || p.ident.sym == "this" || p.ident.sym == "_")
                    {
                        continue;
                    }
                    let val_opt = meta_item
                        .args
                        .iter()
                        .find_map(|a| match a {
                            MetaArg::Named(id, e) if id.sym == p.ident.sym => {
                                Some(Self::eval_meta_arg_str(e))
                            }
                            _ => None,
                        })
                        .or_else(|| {
                            if arg_idx < meta_item.args.len() {
                                let a = &meta_item.args[arg_idx];
                                arg_idx += 1;
                                match a {
                                    MetaArg::Positional(e) => Some(Self::eval_meta_arg_str(e)),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        });

                    if let Some(val) = val_opt {
                        subst.insert(format!("${{{}}}", p.ident.sym), val.clone());
                        subst.insert(format!("${}", p.ident.sym), val.clone());
                        subst.insert(p.ident.sym.clone(), val);
                    }
                }

                let mut subst_ordered: Vec<(&String, &String)> = subst.iter().collect();
                subst_ordered.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));

                let mut expanded = Vec::new();
                for m in &macro_def.methods {
                    let mut instantiated = m.clone();
                    Self::substitute_function(&mut instantiated, &subst_ordered);
                    instantiated.meta.clear(); // Macro Hygiene!
                    expanded.push(instantiated);
                }
                return Ok(expanded);
            }
        }

        Ok(vec![
            self.derive_custom_macro_method(macro_name, cls, meta_item)
        ])
    }

    fn eval_meta_arg_str(expr: &Expr) -> String {
        match expr {
            Expr::Lit(Lit::Str(s)) => s.clone(),
            Expr::Ident(id) => id.sym.clone(),
            Expr::Lit(Lit::Int(n)) => n.to_string(),
            Expr::Lit(Lit::Float(f)) => f.to_string(),
            Expr::Lit(Lit::Bool(b)) => b.to_string(),
            _ => "".to_string(),
        }
    }

    fn substitute_type(ty: &mut Type, subst: &[(&String, &String)]) {
        match ty {
            Type::Ref(id) => {
                for (k, v) in subst {
                    if id.sym == **k {
                        id.sym = (*v).clone();
                    } else if id.sym.contains(*k) {
                        id.sym = id.sym.replace(*k, v);
                    }
                }
                match id.sym.as_str() {
                    "string" => *ty = Type::String,
                    "i32" => *ty = Type::I32,
                    "u32" => *ty = Type::U32,
                    "i64" => *ty = Type::I64,
                    "u64" => *ty = Type::U64,
                    "i128" => *ty = Type::I128,
                    "u128" => *ty = Type::U128,
                    "f32" => *ty = Type::F32,
                    "f64" => *ty = Type::F64,
                    "usize" => *ty = Type::USize,
                    "bool" => *ty = Type::Bool,
                    "void" => *ty = Type::Void,
                    _ => {}
                    _ => {}
                }
            }
            Type::BorrowType { inner, .. } => Self::substitute_type(inner, subst),
            Type::Optional(inner) => Self::substitute_type(inner, subst),
            Type::Array(inner) => Self::substitute_type(inner, subst),
            _ => {}
            _ => {}
        }
    }

    fn substitute_function(func: &mut Function, subst: &[(&String, &String)]) {
        for (k, v) in subst {
            if func.ident.sym == **k {
                func.ident.sym = (*v).clone();
            } else if func.ident.sym.contains(*k) {
                func.ident.sym = func.ident.sym.replace(*k, v);
            }
        }
        if let Some(ret) = &mut func.return_type {
            Self::substitute_type(ret, subst);
        }
        for p in &mut func.params {
            for (k, v) in subst {
                if p.ident.sym == **k {
                    p.ident.sym = (*v).clone();
                } else if p.ident.sym.contains(*k) {
                    p.ident.sym = p.ident.sym.replace(*k, v);
                }
            }
            Self::substitute_type(&mut p.ty, subst);
        }
        for s in &mut func.body.body {
            Self::substitute_stmt(s, subst);
        }
    }

    fn substitute_stmt(stmt: &mut Stmt, subst: &[(&String, &String)]) {
        match stmt {
            Stmt::Expr(e) => Self::substitute_expr(e, subst),
            Stmt::Return(r) => {
                if let Some(arg) = &mut r.arg {
                    Self::substitute_expr(arg, subst);
                }
            }
            Stmt::Break | Stmt::Continue => {}
            Stmt::If(i) => {
                Self::substitute_expr(&mut i.test, subst);
                Self::substitute_stmt(&mut i.cons, subst);
                if let Some(alt) = &mut i.alt {
                    Self::substitute_stmt(alt, subst);
                }
            }
            Stmt::While(w) => {
                Self::substitute_expr(&mut w.test, subst);
                if let Some(c) = &mut w.continue_expr {
                    Self::substitute_expr(c, subst);
                }
                Self::substitute_stmt(&mut w.body, subst);
            }
            Stmt::For(f) => {
                Self::substitute_expr(&mut f.expr, subst);
                Self::substitute_stmt(&mut f.body, subst);
            }
            Stmt::Block(b) | Stmt::ComptimeBlock(b) | Stmt::UnsafeBlock(b) => {
                for s in &mut b.body {
                    Self::substitute_stmt(s, subst);
                }
            }
            Stmt::Var(v) => {
                if let Pattern::Ident(id) = &mut v.pat {
                    for (k, v_sub) in subst {
                        if id.sym == **k {
                            id.sym = (*v_sub).clone();
                        } else if id.sym.contains(*k) {
                            id.sym = id.sym.replace(*k, v_sub);
                        }
                    }
                }
                if let Some(ty) = &mut v.ty {
                    Self::substitute_type(ty, subst);
                }
                if let Some(init) = &mut v.init {
                    Self::substitute_expr(init, subst);
                }
            }
            _ => {}
            _ => {}
        }
    }

    fn substitute_expr(expr: &mut Expr, subst: &[(&String, &String)]) {
        match expr {
            Expr::Ident(id) => {
                for (k, v) in subst {
                    if id.sym == **k {
                        id.sym = (*v).clone();
                    } else if id.sym.contains(*k) {
                        id.sym = id.sym.replace(*k, v);
                    }
                }
            }
            Expr::Member(m) => {
                Self::substitute_expr(&mut m.obj, subst);
                for (k, v) in subst {
                    if m.prop.sym == **k {
                        m.prop.sym = (*v).clone();
                    } else if m.prop.sym.contains(*k) {
                        m.prop.sym = m.prop.sym.replace(*k, v);
                    }
                }
            }
            Expr::Bin(b) => {
                Self::substitute_expr(&mut b.left, subst);
                Self::substitute_expr(&mut b.right, subst);
            }
            Expr::Call(c) | Expr::New(c) | Expr::OptCall(c) => {
                Self::substitute_expr(&mut c.callee, subst);
                for a in &mut c.args {
                    Self::substitute_expr(a, subst);
                }
            }
            Expr::Lit(Lit::Str(s)) => {
                for (k, v) in subst {
                    if s.contains(*k) {
                        *s = s.replace(*k, v);
                    }
                }
            }
            Expr::Template(t) => {
                for q in &mut t.quasis {
                    for (k, v) in subst {
                        if q.contains(*k) {
                            *q = q.replace(*k, v);
                        }
                    }
                }
                for e in &mut t.expressions {
                    Self::substitute_expr(e, subst);
                }
            }
            Expr::Builtin(b) => {
                for a in &mut b.args {
                    Self::substitute_expr(a, subst);
                }
            }
            Expr::Array(items) => {
                for item in items {
                    Self::substitute_expr(item, subst);
                }
            }
            Expr::ArrayRepeat(val, count) => {
                Self::substitute_expr(val, subst);
                Self::substitute_expr(count, subst);
            }
            _ => {}
            _ => {}
        }
    }

    fn is_const_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Lit(_) => true,
            Expr::Ident(_) => true,
            Expr::Member(mem) => Self::is_const_expr(&mem.obj),
            Expr::Array(elements) => elements.iter().all(Self::is_const_expr),
            Expr::ArrayRepeat(val, count) => Self::is_const_expr(val) && Self::is_const_expr(count),
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

    fn extract_zig_code_symbols(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(Expr::Zig(z)) => self.register_zig_block_symbols(&z.code),
            Stmt::Block(b) | Stmt::UnsafeBlock(b) => {
                for s in &b.body {
                    self.extract_zig_code_symbols(s);
                }
            }
            _ => {}
            _ => {}
        }
    }

    fn register_zig_block_symbols(&mut self, code: &str) {
        for line in code.lines() {
            let trimmed = line.trim();
            let stripped = if let Some(s) = trimmed.strip_prefix("pub ") {
                s.trim_start()
            } else {
                trimmed
            };
            let (ident_opt, is_var) = if let Some(rest) = stripped.strip_prefix("const ") {
                (rest.split(&[' ', ':', '='][..]).next(), false)
            } else if let Some(rest) = stripped.strip_prefix("var ") {
                (rest.split(&[' ', ':', '='][..]).next(), true)
            } else if let Some(rest) = stripped.strip_prefix("fn ") {
                (rest.split(&[' ', '(', '<'][..]).next(), false)
            } else if let Some(rest) = stripped.strip_prefix("extern fn ") {
                (rest.split(&[' ', '(', '<'][..]).next(), false)
            } else if let Some(rest) = stripped.strip_prefix("extern \"c\" fn ") {
                (rest.split(&[' ', '(', '<'][..]).next(), false)
            } else {
                (None, false)
            };
            if let Some(ident) = ident_opt {
                let id = ident.trim();
                if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    self.scopes[0].insert(
                        id.to_string(),
                        SymbolKind::Var {
                            ty: Type::Any,
                            kind: if is_var {
                                VarDeclKind::Let
                            } else {
                                VarDeclKind::Const
                            },
                        },
                    );
                }
            }
        }
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

            if self.macro_defs.contains_key(&item.name.sym) {
                let matches_target = target == "class" || target == "struct";
                if !matches_target {
                    return Err(SemaError::MacroInvalidTarget(
                        item.name.sym.clone(),
                        "class, struct".to_string(),
                        target.to_string(),
                        item.span,
                    ));
                }
            } else if self.dec_defs.contains_key(&item.name.sym) {
                let matches_target = target == "method" || target == "function";
                if !matches_target {
                    return Err(SemaError::DecInvalidTarget(
                        item.name.sym.clone(),
                        "method, function".to_string(),
                        target.to_string(),
                        item.span,
                    ));
                }
            } else if let Some(def) = self.meta_defs.get(&item.name.sym) {
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
            Decl::Dec(d) => {
                self.validate_meta_items(&d.meta, "dec")?;
                self.enter_scope();
                for p in &d.params {
                    let p_kind = SymbolKind::Var {
                        ty: p.ty.clone(),
                        kind: VarDeclKind::Let,
                    };
                    self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span)?;
                    self.symbol_map.insert(p.ident.span, p_kind);
                }
                self.analyze_block(&d.body)?;
                self.exit_scope();
            }
            Decl::Macro(m) => {
                self.validate_meta_items(&m.meta, "macro")?;
                self.enter_scope();
                for p in &m.params {
                    let p_kind = SymbolKind::Var {
                        ty: p.ty.clone(),
                        kind: VarDeclKind::Let,
                    };
                    self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span)?;
                    self.symbol_map.insert(p.ident.span, p_kind);
                }
                self.analyze_block(&m.body)?;
                self.exit_scope();
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
                    is_unsafe: f.is_unsafe,
                };
                self.define(f.ident.sym.clone(), kind.clone(), f.ident.span)?;
                self.symbol_map.insert(f.ident.span, kind);

                let mut added_generic_bounds = Vec::new();
                for gp in &f.generic_params {
                    if let GenericParam::Type { ident, bounds } = gp {
                        self.active_generic_bounds.insert(
                            ident.sym.clone(),
                            bounds.iter().map(|b| b.sym.clone()).collect(),
                        );
                        added_generic_bounds.push(ident.sym.clone());
                    }
                }

                let old_async = self.current_fn_is_async;
                let old_unsafe = self.current_fn_is_unsafe;
                self.current_fn_is_async = f.is_async;
                self.current_fn_is_unsafe = f.is_unsafe;
                self.enter_scope();
                for gp in &f.generic_params {
                    match gp {
                        GenericParam::Type { ident: id, .. } => {
                            let _ = self.define(
                                id.sym.clone(),
                                SymbolKind::Var {
                                    ty: Type::Ref(id.clone()),
                                    kind: VarDeclKind::Const,
                                },
                                id.span,
                            );
                        }
                        GenericParam::Const { ident, ty } => {
                            let _ = self.define(
                                ident.sym.clone(),
                                SymbolKind::Var {
                                    ty: ty.clone(),
                                    kind: VarDeclKind::Const,
                                },
                                ident.span,
                            );
                        }
                    }
                }
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
                for k in added_generic_bounds {
                    self.active_generic_bounds.remove(&k);
                }
                self.current_fn_is_async = old_async;
                self.current_fn_is_unsafe = old_unsafe;
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
                // Verify that all declared 'implements' traits have their required methods implemented
                for imp in &c.implements {
                    if let Some(tr) = self.traits.get(&imp.sym) {
                        for tm in &tr.methods {
                            if tm.default_body.is_none() {
                                let has_m = c.methods.iter().any(|m| m.ident.sym == tm.ident.sym)
                                    || self.impls.iter().any(|i| {
                                        i.trait_name.sym == imp.sym
                                            && i.target_name.sym == c.ident.sym
                                            && i.methods.iter().any(|m| m.ident.sym == tm.ident.sym)
                                    });
                                if !has_m {
                                    return Err(SemaError::TraitMissingMethod {
                                        class_name: c.ident.sym.clone(),
                                        trait_name: imp.sym.clone(),
                                        missing_method: tm.ident.sym.clone(),
                                        span: c.ident.span,
                                    });
                                }
                            }
                        }
                    }
                }

                let mut added_class_bounds = Vec::new();
                for gp in &c.generic_params {
                    if let GenericParam::Type { ident, bounds } = gp {
                        self.active_generic_bounds.insert(
                            ident.sym.clone(),
                            bounds.iter().map(|b| b.sym.clone()).collect(),
                        );
                        added_class_bounds.push(ident.sym.clone());
                    }
                }

                let old_class = self.current_class.clone();
                self.current_class = Some(c.ident.sym.clone());
                for m in &c.methods {
                    let mut added_method_bounds = Vec::new();
                    for gp in &m.generic_params {
                        if let GenericParam::Type { ident, bounds } = gp {
                            self.active_generic_bounds.insert(
                                ident.sym.clone(),
                                bounds.iter().map(|b| b.sym.clone()).collect(),
                            );
                            added_method_bounds.push(ident.sym.clone());
                        }
                    }

                    let old_async = self.current_fn_is_async;
                    let old_unsafe = self.current_fn_is_unsafe;
                    self.current_fn_is_async = m.is_async;
                    self.current_fn_is_unsafe = m.is_unsafe;
                    self.enter_scope();
                    for gp in &c.generic_params {
                        match gp {
                            GenericParam::Type { ident: id, .. } => {
                                let _ = self.define(
                                    id.sym.clone(),
                                    SymbolKind::Var {
                                        ty: Type::Ref(id.clone()),
                                        kind: VarDeclKind::Const,
                                    },
                                    id.span,
                                );
                            }
                            GenericParam::Const { ident, ty } => {
                                let _ = self.define(
                                    ident.sym.clone(),
                                    SymbolKind::Var {
                                        ty: ty.clone(),
                                        kind: VarDeclKind::Const,
                                    },
                                    ident.span,
                                );
                            }
                        }
                    }
                    for gp in &m.generic_params {
                        match gp {
                            GenericParam::Type { ident: id, .. } => {
                                let _ = self.define(
                                    id.sym.clone(),
                                    SymbolKind::Var {
                                        ty: Type::Ref(id.clone()),
                                        kind: VarDeclKind::Const,
                                    },
                                    id.span,
                                );
                            }
                            GenericParam::Const { ident, ty } => {
                                let _ = self.define(
                                    ident.sym.clone(),
                                    SymbolKind::Var {
                                        ty: ty.clone(),
                                        kind: VarDeclKind::Const,
                                    },
                                    ident.span,
                                );
                            }
                        }
                    }
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
                                    is_unsafe: cm.is_unsafe,
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
                                        is_unsafe: tm.is_unsafe,
                                    },
                                    tm.ident.span,
                                );
                            }
                        }
                    }
                    for p in &m.params {
                        if p.ident.sym == "this" || p.ident.sym == "_" {
                            let p_kind = SymbolKind::Var {
                                ty: p.ty.clone(),
                                kind: VarDeclKind::Let,
                            };
                            self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span)?;
                            self.symbol_map.insert(p.ident.span, p_kind);
                        }
                    }
                    self.enter_scope();
                    for p in &m.params {
                        if p.ident.sym != "this" && p.ident.sym != "_" {
                            let p_kind = SymbolKind::Var {
                                ty: p.ty.clone(),
                                kind: VarDeclKind::Let,
                            };
                            self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span)?;
                            self.symbol_map.insert(p.ident.span, p_kind);
                        }
                    }
                    for stmt in &m.body.body {
                        self.analyze_stmt(stmt)?;
                    }
                    self.exit_scope();
                    self.exit_scope();
                    for k in added_method_bounds {
                        self.active_generic_bounds.remove(&k);
                    }
                    self.current_fn_is_async = old_async;
                    self.current_fn_is_unsafe = old_unsafe;
                }
                for k in added_class_bounds {
                    self.active_generic_bounds.remove(&k);
                }
                self.current_class = old_class;
            }
            Decl::Trait(t) => {
                self.validate_meta_items(&t.meta, "trait")?;
            }
            Decl::Enum(e) => {
                self.validate_meta_items(&e.meta, "enum")?;
                for m in &e.methods {
                    self.validate_meta_items(&m.meta, "method")?;
                    let old_async = self.current_fn_is_async;
                    let old_unsafe = self.current_fn_is_unsafe;
                    self.current_fn_is_async = m.is_async;
                    self.current_fn_is_unsafe = m.is_unsafe;
                    self.enter_scope();
                    let self_ty = Type::Ref(e.ident.clone());
                    let _ = self.define(
                        "this".to_string(),
                        SymbolKind::Var {
                            ty: self_ty.clone(),
                            kind: VarDeclKind::Let,
                        },
                        e.ident.span,
                    );
                    let _ = self.define(
                        "_".to_string(),
                        SymbolKind::Var {
                            ty: self_ty.clone(),
                            kind: VarDeclKind::Let,
                        },
                        e.ident.span,
                    );
                    let _ = self.define(
                        "self".to_string(),
                        SymbolKind::Var {
                            ty: self_ty.clone(),
                            kind: VarDeclKind::Let,
                        },
                        e.ident.span,
                    );
                    for p in &m.params {
                        if p.ident.sym != "this" && p.ident.sym != "_" && p.ident.sym != "self" {
                            let _ = self.define(
                                p.ident.sym.clone(),
                                SymbolKind::Var {
                                    ty: p.ty.clone(),
                                    kind: VarDeclKind::Let,
                                },
                                p.ident.span,
                            );
                        }
                    }
                    self.analyze_block(&m.body)?;
                    self.exit_scope();
                    self.current_fn_is_async = old_async;
                    self.current_fn_is_unsafe = old_unsafe;
                }
            }
            Decl::TypeAlias(ta) => {
                self.type_aliases
                    .insert(ta.ident.sym.clone(), ta.target_ty.clone());
            }
            _ => {}
        }
        Ok(())
    }

    fn bind_match_pattern(&mut self, pat: &Pattern) -> SemaResult<()> {
        match pat {
            Pattern::Ident(id) => {
                if let Some(SymbolKind::Var { .. }) = self.scopes.last().unwrap().get(&id.sym) {
                    return Err(SemaError::DuplicateVarDecl(id.sym.clone(), id.span));
                }
                self.define(
                    id.sym.clone(),
                    SymbolKind::Var {
                        ty: Type::Any,
                        kind: VarDeclKind::Let,
                    },
                    id.span,
                )?;
            }
            Pattern::Err(inner) => self.bind_match_pattern(inner)?,
            Pattern::Enum(ep) => {
                if let Some(tpats) = &ep.tuple_pats {
                    for tp in tpats {
                        self.bind_match_pattern(tp)?;
                    }
                }
                if let Some(props) = &ep.fields {
                    for p in props {
                        if let Some(vp) = &p.val {
                            self.bind_match_pattern(vp)?;
                        } else {
                            if let Some(SymbolKind::Var { .. }) =
                                self.scopes.last().unwrap().get(&p.key.sym)
                            {
                                return Err(SemaError::DuplicateVarDecl(
                                    p.key.sym.clone(),
                                    p.key.span,
                                ));
                            }
                            self.define(
                                p.key.sym.clone(),
                                SymbolKind::Var {
                                    ty: Type::Any,
                                    kind: VarDeclKind::Let,
                                },
                                p.key.span,
                            )?;
                        }
                    }
                }
            }
            Pattern::Tuple(els) | Pattern::Array(els) => {
                if els.iter().filter(|p| matches!(p, Pattern::Rest(_))).count() > 1 {
                    return Err(SemaError::MultipleRestInPattern(Span { start: 0, end: 0 }));
                }
                for el in els {
                    self.bind_match_pattern(el)?;
                }
            }
            Pattern::Rest(inner) => self.bind_match_pattern(inner)?,
            Pattern::Or(els) => {
                let base_scope = self.scopes.last().unwrap().clone();
                let mut merged = base_scope.clone();
                for el in els {
                    *self.scopes.last_mut().unwrap() = base_scope.clone();
                    self.bind_match_pattern(el)?;
                    for (k, v) in self.scopes.last().unwrap() {
                        merged.insert(k.clone(), v.clone());
                    }
                }
                *self.scopes.last_mut().unwrap() = merged;
            }
            Pattern::Object(props) => {
                let rest_count = props.iter().filter(|p| p.is_rest).count();
                if rest_count > 1 {
                    return Err(SemaError::MultipleRestInPattern(Span { start: 0, end: 0 }));
                }
                for p in props {
                    if p.key.sym == "*" {
                        continue;
                    }
                    if let Some(vp) = &p.val {
                        self.bind_match_pattern(vp)?;
                    } else {
                        if let Some(SymbolKind::Var { .. }) =
                            self.scopes.last().unwrap().get(&p.key.sym)
                        {
                            return Err(SemaError::DuplicateVarDecl(p.key.sym.clone(), p.key.span));
                        }
                        self.define(
                            p.key.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Let,
                            },
                            p.key.span,
                        )?;
                    }
                }
            }
            Pattern::Binding { ident, subpat } => {
                if let Some(SymbolKind::Var { .. }) = self.scopes.last().unwrap().get(&ident.sym) {
                    return Err(SemaError::DuplicateVarDecl(ident.sym.clone(), ident.span));
                }
                self.define(
                    ident.sym.clone(),
                    SymbolKind::Var {
                        ty: Type::Any,
                        kind: VarDeclKind::Let,
                    },
                    ident.span,
                )?;
                self.bind_match_pattern(subpat)?;
            }
            Pattern::TypeMatch { ident, ty } => {
                if let Some(SymbolKind::Var { .. }) = self.scopes.last().unwrap().get(&ident.sym) {
                    return Err(SemaError::DuplicateVarDecl(ident.sym.clone(), ident.span));
                }
                self.define(
                    ident.sym.clone(),
                    SymbolKind::Var {
                        ty: ty.clone(),
                        kind: VarDeclKind::Let,
                    },
                    ident.span,
                )?;
            }
            _ => {}
            _ => {}
        }
        Ok(())
    }

    fn bind_for_pattern(&mut self, pat: &Pattern, elem_ty: &Type) -> SemaResult<()> {
        match pat {
            Pattern::Ident(id) => {
                if let Some(SymbolKind::Var { .. }) = self.scopes.last().unwrap().get(&id.sym) {
                    return Err(SemaError::DuplicateVarDecl(id.sym.clone(), id.span));
                }
                self.define(
                    id.sym.clone(),
                    SymbolKind::Var {
                        ty: elem_ty.clone(),
                        kind: VarDeclKind::Let,
                    },
                    id.span,
                )?;
            }
            Pattern::Tuple(pats) | Pattern::Array(pats) => {
                if pats
                    .iter()
                    .filter(|p| matches!(p, Pattern::Rest(_)))
                    .count()
                    > 1
                {
                    return Err(SemaError::MultipleRestInPattern(Span { start: 0, end: 0 }));
                }
                for p in pats {
                    self.bind_for_pattern(p, &Type::Any)?;
                }
            }
            Pattern::Rest(inner) => {
                self.bind_for_pattern(inner, &Type::Array(Box::new(Type::Any)))?;
            }
            Pattern::Object(props) => {
                let rest_count = props.iter().filter(|p| p.is_rest).count();
                if rest_count > 1 {
                    return Err(SemaError::MultipleRestInPattern(Span { start: 0, end: 0 }));
                }
                for p in props {
                    if p.key.sym == "*" {
                        continue;
                    }
                    if let Some(vp) = &p.val {
                        self.bind_for_pattern(vp, &Type::Any)?;
                    } else {
                        if let Some(SymbolKind::Var { .. }) =
                            self.scopes.last().unwrap().get(&p.key.sym)
                        {
                            return Err(SemaError::DuplicateVarDecl(p.key.sym.clone(), p.key.span));
                        }
                        self.define(
                            p.key.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Let,
                            },
                            p.key.span,
                        )?;
                    }
                }
            }
            _ => self.bind_match_pattern(pat)?,
        }
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> SemaResult<()> {
        match stmt {
            Stmt::Var(var) => {
                let mut inferred_ty = Type::Unknown;
                if let Some(init) = &var.init {
                    inferred_ty = self.infer_type(init)?;
                    self.analyze_expr(init)?;
                }
                if let Some(ref target_ty) = var.ty {
                    if let Some(init) = &var.init {
                        if !self.can_coerce(target_ty, &inferred_ty, Some(init)) {
                            return Err(SemaError::TypeMismatch(
                                target_ty.clone(),
                                inferred_ty,
                                var.span,
                            ));
                        }
                    }
                }
                let ty = var.ty.clone().unwrap_or(inferred_ty);
                self.analyze_pattern(&var.pat, ty.clone(), &var.kind)?;
                if let Pattern::Ident(ident) = &var.pat {
                    let var_name = ident.sym.clone();
                    if let Some(known_len) = self.infer_known_length(&ty, var.init.as_ref()) {
                        self.set_known_length(var_name.clone(), known_len);
                    }
                    if var.kind == VarDeclKind::Const {
                        if let Some(init) = &var.init {
                            if let Some(val) = self.eval_int_expr(init) {
                                self.set_const_int(var_name, val);
                            }
                        }
                    }
                }
            }
            Stmt::Expr(expr) => {
                self.infer_type(expr)?;
                self.analyze_expr(expr)?;
            }
            Stmt::Block(block) => self.analyze_block(block)?,
            Stmt::ComptimeBlock(block) => self.analyze_block(block)?,
            Stmt::UnsafeBlock(block) => {
                self.unsafe_depth += 1;
                let res = self.analyze_block(block);
                self.unsafe_depth -= 1;
                res?;
            }
            Stmt::ArenaBlock(block) => self.analyze_block(block)?,
            Stmt::If(i) => {
                self.infer_type(&i.test)?;
                self.analyze_expr(&i.test)?;
                let not_null_vars = Self::extract_not_null_vars(&i.test);
                let is_null_vars = Self::extract_is_null_vars(&i.test);
                if let Some(cap) = &i.capture {
                    self.enter_scope();
                    self.define(
                        cap.sym.clone(),
                        SymbolKind::Var {
                            ty: Type::Any,
                            kind: if i.is_ptr_capture {
                                VarDeclKind::Let
                            } else {
                                VarDeclKind::Const
                            },
                        },
                        cap.span,
                    )?;
                    self.analyze_stmt(&i.cons)?;
                    self.exit_scope();
                } else if !not_null_vars.is_empty() {
                    self.enter_scope();
                    for v in &not_null_vars {
                        if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                            let unwrapped = Self::unwrap_optional_type(&ty);
                            self.define(
                                v.clone(),
                                SymbolKind::Var {
                                    ty: unwrapped,
                                    kind,
                                },
                                i.span,
                            )?;
                        }
                    }
                    self.analyze_stmt(&i.cons)?;
                    self.exit_scope();
                } else {
                    self.analyze_stmt(&i.cons)?;
                }
                if let Some(alt) = &i.alt {
                    if !is_null_vars.is_empty() {
                        self.enter_scope();
                        for v in &is_null_vars {
                            if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                                let unwrapped = Self::unwrap_optional_type(&ty);
                                self.define(
                                    v.clone(),
                                    SymbolKind::Var {
                                        ty: unwrapped,
                                        kind,
                                    },
                                    i.span,
                                )?;
                            }
                        }
                        self.analyze_stmt(alt)?;
                        self.exit_scope();
                    } else {
                        self.analyze_stmt(alt)?;
                    }
                }
            }
            Stmt::While(w) => {
                self.infer_type(&w.test)?;
                self.analyze_expr(&w.test)?;
                let not_null_vars = Self::extract_not_null_vars(&w.test);
                if let Some(cap) = &w.capture {
                    self.enter_scope();
                    self.define(
                        cap.sym.clone(),
                        SymbolKind::Var {
                            ty: Type::Any,
                            kind: if w.is_ptr_capture {
                                VarDeclKind::Let
                            } else {
                                VarDeclKind::Const
                            },
                        },
                        cap.span,
                    )?;
                    if let Some(c) = &w.continue_expr {
                        self.infer_type(c)?;
                        self.analyze_expr(c)?;
                    }
                    self.analyze_stmt(&w.body)?;
                    self.exit_scope();
                } else if !not_null_vars.is_empty() {
                    self.enter_scope();
                    for v in &not_null_vars {
                        if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                            let unwrapped = Self::unwrap_optional_type(&ty);
                            self.define(
                                v.clone(),
                                SymbolKind::Var {
                                    ty: unwrapped,
                                    kind,
                                },
                                w.span,
                            )?;
                        }
                    }
                    if let Some(c) = &w.continue_expr {
                        self.infer_type(c)?;
                        self.analyze_expr(c)?;
                    }
                    self.analyze_stmt(&w.body)?;
                    self.exit_scope();
                } else {
                    if let Some(c) = &w.continue_expr {
                        self.infer_type(c)?;
                        self.analyze_expr(c)?;
                    }
                    self.analyze_stmt(&w.body)?;
                }
            }
            Stmt::For(f) => {
                if f.raw_header.is_some() {
                    self.enter_scope();
                    if let Pattern::Ident(id) = &f.pat {
                        for part in id.sym.split(',') {
                            let clean_name = part.trim().trim_start_matches('*').trim();
                            if !clean_name.is_empty() {
                                let _ = self.define(
                                    clean_name.to_string(),
                                    SymbolKind::Var {
                                        ty: Type::Any,
                                        kind: VarDeclKind::Let,
                                    },
                                    id.span,
                                );
                            }
                        }
                    }
                    let res = match &*f.body {
                        Stmt::Block(b) => {
                            let mut r = Ok(());
                            for s in &b.body {
                                if let Err(e) = self.analyze_stmt(s) {
                                    r = Err(e);
                                    break;
                                }
                            }
                            r
                        }
                        other => self.analyze_stmt(other),
                    };
                    self.exit_scope();
                    return res;
                }
                let coll_ty = self.infer_type(&f.expr)?;
                self.analyze_expr(&f.expr)?;

                let coll_var_opt = if let Expr::Ident(id) = &f.expr {
                    if self.moved_vars.contains(&id.sym) {
                        return Err(SemaError::UseOfMovedValue(id.sym.clone(), id.span));
                    }
                    if f.mode == IterMode::Move {
                        self.moved_vars.insert(id.sym.clone());
                    }
                    Some(id.sym.clone())
                } else {
                    None
                };

                let elem_ty = match &coll_ty {
                    Type::Array(inner) => (**inner).clone(),
                    _ => Type::Any,
                };

                if let Some(ref coll_name) = coll_var_opt {
                    if f.mode != IterMode::Move {
                        self.loop_borrowed_vars.insert(coll_name.clone());
                    }
                }

                self.enter_scope();
                let bind_res = self.bind_for_pattern(&f.pat, &elem_ty);
                let res = match bind_res {
                    Ok(()) => match &*f.body {
                        Stmt::Block(b) => {
                            let mut r = Ok(());
                            for s in &b.body {
                                if let Err(e) = self.analyze_stmt(s) {
                                    r = Err(e);
                                    break;
                                }
                            }
                            r
                        }
                        other => self.analyze_stmt(other),
                    },
                    Err(e) => Err(e),
                };
                self.exit_scope();

                if let Some(ref coll_name) = coll_var_opt {
                    if f.mode != IterMode::Move {
                        self.loop_borrowed_vars.remove(coll_name);
                    }
                }
                res?;
            }
            Stmt::Return(r) => {
                if let Some(arg) = &r.arg {
                    self.infer_type(arg)?;
                    self.analyze_expr(arg)?;
                }
            }
            Stmt::Break | Stmt::Continue => {}
            Stmt::Match(m) => {
                self.infer_type(&m.disc)?;
                self.analyze_expr(&m.disc)?;
                for arm in &m.arms {
                    self.enter_scope();
                    self.bind_match_pattern(&arm.pat)?;
                    if let Some(cap) = &arm.capture {
                        let _ = self.define(
                            cap.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Let,
                            },
                            cap.span,
                        );
                    }
                    if let Some(g) = &arm.guard {
                        self.infer_type(g)?;
                        self.analyze_expr(g)?;
                    }
                    match &*arm.body {
                        Stmt::Block(b) => {
                            for s in &b.body {
                                self.analyze_stmt(s)?;
                            }
                        }
                        other => self.analyze_stmt(other)?,
                    }
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
            Stmt::Func(f) => {
                let params = f.params.iter().map(|p| p.ty.clone()).collect();
                let kind = SymbolKind::Func {
                    params,
                    ret: f.return_type.clone(),
                    is_async: f.is_async,
                    is_unsafe: f.is_unsafe,
                };
                let _ = self.define(f.ident.sym.clone(), kind.clone(), f.ident.span);
                self.symbol_map.insert(f.ident.span, kind);

                let old_async = self.current_fn_is_async;
                let old_unsafe = self.current_fn_is_unsafe;
                self.current_fn_is_async = f.is_async;
                self.current_fn_is_unsafe = f.is_unsafe;
                self.enter_scope();
                for p in &f.params {
                    let p_kind = SymbolKind::Var {
                        ty: p.ty.clone(),
                        kind: VarDeclKind::Let,
                    };
                    let _ = self.define(p.ident.sym.clone(), p_kind.clone(), p.ident.span);
                    self.symbol_map.insert(p.ident.span, p_kind);
                }
                for stmt in &f.body.body {
                    self.analyze_stmt(stmt)?;
                }
                self.exit_scope();
                self.current_fn_is_async = old_async;
                self.current_fn_is_unsafe = old_unsafe;
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_expr(&mut self, expr: &Expr) -> SemaResult<()> {
        match expr {
            Expr::Ident(ident) => {
                if self.moved_vars.contains(&ident.sym) {
                    return Err(SemaError::UseOfMovedValue(ident.sym.clone(), ident.span));
                }
                if let Some(kind) = self.resolve(&ident.sym) {
                    self.symbol_map.insert(ident.span, kind.clone());
                }
            }
            Expr::Bin(bin) => {
                if bin.op == BinaryOp::Assign {
                    if let Expr::Ident(id) = &*bin.left {
                        if id.sym == "_" {
                            self.analyze_expr(&bin.right)?;
                            return Ok(());
                        }
                        if let Some(SymbolKind::Var {
                            kind: VarDeclKind::Const,
                            ..
                        }) = self.resolve(&id.sym)
                        {
                            return Err(SemaError::ConstAssignment(id.sym.clone(), bin.span));
                        }
                        if self.loop_borrowed_vars.contains(&id.sym) {
                            return Err(SemaError::MutateWhileBorrowedInFor(
                                id.sym.clone(),
                                id.span,
                            ));
                        }
                    } else if let Expr::Index { obj, index } = &*bin.left {
                        if let Expr::Ident(id) = &**obj {
                            if let Some(SymbolKind::Var {
                                kind: VarDeclKind::Const,
                                ..
                            }) = self.resolve(&id.sym)
                            {
                                return Err(SemaError::ConstAssignment(id.sym.clone(), bin.span));
                            }
                        }
                        self.check_index_expr(obj, index, true, bin.span)?;
                    }
                }
                self.analyze_expr(&bin.left)?;
                if bin.op == BinaryOp::Pipe {
                    let transformed = transform_pipeline_rhs(&bin.right, "__pipe_mock");
                    self.analyze_expr(&transformed)?;
                } else if bin.op == BinaryOp::LogicalAnd {
                    let not_null_vars = Self::extract_not_null_vars(&bin.left);
                    if !not_null_vars.is_empty() {
                        self.enter_scope();
                        for v in &not_null_vars {
                            if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                                let unwrapped = Self::unwrap_optional_type(&ty);
                                let _ = self.define(
                                    v.clone(),
                                    SymbolKind::Var {
                                        ty: unwrapped,
                                        kind,
                                    },
                                    bin.span,
                                );
                            }
                        }
                        self.analyze_expr(&bin.right)?;
                        self.exit_scope();
                    } else {
                        self.analyze_expr(&bin.right)?;
                    }
                } else {
                    self.analyze_expr(&bin.right)?;
                }
            }
            Expr::PipePlaceholder(span) => {
                return Err(SemaError::PipePlaceholderOutsidePipeline(*span));
            }
            Expr::Call(call) | Expr::OptCall(call) => {
                if let Expr::Ident(fid) = &*call.callee {
                    if fid.sym == "num" || fid.sym == "Number" {
                        return Err(SemaError::InvalidCast(
                            "CP does not support 'num()' or 'Number()'. CP is strongly typed: use specific numeric types like i32(x), u32(x), usize(x), or f64(x).".to_string(),
                            call.span,
                        ));
                    }
                    if let Some(SymbolKind::Func {
                        is_unsafe: true, ..
                    }) = self.resolve(&fid.sym)
                    {
                        if !self.is_in_unsafe() {
                            return Err(SemaError::UnsafeRequired(
                                format!(
                                    "Call to unsafe function '{}' requires an unsafe block or unsafe function",
                                    fid.sym
                                ),
                                call.span,
                            ));
                        }
                    }
                }
                if let Expr::Member(mem) = &*call.callee {
                    if let Ok(obj_ty) = self.infer_type(&mem.obj) {
                        let base_ty = match &obj_ty {
                            Type::BorrowType { inner, .. } | Type::RawPtr { inner, .. } => {
                                inner.as_ref()
                            }
                            other => other,
                        };
                        if let Type::Ref(cid) = base_ty {
                            if let Some(cls) = self.classes.get(&cid.sym) {
                                if let Some(m) =
                                    cls.methods.iter().find(|m| m.ident.sym == mem.prop.sym)
                                {
                                    let is_lifecycle = m.ident.sym == "init"
                                        || m.ident.sym == "release"
                                        || m.ident.sym == "_create"
                                        || m.ident.sym.starts_with("__");
                                    if !m.is_pub
                                        && !is_lifecycle
                                        && self.current_class.as_deref() != Some(&cid.sym)
                                    {
                                        return Err(SemaError::PrivateAccess(
                                            format!(
                                                "Cannot call private method '{}' of class '{}'",
                                                mem.prop.sym, cid.sym
                                            ),
                                            mem.prop.span,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    if let Expr::Ident(id) = &*mem.obj {
                        if self.loop_borrowed_vars.contains(&id.sym) {
                            let prop = mem.prop.sym.as_str();
                            if prop == "push"
                                || prop == "push_back"
                                || prop == "push_front"
                                || prop == "clear"
                                || prop == "delete"
                                || prop == "pop"
                                || prop == "pop_back"
                                || prop == "pop_front"
                                || prop == "set"
                                || prop == "put_all"
                                || prop == "add"
                                || prop == "insert"
                                || prop == "remove"
                            {
                                return Err(SemaError::MutateWhileBorrowedInFor(
                                    id.sym.clone(),
                                    mem.prop.span,
                                ));
                            }
                        }
                    }
                }
                // Generic trait bounds checking
                if let Expr::Ident(id) = &*call.callee {
                    if let Some(func) = self.functions.get(&id.sym).cloned() {
                        if !func.generic_params.is_empty() {
                            for gp in &func.generic_params {
                                if let GenericParam::Type {
                                    ident: gp_id,
                                    bounds,
                                } = gp
                                {
                                    if !bounds.is_empty() {
                                        for (idx, p) in func.params.iter().enumerate() {
                                            let mut p_ty = &p.ty;
                                            while let Type::BorrowType { inner, .. } = p_ty {
                                                p_ty = inner;
                                            }
                                            if let Type::Ref(pid) = p_ty {
                                                if pid.sym == gp_id.sym && idx < call.args.len() {
                                                    if let Ok(arg_ty) =
                                                        self.infer_type(&call.args[idx])
                                                    {
                                                        for bound in bounds {
                                                            self.check_trait_bound(
                                                                &arg_ty,
                                                                &bound.sym,
                                                                call.args[idx].span(),
                                                            )?;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if let Expr::Member(mem) = &*call.callee {
                    if let Ok(obj_ty) = self.infer_type(&mem.obj) {
                        let mut base_ty = &obj_ty;
                        while let Type::BorrowType { inner, .. } = base_ty {
                            base_ty = inner;
                        }
                        if let Type::Ref(cid) = base_ty {
                            if let Some(cls) = self.classes.get(&cid.sym).cloned() {
                                if let Some(m) =
                                    cls.methods.iter().find(|m| m.ident.sym == mem.prop.sym)
                                {
                                    if !m.generic_params.is_empty() {
                                        for gp in &m.generic_params {
                                            if let GenericParam::Type {
                                                ident: gp_id,
                                                bounds,
                                            } = gp
                                            {
                                                if !bounds.is_empty() {
                                                    let method_params: Vec<&Param> = m
                                                        .params
                                                        .iter()
                                                        .filter(|p| {
                                                            p.ident.sym != "this"
                                                                && p.ident.sym != "_"
                                                                && p.ident.sym != "self"
                                                        })
                                                        .collect();
                                                    for (idx, p) in method_params.iter().enumerate()
                                                    {
                                                        let mut p_ty = &p.ty;
                                                        while let Type::BorrowType {
                                                            inner, ..
                                                        } = p_ty
                                                        {
                                                            p_ty = inner;
                                                        }
                                                        if let Type::Ref(pid) = p_ty {
                                                            if pid.sym == gp_id.sym
                                                                && idx < call.args.len()
                                                            {
                                                                if let Ok(arg_ty) =
                                                                    self.infer_type(&call.args[idx])
                                                                {
                                                                    for bound in bounds {
                                                                        self.check_trait_bound(
                                                                            &arg_ty,
                                                                            &bound.sym,
                                                                            call.args[idx].span(),
                                                                        )?;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                self.analyze_expr(&call.callee)?;
                for arg in &call.args {
                    self.analyze_expr(arg)?;
                }
            }
            Expr::New(call) => {
                if let Expr::Ident(id) = &*call.callee {
                    if let Some(cls) = self.classes.get(&id.sym).cloned() {
                        if cls.is_static {
                            return Err(SemaError::CannotInstantiateStatic(
                                id.sym.clone(),
                                call.span,
                            ));
                        }
                        if !cls.generic_params.is_empty() {
                            for gp in &cls.generic_params {
                                if let GenericParam::Type {
                                    ident: gp_id,
                                    bounds,
                                } = gp
                                {
                                    if !bounds.is_empty() {
                                        if let Some(init_m) =
                                            cls.methods.iter().find(|m| m.ident.sym == "init")
                                        {
                                            let init_params: Vec<&Param> = init_m
                                                .params
                                                .iter()
                                                .filter(|p| {
                                                    p.ident.sym != "this"
                                                        && p.ident.sym != "_"
                                                        && p.ident.sym != "self"
                                                })
                                                .collect();
                                            for (idx, p) in init_params.iter().enumerate() {
                                                let mut p_ty = &p.ty;
                                                while let Type::BorrowType { inner, .. } = p_ty {
                                                    p_ty = inner;
                                                }
                                                if let Type::Ref(pid) = p_ty {
                                                    if pid.sym == gp_id.sym && idx < call.args.len()
                                                    {
                                                        if let Ok(arg_ty) =
                                                            self.infer_type(&call.args[idx])
                                                        {
                                                            for bound in bounds {
                                                                self.check_trait_bound(
                                                                    &arg_ty,
                                                                    &bound.sym,
                                                                    call.args[idx].span(),
                                                                )?;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                self.analyze_expr(&call.callee)?;
                for arg in &call.args {
                    self.analyze_expr(arg)?;
                }
            }
            Expr::Member(member) | Expr::OptMember(member) => {
                self.analyze_expr(&member.obj)?;
                if let Ok(obj_ty) = self.infer_type(&member.obj) {
                    let base_ty = match &obj_ty {
                        Type::BorrowType { inner, .. } | Type::RawPtr { inner, .. } => {
                            inner.as_ref()
                        }
                        other => other,
                    };
                    if let Type::Ref(cid) = base_ty {
                        if let Some(cls) = self.classes.get(&cid.sym) {
                            if let Some(field) =
                                cls.fields.iter().find(|f| f.ident.sym == member.prop.sym)
                            {
                                if !field.is_pub && self.current_class.as_deref() != Some(&cid.sym)
                                {
                                    return Err(SemaError::PrivateAccess(
                                        format!(
                                            "Cannot access private field '{}' of class '{}'",
                                            member.prop.sym, cid.sym
                                        ),
                                        member.prop.span,
                                    ));
                                }
                            }
                        }
                    }
                }
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
                    self.bind_match_pattern(&arm.pat)?;
                    if let Some(cap) = &arm.capture {
                        let _ = self.define(
                            cap.sym.clone(),
                            SymbolKind::Var {
                                ty: Type::Any,
                                kind: VarDeclKind::Let,
                            },
                            cap.span,
                        );
                    }
                    if let Some(g) = &arm.guard {
                        self.analyze_expr(g)?;
                    }
                    match &*arm.body {
                        Stmt::Block(b) => {
                            for s in &b.body {
                                self.analyze_stmt(s)?;
                            }
                        }
                        other => self.analyze_stmt(other)?,
                    }
                    self.exit_scope();
                }
            }
            Expr::Array(elements) => {
                for el in elements {
                    self.analyze_expr(el)?;
                }
            }
            Expr::ArrayRepeat(val, count) => {
                self.analyze_expr(val)?;
                self.analyze_expr(count)?;
            }
            Expr::Object(fields) | Expr::Dict(fields) => {
                for f in fields {
                    self.analyze_expr(&f.val)?;
                }
            }
            Expr::Index { obj, index } => {
                self.analyze_expr(obj)?;
                self.analyze_expr(index)?;
                let span = crate::Span {
                    start: inner_span(obj).start,
                    end: inner_span(index).end,
                };
                self.check_index_expr(obj, index, false, span)?;
            }
            Expr::Await(inner)
            | Expr::Question(inner)
            | Expr::ErrWrap(inner)
            | Expr::BoxNew(inner) => {
                self.analyze_expr(inner)?;
            }
            Expr::Move(inner) => {
                self.analyze_expr(inner)?;
                if let Expr::Ident(id) = &**inner {
                    self.moved_vars.insert(id.sym.clone());
                }
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
            Expr::Range { start, end, .. } => {
                self.analyze_expr(start)?;
                self.analyze_expr(end)?;
            }
            Expr::Cast {
                expr,
                target_ty,
                span,
            } => {
                self.analyze_expr(expr)?;
                let src_ty = self.infer_type(expr).unwrap_or(Type::Unknown);

                // Layer 3: Raw pointer casting is only allowed inside unsafe block
                let is_src_ptr = matches!(src_ty, Type::RawPtr { .. });
                let is_target_ptr = matches!(target_ty, Type::RawPtr { .. });
                if (is_src_ptr || is_target_ptr) && !self.is_in_unsafe() {
                    return Err(SemaError::UnsafeRequired(
                        "Raw pointer casting is only allowed inside unsafe block".to_string(),
                        *span,
                    ));
                }

                // Layer 4: Semantic conversions forbidden via `as`
                if src_ty == Type::String && Self::is_numeric_type(target_ty) {
                    return Err(SemaError::InvalidCast(
                        format!(
                            "Cannot cast 'string' to numeric type '{}' via 'as'; use '{}.parse(...) ?' instead",
                            Self::type_repr(target_ty),
                            Self::type_repr(target_ty)
                        ),
                        *span,
                    ));
                }
                if Self::is_numeric_type(&src_ty) && *target_ty == Type::String {
                    return Err(SemaError::InvalidCast(
                        format!(
                            "Cannot cast numeric type '{}' to 'string' via 'as'; use '.to_string()' instead",
                            Self::type_repr(&src_ty)
                        ),
                        *span,
                    ));
                }
            }
            Expr::Deref(inner) => {
                self.analyze_expr(inner)?;
                if let Ok(ty) = self.infer_type(inner) {
                    if matches!(ty, Type::RawPtr { .. }) && !self.is_in_unsafe() {
                        return Err(SemaError::UnsafeRequired(
                            "Dereferencing raw pointer requires an unsafe block or unsafe function"
                                .to_string(),
                            inner_span(inner),
                        ));
                    }
                }
            }
            Expr::Unsafe(block) => {
                self.unsafe_depth += 1;
                let res = self.analyze_block(block);
                self.unsafe_depth -= 1;
                res?;
            }
            Expr::Comptime(block) => {
                self.analyze_block(block)?;
            }
            Expr::Arena(block) => {
                self.analyze_block(block)?;
            }
            Expr::Template(t) => {
                for e in &t.expressions {
                    self.analyze_expr(e)?;
                }
            }
            Expr::If {
                test, cons, alt, ..
            } => {
                self.analyze_expr(test)?;
                let not_null_vars = Self::extract_not_null_vars(test);
                let is_null_vars = Self::extract_is_null_vars(test);
                if !not_null_vars.is_empty() {
                    self.enter_scope();
                    for v in &not_null_vars {
                        if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                            let unwrapped = Self::unwrap_optional_type(&ty);
                            let _ = self.define(
                                v.clone(),
                                SymbolKind::Var {
                                    ty: unwrapped,
                                    kind,
                                },
                                Span { start: 0, end: 0 },
                            );
                        }
                    }
                    self.analyze_expr(cons)?;
                    self.exit_scope();
                } else {
                    self.analyze_expr(cons)?;
                }
                if let Some(a) = alt {
                    if !is_null_vars.is_empty() {
                        self.enter_scope();
                        for v in &is_null_vars {
                            if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                                let unwrapped = Self::unwrap_optional_type(&ty);
                                let _ = self.define(
                                    v.clone(),
                                    SymbolKind::Var {
                                        ty: unwrapped,
                                        kind,
                                    },
                                    Span { start: 0, end: 0 },
                                );
                            }
                        }
                        self.analyze_expr(a)?;
                        self.exit_scope();
                    } else {
                        self.analyze_expr(a)?;
                    }
                }
            }
            Expr::LabeledBlock { block, .. } => {
                self.analyze_block(block)?;
            }
            Expr::LabeledBreak(_, _, val) => {
                if let Some(v) = val {
                    self.analyze_expr(v)?;
                }
            }
            _ => {}
            _ => {}
        }
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.known_lengths_stack.push(HashMap::new());
        self.const_int_vars_stack.push(HashMap::new());
    }
    fn exit_scope(&mut self) {
        self.scopes.pop();
        self.known_lengths_stack.pop();
        self.const_int_vars_stack.pop();
    }

    pub fn get_known_length(&self, name: &str) -> Option<usize> {
        for scope in self.known_lengths_stack.iter().rev() {
            if let Some(len) = scope.get(name) {
                return Some(*len);
            }
        }
        None
    }

    pub fn set_known_length(&mut self, name: String, len: usize) {
        if let Some(scope) = self.known_lengths_stack.last_mut() {
            scope.insert(name, len);
        }
    }

    pub fn get_const_int(&self, name: &str) -> Option<i64> {
        for scope in self.const_int_vars_stack.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(*val);
            }
        }
        None
    }

    pub fn set_const_int(&mut self, name: String, val: i64) {
        if let Some(scope) = self.const_int_vars_stack.last_mut() {
            scope.insert(name, val);
        }
    }

    pub fn eval_int_expr(&self, expr: &Expr) -> Option<i64> {
        match expr {
            Expr::Lit(Lit::Int(n)) => Some(*n),
            Expr::Ident(id) => self.get_const_int(&id.sym),
            Expr::Bin(bin) => {
                let l = self.eval_int_expr(&bin.left)?;
                let r = self.eval_int_expr(&bin.right)?;
                match bin.op {
                    BinaryOp::Add => Some(l.wrapping_add(r)),
                    BinaryOp::Sub => Some(l.wrapping_sub(r)),
                    BinaryOp::Mul => Some(l.wrapping_mul(r)),
                    BinaryOp::Div => {
                        if r != 0 {
                            Some(l.wrapping_div(r))
                        } else {
                            None
                        }
                    }
                    BinaryOp::Mod => {
                        if r != 0 {
                            Some(l.wrapping_rem(r))
                        } else {
                            None
                        }
                    }
                    BinaryOp::Shl => {
                        if (0..64).contains(&r) {
                            Some(l << r)
                        } else {
                            None
                        }
                    }
                    BinaryOp::Shr => {
                        if (0..64).contains(&r) {
                            Some(l >> r)
                        } else {
                            None
                        }
                    }
                    BinaryOp::BitAnd => Some(l & r),
                    BinaryOp::BitOr => Some(l | r),
                    BinaryOp::BitXor => Some(l ^ r),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn infer_known_length(&self, ty: &Type, init_opt: Option<&Expr>) -> Option<usize> {
        let mut curr_ty = ty;
        while let Type::BorrowType { inner, .. } = curr_ty {
            curr_ty = inner;
        }
        if let Type::FixedArray(_, len) = curr_ty {
            return Some(*len);
        }

        if let Some(init) = init_opt {
            match init {
                Expr::Lit(Lit::Str(s)) => {
                    return Some(s.len());
                }
                Expr::Array(elements) => {
                    return Some(elements.len());
                }
                Expr::ArrayRepeat(_, count) => {
                    if let Some(n) = self.eval_int_expr(count) {
                        if n >= 0 {
                            return Some(n as usize);
                        }
                    }
                }
                Expr::Borrow(inner, _) => match &**inner {
                    Expr::Ident(id) => {
                        if let Some(len) = self.get_known_length(&id.sym) {
                            return Some(len);
                        }
                    }
                    Expr::Index { obj, index } => {
                        if let Expr::Ident(id) = &**obj {
                            if let Some(buf_len) = self.get_known_length(&id.sym) {
                                if let Some(idx_val) = self.eval_int_expr(index) {
                                    if idx_val >= 0 && (idx_val as usize) <= buf_len {
                                        return Some(buf_len - idx_val as usize);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Expr::Slice(slice) => {
                    if let (Some(s_expr), Some(e_expr)) = (&slice.start, &slice.end) {
                        if let (Some(s_val), Some(e_val)) =
                            (self.eval_int_expr(s_expr), self.eval_int_expr(e_expr))
                        {
                            if e_val >= s_val && s_val >= 0 {
                                return Some((e_val - s_val) as usize);
                            }
                        }
                    }
                }
                Expr::Ident(id) => {
                    if let Some(len) = self.get_known_length(&id.sym) {
                        return Some(len);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn check_index_expr(
        &mut self,
        obj: &Expr,
        index: &Expr,
        is_write: bool,
        span: Span,
    ) -> SemaResult<()> {
        let idx_opt = self.eval_int_expr(index);

        if let Some(idx) = idx_opt {
            if idx < 0 {
                return Err(SemaError::NegativeIndex { index: idx, span });
            }
        }

        let mut known_len = None;
        if let Expr::Ident(id) = obj {
            known_len = self.get_known_length(&id.sym);
        }
        if known_len.is_none() {
            if let Ok(obj_ty) = self.infer_type(obj) {
                let mut curr_ty = &obj_ty;
                while let Type::BorrowType { inner, .. } = curr_ty {
                    curr_ty = inner;
                }
                if let Type::FixedArray(_, len) = curr_ty {
                    known_len = Some(*len);
                }
            }
        }
        if known_len.is_none() {
            if let Expr::Lit(Lit::Str(s)) = obj {
                known_len = Some(s.len());
            }
        }

        if let (Some(idx), Some(len)) = (idx_opt, known_len) {
            if (idx as usize) >= len {
                return Err(SemaError::CompileTimeIndexOutOfBounds {
                    index: idx,
                    len,
                    span,
                });
            }
        }

        if is_write {
            let obj_ty_res = self.infer_type(obj);
            if let Ok(ty) = obj_ty_res {
                let mut base_ty = &ty;
                while let Type::BorrowType { inner, .. } = base_ty {
                    base_ty = inner;
                }
                let is_unbounded_ptr_or_cstr = matches!(
                    base_ty,
                    Type::ManyRawPtr(_)
                        | Type::CPtr(_)
                        | Type::SentinelPtr(_, _)
                        | Type::CStr
                        | Type::RawPtr { .. }
                ) || matches!(base_ty, Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr");

                if is_unbounded_ptr_or_cstr && !self.is_in_unsafe() {
                    let is_statically_proven_safe =
                        if let (Some(idx), Some(len)) = (idx_opt, known_len) {
                            idx >= 0 && (idx as usize) < len
                        } else {
                            false
                        };

                    if !is_statically_proven_safe {
                        let var_name = if let Expr::Ident(id) = obj {
                            id.sym.clone()
                        } else {
                            "pointer".to_string()
                        };
                        return Err(SemaError::UnboundedRawPointerMutation(var_name, span));
                    }
                }
            }
        }

        Ok(())
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
            _ => {}
        }
        vars
    }

    fn is_int_type(t: &Type) -> bool {
        match t {
            Type::Conditional(_, cons, alt) => Self::is_int_type(cons) && Self::is_int_type(alt),
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
            | Type::USize => true,
            Type::Ref(id) => {
                matches!(
                    id.sym.as_str(),
                    "c_int"
                        | "c_uint"
                        | "c_short"
                        | "c_ushort"
                        | "c_long"
                        | "c_ulong"
                        | "c_char"
                        | "c_uchar"
                        | "ares_socket_t"
                        | "ares_socklen_t"
                        | "ares_ssize_t"
                ) || ((id.sym.starts_with('u') || id.sym.starts_with('i'))
                    && id.sym.len() > 1
                    && id.sym[1..].chars().all(|c| c.is_ascii_digit()))
            }
            _ => false,
        }
    }

    fn is_float_type(t: &Type) -> bool {
        matches!(t, Type::F32 | Type::F64)
    }

    fn is_numeric_type(t: &Type) -> bool {
        Self::is_int_type(t) || Self::is_float_type(t)
    }

    fn type_repr(t: &Type) -> String {
        match t {
            Type::U8 => "u8".to_string(),
            Type::I8 => "i8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::I16 => "i16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::I32 => "i32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U128 => "u128".to_string(),
            Type::I128 => "i128".to_string(),
            Type::USize => "usize".to_string(),
            Type::ISize => "isize".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Any => "any".to_string(),
            Type::Ref(id) => id.sym.clone(),
            Type::Conditional(_cond, cons, alt) => {
                format!(
                    "@if (...) {} else {}",
                    Self::type_repr(cons),
                    Self::type_repr(alt)
                )
            }
            _ => format!("{:?}", t),
        }
    }

    fn int_fits_in_type(val: i64, target: &Type) -> bool {
        match target {
            Type::Conditional(_, cons, alt) => {
                Self::int_fits_in_type(val, cons) || Self::int_fits_in_type(val, alt)
            }
            Type::U8 => (0..=255).contains(&val),
            Type::I8 => (-128..=127).contains(&val),
            Type::U16 => (0..=65535).contains(&val),
            Type::I16 => (-32768..=32767).contains(&val),
            Type::U32 => val >= 0 && val <= u32::MAX as i64,
            Type::I32 => val >= i32::MIN as i64 && val <= i32::MAX as i64,
            Type::U64 | Type::U128 | Type::USize => true,
            Type::I64 | Type::I128 | Type::ISize => true,
            Type::F32 | Type::F64 => true,
            Type::Ref(id)
                if matches!(
                    id.sym.as_str(),
                    "c_int"
                        | "c_uint"
                        | "c_short"
                        | "c_ushort"
                        | "c_long"
                        | "c_ulong"
                        | "c_char"
                        | "c_uchar"
                        | "ares_socket_t"
                        | "ares_socklen_t"
                        | "ares_ssize_t"
                ) =>
            {
                true
            }
            Type::Ref(id)
                if (id.sym.starts_with('u') || id.sym.starts_with('i'))
                    && id.sym[1..].chars().all(|c| c.is_ascii_digit()) =>
            {
                if let Ok(bits) = id.sym[1..].parse::<u32>() {
                    if id.sym.starts_with('u') {
                        val >= 0 && (bits >= 64 || val < (1i64 << bits.min(62)))
                    } else {
                        bits >= 64
                            || (val >= -(1i64 << (bits - 1).min(62))
                                && val < (1i64 << (bits - 1).min(62)))
                    }
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    fn eval_const_int_expr(expr: &Expr) -> Option<i64> {
        match expr {
            Expr::Lit(Lit::Int(n)) => Some(*n),
            Expr::Bin(bin) => {
                let l = Self::eval_const_int_expr(&bin.left)?;
                let r = Self::eval_const_int_expr(&bin.right)?;
                match bin.op {
                    BinaryOp::Add => Some(l.wrapping_add(r)),
                    BinaryOp::Sub => Some(l.wrapping_sub(r)),
                    BinaryOp::Mul => Some(l.wrapping_mul(r)),
                    BinaryOp::Div => {
                        if r != 0 {
                            Some(l.wrapping_div(r))
                        } else {
                            None
                        }
                    }
                    BinaryOp::Mod => {
                        if r != 0 {
                            Some(l.wrapping_rem(r))
                        } else {
                            None
                        }
                    }
                    BinaryOp::Shl => {
                        if (0..64).contains(&r) {
                            Some(l << r)
                        } else {
                            None
                        }
                    }
                    BinaryOp::Shr => {
                        if (0..64).contains(&r) {
                            Some(l >> r)
                        } else {
                            None
                        }
                    }
                    BinaryOp::BitAnd => Some(l & r),
                    BinaryOp::BitOr => Some(l | r),
                    BinaryOp::BitXor => Some(l ^ r),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn is_safe_widening(target: &Type, src: &Type) -> bool {
        if target == src {
            return true;
        }
        // Small unsigned -> larger unsigned
        match (src, target) {
            (Type::U8, Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::USize) => true,
            (Type::U16, Type::U32 | Type::U64 | Type::U128 | Type::USize) => true,
            (Type::U32, Type::U64 | Type::U128 | Type::USize) => true,
            (Type::U64, Type::U128) => true,
            (Type::U64, Type::USize) | (Type::USize, Type::U64) => true,

            // Small signed -> larger signed
            (Type::I8, Type::I16 | Type::I32 | Type::I64 | Type::I128 | Type::ISize) => true,
            (Type::I16, Type::I32 | Type::I64 | Type::I128 | Type::ISize) => true,
            (Type::I32, Type::I64 | Type::I128 | Type::ISize) => true,
            (Type::I64, Type::I128) => true,
            (Type::I64, Type::ISize) | (Type::ISize, Type::I64) => true,

            // Int / Float to larger Float
            (Type::I32, Type::F64) => true,
            (Type::F32, Type::F64) => true,
            (Type::I8 | Type::U8 | Type::I16 | Type::U16, Type::F32 | Type::F64) => true,

            _ => false,
        }
    }

    fn is_subclass(&self, derived: &str, base: &str) -> bool {
        let mut curr = derived;
        while let Some(cls) = self.classes.get(curr) {
            if let Some(ref ext) = cls.extends {
                if ext.sym == base {
                    return true;
                }
                curr = &ext.sym;
            } else {
                break;
            }
        }
        false
    }

    fn is_builtin_trait(name: &str) -> bool {
        matches!(name, "Comparable" | "ToString" | "Clone" | "Hash" | "Debug")
    }

    fn is_primitive_scalar(ty: &Type) -> bool {
        matches!(
            ty,
            Type::U8
                | Type::I8
                | Type::U16
                | Type::I16
                | Type::I32
                | Type::U32
                | Type::I64
                | Type::U64
                | Type::I128
                | Type::U128
                | Type::USize
                | Type::ISize
                | Type::F32
                | Type::F64
                | Type::Bool
                | Type::String
        )
    }

    pub fn check_trait_bound(&self, ty: &Type, bound_name: &str, span: Span) -> SemaResult<()> {
        if bound_name == "Any" {
            return Ok(());
        }

        let mut actual_ty = ty;
        while let Type::BorrowType { inner, .. } = actual_ty {
            actual_ty = inner;
        }

        // 1. If actual_ty is a generic type parameter in the current scope with this bound
        if let Type::Ref(id) = actual_ty {
            if let Some(bounds) = self.active_generic_bounds.get(&id.sym) {
                if bounds.iter().any(|b| b == bound_name) {
                    return Ok(());
                }
            }
        }

        // 2. Built-in traits inherently satisfied by primitive scalar types
        if Self::is_builtin_trait(bound_name) && Self::is_primitive_scalar(actual_ty) {
            return Ok(());
        }

        // 3. For classes/structs (Type::Ref)
        if let Type::Ref(id) = actual_ty {
            if let Some(cls) = self.classes.get(&id.sym) {
                // A. Explicit 'implements'
                if cls.implements.iter().any(|i| i.sym == bound_name) {
                    return Ok(());
                }

                // B. External 'impl Trait for Target'
                if self
                    .impls
                    .iter()
                    .any(|i| i.trait_name.sym == bound_name && i.target_name.sym == id.sym)
                {
                    return Ok(());
                }

                // C. Trait declared in code: check required methods
                if let Some(tr) = self.traits.get(bound_name) {
                    let mut missing = None;
                    for tm in &tr.methods {
                        if tm.default_body.is_none() {
                            let has_m = cls.methods.iter().any(|m| m.ident.sym == tm.ident.sym)
                                || self.impls.iter().any(|i| {
                                    i.trait_name.sym == bound_name
                                        && i.target_name.sym == id.sym
                                        && i.methods.iter().any(|m| m.ident.sym == tm.ident.sym)
                                });
                            if !has_m {
                                missing = Some(tm.ident.sym.clone());
                                break;
                            }
                        }
                    }
                    if missing.is_none() {
                        return Ok(());
                    } else {
                        return Err(SemaError::TraitBoundNotSatisfied {
                            type_name: id.sym.clone(),
                            trait_name: bound_name.to_string(),
                            missing_method: missing,
                            span,
                        });
                    }
                }

                // D. Built-in traits satisfied by method conventions on classes
                match bound_name {
                    "ToString"
                        if cls
                            .methods
                            .iter()
                            .any(|m| m.ident.sym == "toString" || m.ident.sym == "to_string") =>
                    {
                        return Ok(());
                    }
                    "Comparable"
                        if cls
                            .methods
                            .iter()
                            .any(|m| m.ident.sym == "compare" || m.ident.sym == "cmp") =>
                    {
                        return Ok(());
                    }
                    "Clone" if cls.methods.iter().any(|m| m.ident.sym == "clone") => {
                        return Ok(());
                    }
                    "Hash" if cls.methods.iter().any(|m| m.ident.sym == "hash") => {
                        return Ok(());
                    }
                    "Debug"
                        if cls
                            .methods
                            .iter()
                            .any(|m| m.ident.sym == "debug" || m.ident.sym == "format") =>
                    {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // 4. Any / Auto / Unknown
        if matches!(actual_ty, Type::Any | Type::Auto | Type::Unknown) {
            return Ok(());
        }

        Err(SemaError::TraitBoundNotSatisfied {
            type_name: Self::type_repr(actual_ty),
            trait_name: bound_name.to_string(),
            missing_method: None,
            span,
        })
    }

    pub fn resolve_type(&self, ty: &Type) -> Type {
        let mut curr = ty.clone();
        for _ in 0..16 {
            match &curr {
                Type::Ref(id) => {
                    if let Some(target) = self.type_aliases.get(&id.sym) {
                        curr = target.clone();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        curr
    }

    pub fn can_coerce(&self, target: &Type, src: &Type, init_expr: Option<&Expr>) -> bool {
        if target == src {
            return true;
        }
        if *target == Type::Any
            || *src == Type::Any
            || *target == Type::Unknown
            || *src == Type::Unknown
            || *target == Type::Auto
            || *src == Type::Auto
        {
            return true;
        }

        let resolved_target = self.resolve_type(target);
        let resolved_src = self.resolve_type(src);
        if &resolved_target != target || &resolved_src != src {
            if self.can_coerce(&resolved_target, &resolved_src, init_expr) {
                return true;
            }
        }

        if let Type::Conditional(_, cons, alt) = target {
            if self.can_coerce(cons, src, init_expr) || self.can_coerce(alt, src, init_expr) {
                return true;
            }
        }
        if let Type::Conditional(_, cons, alt) = src {
            if self.can_coerce(target, cons, init_expr) || self.can_coerce(target, alt, init_expr) {
                return true;
            }
        }

        // Layer 1.1: Literal and Const Expr fitting
        if let Some(expr) = init_expr {
            if let Some(val) = Self::eval_const_int_expr(expr) {
                if Self::int_fits_in_type(val, target) {
                    return true;
                }
            }
            match expr {
                Expr::Lit(lit) => match lit {
                    Lit::Int(val) => {
                        if Self::int_fits_in_type(*val, target) {
                            return true;
                        }
                    }
                    Lit::Float(_) => {
                        if matches!(target, Type::F32 | Type::F64) {
                            return true;
                        }
                    }
                    Lit::Null => {
                        if matches!(target, Type::Optional(_)) {
                            return true;
                        }
                    }
                    Lit::Str(_) => {
                        if matches!(target, Type::CStr)
                            || matches!(target, Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr")
                        {
                            return true;
                        }
                    }
                    _ => {}
                    _ => {}
                },
                Expr::NullCoalesce { right, .. } => {
                    if self.can_coerce(target, src, Some(right)) {
                        return true;
                    }
                }
                Expr::QuestionFallback {
                    fallback: right,
                    is_return_fallback,
                    ..
                } => {
                    if *is_return_fallback {
                        return true;
                    }
                    if self.can_coerce(target, src, Some(right)) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        // Layer 1.2: Numeric safe widening
        if Self::is_safe_widening(target, src) {
            return true;
        }

        if let Type::Ref(id) = target {
            if matches!(
                id.sym.as_str(),
                "c_int"
                    | "c_uint"
                    | "c_short"
                    | "c_ushort"
                    | "c_long"
                    | "c_ulong"
                    | "c_char"
                    | "c_uchar"
                    | "ares_socket_t"
                    | "ares_socklen_t"
                    | "ares_ssize_t"
            ) || ((id.sym.starts_with('u') || id.sym.starts_with('i'))
                && id.sym.len() > 1
                && id.sym[1..].chars().all(|c| c.is_ascii_digit()))
            {
                if Self::is_int_type(src) {
                    return true;
                }
            }
        }
        if let Type::Ref(id) = src {
            if matches!(
                id.sym.as_str(),
                "c_int"
                    | "c_uint"
                    | "c_short"
                    | "c_ushort"
                    | "c_long"
                    | "c_ulong"
                    | "c_char"
                    | "c_uchar"
                    | "ares_socket_t"
                    | "ares_socklen_t"
                    | "ares_ssize_t"
            ) {
                if Self::is_int_type(target) {
                    return true;
                }
            }
        }

        if (matches!(target, Type::CStr)
            || matches!(target, Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr"))
            && (matches!(src, Type::String)
                || matches!(src, Type::CStr)
                || matches!(src, Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr"))
        {
            return true;
        }
        if matches!(target, Type::String)
            && (matches!(src, Type::CStr)
                || matches!(src, Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr")
                || matches!(src, Type::Array(inner) if **inner == Type::U8)
                || matches!(src, Type::FixedArray(inner, _) if **inner == Type::U8))
        {
            return true;
        }

        if (matches!(target, Type::CStr)
            || matches!(target, Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr"))
        {
            if let Type::BorrowType { inner, .. } = src {
                if matches!(**inner, Type::U8 | Type::I8) {
                    return true;
                }
            }
            if let Type::ManyRawPtr(inner) = src {
                if matches!(**inner, Type::U8 | Type::I8) {
                    return true;
                }
            }
        }

        // Layer 1.3: Structural wrapping (Optional, ErrorUnion, ResultType)
        match target {
            Type::Optional(t_inner) => {
                if src == &Type::Optional(Box::new(Type::Unknown)) {
                    return true;
                }
                if let Type::Optional(s_inner) = src {
                    return self.can_coerce(t_inner, s_inner, None);
                }
                return self.can_coerce(t_inner, src, init_expr);
            }
            Type::ErrorUnion(t_inner) => {
                if let Type::ErrorUnion(s_inner) = src {
                    return self.can_coerce(t_inner, s_inner, None);
                }
                return self.can_coerce(t_inner, src, init_expr);
            }
            Type::ResultType { ok: t_ok, .. } => {
                if let Type::ResultType { ok: s_ok, .. } = src {
                    return self.can_coerce(t_ok, s_ok, None);
                }
                return self.can_coerce(t_ok, src, init_expr);
            }
            _ => {}
            _ => {}
        }

        // Layer 1.4: Borrow tightening (*T -> &T) or same mutability
        if let (
            Type::BorrowType {
                inner: t_inner,
                is_mut: t_mut,
                ..
            },
            Type::BorrowType {
                inner: s_inner,
                is_mut: s_mut,
                ..
            },
        ) = (target, src)
        {
            if (*t_mut == *s_mut || (!*t_mut && *s_mut)) && self.can_coerce(t_inner, s_inner, None)
            {
                return true;
            }
        }

        // Layer 1.5: Subclass to base class or matching type ref
        if let (Type::Ref(base), Type::Ref(derived)) = (target, src) {
            if base.sym == derived.sym || self.is_subclass(&derived.sym, &base.sym) {
                return true;
            }
        }

        // Object literal initializing a struct/class reference
        if let Type::Ref(_) = target {
            if matches!(src, Type::Object(_)) || matches!(init_expr, Some(Expr::Object(_))) {
                return true;
            }
        }

        // Raw pointers (same depth and coerceable inner)
        if let (
            Type::RawPtr {
                inner: t_inner,
                depth: t_d,
            },
            Type::RawPtr {
                inner: s_inner,
                depth: s_d,
            },
        ) = (target, src)
        {
            if t_d == s_d && self.can_coerce(t_inner, s_inner, None) {
                return true;
            }
        }
        if let (
            Type::CPtr(t_inner) | Type::ManyRawPtr(t_inner),
            Type::CPtr(s_inner) | Type::ManyRawPtr(s_inner),
        ) = (target, src)
        {
            if self.can_coerce(t_inner, s_inner, None) {
                return true;
            }
        }
        if let Type::CPtr(t_inner) | Type::ManyRawPtr(t_inner) = target {
            if let Type::BorrowType { inner: s_inner, .. } = src {
                if self.can_coerce(t_inner, s_inner, None) {
                    return true;
                }
            }
            if let Type::RawPtr { inner: s_inner, .. } = src {
                if self.can_coerce(t_inner, s_inner, None) {
                    return true;
                }
            }
        }
        if let Type::Array(t_inner) = target {
            if let Type::BorrowType { inner: s_inner, .. } = src {
                if let Type::Tuple(s_elems) = &**s_inner {
                    if s_elems.is_empty() {
                        return true;
                    }
                }
                if self.can_coerce(&Type::Array(t_inner.clone()), s_inner, init_expr) {
                    return true;
                }
            }
        }
        if let (Type::SentinelPtr(t_inner, t_s), Type::SentinelPtr(s_inner, s_s)) = (target, src) {
            if t_s == s_s && self.can_coerce(t_inner, s_inner, None) {
                return true;
            }
        }
        if let (Type::SentinelSlice(t_inner, t_s), Type::SentinelSlice(s_inner, s_s)) =
            (target, src)
        {
            let t_clean = t_s.split(':').next().unwrap_or(t_s);
            let s_clean = s_s.split(':').next().unwrap_or(s_s);
            if t_clean == s_clean && self.can_coerce(t_inner, s_inner, None) {
                return true;
            }
        }
        if let Type::SentinelSlice(t_inner, t_s) = target {
            let t_clean = t_s.split(':').next().unwrap_or(t_s);
            if **t_inner == Type::U8 && t_clean == "0" && src == &Type::String {
                return true;
            }
        }
        if let Type::Array(t_inner) = target {
            if let Type::SentinelSlice(s_inner, _) = src {
                if self.can_coerce(t_inner, s_inner, None) {
                    return true;
                }
            }
        }
        if let Type::SentinelSlice(_, _) = target {
            if let Type::BorrowType { inner, .. } = src {
                if self.can_coerce(target, inner, init_expr) {
                    return true;
                }
            }
        }
        if let Type::BorrowType { inner, .. } = target {
            if let Type::SentinelSlice(_, _) = src {
                if self.can_coerce(inner, src, init_expr) {
                    return true;
                }
            }
        }
        if let (Type::ManyRawPtr(t_inner), Type::CPtr(s_inner))
        | (Type::CPtr(t_inner), Type::ManyRawPtr(s_inner)) = (target, src)
        {
            if self.can_coerce(t_inner, s_inner, None) {
                return true;
            }
        }

        // Layer 1.6: Struct to Trait (&dyn Trait)
        if let Type::BoxDyn(t) | Type::BareDyn(t) = target {
            if let Type::Ref(s) = src {
                if let Some(cls) = self.classes.get(&s.sym) {
                    if cls.implements.iter().any(|i| i.sym == t.sym) {
                        return true;
                    }
                }
            }
            if let Type::BorrowType { inner, .. } = src {
                if let Type::Ref(s) = &**inner {
                    if let Some(cls) = self.classes.get(&s.sym) {
                        if cls.implements.iter().any(|i| i.sym == t.sym) {
                            return true;
                        }
                    }
                }
            }
        }

        // Containers
        match (target, src) {
            (Type::Array(t_inner), Type::Array(s_inner)) => {
                if let Some(Expr::Array(elements)) = init_expr {
                    elements
                        .iter()
                        .all(|el| self.can_coerce(t_inner, s_inner, Some(el)))
                } else if let Some(Expr::ArrayRepeat(val, _)) = init_expr {
                    self.can_coerce(t_inner, s_inner, Some(val))
                } else {
                    self.can_coerce(t_inner, s_inner, None)
                }
            }
            (Type::FixedArray(t_elem, t_len), Type::FixedArray(s_elem, s_len)) => {
                if t_len != s_len {
                    return false;
                }
                if let Some(Expr::Array(elements)) = init_expr {
                    elements
                        .iter()
                        .all(|el| self.can_coerce(t_elem, s_elem, Some(el)))
                } else if let Some(Expr::ArrayRepeat(val, _)) = init_expr {
                    self.can_coerce(t_elem, s_elem, Some(val))
                } else {
                    self.can_coerce(t_elem, s_elem, None)
                }
            }
            (Type::FixedArray(elem, _), Type::Array(t2))
            | (Type::Array(t2), Type::FixedArray(elem, _)) => {
                if let Some(Expr::Array(elements)) = init_expr {
                    elements
                        .iter()
                        .all(|el| self.can_coerce(elem, t2, Some(el)))
                } else if let Some(Expr::ArrayRepeat(val, _)) = init_expr {
                    self.can_coerce(elem, t2, Some(val))
                } else {
                    self.can_coerce(elem, t2, None)
                }
            }
            (Type::FixedArray(t_elem, t_len), Type::Tuple(s_elems)) => {
                if *t_len != s_elems.len() {
                    return false;
                }
                if let Some(Expr::Array(elements)) = init_expr {
                    elements
                        .iter()
                        .zip(s_elems)
                        .all(|(el, s)| self.can_coerce(t_elem, s, Some(el)))
                } else {
                    s_elems.iter().all(|s| self.can_coerce(t_elem, s, None))
                }
            }
            (Type::Array(t_elem), Type::Tuple(s_elems)) => {
                if let Some(Expr::Array(elements)) = init_expr {
                    elements
                        .iter()
                        .zip(s_elems)
                        .all(|(el, s)| self.can_coerce(t_elem, s, Some(el)))
                } else {
                    s_elems.iter().all(|s| self.can_coerce(t_elem, s, None))
                }
            }
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return false;
                }
                t1.iter().zip(t2).all(|(a, b)| self.can_coerce(a, b, None))
            }
            _ => false,
        }
    }

    fn types_eq(&self, a: &Type, b: &Type) -> bool {
        if a == b {
            return true;
        }
        if *a == Type::Any || *b == Type::Any || *a == Type::Unknown || *b == Type::Unknown {
            return true;
        }
        let res_a = self.resolve_type(a);
        let res_b = self.resolve_type(b);
        if &res_a != a || &res_b != b {
            if self.types_eq(&res_a, &res_b) {
                return true;
            }
        }
        if Self::is_int_type(a) && Self::is_int_type(b) {
            return true;
        }
        if matches!(a, Type::F32 | Type::F64) && matches!(b, Type::F32 | Type::F64) {
            return true;
        }
        match (a, b) {
            (Type::Conditional(_, c1, a1), Type::Conditional(_, c2, a2)) => {
                self.types_eq(c1, c2) && self.types_eq(a1, a2)
            }
            (Type::Conditional(_, c1, a1), other) | (other, Type::Conditional(_, c1, a1)) => {
                self.types_eq(c1, other) || self.types_eq(a1, other)
            }
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
            (Type::ManyRawPtr(i1), Type::ManyRawPtr(i2)) => self.types_eq(i1, i2),
            (Type::CPtr(i1), Type::CPtr(i2)) => self.types_eq(i1, i2),
            (Type::CPtr(i1), Type::ManyRawPtr(i2)) | (Type::ManyRawPtr(i1), Type::CPtr(i2)) => {
                self.types_eq(i1, i2)
            }
            (Type::SentinelPtr(i1, s1), Type::SentinelPtr(i2, s2)) => {
                s1 == s2 && self.types_eq(i1, i2)
            }
            (Type::SentinelSlice(i1, s1), Type::SentinelSlice(i2, s2)) => {
                s1.split(':').next() == s2.split(':').next() && self.types_eq(i1, i2)
            }
            (Type::SentinelSlice(i1, _), Type::Array(i2))
            | (Type::Array(i2), Type::SentinelSlice(i1, _)) => self.types_eq(i1, i2),
            (Type::Array(t1), Type::Array(t2)) => self.types_eq(t1, t2),
            (Type::FixedArray(t1, s1), Type::FixedArray(t2, s2)) => {
                s1 == s2 && self.types_eq(t1, t2)
            }
            (Type::FixedArray(elem, _), Type::Array(t2))
            | (Type::Array(t2), Type::FixedArray(elem, _)) => self.types_eq(elem, t2),
            (Type::Array(elem), Type::Tuple(tys)) | (Type::Tuple(tys), Type::Array(elem)) => {
                tys.iter().all(|t| self.types_eq(elem, t))
            }
            (Type::FixedArray(elem, len), Type::Tuple(tys))
            | (Type::Tuple(tys), Type::FixedArray(elem, len)) => {
                *len == tys.len() && tys.iter().all(|t| self.types_eq(elem, t))
            }
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
            (Type::Auto, _) | (_, Type::Auto) => true,
            _ => a == b,
        }
    }

    fn analyze_pattern(&mut self, pat: &Pattern, ty: Type, kind: &VarDeclKind) -> SemaResult<()> {
        match pat {
            Pattern::Ident(ident) => {
                if self.scopes.len() > 1 {
                    if let Some(SymbolKind::Var { .. }) =
                        self.scopes.last().unwrap().get(&ident.sym)
                    {
                        return Err(SemaError::DuplicateVarDecl(ident.sym.clone(), ident.span));
                    }
                }
                self.define(
                    ident.sym.clone(),
                    SymbolKind::Var {
                        ty,
                        kind: kind.clone(),
                    },
                    ident.span,
                )
            }
            Pattern::Rest(inner) => self.analyze_pattern(inner, ty, kind),
            Pattern::Tuple(elements) => {
                let mut base_ty = ty;
                while let Type::BorrowType { inner, .. } = base_ty {
                    base_ty = *inner;
                }
                let rest_positions: Vec<usize> = elements
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| {
                        if matches!(p, Pattern::Rest(_)) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                if rest_positions.len() > 1 {
                    return Err(SemaError::MultipleRestInPattern(Span { start: 0, end: 0 }));
                }
                if let Some(&rest_idx) = rest_positions.first() {
                    let prefix_len = rest_idx;
                    let suffix_len = elements.len() - 1 - rest_idx;
                    match base_ty {
                        Type::Tuple(element_types) => {
                            if element_types.len() < prefix_len + suffix_len {
                                return Err(SemaError::TypeMismatch(
                                    Type::Tuple(vec![]),
                                    Type::Tuple(element_types),
                                    Span { start: 0, end: 0 },
                                ));
                            }
                            for i in 0..prefix_len {
                                self.analyze_pattern(&elements[i], element_types[i].clone(), kind)?;
                            }
                            if let Pattern::Rest(inner) = &elements[rest_idx] {
                                let slice_elem_ty = element_types
                                    .get(rest_idx)
                                    .cloned()
                                    .or_else(|| element_types.first().cloned())
                                    .unwrap_or(Type::Any);
                                self.analyze_pattern(
                                    inner,
                                    Type::Array(Box::new(slice_elem_ty)),
                                    kind,
                                )?;
                            }
                            for i in (rest_idx + 1)..elements.len() {
                                let offset_from_end = elements.len() - i;
                                let elem_ty =
                                    element_types[element_types.len() - offset_from_end].clone();
                                self.analyze_pattern(&elements[i], elem_ty, kind)?;
                            }
                        }
                        Type::Array(inner) => {
                            for (i, el) in elements.iter().enumerate() {
                                if i == rest_idx {
                                    self.analyze_pattern(el, Type::Array(inner.clone()), kind)?;
                                } else {
                                    self.analyze_pattern(el, (*inner).clone(), kind)?;
                                }
                            }
                        }
                        Type::Any | Type::Unknown => {
                            for (i, el) in elements.iter().enumerate() {
                                if i == rest_idx {
                                    self.analyze_pattern(
                                        el,
                                        Type::Array(Box::new(Type::Any)),
                                        kind,
                                    )?;
                                } else {
                                    self.analyze_pattern(el, Type::Any, kind)?;
                                }
                            }
                        }
                        _ => {
                            return Err(SemaError::TypeMismatch(
                                Type::Tuple(vec![]),
                                base_ty,
                                Span { start: 0, end: 0 },
                            ))
                        }
                    }
                    Ok(())
                } else {
                    let element_types = match base_ty {
                        Type::Tuple(tys) => tys,
                        Type::Array(inner) => vec![(*inner).clone(); elements.len()],
                        Type::Any | Type::Unknown => vec![Type::Any; elements.len()],
                        _ => {
                            return Err(SemaError::TypeMismatch(
                                Type::Tuple(vec![]),
                                base_ty,
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
            }
            Pattern::Object(props) => {
                let rest_count = props.iter().filter(|p| p.is_rest).count();
                if rest_count > 1 {
                    return Err(SemaError::MultipleRestInPattern(Span { start: 0, end: 0 }));
                }
                let mut base_ty = ty;
                while let Type::BorrowType { inner, .. } = base_ty {
                    base_ty = *inner;
                }
                let fields = match base_ty {
                    Type::Object(fs) => fs,
                    Type::Any | Type::Unknown => vec![],
                    _ => {
                        return Err(SemaError::TypeMismatch(
                            Type::Object(vec![]),
                            base_ty,
                            Span { start: 0, end: 0 },
                        ))
                    }
                };
                let non_rest_keys: std::collections::HashSet<String> = props
                    .iter()
                    .filter(|p| !p.is_rest)
                    .map(|p| p.key.sym.clone())
                    .collect();

                for prop in props {
                    if prop.is_rest {
                        if prop.key.sym == "*" || matches!(prop.val, Some(Pattern::Wildcard)) {
                            continue;
                        }
                        let remaining_fields: Vec<Field> = fields
                            .iter()
                            .filter(|f| !non_rest_keys.contains(&f.ident.sym))
                            .cloned()
                            .collect();
                        let rest_ty = Type::Object(remaining_fields);
                        if let Some(inner_pat) = &prop.val {
                            self.analyze_pattern(inner_pat, rest_ty, kind)?;
                        } else {
                            self.analyze_pattern(&Pattern::Ident(prop.key.clone()), rest_ty, kind)?;
                        }
                        continue;
                    }

                    if !fields.is_empty() {
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
                    } else {
                        if let Some(inner_pat) = &prop.val {
                            self.analyze_pattern(inner_pat, Type::Any, kind)?;
                        } else {
                            self.analyze_pattern(
                                &Pattern::Ident(prop.key.clone()),
                                Type::Any,
                                kind,
                            )?;
                        }
                    }
                }
                Ok(())
            }
            Pattern::Binding { ident, subpat } => {
                if let Some(SymbolKind::Var { .. }) = self.scopes.last().unwrap().get(&ident.sym) {
                    return Err(SemaError::DuplicateVarDecl(ident.sym.clone(), ident.span));
                }
                self.define(
                    ident.sym.clone(),
                    SymbolKind::Var {
                        ty: ty.clone(),
                        kind: kind.clone(),
                    },
                    ident.span,
                )?;
                self.analyze_pattern(subpat, ty, kind)?;
                Ok(())
            }
            Pattern::TypeMatch { ident, ty: pty } => {
                if let Some(SymbolKind::Var { .. }) = self.scopes.last().unwrap().get(&ident.sym) {
                    return Err(SemaError::DuplicateVarDecl(ident.sym.clone(), ident.span));
                }
                self.define(
                    ident.sym.clone(),
                    SymbolKind::Var {
                        ty: pty.clone(),
                        kind: kind.clone(),
                    },
                    ident.span,
                )?;
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

    fn infer_type(&mut self, expr: &Expr) -> SemaResult<Type> {
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
                    || ident.sym == "super"
                    || ident.sym == "Box"
                    || ident.sym == "Err"
                    || ident.sym == "err"
                    || ident.sym == "Error"
                    || ident.sym == "reflect"
                    || ident.sym == "unreachable"
                    || ident.sym.starts_with("c_")
                    || ((ident.sym.starts_with('u') || ident.sym.starts_with('i'))
                        && ident.sym.len() > 1
                        && ident.sym[1..].chars().all(|c| c.is_ascii_digit()))
                    || matches!(
                        ident.sym.as_str(),
                        "u8" | "i8"
                            | "u16"
                            | "i16"
                            | "u32"
                            | "i32"
                            | "u64"
                            | "i64"
                            | "u128"
                            | "i128"
                            | "usize"
                            | "isize"
                            | "f32"
                            | "f64"
                            | "bool"
                            | "void"
                            | "anytype"
                            | "anyopaque"
                    )
                {
                    return Ok(Type::Any);
                }
                match self.resolve(&ident.sym) {
                    Some(SymbolKind::Var { ty, .. }) => Ok(ty.clone()),
                    Some(SymbolKind::Func { params, ret, .. }) => Ok(Type::Fn(
                        params.clone(),
                        Box::new(ret.clone().unwrap_or(Type::I32)),
                    )),
                    Some(SymbolKind::Class(c)) => Ok(Type::Ref(c.ident.clone())),
                    Some(SymbolKind::Trait(t)) => Ok(Type::Ref(t.ident.clone())),
                    Some(SymbolKind::Enum(e)) => Ok(Type::Ref(e.ident.clone())),
                    _ => {
                        if ident.sym == "this" || ident.sym == "_" {
                            Ok(Type::Any)
                        } else {
                            Err(SemaError::UndefinedSymbol(ident.sym.clone(), ident.span))
                        }
                    }
                }
            }
            Expr::Bin(bin) => {
                if bin.op == BinaryOp::Pipe {
                    let transformed = transform_pipeline_rhs(&bin.right, "__pipe_mock");
                    return self.infer_type(&transformed);
                }
                let left = self.infer_type(&bin.left)?;
                let right = if bin.op == BinaryOp::LogicalAnd {
                    let not_null_vars = Self::extract_not_null_vars(&bin.left);
                    if !not_null_vars.is_empty() {
                        self.enter_scope();
                        for v in &not_null_vars {
                            if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                                let unwrapped = Self::unwrap_optional_type(&ty);
                                let _ = self.define(
                                    v.clone(),
                                    SymbolKind::Var {
                                        ty: unwrapped,
                                        kind,
                                    },
                                    bin.span,
                                );
                            }
                        }
                        let r = self.infer_type(&bin.right)?;
                        self.exit_scope();
                        r
                    } else {
                        self.infer_type(&bin.right)?
                    }
                } else {
                    self.infer_type(&bin.right)?
                };
                if bin.op == BinaryOp::Assign {
                    if let Expr::Ident(ident) = &*bin.left {
                        if ident.sym == "_" {
                            return Ok(right);
                        }
                        if let Some(SymbolKind::Var {
                            kind: VarDeclKind::Const,
                            ..
                        }) = self.resolve(&ident.sym)
                        {
                            return Err(SemaError::ConstAssignment(ident.sym.clone(), bin.span));
                        }
                    } else if let Expr::Index { obj, .. } = &*bin.left {
                        if let Expr::Ident(ident) = &**obj {
                            if let Some(SymbolKind::Var {
                                kind: VarDeclKind::Const,
                                ..
                            }) = self.resolve(&ident.sym)
                            {
                                return Err(SemaError::ConstAssignment(
                                    ident.sym.clone(),
                                    bin.span,
                                ));
                            }
                        }
                    }
                    if !self.can_coerce(&left, &right, Some(&*bin.right)) {
                        let mut can_assign_to_outer = false;
                        if let Expr::Ident(ident) = &*bin.left {
                            for scope in self.scopes.iter().rev().skip(1) {
                                if let Some(SymbolKind::Var { ty: outer_ty, kind }) =
                                    scope.get(&ident.sym)
                                {
                                    if *kind != VarDeclKind::Const
                                        && self.can_coerce(outer_ty, &right, Some(&*bin.right))
                                    {
                                        can_assign_to_outer = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !can_assign_to_outer {
                            return Err(SemaError::TypeMismatch(left, right, bin.span));
                        }
                    }
                    if let Expr::Ident(ident) = &*bin.left {
                        if let Some(scope) = self.scopes.last_mut() {
                            if let Some(SymbolKind::Var { ty, .. }) = scope.get_mut(&ident.sym) {
                                *ty = right.clone();
                            }
                        }
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
                } else if bin.op == BinaryOp::LogicalOr || bin.op == BinaryOp::LogicalAnd {
                    Ok(Type::Bool)
                } else if bin.op == BinaryOp::Add && (left == Type::String || right == Type::String)
                {
                    Ok(Type::String)
                } else {
                    if bin.op == BinaryOp::Add || bin.op == BinaryOp::Sub {
                        let is_left_single_ptr = matches!(left, Type::RawPtr { .. });
                        let is_right_single_ptr = matches!(right, Type::RawPtr { .. });
                        if is_left_single_ptr || is_right_single_ptr {
                            return Err(SemaError::InvalidPointerArithmetic(
                                "Cannot perform pointer arithmetic on single-item pointer '*T!'; use '[*]T!' instead".to_string(),
                                bin.span,
                            ));
                        }
                    }
                    if !self.types_eq(&left, &right)
                        && !self.can_coerce(&left, &right, Some(&*bin.right))
                        && !self.can_coerce(&right, &left, Some(&*bin.left))
                    {
                        return Err(SemaError::TypeMismatch(left, right, bin.span));
                    }
                    Ok(left)
                }
            }
            Expr::Call(call) => {
                if let Expr::Ident(fid) = &*call.callee {
                    if fid.sym == "num" || fid.sym == "Number" {
                        return Err(SemaError::InvalidCast(
                            "CP does not support 'num()' or 'Number()'. CP is strongly typed: use specific numeric types like i32(x), u32(x), usize(x), or f64(x).".to_string(),
                            call.span,
                        ));
                    }
                    if fid.sym == "join" || fid.sym == "@task_join" {
                        let elem_tys = vec![Type::Any; call.args.len()];
                        return Ok(Type::Tuple(elem_tys));
                    }
                    if fid.sym == "spawn"
                        || fid.sym == "@task_spawn"
                        || fid.sym == "@task_spawn_cpu"
                    {
                        return Ok(Type::Ref(Ident {
                            span: fid.span,
                            sym: "Task".to_string(),
                        }));
                    }
                    if fid.sym == "@alloc" {
                        return Ok(Type::ManyRawPtr(Box::new(Type::U8)));
                    }
                    if let Some(SymbolKind::Func { ret: Some(ret), .. }) = self.resolve(&fid.sym) {
                        return Ok(ret.clone());
                    }
                    if let Some(SymbolKind::Enum(_)) = self.resolve(&fid.sym) {
                        return Ok(Type::Ref(fid.clone()));
                    }
                    match fid.sym.as_str() {
                        "i8" => return Ok(Type::I8),
                        "u8" => return Ok(Type::U8),
                        "i16" => return Ok(Type::I16),
                        "u16" => return Ok(Type::U16),
                        "i32" => return Ok(Type::I32),
                        "u32" => return Ok(Type::U32),
                        "i64" => return Ok(Type::I64),
                        "u64" => return Ok(Type::U64),
                        "i128" => return Ok(Type::I128),
                        "u128" => return Ok(Type::U128),
                        "isize" => return Ok(Type::ISize),
                        "usize" => return Ok(Type::USize),
                        "f32" => return Ok(Type::F32),
                        "f64" => return Ok(Type::F64),
                        "bool" => return Ok(Type::Bool),
                        "string" | "String" => return Ok(Type::String),
                        _ => {}
                    }
                }
                if let Expr::Member(mem) = &*call.callee {
                    if mem.prop.sym == "toString" && call.args.is_empty() {
                        return Ok(Type::String);
                    }
                    if let Expr::Ident(oid) = &*mem.obj {
                        if (oid.sym == "string" || oid.sym == "String") && mem.prop.sym == "from" {
                            return Ok(Type::String);
                        }
                        if oid.sym == "reflect" {
                            if mem.prop.sym == "hasMethod"
                                || mem.prop.sym == "hasField"
                                || mem.prop.sym == "has"
                            {
                                return Ok(Type::Bool);
                            }
                        }
                        if let Some(SymbolKind::Enum(e)) = self.resolve(&oid.sym) {
                            if let Some(m) = e.methods.iter().find(|m| m.ident.sym == mem.prop.sym)
                            {
                                return Ok(m.return_type.clone().unwrap_or(Type::Void));
                            }
                            if mem.prop.sym == "fromName"
                                || mem.prop.sym == "fromString"
                                || mem.prop.sym == "get"
                            {
                                return Ok(Type::Optional(Box::new(Type::Ref(oid.clone()))));
                            }
                            if mem.prop.sym == "has" {
                                return Ok(Type::Bool);
                            }
                        }
                        if oid.sym == "spawn" && mem.prop.sym == "cpu" {
                            return Ok(Type::Ref(Ident {
                                span: mem.prop.span,
                                sym: "Task".to_string(),
                            }));
                        }
                    }
                }
                let mut callee_ty = self.infer_type(&call.callee)?;
                if let Type::Ref(id) = &callee_ty {
                    if let Some(target) = self.type_aliases.get(&id.sym).cloned() {
                        callee_ty = target;
                    }
                }
                while let Type::RawPtr { inner, .. } | Type::BorrowType { inner, .. } = &callee_ty {
                    callee_ty = *inner.clone();
                }
                match callee_ty {
                    Type::Fn(params, ret) => {
                        if let Type::Ref(rid) = &*ret {
                            for (idx, p) in params.iter().enumerate() {
                                let mut p_ty = p;
                                while let Type::BorrowType { inner, .. } = p_ty {
                                    p_ty = inner;
                                }
                                if let Type::Ref(pid) = p_ty {
                                    if pid.sym == rid.sym && idx < call.args.len() {
                                        if let Ok(arg_ty) = self.infer_type(&call.args[idx]) {
                                            return Ok(arg_ty);
                                        }
                                    }
                                }
                            }
                        }
                        Ok(*ret)
                    }
                    Type::CFn(_, ret) => Ok(*ret),
                    Type::Any | Type::Auto => Ok(Type::Any),
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
            Expr::EnumInit(init) => {
                if !init.enum_name.sym.is_empty() {
                    Ok(Type::Ref(init.enum_name.clone()))
                } else {
                    Ok(Type::Any)
                }
            }
            Expr::Spawn(_) => Ok(Type::PID),
            Expr::Receive(ty) => Ok(ty.as_ref().map(|t| (**t).clone()).unwrap_or(Type::I32)),
            Expr::Slice(slice) => {
                let mut o_ty = self.infer_type(&slice.obj)?;
                while let Type::BorrowType { inner, .. } | Type::RawPtr { inner, .. } = o_ty {
                    o_ty = *inner;
                }
                let elem = match o_ty {
                    Type::ManyRawPtr(elem)
                    | Type::CPtr(elem)
                    | Type::SentinelPtr(elem, _)
                    | Type::SentinelSlice(elem, _)
                    | Type::Array(elem)
                    | Type::FixedArray(elem, _) => elem,
                    Type::CStr => Box::new(Type::U8),
                    Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr" => Box::new(Type::U8),
                    Type::String => Box::new(Type::U8),
                    Type::Ref(id) if id.sym.ends_with("WPathBuffer") || id.sym.contains("u16") => {
                        Box::new(Type::U16)
                    }
                    _ => Box::new(Type::Auto),
                };
                if let Some(s_expr) = &slice.sentinel {
                    let sentinel_str = match &**s_expr {
                        Expr::Lit(Lit::Int(n)) => n.to_string(),
                        _ => "0".to_string(),
                    };
                    Ok(Type::SentinelSlice(elem, sentinel_str))
                } else {
                    Ok(Type::Array(elem))
                }
            }
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
                Ok(Type::Object(fs))
            }
            Expr::Dict(_) => Ok(Type::Dict(Box::new(Type::String), Box::new(Type::Any))),
            Expr::Index { obj, .. } => {
                let mut o_ty = self.infer_type(obj)?;
                while let Type::BorrowType { inner, .. } = o_ty {
                    o_ty = *inner;
                }
                match o_ty {
                    Type::Array(elem) => Ok(*elem),
                    Type::FixedArray(elem, _) => Ok(*elem),
                    Type::ManyRawPtr(elem)
                    | Type::CPtr(elem)
                    | Type::SentinelPtr(elem, _)
                    | Type::SentinelSlice(elem, _) => Ok(*elem),
                    Type::CStr => Ok(Type::U8),
                    Type::String => Ok(Type::U8),
                    Type::Ref(id) if id.sym == "cstring" || id.sym == "cstr" => Ok(Type::U8),
                    Type::Tuple(tys) => Ok(tys.into_iter().next().unwrap_or(Type::Any)),
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
            Expr::ArrayRepeat(val, count) => {
                let val_ty = self.infer_type(val)?;
                if let Expr::Lit(Lit::Int(n)) = &**count {
                    Ok(Type::FixedArray(Box::new(val_ty), *n as usize))
                } else {
                    Ok(Type::Array(Box::new(val_ty)))
                }
            }
            Expr::Template(t) => {
                for e in &t.expressions {
                    let _ = self.infer_type(e)?;
                }
                Ok(Type::String)
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
                "@get_arena" | "@arena" => Ok(Type::Ref(Ident {
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
            Expr::Range { .. } => Ok(Type::I32),
            Expr::Cast {
                expr, target_ty, ..
            } => {
                if let Type::RawPtr { inner, depth } = target_ty {
                    if **inner == Type::Any {
                        if let Expr::Borrow(b, _) = &**expr {
                            let mut base_ty = self.infer_type(b)?;
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
                            return Ok(Type::RawPtr {
                                inner: Box::new(base_ty),
                                depth: *depth,
                            });
                        }
                    }
                }
                Ok(target_ty.clone())
            }
            Expr::Deref(inner) => {
                let inner_ty = self.infer_type(inner)?;
                match inner_ty {
                    Type::RawPtr { inner, depth } => {
                        if depth > 1 {
                            Ok(Type::RawPtr {
                                inner,
                                depth: depth - 1,
                            })
                        } else {
                            Ok(*inner)
                        }
                    }
                    Type::BorrowType { inner, .. } => Ok(*inner),
                    other => Ok(other),
                }
            }
            Expr::Unsafe(b) | Expr::Comptime(b) | Expr::Arena(b) => {
                if let Some(Stmt::Expr(e)) = b.body.last() {
                    return Ok(self.infer_type(e).unwrap_or(Type::Any));
                } else if let Some(Stmt::Return(r)) = b.body.last() {
                    if let Some(arg) = &r.arg {
                        return Ok(self.infer_type(arg).unwrap_or(Type::Any));
                    }
                }
                Ok(Type::Void)
            }
            Expr::Match(m) => {
                for arm in &m.arms {
                    let arm_ty = match &*arm.body {
                        Stmt::Expr(Expr::Return(..))
                        | Stmt::Expr(Expr::Break(..))
                        | Stmt::Expr(Expr::LabeledBreak(..))
                        | Stmt::Expr(Expr::Continue(..)) => Type::Void,
                        Stmt::Expr(e) => self.infer_type(e).unwrap_or(Type::Any),
                        Stmt::Block(b) => {
                            if let Some(last) = b.body.last() {
                                match last {
                                    Stmt::Expr(Expr::Return(..))
                                    | Stmt::Expr(Expr::Break(..))
                                    | Stmt::Expr(Expr::LabeledBreak(..))
                                    | Stmt::Expr(Expr::Continue(..))
                                    | Stmt::Return(..)
                                    | Stmt::Break
                                    | Stmt::Continue => Type::Void,
                                    Stmt::Expr(e) => self.infer_type(e).unwrap_or(Type::Any),
                                    _ => Type::Void,
                                }
                            } else {
                                Type::Void
                            }
                        }
                        Stmt::Return(_) | Stmt::Break | Stmt::Continue => Type::Void,
                        _ => Type::Void,
                    };
                    if arm_ty != Type::Any && arm_ty != Type::Unknown && arm_ty != Type::Void {
                        return Ok(arm_ty);
                    }
                }
                Ok(Type::Any)
            }
            Expr::Member(mem) => {
                if mem.prop.sym == "toString" {
                    return Ok(Type::Fn(vec![], Box::new(Type::String)));
                }
                if let Expr::Ident(id) = &*mem.obj {
                    if let Some(SymbolKind::Enum(e)) = self.resolve(&id.sym) {
                        if let Some(m) = e.methods.iter().find(|m| m.ident.sym == mem.prop.sym) {
                            return Ok(Type::Fn(
                                m.params.iter().map(|p| p.ty.clone()).collect(),
                                Box::new(m.return_type.clone().unwrap_or(Type::Void)),
                            ));
                        }
                        return Ok(Type::Ref(id.clone()));
                    }
                }
                if let Ok(obj_ty) = self.infer_type(&mem.obj) {
                    let base_ty = match &obj_ty {
                        Type::BorrowType { inner, .. } | Type::RawPtr { inner, .. } => {
                            inner.as_ref()
                        }
                        other => other,
                    };
                    if let Type::Ref(id) = base_ty {
                        if let Some(SymbolKind::Enum(_)) = self.resolve(&id.sym) {
                            if mem.prop.sym == "name" {
                                return Ok(Type::String);
                            } else if mem.prop.sym == "value" {
                                return Ok(Type::I32);
                            }
                        }
                        let mut cur_cls_name = Some(id.sym.clone());
                        while let Some(cls_name) = cur_cls_name {
                            if let Some(cls) = self.classes.get(&cls_name) {
                                if let Some(field) =
                                    cls.fields.iter().find(|f| f.ident.sym == mem.prop.sym)
                                {
                                    return Ok(field.ty.clone());
                                }
                                cur_cls_name = cls.extends.as_ref().map(|e| e.sym.clone());
                            } else {
                                break;
                            }
                        }
                    } else if (*base_ty == Type::String
                        || *base_ty == Type::CStr
                        || matches!(
                            base_ty,
                            Type::Array(_)
                                | Type::FixedArray(_, _)
                                | Type::SentinelSlice(_, _)
                                | Type::SentinelPtr(_, _)
                        ))
                        && (mem.prop.sym == "len" || mem.prop.sym == "length")
                    {
                        return Ok(Type::USize);
                    }
                }
                Ok(Type::Any)
            }
            Expr::If {
                test, cons, alt, ..
            } => {
                let not_null_vars = Self::extract_not_null_vars(test);
                let is_null_vars = Self::extract_is_null_vars(test);
                let cons_ty = if !not_null_vars.is_empty() {
                    self.enter_scope();
                    for v in &not_null_vars {
                        if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                            let unwrapped = Self::unwrap_optional_type(&ty);
                            let _ = self.define(
                                v.clone(),
                                SymbolKind::Var {
                                    ty: unwrapped,
                                    kind,
                                },
                                Span { start: 0, end: 0 },
                            );
                        }
                    }
                    let res = self.infer_type(cons)?;
                    self.exit_scope();
                    res
                } else {
                    self.infer_type(cons)?
                };
                if let Some(a) = alt {
                    let alt_ty = if !is_null_vars.is_empty() {
                        self.enter_scope();
                        for v in &is_null_vars {
                            if let Some(SymbolKind::Var { ty, kind }) = self.resolve(v).cloned() {
                                let unwrapped = Self::unwrap_optional_type(&ty);
                                let _ = self.define(
                                    v.clone(),
                                    SymbolKind::Var {
                                        ty: unwrapped,
                                        kind,
                                    },
                                    Span { start: 0, end: 0 },
                                );
                            }
                        }
                        let res = self.infer_type(a)?;
                        self.exit_scope();
                        res
                    } else {
                        self.infer_type(a)?
                    };
                    if cons_ty == alt_ty {
                        Ok(cons_ty)
                    } else {
                        Ok(Type::Any)
                    }
                } else {
                    Ok(cons_ty)
                }
            }
            Expr::LabeledBlock { block, .. } => {
                for s in block.body.iter().rev() {
                    match s {
                        Stmt::Expr(Expr::LabeledBreak(_, _, Some(val)))
                        | Stmt::Expr(Expr::Break(_, Some(val))) => {
                            return Ok(self.infer_type(val).unwrap_or(Type::Any));
                        }
                        Stmt::Expr(e) => {
                            return Ok(self.infer_type(e).unwrap_or(Type::Any));
                        }
                        _ => {}
                        _ => {}
                    }
                }
                Ok(Type::Void)
            }
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
        Expr::PipePlaceholder(span) => *span,
        Expr::Range { span, .. } => *span,
        Expr::Template(t) => t.span,
        _ => Span { start: 0, end: 0 },
    }
}
