use std::mem::{Discriminant, discriminant};

use crate::{
    ast::{Expr, ExprInfo, Stmt},
    errors::{InterpreterError, Result},
    runtime::VariableValue,
    scanner::models::{Token, TokenInfo},
};

pub struct Parser {
    pos: usize,
    had_errors: bool,
    expr_node_id: usize,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            pos: 0,
            had_errors: false,
            expr_node_id: 0,
        }
    }

    pub fn parse(&mut self, tokens: &[TokenInfo]) -> Option<Vec<Stmt>> {
        self.pos = 0;
        self.had_errors = false;

        let mut statements = Vec::new();
        while !self.is_at_end(tokens) {
            let result = self.declaration(tokens);

            match result {
                Ok(res) => {
                    statements.push(res);
                }
                Err(e @ InterpreterError::Parser { .. }) => {
                    self.had_errors = true;
                    eprintln!("{e}");

                    self.synchronize(tokens);
                }
                _ => unreachable!(),
            };
        }

        if self.had_errors {
            None
        } else {
            Some(statements)
        }
    }

    fn declaration(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Var)]);
        if has_advanced {
            return self.var_declaration(tokens);
        }

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Fun)]);
        if has_advanced {
            return self.function(tokens, "function");
        }

        self.statement(tokens)
    }

    fn var_declaration(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        let name_token = self.consume(
            tokens,
            // discriminant ignores values
            discriminant(&Token::Identifier {
                lexeme: String::new(),
            }),
            String::from("Expect variable name."),
        )?;
        let name = name_token.token.lexeme().to_string();

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Equal)]);

        let var_stmt = if has_advanced {
            let expr_result = self.expression(tokens)?;

            Stmt::Var {
                line: name_token.line,
                name,
                initializer: Some(expr_result),
            }
        } else {
            Stmt::Var {
                line: name_token.line,
                name,
                initializer: None,
            }
        };

        self.consume(
            tokens,
            discriminant(&Token::Semicolon),
            String::from("Expect ';' after variable declaration."),
        )?;

        Ok(var_stmt)
    }

    fn function(&mut self, tokens: &[TokenInfo], kind: &str) -> Result<Stmt> {
        let name = self.consume(
            tokens,
            discriminant(&Token::Identifier {
                lexeme: String::new(),
            }),
            format!("Expect {kind} name"),
        )?;

        self.consume(
            tokens,
            discriminant(&Token::LeftParen),
            format!("Expect '(' after {kind} name"),
        )?;

        let mut params = vec![];

        if !self.check(tokens, discriminant(&Token::RightParen)) {
            loop {
                if params.len() >= 255 {
                    let warning = InterpreterError::parser_error(
                        self.pos,
                        self.peek(tokens),
                        String::from("Can't have more than 255 parameters."),
                    );
                    eprint!("{warning}");
                    self.had_errors = true;
                }

                let param = self.consume(
                    tokens,
                    discriminant(&Token::Identifier {
                        lexeme: String::new(),
                    }),
                    "Expect parameter name".to_string(),
                )?;
                params.push(param);

                let advance_result =
                    self.advance_on_match(tokens, vec![discriminant(&Token::Comma)]);

                let has_advanced = advance_result;
                if !has_advanced {
                    break;
                }
            }
        }

        self.consume(
            tokens,
            discriminant(&Token::RightParen),
            "Expect ')' after parameters.".to_string(),
        )?;

        self.consume(
            tokens,
            discriminant(&Token::LeftBrace),
            format!("Expect '{{' before {kind} body."),
        )?;

        let body = self.block(tokens)?;

        let params = params
            .iter()
            .map(|p| (p.line, p.token.lexeme().to_string()))
            .collect();

        Ok(Stmt::Function {
            line: name.line,
            name: name.token.lexeme().to_string(),
            params,
            body,
        })
    }

    fn statement(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Print)]);
        if has_advanced {
            return self.print_statement(tokens);
        };

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::LeftBrace)]);
        if has_advanced {
            return self.block(tokens).map(|b| Stmt::Block { statements: b });
        };

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::If)]);
        if has_advanced {
            return self.if_statement(tokens);
        };

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::While)]);
        if has_advanced {
            return self.while_statement(tokens);
        };

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::For)]);
        if has_advanced {
            return self.for_statement(tokens);
        };

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Return)]);
        if has_advanced {
            return self.return_statement(tokens);
        };

        self.expr_statement(tokens)
    }

    fn print_statement(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        let expr = self.expression(tokens)?;

        self.consume(
            tokens,
            discriminant(&Token::Semicolon),
            String::from("Expect ';' after value."),
        )?;

        Ok(Stmt::Print { expr })
    }

    fn block(&mut self, tokens: &[TokenInfo]) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(tokens, discriminant(&Token::RightBrace)) && !self.is_at_end(tokens) {
            let stmt_result = self.declaration(tokens)?;

            statements.push(stmt_result);
        }

        self.consume(
            tokens,
            discriminant(&Token::RightBrace),
            String::from("Expect '}' after block."),
        )?;

        Ok(statements)
    }

    fn if_statement(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        self.consume(
            tokens,
            discriminant(&Token::LeftParen),
            String::from("Expect '(' after 'if'."),
        )?;

        let cond = self.expression(tokens)?;

        self.consume(
            tokens,
            discriminant(&Token::RightParen),
            String::from("Expect ')' after if condition."),
        )?;

        let then_branch = self.statement(tokens)?;

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Else)]);

        let else_branch = if has_advanced {
            self.statement(tokens).map(Some)?
        } else {
            None
        };

        Ok(Stmt::If {
            condition: cond,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn while_statement(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        self.consume(
            tokens,
            discriminant(&Token::LeftParen),
            String::from("Expect '(' after 'while'."),
        )?;

        let expr = self.expression(tokens)?;

        self.consume(
            tokens,
            discriminant(&Token::RightParen),
            String::from("Expect ')' after while condition."),
        )?;

        let stmt = self.statement(tokens)?;

        Ok(Stmt::While {
            condition: expr,
            stmt: Box::new(stmt),
        })
    }

    fn for_statement(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        self.consume(
            tokens,
            discriminant(&Token::LeftParen),
            String::from("Expect '(' after 'for'."),
        )?;

        let initializer: Option<Stmt>;

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Semicolon)]);
        if has_advanced {
            initializer = None;
        } else {
            let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Var)]);
            if has_advanced {
                let stmt = self.var_declaration(tokens)?;

                initializer = Some(stmt);
            } else {
                let stmt = self.expr_statement(tokens)?;

                initializer = Some(stmt);
            }
        }

        let condition = if !self.check(tokens, discriminant(&Token::Semicolon)) {
            self.expression(tokens).map(Some)?
        } else {
            None
        };
        let condition_line = self.peek(tokens).line;

        self.consume(
            tokens,
            discriminant(&Token::Semicolon),
            String::from("Expect ';' after loop condition."),
        )?;

        let increment = if !self.check(tokens, discriminant(&Token::RightParen)) {
            self.expression(tokens).map(Some)?
        } else {
            None
        };

        self.consume(
            tokens,
            discriminant(&Token::RightParen),
            String::from("Expect ')' after for clauses."),
        )?;

        let mut body = self.statement(tokens)?;

        // Desugaring For Loop to While Loop
        if let Some(incr) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expr { expr: incr }],
            }
        }

        let condition = condition.unwrap_or(ExprInfo::new(
            self.next_expr_id(),
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

        Ok(body)
    }

    fn return_statement(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        let line = self.previous(tokens).line;

        let value = if !self.check(tokens, discriminant(&Token::Semicolon)) {
            let expr_result = self.expression(tokens)?;
            Some(expr_result)
        } else {
            None
        };

        self.consume(
            tokens,
            discriminant(&Token::Semicolon),
            String::from("Expect ';' after return value."),
        )?;

        Ok(Stmt::Return { line, value })
    }

    fn expr_statement(&mut self, tokens: &[TokenInfo]) -> Result<Stmt> {
        let expr = self.expression(tokens)?;

        self.consume(
            tokens,
            discriminant(&Token::Semicolon),
            String::from("Expect ';' after expression."),
        )?;

        Ok(Stmt::Expr { expr })
    }

    fn expression(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        self.assignment(tokens)
    }

    fn assignment(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let expr = self.logic_or(tokens)?;

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Equal)]);

        if has_advanced {
            let equals_token = self.previous(tokens);
            let value = self.assignment(tokens)?;

            match *expr.expr {
                Expr::Variable { name } => Ok(ExprInfo::assignment(
                    self.next_expr_id(),
                    name,
                    expr.line,
                    value,
                )),
                _ => Err(InterpreterError::parser_error(
                    self.pos,
                    equals_token,
                    String::from("Invalid assignment target."),
                )),
            }
        } else {
            Ok(expr)
        }
    }

    fn logic_or(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let mut expr = self.logic_and(tokens)?;

        loop {
            let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Or)]);

            if has_advanced {
                let operator = self.previous(tokens);
                let comparison_result = self.logic_and(tokens)?;
                let right = comparison_result;
                expr = ExprInfo::binary(self.pos, self.next_expr_id(), expr, operator, right)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn logic_and(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let mut expr = self.equality(tokens)?;

        loop {
            let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::And)]);

            if has_advanced {
                let operator = self.previous(tokens);
                let comparison_result = self.equality(tokens)?;
                let right = comparison_result;
                expr = ExprInfo::binary(self.pos, self.next_expr_id(), expr, operator, right)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn equality(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let mut expr = self.comparison(tokens)?;

        loop {
            let has_advanced = self.advance_on_match(
                tokens,
                vec![
                    discriminant(&Token::BangEqual),
                    discriminant(&Token::EqualEqual),
                ],
            );

            if has_advanced {
                let operator = self.previous(tokens);
                let comparison_result = self.comparison(tokens)?;
                let right = comparison_result;
                expr = ExprInfo::binary(self.pos, self.next_expr_id(), expr, operator, right)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn comparison(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let mut expr = self.term(tokens)?;

        loop {
            let has_advanced = self.advance_on_match(
                tokens,
                vec![
                    discriminant(&Token::Greater),
                    discriminant(&Token::GreaterEqual),
                    discriminant(&Token::Less),
                    discriminant(&Token::LessEqual),
                ],
            );

            if has_advanced {
                let operator = self.previous(tokens);
                let comparison_result = self.comparison(tokens)?;
                let right = comparison_result;
                expr = ExprInfo::binary(self.pos, self.next_expr_id(), expr, operator, right)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn term(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let mut expr = self.factor(tokens)?;

        loop {
            let has_advanced = self.advance_on_match(
                tokens,
                vec![discriminant(&Token::Minus), discriminant(&Token::Plus)],
            );

            if has_advanced {
                let operator = self.previous(tokens);
                let comparison_result = self.comparison(tokens)?;
                let right = comparison_result;
                expr = ExprInfo::binary(self.pos, self.next_expr_id(), expr, operator, right)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn factor(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let mut expr = self.unary(tokens)?;

        loop {
            let has_advanced = self.advance_on_match(
                tokens,
                vec![discriminant(&Token::Star), discriminant(&Token::Slash)],
            );

            if has_advanced {
                let operator = self.previous(tokens);
                let comparison_result = self.comparison(tokens)?;
                let right = comparison_result;
                expr = ExprInfo::binary(self.pos, self.next_expr_id(), expr, operator, right)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn unary(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let has_advanced = self.advance_on_match(
            tokens,
            vec![discriminant(&Token::Bang), discriminant(&Token::Minus)],
        );

        if has_advanced {
            let operator = self.previous(tokens);
            let op_pos = self.pos;
            let arg = self.unary(tokens)?;

            ExprInfo::unary(op_pos, self.next_expr_id(), operator, arg)
        } else {
            self.call(tokens)
        }
    }

    fn call(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let mut expr = self.primary(tokens)?;

        loop {
            let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::LeftParen)]);

            if has_advanced {
                expr = self.finish_call(tokens, expr)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn finish_call(&mut self, tokens: &[TokenInfo], callee: ExprInfo) -> Result<ExprInfo> {
        let mut args = vec![];

        if !self.check(tokens, discriminant(&Token::RightParen)) {
            loop {
                if args.len() >= 255 {
                    let warning = InterpreterError::parser_error(
                        self.pos,
                        self.peek(tokens),
                        String::from("Can't have more than 255 arguments."),
                    );
                    eprint!("{warning}");
                    self.had_errors = true;
                }

                let arg = self.expression(tokens)?;
                args.push(arg);

                let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Comma)]);

                if !has_advanced {
                    break;
                }
            }
        }

        let paren = self.consume(
            tokens,
            discriminant(&Token::RightParen),
            String::from("Expect ')' after arguments."),
        )?;

        Ok(ExprInfo::call(
            paren.line,
            self.next_expr_id(),
            callee,
            args,
        ))
    }

    // TODO: refactor use one big match instead of multiple if-blocks
    fn primary(&mut self, tokens: &[TokenInfo]) -> Result<ExprInfo> {
        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::False)]);
        if has_advanced {
            return Ok(ExprInfo::new(
                self.next_expr_id(),
                Expr::Literal {
                    value: VariableValue::Boolean { value: false },
                },
                self.peek(tokens).line,
            ));
        }

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::True)]);
        if has_advanced {
            return Ok(ExprInfo::new(
                self.next_expr_id(),
                Expr::Literal {
                    value: VariableValue::Boolean { value: true },
                },
                self.peek(tokens).line,
            ));
        }

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::Nil)]);
        if has_advanced {
            return Ok(ExprInfo::new(
                self.next_expr_id(),
                Expr::Nil,
                self.peek(tokens).line,
            ));
        }

        // string, number and identifier support
        match &self.peek(tokens).token {
            Token::StrLiteral { lexeme: _, value } => {
                self.advance(tokens);
                return Ok(ExprInfo::new(
                    self.next_expr_id(),
                    Expr::Literal {
                        value: VariableValue::Str {
                            value: value.to_string(),
                        },
                    },
                    self.peek(tokens).line,
                ));
            }
            Token::NumLiteral { lexeme: _, value } => {
                self.advance(tokens);
                return Ok(ExprInfo::new(
                    self.next_expr_id(),
                    Expr::Literal {
                        value: VariableValue::Num { value: *value },
                    },
                    self.peek(tokens).line,
                ));
            }
            Token::Identifier { lexeme } => {
                self.advance(tokens);
                return Ok(ExprInfo::new(
                    self.next_expr_id(),
                    Expr::Variable {
                        name: String::from(lexeme),
                    },
                    self.peek(tokens).line,
                ));
            }
            _ => {}
        }

        let has_advanced = self.advance_on_match(tokens, vec![discriminant(&Token::LeftParen)]);
        let grouping_start_line = self.peek(tokens).line;
        if has_advanced {
            let expr = self.expression(tokens)?;

            self.consume(
                tokens,
                discriminant(&Token::RightParen),
                String::from("Expect ')' after expression."),
            )?;

            return Ok(ExprInfo::new(
                self.next_expr_id(),
                Expr::Grouping { expr },
                grouping_start_line,
            ));
        }

        Err(InterpreterError::parser_error(
            self.pos,
            self.peek(tokens),
            String::from("Expect expression."),
        ))
    }

    fn consume<'a>(
        &mut self,
        tokens: &'a [TokenInfo],
        tpe: Discriminant<Token>,
        error_message: String,
    ) -> Result<&'a TokenInfo> {
        if self.check(tokens, tpe) {
            Ok(self.advance(tokens))
        } else {
            Err(InterpreterError::parser_error(
                self.pos,
                self.peek(tokens),
                error_message,
            ))
        }
    }

    fn advance_on_match(&mut self, tokens: &[TokenInfo], types: Vec<Discriminant<Token>>) -> bool {
        for tpe in types {
            if self.check(tokens, tpe) {
                self.advance(tokens);
                return true;
            }
        }

        false
    }

    fn check(&mut self, tokens: &[TokenInfo], tpe: Discriminant<Token>) -> bool {
        if self.is_at_end(tokens) {
            false
        } else {
            discriminant(&self.peek(tokens).token) == tpe
        }
    }

    fn advance<'a>(&mut self, tokens: &'a [TokenInfo]) -> &'a TokenInfo {
        if !self.is_at_end(tokens) {
            self.pos += 1
        };

        self.previous(tokens)
    }

    fn is_at_end(&self, tokens: &[TokenInfo]) -> bool {
        matches!(self.peek(tokens).token, Token::Eof)
    }

    fn peek<'a>(&self, tokens: &'a [TokenInfo]) -> &'a TokenInfo {
        &tokens[self.pos]
    }

    fn previous<'a>(&self, tokens: &'a [TokenInfo]) -> &'a TokenInfo {
        &tokens[self.pos - 1]
    }

    // TODO: refactor
    fn synchronize(&mut self, tokens: &[TokenInfo]) {
        let mut token = self.advance(tokens);

        while !self.is_at_end(tokens) {
            if discriminant(&token.token) == STMT_END {
                return;
            }

            match discriminant(&self.peek(tokens).token) {
                t if STMT_START_TOKENS.contains(&t) => {
                    return;
                }
                _ => {
                    token = self.advance(tokens);
                }
            }
        }
    }

    fn next_expr_id(&mut self) -> usize {
        self.expr_node_id += 1;
        self.expr_node_id
    }
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
