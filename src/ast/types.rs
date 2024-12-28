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

    // A dummy node that does nothing and doesn't technically exist. However,
    // it makes traversal math easier to always include it in the beginning.
    StmtSentinel,
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

    const SENTINEL: AstNode = AstNode {
        kind: AstNodeKind::StmtSentinel,
        subtree_size: 1,
    };

    struct TreeNode {
        kind: AstNodeKind,
        subtree_size: u32,
        children: Vec<TreeNode>,
    }

    struct Traversals {
        postorder: Vec<AstNode>,
        preorder: Vec<AstNode>,
        ast: AstNodeVec,
    }

    impl From<AstNodeKind> for TreeNode {
        fn from(value: AstNodeKind) -> Self {
            return Self::new(value);
        }
    }

    impl TreeNode {
        fn new(kind: AstNodeKind) -> Self {
            return Self {
                kind,
                subtree_size: 1,
                children: Vec::new(),
            };
        }

        fn add<T: Into<Self>>(mut self, child: T) -> Self {
            let child: Self = child.into();
            self.subtree_size += child.subtree_size;
            self.children.push(child);

            return self;
        }

        fn postorder_append(&self, output: &mut Vec<AstNode>) {
            for child in &self.children {
                child.postorder_append(output);
            }

            output.push(AstNode {
                kind: self.kind,
                subtree_size: self.subtree_size,
            });
        }

        fn preorder_append(&self, output: &mut Vec<AstNode>) {
            output.push(AstNode {
                kind: self.kind,
                subtree_size: self.subtree_size,
            });

            for child in &self.children {
                child.preorder_append(output);
            }
        }

        fn traverse(tree: &Vec<Self>) -> Traversals {
            let mut ast = AstNodeVec::new();

            let mut postorder = Vec::new();
            let mut preorder = Vec::new();
            for node in tree {
                node.postorder_append(&mut postorder);
                node.preorder_append(&mut preorder);
            }

            for node in &postorder {
                ast.push(node.clone());
            }

            return Traversals {
                postorder,
                preorder,
                ast,
            };
        }
    }

    #[test]
    fn traverse_easy() {
        use AstNodeKind::*;

        let tree = vec![
            TreeNode::new(StmtSentinel),
            TreeNode::new(StmtIf)
                .add(StmtIfIntro)
                .add(ExprBoolean)
                .add(TreeNode::new(StmtBlock).add(StmtBlockIntro)),
        ];

        let traverse = TreeNode::traverse(&tree);

        let mut postorder = Vec::with_capacity(traverse.ast.len());
        postorder.push(SENTINEL);
        postorder.extend(traverse.ast.postorder().map(|a| a.to_owned()));

        let mut preorder = Vec::with_capacity(traverse.ast.len());
        preorder.push(SENTINEL);
        preorder.extend(traverse.ast.preorder().map(|a| a.to_owned()));

        assert_eq!(&postorder, &traverse.postorder);
        assert_eq!(&preorder, &traverse.preorder);
    }
}
