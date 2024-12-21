use crate::tokens::*;
use crate::util::*;
use std::ops::*;
use std::simd::prelude::*;

#[derive(Default)]
pub struct LexState {
    pub begin_index: usize,
    pub index: usize,
    pub template_level: usize,
    pub tokens: TokenVec,
}

impl LexState {
    fn peek(&self, bytes: &[u8]) -> Option<u8> {
        return bytes.get(self.index).map(|s| *s);
    }

    fn peek_n<const N: usize>(&self, bytes: &[u8]) -> Option<[u8; N]> {
        return bytes[self.index..].try_into().ok();
    }

    fn incr(&mut self) {
        self.index += 1;
    }

    fn incr_count(&mut self, count: usize) {
        self.index += count;
    }

    fn pop(&mut self, bytes: &[u8]) -> Option<u8> {
        let a = self.peek(bytes);
        self.incr();
        return a;
    }

    fn text<'a, 'b>(&'a self, bytes: &'b [u8]) -> &'b [u8] {
        return &bytes[self.span()];
    }

    fn span(&self) -> Range<usize> {
        return self.begin_index..self.index;
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            text_index: self.begin_index,
            extra: self.index as u32,
        });

        self.begin_index = self.index;
    }

    fn add_token_extra(&mut self, kind: TokenKind, extra: u32) {
        self.tokens.push(Token {
            kind,
            text_index: self.begin_index,
            extra,
        });

        self.begin_index = self.index;
    }

    fn peek_32(&self, bytes: &[u8]) -> Simd<u8, 32> {
        return Simd::load_or_default(&bytes[self.index..]);
    }

    fn e(&mut self, r: Result<(), String>) -> Result<(), LexResult> {
        match r {
            Ok(()) => return Ok(()),
            Err(e) => {
                let tokens = core::mem::replace(&mut self.tokens, TokenVec::new());

                return Err(LexResult { tokens, error: e });
            }
        }
    }
}

pub struct LexResult {
    pub tokens: TokenVec,
    pub error: String,
}

pub fn lex(text: &str, symbols: &mut Symbols) -> Result<TokenVec, LexResult> {
    let mut state_data = LexState::default();
    let state = &mut state_data;
    let bytes = text.as_bytes();

    while let Some(byte) = state.pop(bytes) {
        // Supposedly LLVM will automatically do the "computed-goto" trick here.
        // We'll profile/disassemble later ig.
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => lex_whitespace(state, bytes),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => lex_word(state, bytes, symbols),
            b'.' => lex_dot_with_suffix(state, bytes),
            b'0' => lex_number_with_prefix(state, bytes),
            b'1'..=b'9' => lex_number(state, bytes, false),

            b'\'' => {
                let result = lex_string(state, bytes, StringOpener::Quote);
                state.e(result)?;
            }
            b'"' => {
                let result = lex_string(state, bytes, StringOpener::DubQuote);
                state.e(result)?;
            }
            b'`' => lex_template(state, bytes, true),

            b'/' => lex_comment_or_div(state, bytes),

            b';' => state.add_token(TokenKind::Semicolon),
            b':' => state.add_token(TokenKind::Colon),

            b'[' => state.add_token(TokenKind::LBracket),
            b']' => state.add_token(TokenKind::RBracket),

            b'(' => state.add_token(TokenKind::LParen),
            b')' => state.add_token(TokenKind::RParen),

            b'{' => state.add_token(TokenKind::LBrace),
            b'}' => {
                if state.template_level > 0 {
                    lex_template(state, bytes, false);
                }

                state.add_token(TokenKind::RBrace)
            }

            b'+' => {
                if let Some(b'+') = state.peek(bytes) {
                    state.incr();
                    state.add_token(TokenKind::PlusPlus);
                    continue;
                } else {
                    state.add_token(TokenKind::Add)
                }
            }
            b'-' => {
                if let Some(b'-') = state.peek(bytes) {
                    state.incr();
                    state.add_token(TokenKind::MinusMinus);
                    continue;
                } else {
                    state.add_token(TokenKind::Sub)
                }
            }

            b'*' => state.add_token(TokenKind::Mult),

            _ => {
                return Err(LexResult {
                    tokens: state_data.tokens,
                    error: (format!("unrecognized token: {} ({})", char::from(byte), byte)),
                })
            }
        }
    }

    return Ok(state_data.tokens);
}

