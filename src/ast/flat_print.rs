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

struct PrettyPrinter {
}
