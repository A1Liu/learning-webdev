use crate::*;
use std::ops::*;
use std::rc::Rc;

// Ideas
// 1. Switch from RC to arena
// 2. Segment it out
// 3. Don't make the whole notation tree for short lines?

// Using a first implementation which is copied ~100% from Justin Pombrio
#[derive(Debug, Clone)]
pub struct Notation(Rc<NotationInner>);

#[derive(Debug, Clone)]
pub enum NotationInner {
    Newline,
    Text(String, u32),
    Flat(Notation),
    Indent(Notation),
    Braced(Notation),
    Concat(Notation, Notation),
    Choice(Notation, Notation),
}

impl Notation {
    /// Display a newline
    pub fn nl() -> Notation {
        Notation(Rc::new(NotationInner::Newline))
    }

    /// Display text exactly as-is. The text should not contain a newline!
    pub fn txt(s: impl ToString) -> Notation {
        let string = s.to_string();
        let width = string.len() as u32; // unicode_width::UnicodeWidthStr::width(&string as &str) as u32;
        Notation(Rc::new(NotationInner::Text(string, width)))
    }

    /// Use the leftmost option of every choice in the contained Notation.
    /// If the contained Notation follows the recommendation of not
    /// putting newlines in the left-most options of choices, then this
    /// `flat` will be displayed all on one line.
    pub fn flat(notation: Notation) -> Notation {
        Notation(Rc::new(NotationInner::Flat(notation)))
    }

    /// Increase the indentation level of the contained notation by the
    /// given width. The indentation level determines the number of spaces
    /// put after `Newline`s. (It therefore doesn't affect the first line
    /// of a notation.)
    pub fn indent(notation: Notation) -> Notation {
        Notation(Rc::new(NotationInner::Indent(notation)))
    }

    fn braced(notation: Notation) -> Notation {
        Notation(Rc::new(NotationInner::Braced(notation)))
    }
}

impl BitAnd<Notation> for Notation {
    type Output = Notation;

    /// Display both notations. The first character of the right
    /// notation immediately follows the last character of the
    /// left notation.
    fn bitand(self, other: Notation) -> Self {
        Self(Rc::new(NotationInner::Concat(self, other)))
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;

    /// If inside a `flat`, _or_ the first line of the left notation
    /// fits within the required width, then display the left
    /// notation. Otherwise, display the right notation.
    fn bitor(self, other: Notation) -> Notation {
        Notation(Rc::new(NotationInner::Choice(self, other)))
    }
}

#[derive(Debug, Clone, Copy)]
struct Chunk<'a> {
    notation: &'a Notation,
    indent: u32,
    flat: bool,
}

impl<'a> Chunk<'a> {
    fn with_notation(self, notation: &'a Notation) -> Chunk<'a> {
        let mut ret = self.clone();
        ret.notation = notation;
        return ret;
    }

    fn indented(self, indent: u32) -> Chunk<'a> {
        let mut ret = self.clone();
        ret.indent += indent;
        return ret;
    }

    fn flat(self) -> Chunk<'a> {
        let mut ret = self.clone();
        ret.flat = true;
        return ret;
    }
}

struct PrettyPrinter<'a> {
    /// Maximum line width that we'll try to stay within
    width: u32,
    /// Current column position
    col: u32,
    /// A stack of chunks to print. The _top_ of the stack is the
    /// _end_ of the vector, which represents the _earliest_ part
    /// of the document to print.
    chunks: Vec<Chunk<'a>>,

    indent_unit: u32,
}