#[repr(u8)]
pub enum StringOpener {
    Quote = b'\'',
    DubQuote = b'"',
}

const NEWLINE_SIMD: Simd<u8, 32> = Simd::from_array([b'\n'; 32]);
const ZERO_SIMD: Simd<u8, 32> = Simd::from_array([0; 32]);

pub fn lex_comment_or_div(state: &mut LexState, bytes: &[u8]) {
    let star_filter = crate::simd::FilterShiftR::<1>::new(b'*');
    const SLASH_SIMD: Simd<u8, 32> = Simd::from_array([b'/'; 32]);

    match state.peek(bytes).unwrap_or(0) {
        b'/' => loop {
            let text = state.peek_32(bytes);
            let newline_mask = text.simd_eq(NEWLINE_SIMD);
            let zero_mask = text.simd_eq(ZERO_SIMD);
            let end_mask = newline_mask | zero_mask;

            if let Some(index) = end_mask.first_set() {
                state.incr_count(index + 1);
                state.add_token(TokenKind::LineComment);
                break;
            }

            state.incr_count(32);
        },

        b'*' => loop {
            state.incr();

            let text = state.peek_32(bytes);
            let zero_mask = text.simd_eq(ZERO_SIMD);

            let star_mask = star_filter.check_eq(text);
            let slash_mask = text.simd_eq(SLASH_SIMD);

            let comment_end_mask = star_mask & slash_mask;

            match comment_end_mask.first_set() {
                None => {}
                Some(index) => {
                    state.incr_count(index + 1);
                    state.add_token(TokenKind::Comment);
                    break;
                }
            }

            if let Some(index) = zero_mask.first_set() {
                // This is technically an error.
                state.incr_count(index);
                state.add_token(TokenKind::Comment);
                break;
            }

            state.incr_count(31);
        },

        _ => {
            state.add_token(TokenKind::Div);
        }
    }
}

pub fn lex_string(state: &mut LexState, bytes: &[u8], opener: StringOpener) -> Result<(), String> {
    const NEWLINE_MASK: Simd<u8, 32> = Simd::from_array([b'\n'; 32]);
    let backslash_filter = crate::simd::FilterShiftR::<1>::new(b'\\');
    let mask = Simd::from_array([opener as u8; 32]);

    loop {
        let text = state.peek_32(bytes);

        let zero_mask = text.simd_eq(ZERO_SIMD);
        let not_escaped_mask = backslash_filter.check_ne(text);
        let quote_mask = text.simd_eq(mask);
        let newline_mask = text.simd_eq(NEWLINE_MASK);

        let quote_end_mask = quote_mask & not_escaped_mask;
        let newline_end_mask = newline_mask & not_escaped_mask;

        let error_end_mask = zero_mask | newline_end_mask;

        let (first_set, is_error) = match (quote_end_mask.first_set(), error_end_mask.first_set()) {
            (Some(quote), Some(error)) => (std::cmp::min(quote, error), error < quote),
            (Some(quote), None) => (quote, false),
            (None, Some(error)) => (error, true),
            (None, None) => {
                state.incr_count(31);
                continue;
            }
        };

        state.incr_count(first_set);

        if is_error {
            let zero_first = zero_mask.first_set().unwrap_or(32);
            let newline_first = newline_end_mask.first_set().unwrap_or(32);
            if zero_first < newline_first {
                return Err(format!("File ended without finishing string"));
            } else {
                return Err(format!("String ended with newline instead of quote"));
            }
        }

        state.incr();

        state.add_token(TokenKind::String);

        return Ok(());
    }
}

