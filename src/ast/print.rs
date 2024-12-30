use super::wadler_print::*;
use crate::*;

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
                    break 'end Notation::braced(NOTE & "{" & NL & "}" & DONE);
                }

                let mut note = NOTE & NL;

                for child in children {
                    note = NOTE & NL & child & note;
                }

                Notation::braced(NOTE & "{" & -note & "}" & DONE)
            }
            StmtIf => {
                // if (true) {  if (true)
                //   blarg        blarg
                // } else {     else {
                //   merp         merp
                // }            }

                // if (true)    if (true) {
                //   blarg        merp
                // else         } else
                //   merp         merp

                let mut children = self.collect_tree(*node.subtree_size);

                let cond = &children.pop().unwrap();
                let if_cond = &children.pop().unwrap();
                let else_cond = &children.pop();

                let suffix = match else_cond {
                    None => Notation::txt(""),
                    Some(else_cond) => {
                        let else_cond_flat = NOTE & " " & else_cond & DONE;
                        let else_cond_vert = NOTE & NL & -else_cond & DONE;
                        let else_cond = else_cond_flat | else_cond_vert;

                        let else_cond = NOTE & "else" & else_cond & DONE;

                        match if_cond.0.as_ref() {
                            NotationInner::Braced(_) => NOTE & " " & else_cond & DONE,
                            _ => NOTE & NL & else_cond & DONE,
                        }
                    }
                };

                let if_cond_flat = NOTE & " " & if_cond & DONE;
                let if_cond_vert = NOTE & NL & -if_cond & DONE;
                let if_cond = if_cond_flat | if_cond_vert;

                let cond_flat = NOTE & cond & DONE;
                let cond_vert = NOTE & NL & -cond & NL & DONE;
                let cond = cond_flat | cond_vert;

                NOTE & "if (" & cond & ")" & if_cond & suffix & DONE
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

        // println!(
        //     "{:?}",
        //     &ast.iter().map(|a| a.to_owned()).collect::<Vec<_>>()
        // );

        let mut builder = NotationBuilder::default();
        let notation = builder.build(&ast);

        let mut printer = WadlerPrinter::new(&notation, 80);

        let output = printer.print();

        pretty_assertions::assert_eq!(source, output);
    }
}
