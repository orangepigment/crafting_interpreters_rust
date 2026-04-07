pub mod models;

use crate::{
    errors::{InterpreterError, Result},
    scanner::models::{ScannerPosition, Token, TokenInfo},
};

// TODO: write tests at least for small helper methods?
pub fn scan_tokens(source: &str) -> Result<Vec<TokenInfo>> {
    let mut tokens: Vec<TokenInfo> = Vec::new();
    let source: Vec<char> = source.chars().collect();
    let input_len = source.len();

    let mut pos = ScannerPosition {
        start: 0,
        current: 0,
        line: 1,
    };

    while !is_at_end(pos.current, input_len) {
        // We are at the beginning of the next lexeme.
        let pos_and_token = scan_token(&pos, &source)?;
        pos = pos_and_token.0.at_start();

        if let Some(t) = pos_and_token.1 {
            tokens.push(t);
        }
    }

    tokens.push(TokenInfo {
        token: Token::Eof,
        line: pos.line,
    });

    Ok(tokens)
}

fn is_at_end(current: usize, input_length: usize) -> bool {
    current >= input_length
}

fn scan_token(
    pos: &ScannerPosition,
    source: &[char],
) -> Result<(ScannerPosition, Option<TokenInfo>)> {
    let (pos, c) = advance(pos, source);
    match c {
        '(' => {
            let token = TokenInfo::new(Token::LeftParen, pos.line);
            Ok((pos, Some(token)))
        }
        ')' => {
            let token = TokenInfo::new(Token::RightParen, pos.line);
            Ok((pos, Some(token)))
        }
        '{' => {
            let token = TokenInfo::new(Token::LeftBrace, pos.line);
            Ok((pos, Some(token)))
        }
        '}' => {
            let token = TokenInfo::new(Token::RightBrace, pos.line);
            Ok((pos, Some(token)))
        }
        ',' => {
            let token = TokenInfo::new(Token::Comma, pos.line);
            Ok((pos, Some(token)))
        }
        '.' => {
            let token = TokenInfo::new(Token::Dot, pos.line);
            Ok((pos, Some(token)))
        }
        '-' => {
            let token = TokenInfo::new(Token::Minus, pos.line);
            Ok((pos, Some(token)))
        }
        '+' => {
            let token = TokenInfo::new(Token::Plus, pos.line);
            Ok((pos, Some(token)))
        }
        ';' => {
            let token = TokenInfo::new(Token::Semicolon, pos.line);
            Ok((pos, Some(token)))
        }
        '*' => {
            let token = TokenInfo::new(Token::Star, pos.line);
            Ok((pos, Some(token)))
        }
        '!' => match advance_on_match(pos, source, '=') {
            (pos, true) => {
                let token = TokenInfo::new(Token::BangEqual, pos.line);
                Ok((pos, Some(token)))
            }
            (pos, false) => {
                let token = TokenInfo::new(Token::Bang, pos.line);
                Ok((pos, Some(token)))
            }
        },
        '=' => match advance_on_match(pos, source, '=') {
            (pos, true) => {
                let token = TokenInfo::new(Token::EqualEqual, pos.line);
                Ok((pos, Some(token)))
            }
            (pos, false) => {
                let token = TokenInfo::new(Token::Equal, pos.line);
                Ok((pos, Some(token)))
            }
        },
        '<' => match advance_on_match(pos, source, '=') {
            (pos, true) => {
                let token = TokenInfo::new(Token::LessEqual, pos.line);
                Ok((pos, Some(token)))
            }
            (pos, false) => {
                let token = TokenInfo::new(Token::Less, pos.line);
                Ok((pos, Some(token)))
            }
        },
        '>' => match advance_on_match(pos, source, '=') {
            (pos, true) => {
                let token = TokenInfo::new(Token::GreaterEqual, pos.line);
                Ok((pos, Some(token)))
            }
            (pos, false) => {
                let token = TokenInfo::new(Token::Greater, pos.line);
                Ok((pos, Some(token)))
            }
        },
        '/' => match advance_on_match(pos, source, '/') {
            (pos, true) => {
                // A comment goes until the end of the line.
                let mut pos = pos;
                loop {
                    match peek(&pos, source) {
                        '\n' | '\0' => break,
                        _ => pos = advance(&pos, source).0,
                    }
                }

                Ok((pos, None))
            }
            (pos, false) => {
                let token = TokenInfo::new(Token::Slash, pos.line);
                Ok((pos, Some(token)))
            }
        },
        ' ' | '\r' | '\t' => Ok((pos, None)),
        '\n' => Ok((pos.next_line(), None)),
        '"' => string_token(&pos, source).map(|ps| (ps.0, Some(ps.1))),
        '0'..='9' => number_token(&pos, source).map(|ps| (ps.0, Some(ps.1))),
        'a'..='z' | 'A'..='Z' | '_' => identifier_token(&pos, source).map(|ps| (ps.0, Some(ps.1))),
        unexpected => Err(InterpreterError::scanner_error(
            pos.line,
            format!("Unexpected character '{unexpected}'"),
        )),
    }
}

