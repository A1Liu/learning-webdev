use soa_derive::*;
use strum::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, IntoStaticStr)]
#[repr(u8)]
pub enum AstNodeKind {
    // Expressions
    ExprString,
    ExprNumber,

    ExprTemplateIntro,
    ExprTemplate,

    ExprFunctionIntro,
    ExprFunction,

    ExprParenIntro,
    ExprParen,

    // Not an expr, but... sort of one. Maybe.
    ExprParamsIntro,
    ExprParams,

    // Statements
    StmtIfIntro,
    StmtIf,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, StructOfArray)]
pub struct AstNode {
    pub kind: AstNodeKind,
    pub subtree_size: u32,
}

// Flagged as dead code unfortunately
#[allow(dead_code)]
const fn check_astnodekind_size() {
    assert!(size_of::<AstNodeKind>() == 1);
}

const _: () = check_astnodekind_size();
