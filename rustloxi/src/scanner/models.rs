use std::fmt;

#[derive(Debug)]
pub struct TokenInfo {
    pub token: Token,
    pub line: u32,
}

impl TokenInfo {
    pub fn new(token: Token, line: u32) -> TokenInfo {
        TokenInfo { token, line }
    }
}

#[derive(Debug)]
pub enum Token {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,

    StrLiteral { lexeme: String, value: String },
    NumLiteral { lexeme: String, value: f64 },
    Identifier { lexeme: String },
}

impl Token {
    pub fn try_from_word(word: &str) -> Option<Token> {
        match word {
            "and" => Some(Token::And),
            "class" => Some(Token::Class),
            "else" => Some(Token::Else),
            "false" => Some(Token::False),
            "for" => Some(Token::For),
            "fun" => Some(Token::Fun),
            "if" => Some(Token::If),
            "nil" => Some(Token::Nil),
            "or" => Some(Token::Or),
            "print" => Some(Token::Print),
            "return" => Some(Token::Return),
            "super" => Some(Token::Super),
            "this" => Some(Token::This),
            "true" => Some(Token::True),
            "var" => Some(Token::Var),
            "while" => Some(Token::While),
            _ => None,
        }
    }

    fn tpe(&self) -> &str {
        match &self {
            Token::LeftParen => "LEFT_PAREN",
            Token::RightParen => "RIGHT_PAREN",
            Token::LeftBrace => "LEFT_BRACE",
            Token::RightBrace => "RIGHT_BRACE",
            Token::Comma => "COMMA",
            Token::Dot => "DOT",
            Token::Minus => "MINUS",
            Token::Plus => "PLUS",
            Token::Semicolon => "SEMICOLON",
            Token::Slash => "SLASH",
            Token::Star => "STAR",
            Token::Bang => "BANG",
            Token::BangEqual => "BANG_EQUAL",
            Token::Equal => "EQUAL",
            Token::EqualEqual => "EQUAL_EQUAL",
            Token::Greater => "GREATER",
            Token::GreaterEqual => "GREATER_EQUAL",
            Token::Less => "LESS",
            Token::LessEqual => "LESS_EQUAL",
            Token::And => "AND",
            Token::Class => "CLASS",
            Token::Else => "ELSE",
            Token::False => "FALSE",
            Token::Fun => "FUN",
            Token::For => "FOR",
            Token::If => "IF",
            Token::Nil => "NIL",
            Token::Or => "OR",
            Token::Print => "PRINT",
            Token::Return => "RETURN",
            Token::Super => "SUPER",
            Token::This => "THIS",
            Token::True => "TRUE",
            Token::Var => "VAR",
            Token::While => "WHILE",
            Token::Eof => "EOF",
            Token::StrLiteral { .. } => "STRING",
            Token::NumLiteral { .. } => "NUMBER",
            Token::Identifier { .. } => "IDENTIFIER",
        }
    }
}

impl Token {
    pub fn lexeme(&self) -> &str {
        match &self {
            Token::LeftParen => "(",
            Token::RightParen => ")",
            Token::LeftBrace => "{",
            Token::RightBrace => "}",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::Minus => "-",
            Token::Plus => "+",
            Token::Semicolon => ";",
            Token::Slash => "/",
            Token::Star => "*",
            Token::Bang => "!",
            Token::BangEqual => "!=",
            Token::Equal => "=",
            Token::EqualEqual => "==",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::And => "and",
            Token::Class => "class",
            Token::Else => "else",
            Token::False => "false",
            Token::Fun => "fun",
            Token::For => "for",
            Token::If => "if",
            Token::Nil => "nil",
            Token::Or => "or",
            Token::Print => "or",
            Token::Return => "return",
            Token::Super => "super",
            Token::This => "this",
            Token::True => "true",
            Token::Var => "var",
            Token::While => "while",
            Token::Eof => "", // EOF lexeme is empty string
            Token::StrLiteral { lexeme, value: _ } => lexeme.as_str(),
            Token::NumLiteral { lexeme, value: _ } => lexeme.as_str(),
            Token::Identifier { lexeme } => lexeme.as_str(),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We tweak output format to match canonical implementation
        match self {
            Token::StrLiteral { lexeme, value } => {
                write!(f, "STRING {lexeme} {value}")
            }
            Token::NumLiteral { lexeme, value } => {
                write!(f, "NUMBER {lexeme} {value:?}")
            }
            Token::Identifier { lexeme } => {
                write!(f, "IDENTIFIER {lexeme} null")
            }
            non_literal => {
                write!(f, "{0} {1} null", non_literal.tpe(), non_literal.lexeme())
            }
        }
    }
}

// TODO: add a generic field for storing Token/other output?
//  Or add a wrapper struct for that?
pub struct ScannerPosition {
    pub start: usize,
    pub current: usize,
    pub line: u32,
}

impl ScannerPosition {
    pub fn next_pos(&self) -> Self {
        Self {
            current: self.current + 1,
            ..*self
        }
    }

    pub fn next_line(&self) -> Self {
        Self {
            line: self.line + 1,
            ..*self
        }
    }

    pub fn at_start(&self) -> Self {
        Self {
            start: self.current,
            ..*self
        }
    }
}
