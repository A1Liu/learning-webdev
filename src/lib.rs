#![allow(dead_code)]

#![feature(test)]
#![feature(portable_simd)]

pub mod ast;
pub mod lexer;
pub mod simd;
pub mod tokens;
pub mod util;

pub use ast::*;
pub use lexer::{lex, lex_with_options, LexOptions};
pub use util::Symbols;

#[cfg(test)]
pub use util::tests::*;
