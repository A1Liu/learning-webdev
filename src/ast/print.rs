use super::*;
use crate::util::*;

struct SimpleLinePrintContext {
    suffix: String,
    indentation_level: u32,
}

enum LinePrintContext {
    // Pop items off in between children
    Stacked(Vec<SimpleLinePrintContext>),

    Simple(SimpleLinePrintContext),
}

// need prefix order traversal
struct LinePrintStackEntry {
    context: LinePrintContext,

    // Insert between each child
    separator: String,
}

pub struct LinePrinter<'a> {
    current_indent: u32,
    stack: Vec<LinePrintStackEntry>,
    context: LinePrintStackEntry,
    output: &'a mut dyn std::io::Write,
}

impl<'a> LinePrinter<'a> {
    pub fn print_tree(&mut self, tree: &AstNodeVec, symbols: &Symbols) -> std::io::Result<()> {
        for node in tree.preorder() {
            let new_stack_entry = match node.kind {
                AstNodeKind::StmtIfIntro => {
                    // NOTE:
                    //   need a stack, even for this ffs
                    //
                    // NOTE:
                    //   need to do a traversal here to get infix ordering
                    //   on expressions. :/
                    //
                    write!(self.output, "if (")?;
                }

                _ => {}
            };
        }

        return Ok(());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test_resources("test/easy/conditional.*")]
    fn print_easy(path: &str) {
        let source = std::fs::read_to_string(path).expect("Should have been able to read the file");

        let mut symbols = Symbols::new();
        let tokens = lex_with_options(
            &source,
            &mut symbols,
            LexOptions {
                include_comments: true,
                include_spacing: false,
            },
        )
        .map_err(|e| e.error)
        .expect("doesn't error");

        let ast = parse(&tokens);
    }
}
