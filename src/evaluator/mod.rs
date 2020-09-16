use crate::error;
use crate::lexer::token::Token;
use crate::object::{Environment, Object};
use crate::parser::ast::*;

type Error = crate::error::MonkeyErr;

// To make unique objects
const TRUE: Object = Object::Boolean { value: true };
const FALSE: Object = Object::Boolean { value: false };
const NULL: Object = Object::Null;

pub fn eval_program(stmts: Vec<Statement>, env: &mut Environment) -> error::Result<Object> {
    let mut result: Object = NULL;
    for statement in stmts {
        result = eval(statement, env)?;
        if let Object::ReturnValue { value } = result {
            return Ok(*value);
        }
    }
    Ok(result)
}

pub fn eval(node: Statement, env: &mut Environment) -> error::Result<Object> {
    match node {
        Statement::LetStmt { name, value } => {
            let val = eval(value.into(), env)?;
            Ok(env.set(name, val))
        }
        Statement::ReturnStmt { value } => Ok(Object::ReturnValue {
            value: Box::new(eval(value.into(), env)?),
        }),
        Statement::ExpressionStmt { expression } => match expression {
            Expression::Integer(value) => Ok(Object::Integer { value }),
            Expression::Complex { re, im } => Ok(Object::Complex { re, im }),
            Expression::Ident(value) => eval_identifier(value, env),
            Expression::Boolean(value) => {
                if value {
                    Ok(TRUE)
                } else {
                    Ok(FALSE)
                }
            }
            Expression::Prefix { operator, right } => {
                eval_prefix_expr(operator, eval(right.into(), env)?)
            }

            Expression::Infix {
                left,
                operator,
                right,
            } => eval_infix_expr(operator, eval(left.into(), env)?, eval(right.into(), env)?),
            Expression::IfExpr {
                condition,
                consequence,
                alternative,
            } => eval_if_expr(eval(condition.into(), env)?, consequence, alternative, env),
            Expression::Function { parameter, body } => Ok(Object::Function {
                parameter,
                body,
                env: Box::new(env.clone()),
            }),
            Expression::Call {
                function,
                arguments,
            } => {
                let function = eval(function.into(), env)?;
                let args = eval_expressions(arguments, env)?;
                apply_function(function, args)
            }
        },
    }
}

fn eval_expressions(exps: Vec<Expression>, env: &mut Environment) -> error::Result<Vec<Object>> {
    let mut result: Vec<Object> = Vec::new();

    for e in exps {
        let evaluated = eval(e.into(), env)?;
        result.push(evaluated);
    }

    Ok(result)
}

fn apply_function(function: Object, args: Vec<Object>) -> error::Result<Object> {
    if let Object::Function {
        parameter,
        body,
        env,
    } = function
    {
        let mut extended_env = extended_function_env(parameter, args, &env);
        let evaluated = eval_stmts(body, &mut extended_env)?;
        if let Object::ReturnValue { value } = evaluated {
            Ok(*value)
        } else {
            Ok(evaluated)
        }
    } else {
        Err(Error::EvalNotFunction { got: function })
    }
}

fn extended_function_env(
    parameter: Vec<String>,
    args: Vec<Object>,
    env: &Environment,
) -> Environment {
    let mut env = Environment::new_enclosed(env);
    for i in 0..parameter.len() {
        env.set(
            parameter.get(i).unwrap().to_owned(),
            args.get(i).unwrap().to_owned(),
        );
    }
    env
}

fn eval_identifier(ident: String, env: &mut Environment) -> error::Result<Object> {
    let value = env.get(&ident);
    if let Some(val) = value {
        Ok(val.clone())
    } else {
        Err(Error::EvalIdentNotFound { name_got: ident })
    }
}

fn eval_stmts(block: BlockStmt, env: &mut Environment) -> error::Result<Object> {
    let mut result: Object = NULL;
    for statement in block {
        result = eval(statement, env)?;
        if let Object::ReturnValue { .. } = result {
            return Ok(result);
        }
    }
    Ok(result)
}

