pub mod lexer;
pub mod ast;
pub mod parser;
pub mod sema;
pub mod codegen;
pub mod dcp;
pub mod lsp;

pub use lexer::{Token, Lexer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