pub fn lex_template(state: &mut LexState, bytes: &[u8], beginning: bool) {
    println!("TEST");

    const TICK_MASK: Simd<u8, 32> = Simd::from_array([b'`'; 32]);
    const LBRACE_MASK: Simd<u8, 32> = Simd::from_array([b'{'; 32]);

    let slash_1_filter = crate::simd::FilterShiftR::<1>::new(b'\\');
    let dollar_1_filter = crate::simd::FilterShiftR::<1>::new(b'$');
    let slash_2_filter = crate::simd::FilterShiftR::<2>::new(b'\\');

    loop {
        let text = state.peek_32(bytes);

        let tick_mask = text.simd_eq(TICK_MASK);
        let lbrace_mask = text.simd_eq(LBRACE_MASK);

        let slash_mask_1 = slash_1_filter.check_ne(text);

        let slash_mask_2 = slash_2_filter.check_ne(text);
        let dollar_mask_1 = dollar_1_filter.check_eq(text);

        let tick_end_mask = tick_mask & slash_mask_1;
        let lbrace_end_mask = lbrace_mask & dollar_mask_1 & slash_mask_2;

        let index = match (tick_end_mask | lbrace_end_mask).first_set() {
            Some(i) => i,
            None => {
                state.incr_count(30);
                continue;
            }
        };

        state.incr_count(index + 1);

        let first_tick = tick_end_mask.first_set().unwrap_or(33);
        let first_lbrace = lbrace_end_mask.first_set().unwrap_or(33);

        match (first_tick < first_lbrace, beginning) {
            (true, true) => state.add_token(TokenKind::StrTemplate),
            (false, false) => state.add_token(TokenKind::StrTemplateMid),

            (true, false) => {
                state.template_level -= 1;
                state.add_token(TokenKind::StrTemplateEnd);
            }
            (false, true) => {
                state.template_level += 1;
                state.add_token(TokenKind::StrTemplateBegin);
            }
        }

        // panic!("");
        break;
    }
}

pub fn lex_dot_with_suffix(state: &mut LexState, bytes: &[u8]) {
    match state.peek_n::<2>(bytes) {
        Some([b'.', b'.']) => {
            state.incr_count(2);
            state.add_token(TokenKind::Spread);
        }
        Some([b'.', _]) => {
            state.add_token(TokenKind::Dot);
        }
        Some([b'0'..=b'9', _]) => {
            lex_number(state, bytes, true);
        }
        Some(_) | None => {
            state.add_token(TokenKind::Dot);
        }
    }
}

pub fn lex_number_with_prefix(state: &mut LexState, bytes: &[u8]) {
    match state.peek(bytes).unwrap_or(0) {
        b'n' => state.add_token(TokenKind::BigInt),
        b'e' => return lex_number(state, bytes, true),
        b'.' => {
            state.incr();
            return lex_number(state, bytes, true);
        }

        b'b' | b'B' => {
            state.incr();
            unimplemented!();
        }
        b'x' | b'X' => {
            state.incr();
            unimplemented!();
        }
        b'o' | b'O' => {
            state.incr();
            unimplemented!();
        }

        b'0'..=b'9' => {
            unimplemented!();
        }

        _ => {
            state.add_token(TokenKind::Number);
        }
    }
}

pub fn lex_number(state: &mut LexState, bytes: &[u8], mut has_dot: bool) {
    // TODO: SIMD-ify?
    let mut has_exponent = false;

    loop {
        match state.peek(bytes).unwrap_or(0) {
            b'0'..=b'9' => {
                // its definitely a number

                state.incr();
            }

            b'.' => {
                if has_exponent {
                    state.add_token(TokenKind::Number);
                    return;
                }

                if has_dot {
                    state.add_token(TokenKind::Number);
                    return;
                }

                has_dot = true;
                state.incr();
            }

            b'e' => {
                if has_exponent {
                    unimplemented!();
                }

                has_exponent = true;
            }

            _ => {
                state.add_token(TokenKind::Number);
                return;
            }
        }
    }
}

