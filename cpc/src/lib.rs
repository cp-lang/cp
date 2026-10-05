pub mod ast;
pub mod ast_visitor;
pub mod codegen;
pub mod dcp;
pub mod lexer;
pub mod lsp;
pub mod parser;
pub mod resolver;
pub mod sema;

pub use lexer::{Lexer, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}
