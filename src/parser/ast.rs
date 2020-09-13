use crate::lexer::token::Token;

pub type Program = Vec<Statement>;
pub type BlockStmt = Vec<Statement>;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    LetStmt { name: String, value: Expression },
    ReturnStmt { value: Expression },
    ExpressionStmt { expression: Expression },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Ident(String),
    Integer(i64),
    Boolean(bool),
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    IfExpr {
        condition: Box<Expression>,
        consequence: BlockStmt,
        alternative: BlockStmt,
    },
    Function {
        parameter: Vec<String>,
        body: BlockStmt,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

impl From<Box<Expression>> for Statement {
    fn from(expr: Box<Expression>) -> Self {
        Self::ExpressionStmt { expression: *expr }
    }
}

impl From<Expression> for Statement {
    fn from(expr: Expression) -> Self {
        Self::ExpressionStmt { expression: expr }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Precedence {
    LOWEST,
    EQUALS,
    LESSGREATER,
    SUM,
    PRODUCT,
    PREFIX,
    CALL,
}

impl Precedence {
    pub fn take_precedence(tok: &Token) -> Self {
        match tok {
            Token::EQ => Precedence::EQUALS,
            Token::NOTEQ => Precedence::EQUALS,
            Token::LT => Precedence::LESSGREATER,
            Token::GT => Precedence::LESSGREATER,
            Token::PLUS => Precedence::SUM,
            Token::MINUS => Precedence::SUM,
            Token::ASTERISK => Precedence::PRODUCT,
            Token::SLASH => Precedence::PRODUCT,
            Token::LPAREN => Precedence::CALL,
            _ => Precedence::LOWEST,
        }
    }
}