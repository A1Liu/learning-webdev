use soa_derive::*;
use strum::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, IntoStaticStr)]
#[repr(u8)]
pub enum AstNodeKind {
    // Expressions
    ExprString,
    ExprNumber,
    ExprBoolean,

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

    StmtBlockIntro,
    StmtBlock,

    StmtSentinel, // A dummy node that does nothing and doesn't technically exist. However, it
                  // makes traversal math easier to always include it in the beginning.
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

struct AstPostorderTraversalStack {
    index: usize,
    expanded: bool,
    subtree_size: u32,
}

pub struct AstPostorderTraversal<'a> {
    tree: &'a AstNodeVec,
    tree_stack: Vec<AstPostorderTraversalStack>,
    index: usize,
}

impl<'a> Iterator for AstPostorderTraversal<'a> {
    type Item = AstNodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut top = self.tree_stack.pop()?;

            if top.expanded || top.subtree_size <= 1 {
                return self.tree.get(top.index);
            }

            // expand
            let mut index = top.index - 1;
            let final_index = top.index - top.subtree_size as usize;

            top.expanded = true;
            self.tree_stack.push(top);

            while index > final_index {
                let node = self.tree.get(index).unwrap();
                let subtree_size = *node.subtree_size;

                self.tree_stack.push(AstPostorderTraversalStack {
                    index,
                    subtree_size,
                    expanded: subtree_size <= 1,
                });

                index -= subtree_size as usize;
            }
        }
    }
}
