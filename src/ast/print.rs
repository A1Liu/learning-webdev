use super::*;


pub struct LinePrinter {
    indent_stack: Vec<PrintStackEntry>,
    stack_context: PrintStackEntry,
}

impl LinePrinter {
    pub fn print_tree(&self, tree: &AstNodeVec) {
        for node in tree {
            match node.kind {
                AstNodeKind::StmtIfIntro => {}

                _ => {}
            }
        }
    }
}

struct PrintStackEntry {
    indent_level: usize,
}

// Use atoms & nested groups
// Start with all groups horizontal, and flip to vertical as needed, starting
// with top-most groups. E.g.
//
// Start with:     |
//                 |< line length limit
// func(a, b, c(d), e(f + g, hijklmnop))
//
// Then first flip outside:
//                 |< line length limit
// func(
//   a,
//   b,
//   c(d),
//   e(f + g, hijklmnop)
// )
//
// Then flip any inner groups that need it:
//                 |< line length limit
// func(
//   a,
//   b,
//   c(d),
//   e(
//     f + g,
//     hijklmnop
//   )
// )
pub struct PrettyPrinter {
    indent_stack: Vec<PrintStackEntry>,
    stack_context: PrintStackEntry,
}

impl PrettyPrinter {
    pub fn print_tree(&self, tree: &AstNodeVec) {
        for node in tree {
            match node.kind {
                AstNodeKind::StmtIfIntro => {}

                _ => {}
            }
        }
    }
}
