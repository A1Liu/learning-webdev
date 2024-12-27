use super::*;
use crate::tokens::*;

#[derive(Clone, Copy, Debug)]
struct StackState {
    start_token: u32,
    start_tree_index: u32,
}

type ParseStackFunc =
    fn(ctx: &mut ParseContext, tokens: TokenSlice, state: StackState) -> Result<(), String>;

#[derive(Clone, Copy, Debug)]
struct StackEntry {
    proc: ParseStackFunc,
    state: Option<StackState>,
}

struct ParseContext {
    index: usize,
    parse_stack: Vec<StackEntry>,
    tree: AstNodeVec,
}

impl ParseContext {
    fn add_node(&mut self, state: &StackState, kind: AstNodeKind) {
        self.tree.push(AstNode {
            kind,
            subtree_size: self.tree.len() as u32 + 1 - state.start_tree_index,
        });
    }

    fn incr(&mut self) {
        self.index += 1;
    }

    fn consume_if<'a>(
        &mut self,
        tokens: &'a TokenSlice<'a>,
        kind: TokenKind,
    ) -> Option<TokenRef<'a>> {
        let token = tokens.get(self.index)?;
        if *token.kind != kind {
            return None;
        }

        self.incr();

        return Some(token);
    }

    fn consume_spaces<'a>(&mut self, tokens: &'a TokenSlice<'a>) -> usize {
        let start = self.index;
        while let Some(token) = tokens.get(self.index) {
            let token_kind = *token.kind;
            match token_kind {
                TokenKind::Whitespace | TokenKind::Comment | TokenKind::LineComment => {
                    self.incr();
                    continue;
                }
                _ => break,
            }
        }

        return self.index - start;
    }

    fn consume_ifs<'a>(
        &mut self,
        tokens: &'a TokenSlice<'a>,
        kinds: &[TokenKind],
    ) -> Option<TokenRef<'a>> {
        let token = tokens.get(self.index)?;
        let token_kind = *token.kind;
        for &kind in kinds {
            if token_kind == kind {
                self.incr();
                return Some(token);
            }
        }

        return None;
    }

    fn peek<'a>(&self, tokens: &'a TokenSlice<'a>) -> Option<TokenKind> {
        return tokens.get(self.index).map(|s| *s.kind);
    }

    fn push_proc(&mut self, proc: ParseStackFunc) {
        self.parse_stack.push(StackEntry { proc, state: None });
    }

    fn push_state(&mut self, state: StackState, proc: ParseStackFunc) {
        self.parse_stack.push(StackEntry {
            proc,
            state: Some(state),
        });
    }

    fn throw(&self, e: String) -> Result<(), String> {
        return Err(e);
    }
}

pub fn parse(tokens: &TokenVec) -> Result<AstNodeVec, String> {
    let mut ctx = ParseContext {
        index: 0,
        parse_stack: Vec::with_capacity(32),
        tree: AstNodeVec::new(),
    };

    ctx.tree.push(AstNode {
        kind: AstNodeKind::StmtSentinel,
        subtree_size: 1,
    });

    ctx.parse_stack.push(StackEntry {
        proc: parse_stmt,
        state: Some(StackState {
            start_token: 0,
            start_tree_index: 1,
        }),
    });

    while let Some(StackEntry { proc, state }) = ctx.parse_stack.pop() {
        let state = state.unwrap_or(StackState {
            start_token: ctx.index as u32,
            start_tree_index: ctx.tree.len() as u32,
        });
        proc(&mut ctx, tokens.as_slice(), state)?;
    }

    return Ok(ctx.tree);
}

fn parse_stmt(ctx: &mut ParseContext, tokens: TokenSlice, state: StackState) -> Result<(), String> {
    ctx.consume_spaces(&tokens);

    let tok = match ctx.peek(&tokens) {
        None => return Ok(()),
        Some(t) => t,
    };

    match tok {
        TokenKind::Key(Key::If) => {
            ctx.incr();
            ctx.add_node(&state, AstNodeKind::StmtIfIntro);

            while let Some(_) = ctx.consume_if(&tokens, TokenKind::Whitespace) {}

            match ctx.peek(&tokens) {
                Some(TokenKind::LParen) => {
                    ctx.incr();
                }
                _ => {
                    ctx.throw(format!("if statement missing opening parenthesis"))?;
                }
            }

            while let Some(TokenKind::Whitespace) = ctx.peek(&tokens) {
                ctx.incr();
            }

            ctx.push_state(state, |ctx, _tokens, state| {
                ctx.add_node(&state, AstNodeKind::StmtIf);
                return Ok(());
            });

            // TODO: use the proper state here
            // if (a) blah();
            //        ^
            ctx.push_proc(parse_stmt);

            ctx.push_state(state, |ctx, tokens, _state| {
                while let Some(TokenKind::Whitespace) = ctx.peek(&tokens) {
                    ctx.incr();
                }

                match ctx.peek(&tokens) {
                    Some(TokenKind::RParen) => {
                        ctx.incr();
                    }
                    _ => {
                        ctx.throw(format!("if statement missing opening parenthesis"))?;
                    }
                }

                return Ok(());
            });

            ctx.push_proc(parse_expr);

            return Ok(());
        }

        TokenKind::Semicolon => {
            ctx.incr();
            return Ok(());
        }

        TokenKind::LBrace => {
            ctx.incr();
            ctx.add_node(&state, AstNodeKind::StmtBlockIntro);

            const BLOCK_END: ParseStackFunc = |ctx, tokens, state| {
                ctx.consume_spaces(&tokens);

                match ctx.peek(&tokens) {
                    Some(TokenKind::RBrace) => {
                        ctx.incr();
                        ctx.add_node(&state, AstNodeKind::StmtBlock);
                    }
                    _ => {
                        ctx.push_state(state, BLOCK_END);
                        ctx.push_state(state, parse_stmt);
                    }
                }

                return Ok(());
            };

            ctx.push_state(state, BLOCK_END);

            return Ok(());
        }

        _ => unimplemented!("TokenKind={:?}", tok),
    }
}

fn parse_expr(ctx: &mut ParseContext, tokens: TokenSlice, state: StackState) -> Result<(), String> {
    let tok = match ctx.peek(&tokens) {
        None => return Ok(()),
        Some(t) => t,
    };

    match tok {
        TokenKind::Number => {
            ctx.incr();
            ctx.add_node(&state, AstNodeKind::ExprNumber);
        }

        TokenKind::Key(Key::True) => {
            ctx.incr();
            ctx.add_node(&state, AstNodeKind::ExprBoolean);
        }

        _ => {
            unimplemented!();
        }
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::*;
    use crate::util::*;

    #[test_resources("test/easy/conditional.*")]
    fn parse_easy(path: &str) {
        let source = std::fs::read_to_string(path).expect("Should have been able to read the file");

        let mut symbols = Symbols::new();
        let tokens = lex(&source, &mut symbols)
            .map_err(|e| e.error)
            .expect("doesn't error");

        let ast = parse(&tokens).expect("doesn't error");

        let mut output = Vec::new();
        for token in &ast {
            println!("{:?}", token.to_owned());

            output.push(format!("{:?}", token.kind));
        }

        let doc = match extract_yaml(&source) {
            None => return,
            Some(d) => d,
        };
        let expected_token_string = doc["ast"].as_str().unwrap_or("");

        let mut expected_tokens = Vec::with_capacity(output.len());
        expected_tokens.push(AstNodeKind::StmtSentinel.into());
        for token in expected_token_string.trim().split(",") {
            let token = token.trim();
            expected_tokens.push(token);
        }

        assert_eq!(&output, &expected_tokens);
    }
}
