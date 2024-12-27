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

impl AstNodeVec {
    pub fn postorder(&self) -> AstTraversal {
        let mut traversal = AstTraversal {
            tree: self,
            tree_stack: Vec::new(),
            pre_order: false,
        };

        let mut index = self.len() - 1;
        let final_index = 0 as usize;

        while index > final_index {
            let node = traversal.tree.get(index).unwrap();
            let subtree_size = *node.subtree_size;

            traversal.tree_stack.push(AstTraversalStack {
                index,
                subtree_size,
                expanded: subtree_size <= 1,
            });

            index -= subtree_size as usize;
        }

        return traversal;
    }

    pub fn preorder(&self) -> AstTraversal {
        let mut traversal = AstTraversal {
            tree: self,
            tree_stack: Vec::new(),
            pre_order: true,
        };

        let mut index = self.len() - 1;
        let final_index = 0 as usize;

        while index > final_index {
            let node = traversal.tree.get(index).unwrap();
            let subtree_size = *node.subtree_size;

            traversal.tree_stack.push(AstTraversalStack {
                index,
                subtree_size,
                expanded: subtree_size <= 1,
            });

            index -= subtree_size as usize;
        }

        return traversal;
    }
}

#[derive(Clone, Copy)]
struct AstTraversalStack {
    index: usize,
    expanded: bool,
    subtree_size: u32,
}

pub struct AstTraversal<'a> {
    tree: &'a AstNodeVec,
    tree_stack: Vec<AstTraversalStack>,
    pre_order: bool,
}

impl<'a> Iterator for AstTraversal<'a> {
    type Item = AstNodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut top = self.tree_stack.pop()?;

            if top.expanded || top.subtree_size <= 1 {
                return Some(self.tree.get(top.index).unwrap());
            }

            // expand
            let mut index = top.index - 1;
            let final_index = top.index - top.subtree_size as usize;

            top.expanded = true;
            if !self.pre_order {
                self.tree_stack.push(top);
            }

            while index > final_index {
                let node = self.tree.get(index).unwrap();
                let subtree_size = *node.subtree_size;

                self.tree_stack.push(AstTraversalStack {
                    index,
                    subtree_size,
                    expanded: subtree_size <= 1,
                });

                index -= subtree_size as usize;
            }

            if self.pre_order {
                self.tree_stack.push(top);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::util::*;

    #[test]
    fn parse_easy() {
        let mut ast = AstNodeVec::with_capacity(10);
        ast.push(AstNode {
            kind: AstNodeKind::StmtSentinel,
            subtree_size: 1,
        });
        ast.push(AstNode {
            kind: AstNodeKind::StmtIfIntro,
            subtree_size: 1,
        });
        ast.push(AstNode {
            kind: AstNodeKind::ExprBoolean,
            subtree_size: 1,
        });
        ast.push(AstNode {
            kind: AstNodeKind::StmtBlockIntro,
            subtree_size: 1,
        });
        ast.push(AstNode {
            kind: AstNodeKind::StmtBlock,
            subtree_size: 2,
        });
        ast.push(AstNode {
            kind: AstNodeKind::StmtIf,
            subtree_size: 5,
        });

        let mut preorder_ast = Vec::with_capacity(10);
        preorder_ast.push(AstNode {
            kind: AstNodeKind::StmtSentinel,
            subtree_size: 1,
        });
        preorder_ast.push(AstNode {
            kind: AstNodeKind::StmtIf,
            subtree_size: 5,
        });
        preorder_ast.push(AstNode {
            kind: AstNodeKind::StmtIfIntro,
            subtree_size: 1,
        });
        preorder_ast.push(AstNode {
            kind: AstNodeKind::ExprBoolean,
            subtree_size: 1,
        });
        preorder_ast.push(AstNode {
            kind: AstNodeKind::StmtBlock,
            subtree_size: 2,
        });
        preorder_ast.push(AstNode {
            kind: AstNodeKind::StmtBlockIntro,
            subtree_size: 1,
        });

        let mut postorder = Vec::with_capacity(ast.len());
        postorder.push(AstNode {
            kind: AstNodeKind::StmtSentinel,
            subtree_size: 1,
        });
        for token in ast.postorder() {
            postorder.push(token.to_owned());
        }

        let mut preorder = Vec::with_capacity(ast.len());
        preorder.push(AstNode {
            kind: AstNodeKind::StmtSentinel,
            subtree_size: 1,
        });
        for token in ast.preorder() {
            preorder.push(token.to_owned());
        }

        let mut source_order = Vec::new();
        for token in &ast {
            source_order.push(token.to_owned());
        }

        let mut source_order = Vec::new();
        for token in &ast {
            source_order.push(token.to_owned());
        }

        assert_eq!(&postorder, &source_order);
        assert_eq!(&preorder, &preorder_ast);
    }
}