fn eval_prefix_expr(operator: Token, right: Object) -> error::Result<Object> {
    match operator {
        Token::BANG => eval_bang_operator_expr(right),
        Token::MINUS => eval_minus_operator_expr(right),
        _ => Err(Error::EvalUnknownPrefix { operator, right }),
    }
}

fn eval_infix_expr(operator: Token, left: Object, right: Object) -> error::Result<Object> {
    // Check whether left and right are number types
    if left.to_complex().is_some() && right.to_complex().is_some() {
        eval_num_infix_expr(operator, left, right)
    } else if Object::is_same_type(&left, &right) {
        match operator {
            Token::EQ => {
                if left == right {
                    Ok(TRUE)
                } else {
                    Ok(FALSE)
                }
            }
            Token::NOTEQ => {
                if left != right {
                    Ok(TRUE)
                } else {
                    Ok(FALSE)
                }
            }
            _ => Err(Error::EvalUnknownInfix {
                left,
                operator,
                right,
            }),
        }
    } else {
        Err(Error::EvalTypeMismatch {
            left,
            operator,
            right,
        })
    }
}

fn eval_if_expr(
    cond: Object,
    consq: BlockStmt,
    alter: BlockStmt,
    env: &mut Environment,
) -> error::Result<Object> {
    if is_truthy(&cond) {
        eval_stmts(consq, env)
    } else if !alter.is_empty() {
        eval_stmts(alter, env)
    } else {
        Ok(NULL)
    }
}

fn eval_bang_operator_expr(right: Object) -> error::Result<Object> {
    match right {
        TRUE => Ok(FALSE),
        FALSE => Ok(TRUE),
        NULL => Ok(TRUE), // This means that NULL is falsty
        _ => Ok(FALSE),   // and the defalut is truthy
    }
}

fn eval_minus_operator_expr(right: Object) -> error::Result<Object> {
    match right {
        Object::Integer { value } => Ok(Object::Integer { value: -value }),
        Object::Complex { re, im } => Ok(Object::Complex { re: -re, im: -im }),
        _ => Err(Error::EvalUnknownPrefix {
            operator: Token::MINUS,
            right,
        }),
    }
}

fn eval_num_infix_expr(operator: Token, lf: Object, rt: Object) -> error::Result<Object> {
    use crate::object::Object::Complex;
    // This function called only when both option are some.
    // So unwraping these does not cause panic.
    match (lf.to_complex().unwrap(), rt.to_complex().unwrap()) {
        (Complex { re: lf, im: 0 }, Complex { re: rt, im: 0 }) => {
            eval_integer_infix_expr(operator, lf, rt)
        }
        (
            Complex {
                re: lf_re,
                im: lf_im,
            },
            Complex {
                re: rt_re,
                im: rt_im,
            },
        ) => eval_complex_infix_expr(operator, lf_re, lf_im, rt_re, rt_im),
        _ => Err(Error::EvalUnknownInfix {
            left: lf,
            operator,
            right: rt,
        }),
    }
}
fn eval_complex_infix_expr(
    operator: Token,
    lf_re: i64,
    lf_im: i64,
    rt_re: i64,
    rt_im: i64,
) -> error::Result<Object> {
    match operator {
        Token::PLUS => Ok(Object::Complex {
            re: lf_re + rt_re,
            im: lf_im + rt_im,
        }),
        Token::MINUS => Ok(Object::Complex {
            re: lf_re - rt_re,
            im: lf_im - rt_im,
        }),
        Token::ASTERISK => Ok(Object::Complex {
            re: lf_re * rt_re - lf_im * rt_im,
            im: lf_re * rt_im + lf_im * rt_re,
        }),
        Token::EQ => Ok(Object::Boolean {
            value: lf_re == rt_re && lf_im == rt_im,
        }),
        Token::NOTEQ => Ok(Object::Boolean {
            value: lf_re != rt_re || lf_im != rt_im,
        }),
        // Division, power operation and ordering are not implemented.
        _ => Err(Error::EvalUnknownInfix {
            left: Object::Complex {
                re: lf_re,
                im: lf_im,
            },
            operator,
            right: Object::Complex {
                re: rt_re,
                im: rt_im,
            },
        }),
    }
}

