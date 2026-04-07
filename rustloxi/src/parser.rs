use std::mem::{Discriminant, discriminant};

use crate::{
    ast::{Expr, ExprInfo, Stmt},
    errors::{InterpreterError, Result},
    scanner::models::{Token, TokenInfo},
    state::VariableValue,
};

pub fn parse(tokens: &[TokenInfo]) -> Option<Vec<Stmt>> {
    let mut had_errors = false;

    let mut statements = Vec::new();
    let mut pos = 0;
    while !is_at_end(pos, tokens) {
        let result = declaration(pos, tokens);

        match result {
            Ok(res) => {
                pos = res.0;
                statements.push(res.1);
            }
            Err(
                e @ InterpreterError::Parser {
                    line: _,
                    pos: err_pos,
                    location: _,
                    message: _,
                },
            ) => {
                had_errors = true;
                eprintln!("{e}");

                pos = synchronize(err_pos, tokens);
            }
            _ => unreachable!(),
        };
    }

    if had_errors { None } else { Some(statements) }
}

fn declaration(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::Var)]);

    if has_advanced {
        var_declaration(pos, tokens)
    } else {
        statement(pos, tokens)
    }
}

fn var_declaration(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let (pos, name) = consume(
        pos,
        tokens,
        // discriminant ignores values
        discriminant(&Token::Identifier {
            lexeme: String::new(),
        }),
        String::from("Expect variable name."),
    )?;
    let name = name.token.lexeme().to_string();

    let (mut pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::Equal)]);

    let var_stmt = if has_advanced {
        let expr_result = expression(pos, tokens)?;
        pos = expr_result.0;

        Stmt::Var {
            name,
            initializer: Some(expr_result.1),
        }
    } else {
        Stmt::Var {
            name,
            initializer: None,
        }
    };

    pos = consume(
        pos,
        tokens,
        discriminant(&Token::Semicolon),
        String::from("Expect ';' after variable declaration."),
    )?
    .0;

    Ok((pos, var_stmt))
}

fn statement(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::Print)]);
    if has_advanced {
        return print_statement(pos, tokens);
    };

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::LeftBrace)]);
    if has_advanced {
        return block_statement(pos, tokens);
    };

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::If)]);
    if has_advanced {
        return if_statement(pos, tokens);
    };

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::While)]);
    if has_advanced {
        return while_statement(pos, tokens);
    };

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::For)]);
    if has_advanced {
        return for_statement(pos, tokens);
    };

    expr_statement(pos, tokens)
}

fn print_statement(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let (pos, expr) = expression(pos, tokens)?;

    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::Semicolon),
        String::from("Expect ';' after value."),
    )?
    .0;

    Ok((pos, Stmt::Print { expr }))
}

fn block_statement(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let mut statements = Vec::new();
    let mut pos = pos;

    while !check(pos, tokens, discriminant(&Token::RightBrace)) && !is_at_end(pos, tokens) {
        let stmt_result = declaration(pos, tokens)?;
        pos = stmt_result.0;

        statements.push(stmt_result.1);
    }

    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::RightBrace),
        String::from("Expect '}' after block."),
    )?
    .0;

    Ok((pos, Stmt::Block { statements }))
}

fn if_statement(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::LeftParen),
        String::from("Expect '(' after 'if'."),
    )?
    .0;

    let (pos, cond) = expression(pos, tokens)?;

    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::RightParen),
        String::from("Expect ')' after if condition."),
    )?
    .0;

    let (pos, then_branch) = statement(pos, tokens)?;

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::Else)]);

    let (pos, else_branch) = if has_advanced {
        statement(pos, tokens).map(|r| (r.0, Some(r.1)))?
    } else {
        (pos, None)
    };

    Ok((
        pos,
        Stmt::If {
            condition: cond,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        },
    ))
}

