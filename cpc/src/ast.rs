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
    pub meta: Vec<MetaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    Expr(Expr),
    Var(VarDecl),
    Return(ReturnStmt),
    If(IfStmt),
    While(WhileStmt),
    Match(MatchExpr),
    Spawn(SpawnExpr),
    Defer(Box<Stmt>),
    ErrDefer(Box<Stmt>),
    ComptimeBlock(BlockStmt),
    Block(BlockStmt),
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
}
