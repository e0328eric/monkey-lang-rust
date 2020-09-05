#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    ILLIGAL,
    EOF,
    // Identifiers + Literals
    IDENT(String),
    INT(i64),

    // Operations
    ASSIGN,
    PLUS,
    MINUS,
    BANG,
    ASTERISK,
    SLASH,
    LT,
    GT,
    EQ,
    NOTEQ,

    // Delimiters
    COMMA,
    SEMICOLON,

    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,

    // Keywords
    FUNCTION,
    LET,
    IF,
    ELSE,
    RETURN,
    TRUE,
    FALSE,
}

impl Token {
    pub fn is_str_keywords(s: &str) -> Option<Token> {
        match s {
            "let" => Some(Token::LET),
            "fn" => Some(Token::FUNCTION),
            "if" => Some(Token::IF),
            "else" => Some(Token::ELSE),
            "return" => Some(Token::RETURN),
            "true" => Some(Token::TRUE),
            "false" => Some(Token::FALSE),
            _ => None,
        }
    }
}

pub fn is_letter(chr: char) -> bool {
    chr.is_ascii_alphabetic() || chr == '_'
}

#[test]
fn check_is_letter() {
    assert!(is_letter('a'));
    assert!(is_letter('P'));
    assert!(is_letter('_'));
    assert!(!is_letter('3'));
    assert!(!is_letter('{'));
}
