use crate::*;
use super::wadler_print::*;

#[derive(Default)]
struct NotationBuilder {
    // notation + subtree size
    note_stack: Vec<(Option<Notation>, u32)>,
}

impl NotationBuilder {
    pub fn collect_tree(&mut self, subtree_size: u32) -> Vec<Notation> {
        let mut output = Vec::new();
        let mut current: u32 = 1;

        while current < subtree_size {
            let (notation, tree_size) = match self.note_stack.pop() {
                Some(a) => a,
                None => break,
            };

            output.extend(notation);
            current += tree_size;
        }

        return output;
    }

    fn get_note(&mut self, node: AstNodeRef) -> Option<Notation> {
        use AstNodeKind::*;

        let notation = match node.kind {
            ExprWord => Notation::txt("word"),
            ExprBoolean => Notation::txt("true"),

            StmtIfIntro => return None,
            StmtBlockIntro => return None,
            StmtBlock => 'end: {
                let children = self.collect_tree(*node.subtree_size);

                if children.len() == 0 {
                    break 'end NOTE & "{" & NL & "}" & DONE;
                }

                let mut note = NOTE & NL;

                for child in children {
                    note = NOTE & NL & child & note;
                }

                NOTE & "{" & -note & "}" & DONE
            }
            StmtIf => 'end: {
                let mut children = self.collect_tree(*node.subtree_size);

                let cond = &children.pop().unwrap();
                let if_cond = &children.pop().unwrap();
                let else_cond = &children.pop();

                let cond = NOTE & "if (" & cond & ")" & DONE;
                let cond = &cond;

                let Some(else_cond) = else_cond else {
                    let if_cond_flat = NOTE & " " & if_cond & DONE;
                    let if_cond_vert = NOTE & NL & if_cond & DONE;
                    let if_cond = if_cond_flat | if_cond_vert;

                    break 'end NOTE & cond & if_cond & DONE;
                };

                let if_cond_flat = NOTE & cond & " " & if_cond & " else " & else_cond & DONE;
                let if_cond_vert = NOTE & cond & NL & -if_cond & NL & "else" & -else_cond & DONE;

                if_cond_flat | if_cond_vert
            }

            UtilSentinel => {
                return None;
            }

            _ => unimplemented!("printing for {:?}", node.kind),
        };

        return Some(notation);
    }

    pub fn build(&mut self, ast: &AstNodeVec) -> Notation {
        for node in ast {
            let notation = self.get_note(node);

            self.note_stack.push((notation, *node.subtree_size));
        }

        let mut note = NOTE;
        while let Some((Some(cur), _)) = self.note_stack.pop() {
            note = NOTE & cur & NL & note;
        }
        return note & DONE;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_resources("test/printing/*")]
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

        let ast = parse(&tokens).expect("doesn't error");

        println!(
            "{:?}",
            &ast.iter().map(|a| a.to_owned()).collect::<Vec<_>>()
        );

        let mut builder = NotationBuilder::default();
        let notation = builder.build(&ast);

        let mut printer = WadlerPrinter::new(&notation, 80);

        let output = printer.print();

        assert_eq!(output, source);
    }
}