impl<'a> PrettyPrinter<'a> {
    fn new(notation: &'a Notation, width: u32) -> PrettyPrinter<'a> {
        let chunk = Chunk {
            notation,
            indent: 0,
            flat: false,
        };
        PrettyPrinter {
            width,
            col: 0,
            chunks: vec![chunk],
            indent_unit: 2,
        }
    }

    fn fits(&self, chunk: Chunk<'a>) -> bool {
        use NotationInner::*;

        let mut remaining = if self.col <= self.width {
            self.width - self.col
        } else {
            return false;
        };
        let mut stack = vec![chunk];
        let mut chunks = &self.chunks as &[Chunk];

        loop {
            let chunk = match stack.pop() {
                Some(chunk) => chunk,
                None => match chunks.split_last() {
                    None => return true,
                    Some((chunk, more_chunks)) => {
                        chunks = more_chunks;
                        *chunk
                    }
                },
            };

            match chunk.notation.0.as_ref() {
                Newline => return true,
                Text(_text, text_width) => {
                    if *text_width <= remaining {
                        remaining -= *text_width;
                    } else {
                        return false;
                    }
                }
                Flat(x) => stack.push(chunk.with_notation(x).flat()),
                Indent(x) => stack.push(chunk.with_notation(x).indented(self.indent_unit)),
                Concat(x, y) => {
                    stack.push(chunk.with_notation(y));
                    stack.push(chunk.with_notation(x));
                }
                Braced(_x) => {
                    if 2 <= remaining {
                        remaining -= 2;
                    } else {
                        return false;
                    }
                }
                Choice(x, y) => {
                    if chunk.flat {
                        stack.push(chunk.with_notation(x));
                    } else {
                        // Relies on the rule that for every choice
                        // `x | y`, the first line of `y` is no longer
                        // than the first line of `x`.
                        stack.push(chunk.with_notation(y));
                    }
                }
            }
        }
    }

    fn print(&mut self) -> String {
        use NotationInner::*;

        let mut output = String::new();
        while let Some(chunk) = self.chunks.pop() {
            match chunk.notation.0.as_ref() {
                Text(text, width) => {
                    output.push_str(text);
                    self.col += width;
                }
                Newline => {
                    output.push('\n');
                    for _ in 0..chunk.indent {
                        output.push(' ');
                    }
                    self.col = chunk.indent;
                }
                Flat(x) => self.chunks.push(chunk.with_notation(x).flat()),
                Indent(x) => self
                    .chunks
                    .push(chunk.with_notation(x).indented(self.indent_unit)),
                Braced(x) => self.chunks.push(chunk.with_notation(x)),

                Concat(x, y) => {
                    self.chunks.push(chunk.with_notation(y));
                    self.chunks.push(chunk.with_notation(x));
                }
                Choice(x, y) => {
                    if chunk.flat || self.fits(chunk.with_notation(x)) {
                        self.chunks.push(chunk.with_notation(x));
                    } else {
                        self.chunks.push(chunk.with_notation(y));
                    }
                }
            }
        }
        output
    }
}

#[derive(Default)]
struct NotationBuilder {
    // notation + subtree size
    note_stack: Vec<(Notation, u32)>,
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

            output.push(notation);
            current += tree_size;
        }

        return output;
    }

    pub fn build(&mut self, ast: &AstNodeVec) -> Notation {
        for node in ast {
            use AstNodeKind::*;

            let notation = match node.kind {
                ExprWord => Notation::txt("word"),
                ExprBoolean => Notation::txt("true"),

                StmtIfIntro => Notation::txt(""),
                StmtBlockIntro => Notation::txt(""),
                StmtBlock => 'end: {
                    let children = self.collect_tree(*node.subtree_size);

                    if children.len() <= 1 {
                        break 'end Notation::txt("{") & Notation::nl() & Notation::txt("}");
                    }

                    let mut note = Notation::nl();

                    for child in children {
                        note = Notation::nl() & child & note;
                    }

                    Notation::txt("{") & Notation::indent(note) & Notation::txt("}")
                }
                StmtIf => {
                    let mut children = self.collect_tree(*node.subtree_size);

                    let _if_intro = children.pop().unwrap();
                    let cond = children.pop().unwrap();
                    let if_cond = children.pop().unwrap();
                    let else_cond = children.pop();

                    dbg!(&cond);
                    dbg!(&if_cond);
                    dbg!(&else_cond);

                    let cond = Notation::txt("if (") & cond & Notation::txt(")");

                    if let Some(else_cond) = else_cond {
                        let if_cond_flat = cond.clone()
                            & Notation::txt(" ")
                            & if_cond.clone()
                            & Notation::txt(" else ")
                            & else_cond.clone();
                        let if_cond_vert = cond.clone()
                            & Notation::nl()
                            & Notation::indent(if_cond)
                            & Notation::nl()
                            & Notation::txt("else")
                            & Notation::indent(else_cond);

                        if_cond_flat | if_cond_vert
                    } else {
                        let if_cond_flat = Notation::txt(" ") & if_cond.clone();
                        let if_cond_vert = Notation::nl() & if_cond;
                        let if_cond = if_cond_flat | if_cond_vert;

                        cond & if_cond
                    }
                }

                UtilSentinel => Notation::txt(""),

                _ => unimplemented!("printing for {:?}", node.kind),
            };

            self.note_stack.push((notation, *node.subtree_size));
        }

        return self.note_stack.pop().unwrap().0;
    }
}

// This is an algorithm I came up with before reading Justin's article/Wadlin's paper.
// In theory they're close to equivalent, but Justin/Wadlin actually had code
// which implemented it, and I struggled quite a bit with actually implementing
// a first prototype.
//
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

#[cfg(test)]
mod tests {
    use super::*;

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

        let ast = parse(&tokens).expect("doesn't error");

        let mut builder = NotationBuilder::default();
        let notation = builder.build(&ast);

        let mut printer = PrettyPrinter::new(&notation, 80);

        let output = printer.print();

        assert_eq!(output, source);
    }
}