// TODO: handle utf-8 characters
pub fn lex_word(state: &mut LexState, bytes: &[u8], symbols: &mut Symbols) {
    const EQ_0: Simd<u8, 32> = Simd::from_array([0u8; 32]);

    loop {
        let text = Simd::from_array(state.peek_32(bytes).to_array().map(|a| a as usize));
        let word_mask = Simd::gather_or_default(&*ALNUM_MAP, text);
        let non_alnum_mask = word_mask.simd_eq(EQ_0);

        if let Some(index) = non_alnum_mask.first_set() {
            state.incr_count(index);
            break;
        }

        state.incr_count(32);
    }

    let word = state.text(bytes);

    if let Some(key) = KEY_HASH_MAP.get(word) {
        state.add_token(TokenKind::Key(*key));
        return;
    }

    // TODO: we can get rid of this unsafe if desired. Whatever.
    let symbol = symbols.add_str(unsafe { core::str::from_utf8_unchecked(word) });

    state.add_token_extra(TokenKind::Word, symbol);
}

pub fn lex_whitespace(state: &mut LexState, bytes: &[u8]) {
    const TAB: Simd<u8, 32> = Simd::from_array([b'\t'; 32]);
    const SPACE: Simd<u8, 32> = Simd::from_array([b' '; 32]);
    const CARRIAGE_RETURN: Simd<u8, 32> = Simd::from_array([b'\r'; 32]);

    loop {
        let bytes = state.peek_32(bytes);

        let newlines = bytes.simd_eq(NEWLINE_SIMD);
        let tabs = bytes.simd_eq(TAB);
        let spaces = bytes.simd_eq(SPACE);
        let carriage_returns = bytes.simd_eq(CARRIAGE_RETURN);

        let whitespace_mask = newlines | carriage_returns | spaces | tabs;
        let whitespace_mask = !whitespace_mask;

        if let Some(index) = whitespace_mask.first_set() {
            state.incr_count(index);
            state.add_token(TokenKind::Whitespace);
            break;
        }

        state.incr_count(32);
    }
}

type MyFunc = for<'a> fn(state: &'a mut LexState, bytes: &'a [u8]);

#[cfg(test)]
mod tests {
    use super::*;
    use test_generator::test_resources;
    use yaml_rust::YamlLoader;

    #[test_resources("test/easy/*")]
    fn lex_easy(path: &str) {
        let source = std::fs::read_to_string(path).expect("Should have been able to read the file");

        let mut symbols = Symbols::new();
        let tokens = lex(&source, &mut symbols)
            .map_err(|e| e.error)
            .expect("doesn't error");

        println!("{}", source);

        let mut output = Vec::new();
        for token in &tokens {
            if *token.kind == TokenKind::Whitespace {
                continue;
            }

            println!("{:?}", token.to_owned());

            output.push(format!("{:?}", token.kind));
        }

        let mut yaml_text = "";
        for item in source.split("/*---") {
            if item == "" {
                continue;
            }
            yaml_text = item;
            break;
        }

        let mut yaml_text_2 = "";
        for item in yaml_text.split("---*/") {
            yaml_text_2 = item;
            break;
        }

        let yaml_text = yaml_text_2;

        let docs = YamlLoader::load_from_str(yaml_text).unwrap();
        if docs.len() == 0 {
            return;
        }

        let doc = &docs[0];
        let expected_token_string = doc["tokens"].as_str().unwrap_or("");

        let mut expected_tokens = Vec::new();
        for token in expected_token_string.trim().split(",") {
            let token = token.trim();
            expected_tokens.push(token);
        }

        assert_eq!(&output, &expected_tokens);
    }
}
