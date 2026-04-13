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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDecl {
    pub span: Span,
    pub ident: Ident,
    pub methods: Vec<MethodSig>,
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub span: Span,
    pub ident: Ident,
    pub fields: Option<Vec<Field>>, 
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorSetDecl {
    pub span: Span,
    pub ident: Ident,
    pub variants: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub span: Span,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: BlockStmt,
    pub is_async: bool, // Added for async fn
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub span: Span,
    pub ident: Ident,
    pub implements: Vec<Ident>,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interface {
    pub span: Span,
    pub ident: Ident,
    pub methods: Vec<MethodSig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    Block(BlockStmt),
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumPattern {
    pub span: Span,
    pub enum_name: Ident,
    pub variant_name: Ident,
    pub fields: Option<Vec<ObjectPatProp>>, 
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
    Slice(SliceExpr),
    Match(Box<MatchExpr>),
    Spawn(Box<SpawnExpr>),
    Receive(Option<Box<Type>>),
    Zig(ZigEscapeExpr),
    Question(Box<Expr>),
    Array(Vec<Expr>),
    Object(Vec<ObjectField>), // Used for {k: v} (Anonymous Structs)
    Arrow(Box<ArrowExpr>),
    EnumInit(EnumInitExpr),
    Template(TemplateLit),
    Await(Box<Expr>), 
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectField {
    pub key: Ident,
    pub val: Expr,
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateLit {
    pub span: Span,
    pub parts: Vec<String>,
    pub exprs: Vec<Expr>,
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
    Add, Sub, Mul, Div, Eq, NotEq, Lt, Gt, LtEq, GtEq,
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
pub enum Type {
    I32, U64, F32, F64, USize, Bool, String, Void,
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Tuple(Vec<Type>), // [T1, T2]
    Object(Vec<Field>), // {k: T}
    Ref(Ident), 
    Fn(Vec<Type>, Box<Type>), 
    PID, 
    Unknown, 
    ErrorUnion(Box<Type>),
    Optional(Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub span: Span,
    pub ident: Ident,
    pub ty: Type,
    pub is_comptime: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub span: Span,
    pub ident: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodSig {
    pub span: Span,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}
