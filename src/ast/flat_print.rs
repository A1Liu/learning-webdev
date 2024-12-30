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

// To implement above, we can use a stack which encodes information in post-order.
//
// Structure is:
// vec![ group_int_b(6), group_int_a(3), atom, atom, group_a(3), atom, group_b(6) ]
//
//
// Groups mark their size, group_a has 2 elements (and its markers are 3 away from each other).
//
// You pop a group and then use a stack traversal to push information down to its children
// (indentation)
//
// When printing, traverse the AST using reverse-post-order (RPO) directly;
// Push items onto the stack. When popping, they'll be in print-order.
//
// nodes can pop, write indentation, etc.



// F3

// F1
// F2
// F3

// F1-name
// F1-params
// F1-body
// F1 end
// F2
// F3

// F1-name
// F1-params
// F1-body
// F1 end
// F2
// F3

// :/ ^ this is just the post order traversal stack.
//
// In theory I can encode all the info I need by hardcoding the different kinds
// of `Choice` nodes. I think. Merp.

enum ListContext {
    CommaList,
}

enum EntryKind {
    Atom(String),
    Group { indent: bool },
    GroupClose { indent: bool },
}

struct StackEntry {
    kind: EntryKind,
    indentation: u32,
    list_context: ListContext,
}

struct PrettyPrinter {
    indent_stack: Vec<u32>,
}
