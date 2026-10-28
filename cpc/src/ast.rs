use crate::Span;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub span: Span,
    pub body: Vec<ModuleItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleItem {
    Decl(Decl),
    Stmt(Stmt),
    Import(ImportStmt),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportStmt {
    pub span: Span,
    pub specifiers: Vec<Ident>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Decl {
    Func(Function),
    Class(Class),
    Trait(TraitDecl),
    Impl(ImplDecl),
    Enum(EnumDecl),
    Interface(Interface),
    ErrorSet(ErrorSetDecl),
    Meta(MetaDef),
    Dec(DecDef),
    Macro(MacroDef),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroDef {
    pub span: Span,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: BlockStmt,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecDef {
    pub span: Span,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: BlockStmt,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaDef {
    pub span: Span,
    pub ident: Ident,
    pub params: Vec<MetaParam>,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaParam {
    pub span: Span,
    pub ident: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaItem {
    pub span: Span,
    pub name: Ident,
    pub args: Vec<MetaArg>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaArg {
    Positional(Expr),
    Named(Ident, Expr),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDecl {
    pub span: Span,
    pub ident: Ident,
    pub methods: Vec<MethodSig>,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplDecl {
    pub span: Span,
    pub trait_name: Ident,
    pub target_name: Ident,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDecl {
    pub span: Span,
    pub ident: Ident,
    pub variants: Vec<EnumVariant>,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub span: Span,
    pub ident: Ident,
    pub fields: Option<Vec<Field>>,
    pub value: Option<Expr>,
    #[serde(default)]
    pub tuple_type: Option<Type>,
    #[serde(default)]
    pub from_err: Option<Ident>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorSetDecl {
    pub span: Span,
    pub ident: Ident,
    pub variants: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interface {
    pub span: Span,
    pub ident: Ident,
    pub methods: Vec<MethodSig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub span: Span,
    pub ident: Ident,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub implements: Vec<Ident>,
    #[serde(default)]
    pub extends: Option<Ident>,
    #[serde(default)]
    pub is_ref: bool,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub span: Span,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: BlockStmt,
    pub is_async: bool,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_unsafe: bool,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    Expr(Expr),
    Var(VarDecl),
    Return(ReturnStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Match(MatchExpr),
    Spawn(SpawnExpr),
    Defer(Box<Stmt>),
    ErrDefer(Box<Stmt>),
    ComptimeBlock(BlockStmt),
    Block(BlockStmt),
    UnsafeBlock(BlockStmt),
    TryCatch {
        span: Span,
        try_block: BlockStmt,
        err_ident: Option<Ident>,
        is_err_mut: bool,
        catch_body: Option<Box<Stmt>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockStmt {
    pub span: Span,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarDecl {
    pub span: Span,
    pub kind: VarDeclKind,
    pub pat: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VarDeclKind {
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Ident(Ident),
    Array(Vec<Pattern>),
    Tuple(Vec<Pattern>),
    Object(Vec<ObjectPatProp>),
    Enum(EnumPattern),
    Lit(Lit),
    Rest(Box<Pattern>),
    Wildcard,
    Null,
    Err(Box<Pattern>),
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    Or(Vec<Pattern>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumPattern {
    pub span: Span,
    pub enum_name: Ident,
    pub variant_name: Ident,
    pub fields: Option<Vec<ObjectPatProp>>,
    #[serde(default)]
    pub tuple_pats: Option<Vec<Pattern>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectPatProp {
    pub key: Ident,
    pub val: Option<Pattern>,
    #[serde(default)]
    pub is_rest: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStmt {
    pub span: Span,
    pub arg: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStmt {
    pub span: Span,
    pub test: Expr,
    pub cons: Box<Stmt>,
    pub alt: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStmt {
    pub span: Span,
    pub test: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IterMode {
    Borrow,    // for x of list
    MutBorrow, // for &x of list
    Move,      // for move x of list
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStmt {
    pub span: Span,
    pub mode: IterMode,
    pub pat: Pattern,
    pub expr: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Ident(Ident),
    Lit(Lit),
    Bin(BinExpr),
    Call(CallExpr),
    New(CallExpr),
    Member(MemberExpr),
    OptMember(MemberExpr),
    OptCall(CallExpr),
    Slice(SliceExpr),
    Match(Box<MatchExpr>),
    Spawn(Box<SpawnExpr>),
    Receive(Option<Box<Type>>),
    Zig(ZigEscapeExpr),
    Question(Box<Expr>),
    QuestionFallback {
        expr: Box<Expr>,
        fallback: Box<Expr>,
        is_return_fallback: bool,
    },
    NullCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Catch {
        expr: Box<Expr>,
        err_ident: Option<Ident>,
        is_err_mut: bool,
        body: Box<Stmt>,
    },
    ErrWrap(Box<Expr>),
    BoxNew(Box<Expr>),
    Downcast {
        obj: Box<Expr>,
        target_ty: Ident,
        is_mut: bool,
    },
    Array(Vec<Expr>),
    Object(Vec<ObjectField>),
    Dict(Vec<ObjectField>),
    Index {
        obj: Box<Expr>,
        index: Box<Expr>,
    },
    Arrow(Box<ArrowExpr>),
    EnumInit(EnumInitExpr),
    Template(TemplateLit),
    Await(Box<Expr>),
    Move(Box<Expr>),
    Borrow(Box<Expr>, bool),
    Builtin(BuiltinCall),
    ReflectType {
        span: Span,
        target_ty: Type,
    },
    ReflectOf {
        span: Span,
        expr: Box<Expr>,
    },
    PipePlaceholder(Span),
    Range {
        span: Span,
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    Cast {
        span: Span,
        expr: Box<Expr>,
        target_ty: Type,
    },
    Deref(Box<Expr>),
    Unsafe(BlockStmt),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinCall {
    pub span: Span,
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectField {
    pub key: Option<Ident>,
    pub val: Expr,
    pub is_spread: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumInitExpr {
    pub span: Span,
    pub enum_name: Ident,
    pub variant_name: Ident,
    pub fields: Option<Vec<EnumInitField>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumInitField {
    pub key: Ident,
    pub val: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrowExpr {
    pub span: Span,
    pub is_async: bool,
    pub params: Vec<Pattern>,
    pub body: Box<ArrowBody>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArrowBody {
    Expr(Expr),
    Block(BlockStmt),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ident {
    pub span: Span,
    pub sym: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    Undefined,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateLit {
    pub span: Span,
    pub quasis: Vec<String>,
    pub expressions: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinExpr {
    pub span: Span,
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BinaryOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Pipe,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpr {
    pub span: Span,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub is_comptime: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberExpr {
    pub span: Span,
    pub obj: Box<Expr>,
    pub prop: Ident,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliceExpr {
    pub span: Span,
    pub obj: Box<Expr>,
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpr {
    pub span: Span,
    pub disc: Box<Expr>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub span: Span,
    pub pat: Pattern,
    #[serde(default)]
    pub guard: Option<Expr>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnExpr {
    pub span: Span,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZigEscapeExpr {
    pub span: Span,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
    Named(Ident),
    Union(Vec<Ident>),
    AnonSet(Vec<Ident>),
    BoxDyn(Ident),
    DefaultError,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    I32,
    U32,
    U64,
    F32,
    F64,
    USize,
    Bool,
    String,
    Void,
    Any,
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Tuple(Vec<Type>),   // [T1, T2]
    Object(Vec<Field>), // {k: T}
    Ref(Ident),
    Fn(Vec<Type>, Box<Type>),
    PID,
    Unknown,
    ErrorUnion(Box<Type>),
    ResultType {
        ok: Box<Type>,
        err: ErrorType,
    },
    Optional(Box<Type>),
    BorrowType {
        inner: Box<Type>,
        is_mut: bool,
        lifetimes: Vec<String>,
    },
    Shared(Box<Type>),
    BoxDyn(Ident),
    BareDyn(Ident),
    RawPtr {
        inner: Box<Type>,
        depth: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub span: Span,
    pub ident: Ident,
    pub ty: Type,
    pub is_comptime: bool,
    pub is_mut: bool,
    pub is_move: bool,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
    #[serde(default)]
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub span: Span,
    pub ident: Ident,
    pub ty: Type,
    #[serde(default)]
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodSig {
    pub span: Span,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    #[serde(default)]
    pub default_body: Option<BlockStmt>,
    #[serde(default)]
    pub is_unsafe: bool,
}

pub fn has_pipe_placeholder(expr: &Expr) -> bool {
    match expr {
        Expr::PipePlaceholder(_) => true,
        Expr::Bin(b) => has_pipe_placeholder(&b.left) || has_pipe_placeholder(&b.right),
        Expr::Call(c) | Expr::OptCall(c) | Expr::New(c) => {
            has_pipe_placeholder(&c.callee) || c.args.iter().any(has_pipe_placeholder)
        }
        Expr::Member(m) | Expr::OptMember(m) => has_pipe_placeholder(&m.obj),
        Expr::Slice(s) => {
            has_pipe_placeholder(&s.obj)
                || s.start.as_ref().map_or(false, |e| has_pipe_placeholder(e))
                || s.end.as_ref().map_or(false, |e| has_pipe_placeholder(e))
        }
        Expr::Match(m) => has_pipe_placeholder(&m.disc),
        Expr::Question(q)
        | Expr::ErrWrap(q)
        | Expr::BoxNew(q)
        | Expr::Await(q)
        | Expr::Move(q)
        | Expr::Borrow(q, _) => has_pipe_placeholder(q),
        Expr::QuestionFallback { expr, fallback, .. } => {
            has_pipe_placeholder(expr) || has_pipe_placeholder(fallback)
        }
        Expr::NullCoalesce { left, right } => {
            has_pipe_placeholder(left) || has_pipe_placeholder(right)
        }
        Expr::Catch { expr, .. } => has_pipe_placeholder(expr),
        Expr::Array(elements) => elements.iter().any(has_pipe_placeholder),
        Expr::Object(fields) | Expr::Dict(fields) => {
            fields.iter().any(|f| has_pipe_placeholder(&f.val))
        }
        Expr::Index { obj, index } => has_pipe_placeholder(obj) || has_pipe_placeholder(index),
        Expr::Deref(d) => has_pipe_placeholder(d),
        Expr::Cast { expr, .. } => has_pipe_placeholder(expr),
        Expr::Range { start, end, .. } => has_pipe_placeholder(start) || has_pipe_placeholder(end),
        Expr::Builtin(b) => b.args.iter().any(has_pipe_placeholder),
        _ => false,
    }
}

pub fn substitute_pipe_placeholder(expr: &Expr, pipe_var: &str) -> Expr {
    match expr {
        Expr::PipePlaceholder(span) => Expr::Ident(Ident {
            span: *span,
            sym: pipe_var.to_string(),
        }),
        Expr::Bin(b) => Expr::Bin(BinExpr {
            span: b.span,
            op: b.op.clone(),
            left: Box::new(substitute_pipe_placeholder(&b.left, pipe_var)),
            right: Box::new(substitute_pipe_placeholder(&b.right, pipe_var)),
        }),
        Expr::Call(c) => Expr::Call(CallExpr {
            span: c.span,
            callee: Box::new(substitute_pipe_placeholder(&c.callee, pipe_var)),
            args: c
                .args
                .iter()
                .map(|a| substitute_pipe_placeholder(a, pipe_var))
                .collect(),
            is_comptime: c.is_comptime,
        }),
        Expr::OptCall(c) => Expr::OptCall(CallExpr {
            span: c.span,
            callee: Box::new(substitute_pipe_placeholder(&c.callee, pipe_var)),
            args: c
                .args
                .iter()
                .map(|a| substitute_pipe_placeholder(a, pipe_var))
                .collect(),
            is_comptime: c.is_comptime,
        }),
        Expr::New(c) => Expr::New(CallExpr {
            span: c.span,
            callee: Box::new(substitute_pipe_placeholder(&c.callee, pipe_var)),
            args: c
                .args
                .iter()
                .map(|a| substitute_pipe_placeholder(a, pipe_var))
                .collect(),
            is_comptime: c.is_comptime,
        }),
        Expr::Member(m) => Expr::Member(MemberExpr {
            span: m.span,
            obj: Box::new(substitute_pipe_placeholder(&m.obj, pipe_var)),
            prop: m.prop.clone(),
        }),
        Expr::OptMember(m) => Expr::OptMember(MemberExpr {
            span: m.span,
            obj: Box::new(substitute_pipe_placeholder(&m.obj, pipe_var)),
            prop: m.prop.clone(),
        }),
        Expr::Question(q) => Expr::Question(Box::new(substitute_pipe_placeholder(q, pipe_var))),
        Expr::Await(a) => Expr::Await(Box::new(substitute_pipe_placeholder(a, pipe_var))),
        Expr::Move(m) => Expr::Move(Box::new(substitute_pipe_placeholder(m, pipe_var))),
        Expr::Borrow(b, is_mut) => {
            Expr::Borrow(Box::new(substitute_pipe_placeholder(b, pipe_var)), *is_mut)
        }
        Expr::Index { obj, index } => Expr::Index {
            obj: Box::new(substitute_pipe_placeholder(obj, pipe_var)),
            index: Box::new(substitute_pipe_placeholder(index, pipe_var)),
        },
        Expr::Array(elements) => Expr::Array(
            elements
                .iter()
                .map(|e| substitute_pipe_placeholder(e, pipe_var))
                .collect(),
        ),
        Expr::Builtin(b) => Expr::Builtin(BuiltinCall {
            span: b.span,
            name: b.name.clone(),
            args: b
                .args
                .iter()
                .map(|a| substitute_pipe_placeholder(a, pipe_var))
                .collect(),
        }),
        Expr::Deref(d) => Expr::Deref(Box::new(substitute_pipe_placeholder(d, pipe_var))),
        Expr::Cast {
            span,
            expr,
            target_ty,
        } => Expr::Cast {
            span: *span,
            expr: Box::new(substitute_pipe_placeholder(expr, pipe_var)),
            target_ty: target_ty.clone(),
        },
        Expr::Range {
            span,
            start,
            end,
            inclusive,
        } => Expr::Range {
            span: *span,
            start: Box::new(substitute_pipe_placeholder(start, pipe_var)),
            end: Box::new(substitute_pipe_placeholder(end, pipe_var)),
            inclusive: *inclusive,
        },
        other => other.clone(),
    }
}

pub fn transform_pipeline_rhs(right: &Expr, pipe_var: &str) -> Expr {
    let mock_span = Span { start: 0, end: 0 };
    let pipe_ident = Expr::Ident(Ident {
        span: mock_span,
        sym: pipe_var.to_string(),
    });

    if has_pipe_placeholder(right) {
        substitute_pipe_placeholder(right, pipe_var)
    } else {
        match right {
            Expr::Ident(id) => Expr::Call(CallExpr {
                span: id.span,
                callee: Box::new(Expr::Ident(id.clone())),
                args: vec![pipe_ident],
                is_comptime: false,
            }),
            Expr::Call(call) => {
                let mut new_call = call.clone();
                new_call.args.insert(0, pipe_ident);
                Expr::Call(new_call)
            }
            Expr::OptCall(call) => {
                let mut new_call = call.clone();
                new_call.args.insert(0, pipe_ident);
                Expr::OptCall(new_call)
            }
            Expr::Member(mem) => Expr::Call(CallExpr {
                span: mem.span,
                callee: Box::new(Expr::Member(mem.clone())),
                args: vec![pipe_ident],
                is_comptime: false,
            }),
            Expr::Await(inner) => {
                let transformed_inner = transform_pipeline_rhs(inner, pipe_var);
                Expr::Await(Box::new(transformed_inner))
            }
            Expr::Question(inner) => {
                let transformed_inner = transform_pipeline_rhs(inner, pipe_var);
                Expr::Question(Box::new(transformed_inner))
            }
            _ => right.clone(),
        }
    }
}
