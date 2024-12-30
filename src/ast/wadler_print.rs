use std::ops::*;
use std::rc::Rc;

pub struct NoteDone;
pub const NOTE: NoteBuilder = NoteBuilder::Empty;
pub const EMPTY: NoteBuilder = NoteBuilder::Empty;
pub const DONE: NoteDone = NoteDone;
pub const NL: NotationInner = NotationInner::Newline;

pub enum NoteBuilder {
    Empty,
    Note(Notation),
}

impl BitAnd<&'static str> for NoteBuilder {
    type Output = NoteBuilder;

    /// Display both notations. The first character of the right
    /// notation immediately follows the last character of the
    /// left notation.
    fn bitand(self, other: &'static str) -> Self::Output {
        let note = Notation::txt(other);
        return self & note;
    }
}

impl BitAnd<NoteBuilder> for NoteBuilder {
    type Output = NoteBuilder;

    fn bitand(self, other: NoteBuilder) -> Self::Output {
        match other {
            Self::Empty => self,
            Self::Note(other) => self & other,
        }
    }
}

impl BitAnd<NotationInner> for NoteBuilder {
    type Output = NoteBuilder;

    fn bitand(self, other: NotationInner) -> Self::Output {
        return self & Notation(Rc::new(other));
    }
}

impl BitAnd<Notation> for NoteBuilder {
    type Output = NoteBuilder;

    fn bitand(self, other: Notation) -> Self::Output {
        match self {
            Self::Empty => Self::Note(other),
            Self::Note(note) => Self::Note(Notation(Rc::new(NotationInner::Concat(note, other)))),
        }
    }
}

impl BitAnd<&Notation> for NoteBuilder {
    type Output = NoteBuilder;

    fn bitand(self, other: &Notation) -> Self::Output {
        return self & other.clone();
    }
}

impl BitAnd<NoteDone> for NoteBuilder {
    type Output = Notation;

    /// Display both notations. The first character of the right
    /// notation immediately follows the last character of the
    /// left notation.
    fn bitand(self, _other: NoteDone) -> Self::Output {
        match self {
            Self::Empty => panic!("failed"),
            Self::Note(note) => note,
        }
    }
}

impl Neg for NoteBuilder {
    type Output = NoteBuilder;

    /// Increase the indentation level of the contained notation by the
    /// given width. The indentation level determines the number of spaces
    /// put after `Newline`s. (It therefore doesn't affect the first line
    /// of a notation.)
    fn neg(self) -> Self::Output {
        let Self::Note(note) = self else {
            panic!("failed");
        };

        Self::Note(Notation(Rc::new(NotationInner::Indent(note))))
    }
}

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
    Indent(Notation),
    Concat(Notation, Notation),
    Choice(Notation, Notation),

    /// Use the leftmost option of every choice in the contained Notation.
    /// If the contained Notation follows the recommendation of not
    /// putting newlines in the left-most options of choices, then this
    /// `flat` will be displayed all on one line.
    Flat(Notation),
}

impl Notation {
    /// Display text exactly as-is. The text should not contain a newline!
    pub fn txt(s: impl ToString) -> Notation {
        let string = s.to_string();
        let width = string.len() as u32; // unicode_width::UnicodeWidthStr::width(&string as &str) as u32;
        Notation(Rc::new(NotationInner::Text(string, width)))
    }
}

impl Neg for &Notation {
    type Output = Notation;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

impl Neg for Notation {
    type Output = Notation;

    /// Display both notations. The first character of the right
    /// notation immediately follows the last character of the
    /// left notation.
    fn neg(self) -> Self::Output {
        Self(Rc::new(NotationInner::Indent(self)))
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
pub struct Chunk<'a> {
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

pub struct WadlerPrinter<'a> {
    /// Maximum line width that we'll try to stay within
    width: u32,
    /// Current column position
    col: u32,
    /// A stack of chunks to print. The _top_ of the stack is the
    /// _end_ of the vector, which represents the _earliest_ part
    /// of the document to print.
    chunks: Vec<Chunk<'a>>,

    indent_unit: u32,

    needs_indent: bool,
}

impl<'a> WadlerPrinter<'a> {
    pub fn new(notation: &'a Notation, width: u32) -> WadlerPrinter<'a> {
        let chunk = Chunk {
            notation,
            indent: 0,
            flat: false,
        };
        WadlerPrinter {
            width,
            col: 0,
            chunks: vec![chunk],
            indent_unit: 2,
            needs_indent: false,
        }
    }

    pub fn fits(&self, chunk: Chunk<'a>) -> bool {
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

    pub fn print(&mut self) -> String {
        use NotationInner::*;

        let mut output = String::new();
        while let Some(chunk) = self.chunks.pop() {
            match chunk.notation.0.as_ref() {
                Text(text, width) => {
                    if self.needs_indent {
                        for _ in 0..chunk.indent {
                            output.push(' ');
                        }
                        self.col = chunk.indent;
                        self.needs_indent = false;
                    }

                    output.push_str(text);
                    self.col += width;
                }
                Newline => {
                    output.push('\n');
                    self.needs_indent = true;
                }
                Flat(x) => self.chunks.push(chunk.with_notation(x).flat()),
                Indent(x) => self
                    .chunks
                    .push(chunk.with_notation(x).indented(self.indent_unit)),

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