fn advance(pos: &ScannerPosition, source: &[char]) -> (ScannerPosition, char) {
    (pos.next_pos(), source[pos.current])
}

fn advance_on_match(
    pos: ScannerPosition,
    source: &[char],
    expected: char,
) -> (ScannerPosition, bool) {
    if is_at_end(pos.current, source.len()) || source[pos.current] != expected {
        (pos, false)
    } else {
        (pos.next_pos(), true)
    }
}

fn peek(pos: &ScannerPosition, source: &[char]) -> char {
    if is_at_end(pos.current, source.len()) {
        '\0'
    } else {
        source[pos.current]
    }
}

fn peek_next(pos: &ScannerPosition, source: &[char]) -> char {
    if pos.current + 1 >= source.len() {
        '\0'
    } else {
        source[pos.current + 1]
    }
}

fn string_token(pos: &ScannerPosition, source: &[char]) -> Result<(ScannerPosition, TokenInfo)> {
    let mut pos = ScannerPosition { ..*pos };

    loop {
        match peek(&pos, source) {
            '"' => break,
            '\0' => {
                return Err(InterpreterError::scanner_error(
                    pos.line,
                    String::from("Unterminated string."),
                ));
            }
            '\n' => pos = pos.next_line(),
            _ => {}
        };

        pos = advance(&pos, source).0;
    }

    // The closing "
    pos = advance(&pos, source).0;

    let value: String = source[pos.start + 1..pos.current - 1].iter().collect();
    let lexeme = format!("\"{value}\"");
    let token = TokenInfo {
        token: Token::StrLiteral { lexeme, value },
        line: pos.line,
    };

    Ok((pos, token))
}

fn number_token(pos: &ScannerPosition, source: &[char]) -> Result<(ScannerPosition, TokenInfo)> {
    let mut pos = ScannerPosition { ..*pos };

    while let '0'..='9' = peek(&pos, source) {
        pos = advance(&pos, source).0;
    }

    // FIX
    // if-let with guards will be shipped in Rust 1.95
    // peek(&pos, source) if let '0'..='9' = peek_next(&pos, source)

    if peek(&pos, source) == '.'
        && let '0'..='9' = peek_next(&pos, source)
    {
        // Consume the "."
        pos = advance(&pos, source).0;

        while let '0'..='9' = peek(&pos, source) {
            pos = advance(&pos, source).0;
        }
    }

    let lexeme: String = source[pos.start..pos.current].iter().collect();

    let value: f64 = lexeme.parse().map_err(|e: std::num::ParseFloatError| {
        InterpreterError::scanner_error(pos.line, e.to_string())
    })?;

    let token = TokenInfo {
        token: Token::NumLiteral { lexeme, value },
        line: pos.line,
    };

    Ok((pos, token))
}

fn identifier_token(
    pos: &ScannerPosition,
    source: &[char],
) -> Result<(ScannerPosition, TokenInfo)> {
    let mut pos = ScannerPosition { ..*pos };

    while let 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' = peek(&pos, source) {
        pos = advance(&pos, source).0;
    }

    let text: String = source[pos.start..pos.current].iter().collect();

    let token = Token::try_from_word(text.as_str()).unwrap_or(Token::Identifier { lexeme: text });

    let token_info = TokenInfo {
        token,
        line: pos.line,
    };
    Ok((pos, token_info))
}
