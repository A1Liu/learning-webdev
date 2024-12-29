use super::*;
use crate::tokens::*;

#[derive(Clone, Copy, Debug)]
struct StackState {
    start_token: u32,
    start_tree_index: u32,
}

type ParseStackFunc = fn(ctx: &mut ParseContext, state: StackState) -> Result<(), String>;

#[derive(Clone, Copy, Debug)]
struct StackEntry {
    proc: ParseStackFunc,
    state: Option<StackState>,
}

struct ParseContext<'a> {
    tokens: TokenSlice<'a>,
    index: usize,
    parse_stack: Vec<StackEntry>,
    tree: AstNodeVec,
}

impl<'a> ParseContext<'a> {
    fn add_node(&mut self, state: &StackState, kind: AstNodeKind) {
        self.add_node_extra(state, kind, 0);
    }

    fn add_node_extra(&mut self, state: &StackState, kind: AstNodeKind, extra: u32) {
        self.tree.push(AstNode {
            kind,
            subtree_size: self.tree.len() as u32 + 1 - state.start_tree_index,
            extra,
        });
    }

    fn incr(&mut self) {
        self.index += 1;
    }

    fn consume_if(&mut self, kind: TokenKind) -> Option<TokenRef> {
        let token = self.tokens.get(self.index)?;
        if *token.kind != kind {
            return None;
        }

        self.index += 1;

        return Some(token);
    }

    fn consume_spaces(&mut self) -> usize {
        let start = self.index;
        while let Some(token) = self.tokens.get(self.index) {
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

    fn consume_ifs(&mut self, kinds: &[TokenKind]) -> Option<TokenRef> {
        let token = self.tokens.get(self.index)?;
        let token_kind = *token.kind;
        for &kind in kinds {
            if token_kind == kind {
                self.index += 1;
                return Some(token);
            }
        }

        return None;
    }

    fn peek(&self) -> Option<TokenKind> {
        return self.tokens.get(self.index).map(|s| *s.kind);
    }

    fn peek_ref(&self) -> Option<TokenRef> {
        return self.tokens.get(self.index);
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
}

pub fn parse(tokens: &TokenVec) -> Result<AstNodeVec, String> {
    let mut ctx = ParseContext {
        index: 0,
        tokens: tokens.as_slice(),
        parse_stack: Vec::with_capacity(32),
        tree: AstNodeVec::new(),
    };

    ctx.tree.push(AstNode {
        kind: AstNodeKind::UtilSentinel,
        subtree_size: 1,
        extra: 0,
    });

    ctx.parse_stack.push(StackEntry {
        proc: parse_stmt,
        state: Some(StackState {
            start_token: 0,
            start_tree_index: 1,
        }),
    });

    let state = StackState {
        start_token: 0,
        start_tree_index: 1,
    };

    fn parse_inf(ctx: &mut ParseContext, _: StackState) -> Result<(), String> {
        if ctx.peek().is_none() {
            return Ok(());
        }

        ctx.push_proc(parse_inf);
        ctx.push_proc(parse_stmt);

        return Ok(());
    }
    ctx.push_state(state, parse_inf);

    while let Some(StackEntry { proc, state }) = ctx.parse_stack.pop() {
        let state = state.unwrap_or(StackState {
            start_token: ctx.index as u32,
            start_tree_index: ctx.tree.len() as u32,
        });
        proc(&mut ctx, state)?;
    }

    return Ok(ctx.tree);
}

fn parse_stmt(ctx: &mut ParseContext, state: StackState) -> Result<(), String> {
    ctx.consume_spaces();

    let tok = match ctx.peek() {
        None => return Ok(()),
        Some(t) => t,
    };

    match tok {
        TokenKind::Key(Key::If) => {
            ctx.incr();
            ctx.add_node(&state, AstNodeKind::StmtIfIntro);

            ctx.consume_spaces();

            let Some(_) = ctx.consume_if(TokenKind::LParen) else {
                return Err(format!("if statement missing opening parenthesis"));
            };

            ctx.consume_spaces();

            ctx.push_state(state, |ctx, state| {
                ctx.add_node(&state, AstNodeKind::StmtIf);
                return Ok(());
            });

            ctx.push_proc(|ctx, _| {
                ctx.consume_spaces();

                let Some(_) = ctx.consume_if(TokenKind::Key(Key::Else)) else {
                    return Ok(());
                };

                ctx.push_proc(parse_stmt);

                return Ok(());
            });

            // TODO: use the proper state here
            // if (a) blah();
            //        ^
            ctx.push_proc(parse_stmt);

            ctx.push_state(state, |ctx, _state| {
                ctx.consume_spaces();

                let Some(_) = ctx.consume_if(TokenKind::RParen) else {
                    return Err(format!("if statement missing opening parenthesis"));
                };

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

            const BLOCK_END: ParseStackFunc = |ctx, state| {
                ctx.consume_spaces();

                let Some(_) = ctx.consume_if(TokenKind::RBrace) else {
                    ctx.push_state(state, BLOCK_END);
                    ctx.push_proc(parse_stmt);
                    return Ok(());
                };

                ctx.add_node(&state, AstNodeKind::StmtBlock);
                return Ok(());
            };

            ctx.push_state(state, BLOCK_END);

            return Ok(());
        }

        _ => {
            ctx.push_proc(|ctx, _state| {
                ctx.consume_spaces();
                ctx.consume_if(TokenKind::Semicolon);
                return Ok(());
            });

            return parse_expr(ctx, state);
        }
    }
}

fn parse_expr(ctx: &mut ParseContext, state: StackState) -> Result<(), String> {
    let tok = match ctx.peek_ref() {
        None => return Ok(()),
        Some(t) => t,
    };

    match tok.kind {
        TokenKind::Number => {
            ctx.incr();
            ctx.add_node(&state, AstNodeKind::ExprNumber);
        }

        TokenKind::Key(Key::True) => {
            ctx.incr();
            ctx.add_node(&state, AstNodeKind::ExprBoolean);
        }

        TokenKind::Word => {
            let extra = *tok.extra;
            ctx.incr();
            ctx.add_node_extra(&state, AstNodeKind::ExprWord, extra);
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
        expected_tokens.push(AstNodeKind::UtilSentinel.into());
        for token in expected_token_string.trim().split(",") {
            let token = token.trim();
            expected_tokens.push(token);
        }

        assert_eq!(&output, &expected_tokens);
    }
}