fn eval_integer_infix_expr(operator: Token, lf: i64, rt: i64) -> error::Result<Object> {
    match operator {
        Token::PLUS => Ok(Object::Integer { value: lf + rt }),
        Token::MINUS => Ok(Object::Integer { value: lf - rt }),
        Token::ASTERISK => Ok(Object::Integer { value: lf * rt }),
        Token::SLASH => Ok(Object::Integer { value: lf / rt }),
        Token::POWER => {
            if rt >= 0 {
                Ok(Object::Integer {
                    value: lf.pow(rt as u32),
                })
            } else {
                Err(Error::EvalPowErr)
            }
        }
        Token::LT => Ok(Object::Boolean { value: lf < rt }),
        Token::GT => Ok(Object::Boolean { value: lf > rt }),
        Token::EQ => Ok(Object::Boolean { value: lf == rt }),
        Token::NOTEQ => Ok(Object::Boolean { value: lf != rt }),
        _ => Err(Error::EvalUnknownInfix {
            left: Object::Integer { value: lf },
            operator,
            right: Object::Integer { value: rt },
        }),
    }
}

fn is_truthy(obj: &Object) -> bool {
    match *obj {
        NULL | FALSE => false,
        _ => true,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn eval_integers() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            5; 10;
            -5; -10;
            5 + 5 + 5 + 5 - 10;
            2 * 2 * 2 * 2 * 2;
            -50 + 100 - 50;
            5 * 2 + 10;
            5 + 2 * 10;
            20 + 2 * -10;
            50 / 2 * 2 + 10;
            2 * (5 + 10);
            3 * 3 * 3 + 10;
            3 * (3 * 3) + 10;
            (5 + 10 * 2 + 15 / 3) * 2 + -10;
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer { value: 5 },
                Object::Integer { value: 10 },
                Object::Integer { value: -5 },
                Object::Integer { value: -10 },
                Object::Integer { value: 10 },
                Object::Integer { value: 32 },
                Object::Integer { value: 0 },
                Object::Integer { value: 20 },
                Object::Integer { value: 25 },
                Object::Integer { value: 0 },
                Object::Integer { value: 60 },
                Object::Integer { value: 30 },
                Object::Integer { value: 37 },
                Object::Integer { value: 37 },
                Object::Integer { value: 50 },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_complex() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            5i; 10i;
            -5i; -10i;
            1 + 4i; 1 - 4i;
            - 1 + 4i; - 1 - 4i;
            (-1) + 4i; (-1) - 4i;
            (-1) + 4i + (-1) - 4i;
            (-1) + 4i - (-1) - 4i;
            ((-1) + 4i) - ((-1) - 4i);
            (-1) + 4i * (-1) - 4i;
            ((-1) + 4i) * ((-1) - 4i);
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Complex { re: 0, im: 5 },
                Object::Complex { re: 0, im: 10 },
                Object::Complex { re: 0, im: -5 },
                Object::Complex { re: 0, im: -10 },
                Object::Complex { re: 1, im: 4 },
                Object::Complex { re: 1, im: -4 },
                Object::Complex { re: -1, im: -4 },
                Object::Complex { re: -1, im: 4 },
                Object::Complex { re: -1, im: 4 },
                Object::Complex { re: -1, im: -4 },
                Object::Complex { re: -2, im: 0 },
                Object::Complex { re: 0, im: 0 },
                Object::Complex { re: 0, im: 8 },
                Object::Complex { re: -1, im: -8 },
                Object::Complex { re: 17, im: 0 },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_boolean() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            false;
            true;
            1 < 2;
            1 > 2;
            1 < 1;
            1 > 1;
            1 == 1;
            1 != 1;
            1 == 2;
            1 != 2;
            true == true;
            true == false;
            false == true;
            true != false;
            false != true;
            false != false;
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_mixed() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new("7; true; 15; false; false; 324;"))
            .parse_program()?
            .into_iter()
            .map(|x| eval(x, &mut env).unwrap())
            .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer { value: 7 },
                Object::Boolean { value: true },
                Object::Integer { value: 15 },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Integer { value: 324 },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_bang_operator() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new("!true; !false !5; !!true; !!false; !!5;"))
            .parse_program()?
            .into_iter()
            .map(|x| eval(x, &mut env).unwrap())
            .collect();
        assert_eq!(
            input,
            vec![
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_if_else_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            if (true) { 10 };
            if (false) { 10 };
            if (1) { 10 };
            if (1 < 2) { 10 };
            if (1 > 2) { 10 };
            if (1 > 2) { 10 } else { 20 };
            if (1 < 2) { 10 } else { 20 };
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer { value: 10 },
                NULL,
                Object::Integer { value: 10 },
                Object::Integer { value: 10 },
                NULL,
                Object::Integer { value: 20 },
                Object::Integer { value: 10 },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_return_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input = eval_program(
            Parser::new(Lexer::new(
                r#"
            9; return 2 * 5; 8;
            "#,
            ))
            .parse_program()?,
            &mut env,
        )
        .unwrap();
        assert_eq!(input, Object::Integer { value: 10 });
        Ok(())
    }

    #[test]
    fn eval_nested_block_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input = eval_program(
            Parser::new(Lexer::new(
                r#"
            if (10 > 1) {
                if (12 > 2) {
                    return 10;
                }

                return 1;
            }
            "#,
            ))
            .parse_program()?,
            &mut env,
        )
        .unwrap();
        assert_eq!(input, Object::Integer { value: 10 });
        Ok(())
    }

    #[test]
    fn eval_let_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            let a = 5; a;
            let a = 5 * 5; a;
            let a = 5; let b = a; b;
            let a = 5; let b = a; let c = a + b + 5; c;
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::DeclareVariable,
                Object::Integer { value: 5 },
                Object::DeclareVariable,
                Object::Integer { value: 25 },
                Object::DeclareVariable,
                Object::DeclareVariable,
                Object::Integer { value: 5 },
                Object::DeclareVariable,
                Object::DeclareVariable,
                Object::DeclareVariable,
                Object::Integer { value: 15 },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_function_object() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            fn(x) { x + 2; };
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![Object::Function {
                parameter: vec!["x".to_string()],
                body: vec![Statement::ExpressionStmt {
                    expression: Expression::Infix {
                        left: Box::new(Expression::Ident("x".to_string())),
                        operator: Token::PLUS,
                        right: Box::new(Expression::Integer(2))
                    }
                }],
                env: Box::new(env)
            }]
        );
        Ok(())
    }

    #[test]
    fn eval_function_application() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            let identity = fn(x) { x; }; identity(5);
            let identity = fn(x) { return x; }; identity(5);
            let double = fn(x) { x * 2; }; double(6);
            let add = fn(x, y) { x + y; }; add(6, 5);
            let add = fn(x, y) { x + y; }; add(6 + 5, add(6, 5));
            fn(x) { x; }(5);
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::DeclareVariable,
                Object::Integer { value: 5 },
                Object::DeclareVariable,
                Object::Integer { value: 5 },
                Object::DeclareVariable,
                Object::Integer { value: 12 },
                Object::DeclareVariable,
                Object::Integer { value: 11 },
                Object::DeclareVariable,
                Object::Integer { value: 22 },
                Object::Integer { value: 5 },
            ]
        );
        Ok(())
    }
}
