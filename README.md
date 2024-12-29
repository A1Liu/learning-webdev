# learning-webdev
Writing a TS compiler just for practice.

## Thoughts
- Architecture
  - Using a post-order data-oriented AST, instead of a node-based tree structure
  - Doing everything in a linear chain of passes, instead of the
    on-demand-and-cache query-compiler approach
  - Avoiding recursion.

## Resources
- Spec - https://tc39.es/ecma262 ; annoying to parse but can be useful
- JS Test suite - https://github.com/tc39/test262
- Typescript test cases - https://github.com/microsoft/TypeScript/blob/main/tests
- Compiler architecture theory
  - Chandler Carruth, Carbon's design - https://www.youtube.com/watch?v=ZI198eFghJk
  - Aaron Hsu, Data-parallel compiler - https://scholarworks.iu.edu/dspace/items/3ab772c9-92c9-4f59-bd95-40aff99e8c7a
  - Andrew Kelley, Zig's re-design - https://vimeo.com/649009599
  - Max Brunsfeld, Tree-sitter (just the parser branching parts) - https://www.youtube.com/watch?v=Jes3bD6P0To
- Compiler architecture practice
  - state-machine over recursion for the parser, and some other ideas - https://github.com/carbon-language/carbon-lang/tree/trunk/toolchain/parse
  - Some of the most important stuff regarding traversal (see `TreeAndSubtrees` and the field `subtree_sizes_`) - https://github.com/carbon-language/carbon-lang/blob/trunk/toolchain/parse/tree_and_subtrees.h
  - State machine in action, else blocks (see `HandleStatementIfThenBlockFinish`) - https://github.com/carbon-language/carbon-lang/blob/trunk/toolchain/parse/handle_statement.cpp
- Formatting
  - A bag-of-heuristics approach - https://journal.stuffwithstuff.com/2015/09/08/the-hardest-program-ive-ever-written/
  - Wadler, Paper on Prettier - https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf
  - Imperative version of Wadler - https://justinpombrio.net/2024/02/23/a-twist-on-Wadlers-printer.html

## Testing
- E.g. `cargo test lexer::tests::lex_easy_test_easy_templates_ts -- --nocapture`
- Thought - LLM test case generation may actually work pretty OK here
