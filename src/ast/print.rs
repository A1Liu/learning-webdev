use super::*;
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

        let _ast = parse(&tokens);
    }
}
