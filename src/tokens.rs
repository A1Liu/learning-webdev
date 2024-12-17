use strum::IntoStaticStr;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Token {
    /// Operators
    Op(Op),

    /// Keywords
    Key(Key),

    LParen = 128,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Comment,
    Whitespace,
    Unknown,

    Word,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, IntoStaticStr)]
#[repr(u8)]
pub enum Op {
    Plus = 1,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, IntoStaticStr)]
#[repr(u8)]
pub enum Key {
    As = 32,
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