fn while_statement(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::LeftParen),
        String::from("Expect '(' after 'while'."),
    )?
    .0;

    let (pos, expr) = expression(pos, tokens)?;

    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::RightParen),
        String::from("Expect ')' after while condition."),
    )?
    .0;

    let (pos, stmt) = statement(pos, tokens)?;

    Ok((
        pos,
        Stmt::While {
            condition: expr,
            stmt: Box::new(stmt),
        },
    ))
}

fn for_statement(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::LeftParen),
        String::from("Expect '(' after 'for'."),
    )?
    .0;

    let initializer: Option<Stmt>;

    let (mut pos, has_advanced) =
        advance_on_match(pos, tokens, vec![discriminant(&Token::Semicolon)]);
    if has_advanced {
        initializer = None;
    } else {
        let (inner_pos, has_advanced) =
            advance_on_match(pos, tokens, vec![discriminant(&Token::Var)]);
        if has_advanced {
            let result = var_declaration(inner_pos, tokens)?;

            pos = result.0;
            initializer = Some(result.1);
        } else {
            let result = expr_statement(inner_pos, tokens)?;

            pos = result.0;
            initializer = Some(result.1);
        }
    }

    let (pos, condition) = if !check(pos, tokens, discriminant(&Token::Semicolon)) {
        expression(pos, tokens).map(|e| (e.0, Some(e.1)))?
    } else {
        (pos, None)
    };
    let condition_line = peek(pos, tokens).line;

    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::Semicolon),
        String::from("Expect ';' after loop condition."),
    )?
    .0;

    let (pos, increment) = if !check(pos, tokens, discriminant(&Token::RightParen)) {
        expression(pos, tokens).map(|e| (e.0, Some(e.1)))?
    } else {
        (pos, None)
    };

    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::RightParen),
        String::from("Expect ')' after for clauses."),
    )?
    .0;

    let (pos, mut body) = statement(pos, tokens)?;

    // Desugaring For Loop to While Loop
    if let Some(incr) = increment {
        body = Stmt::Block {
            statements: vec![body, Stmt::Expr { expr: incr }],
        }
    }

    let condition = condition.unwrap_or(ExprInfo::new(
        Expr::Literal {
            value: VariableValue::Boolean { value: true },
        },
        condition_line,
    ));

    body = Stmt::While {
        condition,
        stmt: Box::new(body),
    };

    if let Some(init) = initializer {
        body = Stmt::Block {
            statements: vec![init, body],
        }
    }

    Ok((pos, body))
}

fn expr_statement(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Stmt)> {
    let (pos, expr) = expression(pos, tokens)?;

    let pos = consume(
        pos,
        tokens,
        discriminant(&Token::Semicolon),
        String::from("Expect ';' after expression."),
    )?
    .0;

    Ok((pos, Stmt::Expr { expr }))
}

fn expression(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    assignment(pos, tokens)
}

fn assignment(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (pos, expr) = logic_or(pos, tokens)?;

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::Equal)]);

    if has_advanced {
        let equals_token = previous(pos, tokens);
        let (pos, value) = assignment(pos, tokens)?;

        match *expr.expr {
            Expr::Variable { name } => Ok((pos, ExprInfo::assignment(name, expr.line, value))),
            _ => Err(InterpreterError::parser_error(
                pos,
                equals_token,
                String::from("Invalid assignment target."),
            )),
        }
    } else {
        Ok((pos, expr))
    }
}

