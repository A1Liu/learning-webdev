use std::collections::HashMap;
use strum::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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

    Comment,
    Whitespace,
    Unknown,

    Dot,
    Eq,

    String,
    StrTemplateBegin,
    StrTemplateMid,
    StrTemplateEnd,

    Number,

    Word,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, EnumIter, IntoStaticStr)]
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
    op: bool,
}

lazy_static::lazy_static! {
    // TODO: switch to a bloom filter maybe?
    pub static ref KEY_HASH_MAP : HashMap<&'static [u8], Key> = {
        let mut map = HashMap::with_capacity(64);

        for keyword in Key::iter() {
            let text: &'static str = keyword.into();
            let bytes = text.as_bytes();

            map.insert(bytes, keyword);
        }

        map
    };

    pub static ref CHAR_MAP : [CharAttrs; 256] = {
        const DEFAULT_CHAR_ATTRS : CharAttrs = CharAttrs {
            alpha: false,
            num: false,
            alnum: false,
            op: false,
        };
        let mut attrs = [DEFAULT_CHAR_ATTRS; 256];

        for i in 128..256 {
            attrs[i] = CharAttrs {
                alnum: true,
                alpha: true,
                num: false,
                op: false,
            };
        }

        for i in b'a'..=b'z' {
            attrs[i as usize] = CharAttrs {
                alnum: true,
                alpha: true,
                num: false,
                op: false,
            };
        }

        for i in b'A'..=b'Z' {
            attrs[i as usize] = CharAttrs {
                alnum: true,
                alpha: true,
                num: false,
                op: false,
            };
        }

        for i in b'0'..=b'9' {
            attrs[i as usize] = CharAttrs {
                alnum: true,
                alpha: false,
                num: true,
                op: false,
            };
        }

        attrs
    };

}

// Flagged as dead code unfortunately
#[allow(dead_code)]
const fn check_tokenkind_size() {
    assert!(size_of::<TokenKind>() == 1);
}

const _: () = check_tokenkind_size();
