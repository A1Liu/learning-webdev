use soa_derive::*;
use std::collections::HashMap;
use strum::*;

use crate::util::Symbols;

#[derive(Debug, StructOfArray)]
pub struct Token {
    pub kind: TokenKind,
    pub text_index: usize,
    pub extra: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, IntoStaticStr)]
pub enum TokenKind {
    /// Keywords
    Key(Key),

    PlusPlus,
    MinusMinus,
    Spread,

    Comma,

    Add,
    Sub,
    Div,
    Mult,

    BoolAnd,
    BoolOr,

    BinAnd,
    BinOr,
    BinXor,

    EqEq,
    EqEqEq,
    Neq,
    Geq,
    Leq,
    Gt,
    Lt,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Semicolon,
 Colon,

    LineComment,
    Comment,
    Whitespace,
    Unknown,

    Dot,
    Eq,

    String,
    StrTemplate,
    StrTemplateBegin,
    StrTemplateMid,
    StrTemplateEnd,

    Number,
    OctNumber,
    HexNumber,
    BinNumber,
    BigInt,

    Word,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, EnumIter, IntoStaticStr)]
#[repr(u8)]
pub enum Key {
    As = 2,
    Async,
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    False,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    New,
    Null,
    Return,
    Super,
    Switch,
    This,
    Throw,
    True,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,
}

#[derive(Copy, Clone)]
pub struct CharAttrs {
    alpha: bool,
    num: bool,
    alnum: bool,
    spec: bool,
}

lazy_static::lazy_static! {
    pub static ref KEY_HASH_MAP : HashMap<Vec<u8>, Key> = {
        let mut map = HashMap::with_capacity(64);

        // TODO: This is dumb. We can keep it for now, but it's dumb, lets use
        // something real in the future.
        for keyword in Key::iter() {
            let text: &'static str = keyword.into();
            let lowercase = text.to_ascii_lowercase();
            let bytes = lowercase.as_bytes().to_owned();

            map.insert(bytes, keyword);
        }

        map
    };

    pub static ref ALNUM_MAP : [u8; 256] = {
        let mut attrs = [0; 256];

        for i in 128..256 {
            attrs[i] = 1;
        }

        for i in b'a'..=b'z' {
            attrs[i as usize] = 1;
        }

        for i in b'A'..=b'Z' {
            attrs[i as usize] = 1;
        }

        for i in b'0'..=b'9' {
            attrs[i as usize] = 1;
        }

        attrs[b'_' as usize] = 1;

        attrs
    };
}

// Flagged as dead code unfortunately
#[allow(dead_code)]
const fn check_tokenkind_size() {
    assert!(size_of::<TokenKind>() == 1);
}

const _: () = check_tokenkind_size();

impl TokenKind {
    pub fn len(&self) -> Option<usize> {
        match self {
            Self::Key(key) => {
                let token_name: &'static str = key.into();
                return Some(token_name.len());
            }

            Self::PlusPlus => return Some(2),
            Self::MinusMinus => return Some(2),

            Self::Dot => return Some(1),
            Self::Spread => return Some(3),

            Self::Comma => return Some(1),

            Self::Add => return Some(1),
            Self::Sub => return Some(1),
            Self::Div => return Some(1),
            Self::Mult => return Some(1),

            Self::BoolAnd => return Some(2),
            Self::BoolOr => return Some(2),

            Self::BinAnd => return Some(1),
            Self::BinOr => return Some(1),
            Self::BinXor => return Some(1),

            Self::Eq => return Some(1),
            Self::EqEq => return Some(2),
            Self::EqEqEq => return Some(3),
            Self::Neq => return Some(2),
            Self::Geq => return Some(2),
            Self::Leq => return Some(2),
            Self::Gt => return Some(1),
            Self::Lt => return Some(1),

            Self::LParen => return Some(1),
            Self::RParen => return Some(1),
            Self::LBracket => return Some(1),
            Self::RBracket => return Some(1),
            Self::LBrace => return Some(1),
            Self::RBrace => return Some(1),

            _ => return None,
        }
    }
}

impl TokenVec {
    pub fn serialize(&self, symbols: &Symbols) -> String {
        let mut output = String::new();
        for token in self {
            let token_name: &'static str = (*token.kind).into();

            match token.kind {
                TokenKind::Word => {
                    output.push_str(token_name);

                    let word_name = symbols.to_str(*token.extra).unwrap();

                    output.push('(');
                    output.push_str(word_name);
                    output.push(')');
                }

                TokenKind::Key(key) => {
                    let key_name: &'static str = key.into();
                    output.push_str(key_name);
                }

                _ => {
                    output.push_str(token_name);
                }
            }

            output.push(' ');
        }

        output.pop();

        return output;
    }
}
