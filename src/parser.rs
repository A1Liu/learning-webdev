use crate::ast::*;
use crate::tokens::*;

#[derive(Clone, Copy, Debug)]
struct StackState {
    start_token: u32,
}

impl StackState {
    pub fn new(token_index: usize) -> StackState {
        return StackState {
            start_token: token_index as u32,
        };
    }
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
    fn add_node(&mut self, start: u32, kind: AstNodeKind) {
        self.tree.push(AstNode {
            kind,
            subtree_size: self.index as u32 + 1 - start,
        });
    }

    fn incr(&mut self) {
        self.index += 1;
    }

    fn peek<'a>(&self, tokens: &'a TokenSlice<'a>) -> Option<TokenKind> {
        return tokens.get(self.index).map(|s| *s.kind);
    }

    fn push_proc(&mut self, proc: ParseStackFunc) {
        self.parse_stack.push(StackEntry { proc, state: None });
    }

    fn push_state(&mut self, proc: ParseStackFunc, state: StackState) {
        self.parse_stack.push(StackEntry {
            proc,
            state: Some(state),
        });
    }

    fn throw(&self, e: String) -> Result<(), String> {
        return Err(e);
    }
}

pub fn parse(tokens: &TokenVec) -> Result<(), String> {
    let mut ctx = ParseContext {
        index: 0,
        parse_stack: Vec::with_capacity(32),
        tree: AstNodeVec::new(),
    };

    ctx.parse_stack.push(StackEntry {
        proc: parse_stmt,
        state: Some(StackState::new(0)),
    });

    while let Some(StackEntry { proc, state }) = ctx.parse_stack.pop() {
        let state = state.unwrap_or(StackState::new(ctx.index));
        proc(&mut ctx, tokens.as_slice(), state)?;
    }

    return Ok(());
}

fn parse_stmt(ctx: &mut ParseContext, tokens: TokenSlice, state: StackState) -> Result<(), String> {
    let tok = match ctx.peek(&tokens) {
        None => return Ok(()),
        Some(t) => t,
    };

    match tok {
        TokenKind::Key(Key::If) => {
            ctx.incr();

            ctx.add_node(state.start_token, AstNodeKind::StmtIfIntro);

            while let Some(TokenKind::Whitespace) = ctx.peek(&tokens) {
                ctx.incr();
            }

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

            ctx.push_state(
                |ctx, _tokens, state| {
                    ctx.add_node(state.start_token, AstNodeKind::StmtIf);
                    return Ok(());
                },
                state,
            );

            // TODO: use the proper state here
            // if (a) blah();
            //        ^
            ctx.push_proc(parse_stmt);

            ctx.push_state(
                |ctx, tokens, _state| {
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
                },
                state,
            );

            return Ok(());

        }

        _ => unimplemented!(),
    }
}

fn parse_expr(ctx: &mut ParseContext, tokens: TokenSlice, state: StackState) -> Result<(), String> {
    let tok = match ctx.peek(&tokens) {
        None => return Ok(()),
        Some(t) => t,
    };

    match tok {
        TokenKind::Number => {
            ctx.add_node(state.start_token, AstNodeKind::ExprNumber);
        }

        _ => {
            unimplemented!();
        }
    }

    return Ok(());
}