fn logic_or(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (mut pos, mut expr) = logic_and(pos, tokens)?;

    loop {
        let advance_result = advance_on_match(pos, tokens, vec![discriminant(&Token::Or)]);

        pos = advance_result.0;
        let has_advanced = advance_result.1;

        if has_advanced {
            let operator = previous(pos, tokens);
            let comparison_result = logic_and(pos, tokens)?;
            pos = comparison_result.0;
            let right = comparison_result.1;
            expr = ExprInfo::binary(pos, expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn logic_and(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (mut pos, mut expr) = equality(pos, tokens)?;

    loop {
        let advance_result = advance_on_match(pos, tokens, vec![discriminant(&Token::And)]);

        pos = advance_result.0;
        let has_advanced = advance_result.1;

        if has_advanced {
            let operator = previous(pos, tokens);
            let comparison_result = equality(pos, tokens)?;
            pos = comparison_result.0;
            let right = comparison_result.1;
            expr = ExprInfo::binary(pos, expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn equality(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (mut pos, mut expr) = comparison(pos, tokens)?;

    loop {
        let advance_result = advance_on_match(
            pos,
            tokens,
            vec![
                discriminant(&Token::BangEqual),
                discriminant(&Token::EqualEqual),
            ],
        );

        pos = advance_result.0;
        let has_advanced = advance_result.1;

        if has_advanced {
            let operator = previous(pos, tokens);
            let comparison_result = comparison(pos, tokens)?;
            pos = comparison_result.0;
            let right = comparison_result.1;
            expr = ExprInfo::binary(pos, expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn comparison(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (mut pos, mut expr) = term(pos, tokens)?;

    loop {
        let advance_result = advance_on_match(
            pos,
            tokens,
            vec![
                discriminant(&Token::Greater),
                discriminant(&Token::GreaterEqual),
                discriminant(&Token::Less),
                discriminant(&Token::LessEqual),
            ],
        );

        pos = advance_result.0;
        let has_advanced = advance_result.1;

        if has_advanced {
            let operator = previous(pos, tokens);
            let comparison_result = comparison(pos, tokens)?;
            pos = comparison_result.0;
            let right = comparison_result.1;
            expr = ExprInfo::binary(pos, expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn term(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (mut pos, mut expr) = factor(pos, tokens)?;

    loop {
        let advance_result = advance_on_match(
            pos,
            tokens,
            vec![discriminant(&Token::Minus), discriminant(&Token::Plus)],
        );

        pos = advance_result.0;
        let has_advanced = advance_result.1;

        if has_advanced {
            let operator = previous(pos, tokens);
            let comparison_result = comparison(pos, tokens)?;
            pos = comparison_result.0;
            let right = comparison_result.1;
            expr = ExprInfo::binary(pos, expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn factor(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (mut pos, mut expr) = unary(pos, tokens)?;

    loop {
        let advance_result = advance_on_match(
            pos,
            tokens,
            vec![discriminant(&Token::Star), discriminant(&Token::Slash)],
        );

        pos = advance_result.0;
        let has_advanced = advance_result.1;

        if has_advanced {
            let operator = previous(pos, tokens);
            let comparison_result = comparison(pos, tokens)?;
            pos = comparison_result.0;
            let right = comparison_result.1;
            expr = ExprInfo::binary(pos, expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn unary(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (pos, has_advanced) = advance_on_match(
        pos,
        tokens,
        vec![discriminant(&Token::Bang), discriminant(&Token::Minus)],
    );

    if has_advanced {
        let operator = previous(pos, tokens);
        let (pos, arg) = unary(pos, tokens)?;

        ExprInfo::unary(pos, operator, arg).map(|expr| (pos, expr))
    } else {
        primary(pos, tokens)
    }
}

// TODO: refactor use one big match instead of multiple if-blocks
fn primary(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, ExprInfo)> {
    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::False)]);
    if has_advanced {
        return Ok((
            pos,
            ExprInfo::new(
                Expr::Literal {
                    value: VariableValue::Boolean { value: false },
                },
                peek(pos, tokens).line,
            ),
        ));
    }

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::True)]);
    if has_advanced {
        return Ok((
            pos,
            ExprInfo::new(
                Expr::Literal {
                    value: VariableValue::Boolean { value: true },
                },
                peek(pos, tokens).line,
            ),
        ));
    }

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::Nil)]);
    if has_advanced {
        return Ok((pos, ExprInfo::new(Expr::Nil, peek(pos, tokens).line)));
    }

    // string, number and identifier support
    match &peek(pos, tokens).token {
        Token::StrLiteral { lexeme: _, value } => {
            let pos = advance(pos, tokens).0;
            return Ok((
                pos,
                ExprInfo::new(
                    Expr::Literal {
                        value: VariableValue::Str {
                            value: value.to_string(),
                        },
                    },
                    peek(pos, tokens).line,
                ),
            ));
        }
        Token::NumLiteral { lexeme: _, value } => {
            let pos = advance(pos, tokens).0;
            return Ok((
                pos,
                ExprInfo::new(
                    Expr::Literal {
                        value: VariableValue::Num { value: *value },
                    },
                    peek(pos, tokens).line,
                ),
            ));
        }
        Token::Identifier { lexeme } => {
            let pos = advance(pos, tokens).0;
            return Ok((
                pos,
                ExprInfo::new(
                    Expr::Variable {
                        name: String::from(lexeme),
                    },
                    peek(pos, tokens).line,
                ),
            ));
        }
        _ => {}
    }

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::LeftParen)]);
    let grouping_start_line = peek(pos, tokens).line;
    if has_advanced {
        let (pos, expr) = expression(pos, tokens)?;

        let pos = consume(
            pos,
            tokens,
            discriminant(&Token::RightParen),
            String::from("Expect ')' after expression."),
        )?
        .0;

        return Ok((
            pos,
            ExprInfo::new(Expr::Grouping { expr }, grouping_start_line),
        ));
    }

    Err(InterpreterError::parser_error(
        pos,
        peek(pos, tokens),
        String::from("Expect expression."),
    ))
}

fn consume(
    pos: usize,
    tokens: &[TokenInfo],
    tpe: Discriminant<Token>,
    error_message: String,
) -> Result<(usize, &TokenInfo)> {
    if check(pos, tokens, tpe) {
        Ok(advance(pos, tokens))
    } else {
        Err(InterpreterError::parser_error(
            pos,
            peek(pos, tokens),
            error_message,
        ))
    }
}

fn advance_on_match(
    pos: usize,
    tokens: &[TokenInfo],
    types: Vec<Discriminant<Token>>,
) -> (usize, bool) {
    let mut pos = pos;
    for tpe in types {
        if check(pos, tokens, tpe) {
            pos = advance(pos, tokens).0;
            return (pos, true);
        }
    }

    (pos, false)
}

fn check(pos: usize, tokens: &[TokenInfo], tpe: Discriminant<Token>) -> bool {
    if is_at_end(pos, tokens) {
        false
    } else {
        discriminant(&peek(pos, tokens).token) == tpe
    }
}

fn advance(pos: usize, tokens: &[TokenInfo]) -> (usize, &TokenInfo) {
    let pos = if !is_at_end(pos, tokens) {
        pos + 1
    } else {
        pos
    };
    (pos, previous(pos, tokens))
}

fn is_at_end(pos: usize, tokens: &[TokenInfo]) -> bool {
    matches!(peek(pos, tokens).token, Token::Eof)
}

fn peek(pos: usize, tokens: &[TokenInfo]) -> &TokenInfo {
    &tokens[pos]
}

fn previous(pos: usize, tokens: &[TokenInfo]) -> &TokenInfo {
    &tokens[pos - 1]
}

const STMT_END: Discriminant<Token> = discriminant(&Token::Semicolon);
const STMT_START_TOKENS: [Discriminant<Token>; 8] = [
    discriminant(&Token::Class),
    discriminant(&Token::Fun),
    discriminant(&Token::Var),
    discriminant(&Token::For),
    discriminant(&Token::If),
    discriminant(&Token::While),
    discriminant(&Token::Print),
    discriminant(&Token::Return),
];

fn synchronize(pos: usize, tokens: &[TokenInfo]) -> usize {
    let (mut pos, mut token) = advance(pos, tokens);

    while !is_at_end(pos, tokens) {
        if discriminant(&token.token) == STMT_END {
            return pos;
        }

        match discriminant(&peek(pos, tokens).token) {
            t if STMT_START_TOKENS.contains(&t) => {
                return pos;
            }
            _ => {
                (pos, token) = advance(pos, tokens);
            }
        }
    }

    pos
}
