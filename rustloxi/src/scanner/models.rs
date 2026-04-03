use std::fmt;

// FIX: TokenInfo can be merged with Token into:
// Token::NonLiteral(line, tpe) Token::Literal(line, tpe, lexeme)
// Line can be accessed via a method
#[derive(Debug)]
pub struct TokenInfo {
    pub token: Token,
    pub line: u32,
}

#[derive(Debug)]
pub enum Token {
    NonLiteral { tpe: NonLiteralType },

    StrLiteral { lexeme: String, value: String },
    NumLiteral { lexeme: String, value: f64 },
    Identifier { lexeme: String },
}

#[derive(Debug)]
pub enum NonLiteralType {
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

    //UnaryOp {op: UnaryOp},

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
}

impl NonLiteralType {
    pub fn try_from_word(word: &str) -> Option<NonLiteralType> {
        match word {
            "and" => Some(NonLiteralType::And),
            "class" => Some(NonLiteralType::Class),
            "else" => Some(NonLiteralType::Else),
            "false" => Some(NonLiteralType::False),
            "for" => Some(NonLiteralType::For),
            "fun" => Some(NonLiteralType::Fun),
            "if" => Some(NonLiteralType::If),
            "nil" => Some(NonLiteralType::Nil),
            "or" => Some(NonLiteralType::Or),
            "print" => Some(NonLiteralType::Print),
            "return" => Some(NonLiteralType::Return),
            "super" => Some(NonLiteralType::Super),
            "this" => Some(NonLiteralType::This),
            "true" => Some(NonLiteralType::True),
            "var" => Some(NonLiteralType::Var),
            "while" => Some(NonLiteralType::While),
            _ => None,
        }
    }

    fn tpe(&self) -> &str {
        match &self {
            NonLiteralType::LeftParen => "LEFT_PAREN",
            NonLiteralType::RightParen => "RIGHT_PAREN",
            NonLiteralType::LeftBrace => "LEFT_BRACE",
            NonLiteralType::RightBrace => "RIGHT_BRACE",
            NonLiteralType::Comma => "COMMA",
            NonLiteralType::Dot => "DOT",
            NonLiteralType::Minus => "MINUS",
            NonLiteralType::Plus => "PLUS",
            NonLiteralType::Semicolon => "SEMICOLON",
            NonLiteralType::Slash => "SLASH",
            NonLiteralType::Star => "STAR",
            NonLiteralType::Bang => "BANG",
            NonLiteralType::BangEqual => "BANG_EQUAL",
            NonLiteralType::Equal => "EQUAL",
            NonLiteralType::EqualEqual => "EQUAL_EQUAL",
            NonLiteralType::Greater => "GREATER",
            NonLiteralType::GreaterEqual => "GREATER_EQUAL",
            NonLiteralType::Less => "LESS",
            NonLiteralType::LessEqual => "LESS_EQUAL",
            NonLiteralType::And => "AND",
            NonLiteralType::Class => "CLASS",
            NonLiteralType::Else => "ELSE",
            NonLiteralType::False => "FALSE",
            NonLiteralType::Fun => "FUN",
            NonLiteralType::For => "FOR",
            NonLiteralType::If => "IF",
            NonLiteralType::Nil => "NIL",
            NonLiteralType::Or => "OR",
            NonLiteralType::Print => "PRINT",
            NonLiteralType::Return => "RETURN",
            NonLiteralType::Super => "SUPER",
            NonLiteralType::This => "THIS",
            NonLiteralType::True => "TRUE",
            NonLiteralType::Var => "VAR",
            NonLiteralType::While => "WHILE",
            NonLiteralType::Eof => "EOF",
        }
    }
}

impl Token {
    fn lexeme(&self) -> &str {
        match &self {
            Token::NonLiteral {
                tpe: NonLiteralType::LeftParen,
            } => "(",
            Token::NonLiteral {
                tpe: NonLiteralType::RightParen,
            } => ")",
            Token::NonLiteral {
                tpe: NonLiteralType::LeftBrace,
            } => "{",
            Token::NonLiteral {
                tpe: NonLiteralType::RightBrace,
            } => "}",
            Token::NonLiteral {
                tpe: NonLiteralType::Comma,
            } => ",",
            Token::NonLiteral {
                tpe: NonLiteralType::Dot,
            } => ".",
            Token::NonLiteral {
                tpe: NonLiteralType::Minus,
            } => "-",
            Token::NonLiteral {
                tpe: NonLiteralType::Plus,
            } => "+",
            Token::NonLiteral {
                tpe: NonLiteralType::Semicolon,
            } => ";",
            Token::NonLiteral {
                tpe: NonLiteralType::Slash,
            } => "/",
            Token::NonLiteral {
                tpe: NonLiteralType::Star,
            } => "*",
            Token::NonLiteral {
                tpe: NonLiteralType::Bang,
            } => "!",
            Token::NonLiteral {
                tpe: NonLiteralType::BangEqual,
            } => "!=",
            Token::NonLiteral {
                tpe: NonLiteralType::Equal,
            } => "=",
            Token::NonLiteral {
                tpe: NonLiteralType::EqualEqual,
            } => "==",
            Token::NonLiteral {
                tpe: NonLiteralType::Greater,
            } => ">",
            Token::NonLiteral {
                tpe: NonLiteralType::GreaterEqual,
            } => ">=",
            Token::NonLiteral {
                tpe: NonLiteralType::Less,
            } => "<",
            Token::NonLiteral {
                tpe: NonLiteralType::LessEqual,
            } => "<=",
            Token::NonLiteral {
                tpe: NonLiteralType::And,
            } => "and",
            Token::NonLiteral {
                tpe: NonLiteralType::Class,
            } => "class",
            Token::NonLiteral {
                tpe: NonLiteralType::Else,
            } => "else",
            Token::NonLiteral {
                tpe: NonLiteralType::False,
            } => "false",
            Token::NonLiteral {
                tpe: NonLiteralType::Fun,
            } => "fun",
            Token::NonLiteral {
                tpe: NonLiteralType::For,
            } => "for",
            Token::NonLiteral {
                tpe: NonLiteralType::If,
            } => "if",
            Token::NonLiteral {
                tpe: NonLiteralType::Nil,
            } => "nil",
            Token::NonLiteral {
                tpe: NonLiteralType::Or,
            } => "or",
            Token::NonLiteral {
                tpe: NonLiteralType::Print,
            } => "or",
            Token::NonLiteral {
                tpe: NonLiteralType::Return,
            } => "return",
            Token::NonLiteral {
                tpe: NonLiteralType::Super,
            } => "super",
            Token::NonLiteral {
                tpe: NonLiteralType::This,
            } => "this",
            Token::NonLiteral {
                tpe: NonLiteralType::True,
            } => "true",
            Token::NonLiteral {
                tpe: NonLiteralType::Var,
            } => "var",
            Token::NonLiteral {
                tpe: NonLiteralType::While,
            } => "while",
            Token::NonLiteral {
                tpe: NonLiteralType::Eof,
            } => "", // EOF lexeme is empty string
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
            Token::NonLiteral { tpe } => {
                write!(f, "{0} {1} null", tpe.tpe(), self.lexeme())
            }
            Token::StrLiteral { lexeme, value } => {
                write!(f, "STRING {lexeme} {value}")
            }
            Token::NumLiteral { lexeme, value } => {
                write!(f, "NUMBER {lexeme} {value:?}")
            }
            Token::Identifier { lexeme } => {
                write!(f, "IDENTIFIER {lexeme} null")
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
