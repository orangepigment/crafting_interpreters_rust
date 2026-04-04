use std::{
    mem::{Discriminant, discriminant},
    rc::Rc,
};

use rustloxi::VariableValue;

use crate::{
    ast::Expr,
    errors::{InterpreterError, Result},
    scanner::models::{Token, TokenInfo},
};

pub fn parse(tokens: &[TokenInfo]) -> Result<Expr> {
    expression(0, tokens).map(|r| r.1)
}

fn expression(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Expr)> {
    equality(pos, tokens)
}

fn equality(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Expr)> {
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
            expr = Expr::binary(expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn comparison(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Expr)> {
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
            expr = Expr::binary(expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn term(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Expr)> {
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
            expr = Expr::binary(expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn factor(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Expr)> {
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
            expr = Expr::binary(expr, operator, right)?;
        } else {
            break Ok((pos, expr));
        }
    }
}

fn unary(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Expr)> {
    let (pos, has_advanced) = advance_on_match(
        pos,
        tokens,
        vec![discriminant(&Token::Bang), discriminant(&Token::Minus)],
    );

    if has_advanced {
        let operator = previous(pos, tokens);
        let (pos, arg) = unary(pos, tokens)?;

        Expr::unary(operator, arg).map(|expr| (pos, expr))
    } else {
        primary(pos, tokens)
    }
}

fn primary(pos: usize, tokens: &[TokenInfo]) -> Result<(usize, Expr)> {
    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::False)]);
    if has_advanced {
        return Ok((
            pos,
            Expr::Literal {
                value: VariableValue::Boolean { value: false },
            },
        ));
    }

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::True)]);
    if has_advanced {
        return Ok((
            pos,
            Expr::Literal {
                value: VariableValue::Boolean { value: true },
            },
        ));
    }

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::Nil)]);
    if has_advanced {
        return Ok((pos, Expr::Nil));
    }

    //string and number support
    match &peek(pos, tokens).token {
        Token::StrLiteral { lexeme: _, value } => {
            let pos = advance(pos, tokens).0;
            return Ok((
                pos,
                Expr::Literal {
                    value: VariableValue::Str {
                        value: value.to_string(),
                    },
                },
            ));
        }
        Token::NumLiteral { lexeme: _, value } => {
            let pos = advance(pos, tokens).0;
            return Ok((
                pos,
                Expr::Literal {
                    value: VariableValue::Num { value: *value },
                },
            ));
        }
        _ => {}
    }

    let (pos, has_advanced) = advance_on_match(pos, tokens, vec![discriminant(&Token::LeftParen)]);
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
            Expr::Grouping {
                expr: Rc::new(expr),
            },
        ));
    }

    Err(InterpreterError::parser_error(
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
