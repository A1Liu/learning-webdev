use crate::tokens::*;
use crate::util::*;
use soa_derive::*;

#[derive(Default)]
pub struct LexState {
    pub begin_index: usize,
    pub index: usize,
    pub within_template: bool,
    pub tokens: TokenVec,
}

impl LexState {
    fn peek(&self, bytes: &[u8]) -> Option<u8> {
        return bytes.get(self.index).map(|s| *s);
    }

    fn incr(&mut self) {
        self.index += 1;
    }

    fn pop(&mut self, bytes: &[u8]) -> Option<u8> {
        let a = self.peek(bytes);
        self.incr();
        return a;
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            text_index: self.begin_index,
            extra: self.index as u32,
        });
    }

    fn add_token_extra(&mut self, kind: TokenKind, extra: u32) {
        self.tokens.push(Token {
            kind,
            text_index: self.begin_index,
            extra,
        });

        self.begin_index = self.index;
    }
}

#[derive(StructOfArray)]
pub struct Token {
    pub kind: TokenKind,
    pub text_index: usize,
    pub extra: u32,
}

pub struct LexResult {
    pub tokens: TokenVec,
    pub error: Option<String>,
}

pub fn lex(text: &str, _symbols: &mut Symbols) -> LexResult {
    let mut state_data = LexState::default();
    let state = &mut state_data;
    let bytes = text.as_bytes();

    while let Some(byte) = state.pop(bytes) {
        // Supposedly LLVM will automatically do the "computed-goto" trick here.
        // We'll profile/disassemble later ig.
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => lex_whitespace(state, bytes),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => lex_word(state, bytes, byte),
            b'.' | b'0'..=b'9' => lex_number(state, bytes),

            b'\'' => lex_string(state, bytes, StringOpener::Quote),
            b'"' => lex_string(state, bytes, StringOpener::DubQuote),
            b'`' => lex_string(state, bytes, StringOpener::Template),

            b'/' => lex_comment_or_div(state, bytes),

            b'[' => state.add_token(TokenKind::LBracket),
            b']' => state.add_token(TokenKind::RBracket),

            b'(' => state.add_token(TokenKind::LParen),
            b')' => state.add_token(TokenKind::RParen),

            b'{' => state.add_token(TokenKind::LBrace),
            b'}' => {
                if state.within_template {
                    lex_string(state, bytes, StringOpener::TemplateBrace);
                }

                state.add_token(TokenKind::RBrace)
            }

            _ => {
                return LexResult {
                    tokens: state_data.tokens,
                    error: Some("unrecognized token".into()),
                }
            }
        }
    }

    return LexResult {
        tokens: state_data.tokens,
        error: None,
    };
}

#[repr(u8)]
pub enum StringOpener {
    Quote,
    DubQuote,
    Template,
    TemplateBrace,
}

pub fn lex_comment_or_div(state: &mut LexState, bytes: &[u8]) {}

pub fn lex_string(state: &mut LexState, bytes: &[u8], closer: StringOpener) {}

pub fn lex_number(state: &mut LexState, bytes: &[u8]) {}

// TODO: handle utf-8 characters
pub fn lex_word(state: &mut LexState, bytes: &[u8], letter: u8) {
    // let word = std::simd::u8x64::from_array(bytes.try_into().expect("Failed"));
    while let Some(byte) = state.peek(bytes) {
        // Use char attrs here
    }
}

pub fn lex_whitespace(state: &mut LexState, bytes: &[u8]) {
    while let Some(cur) = state.peek(bytes) {
        match cur {
            b' ' | b'\t' => {
                state.incr();
            }
            _ => break,
        }
    }

    state.add_token(TokenKind::Whitespace);
}

type MyFunc = for<'a> fn(state: &'a mut LexState, bytes: &'a [u8]);
