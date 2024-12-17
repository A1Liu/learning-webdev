use soa_derive::*;
use crate::tokens::*;
use crate::util::*;

pub struct LexState {
    index: usize,
}

#[derive(StructOfArray)]
pub struct TokenList {
    pub token: Token,
    pub text_index: usize,
    pub extra: usize,
}

pub struct LexResult {
    pub tokens: TokenList,
}

pub fn lex(text: &str, symbols: &mut Symbols) -> Vec<Token> {
    return Vec::new();
}
