use super::{Object, NULL};
use crate::error::{self, MonkeyErr};
use crate::evaluator::gc::GCBox;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltInFnt {
    NotBuiltIn,
    Len,
    First,
    Last,
    Rest,
    Push,
}

impl From<&str> for BuiltInFnt {
    fn from(input: &str) -> Self {
        match input {
            "len" => Self::Len,
            "first" => Self::First,
            "last" => Self::Last,
            "rest" => Self::Rest,
            "push" => Self::Push,
            _ => Self::NotBuiltIn,
        }
    }
}

impl Into<&str> for BuiltInFnt {
    fn into(self) -> &'static str {
        match self {
            Self::Len => "len",
            Self::First => "first",
            Self::Last => "last",
            Self::Push => "push",
            _ => "",
        }
    }
}

impl BuiltInFnt {
    pub fn call(&self, args: Vec<GCBox<Object>>) -> error::Result<GCBox<Object>> {
        match self {
            Self::Len => {
                check_arg_len!(args, 1);

                let arg = &args[0];
                match &**arg {
                    Object::String(s) => Ok(GCBox::new(Object::Integer(s.len() as i64))),
                    Object::Array(array) => Ok(GCBox::new(Object::Integer(array.len() as i64))),
                    _ => Err(MonkeyErr::EvalErr {
                        msg: format!("Argument to `len` not supported, got {}", arg.r#type()),
                    }),
                }
            }
            Self::First => {
                check_arg_len!(args, 1);

                let arg = &args[0];
                if let Object::Array(array) = &**arg {
                    if !array.is_empty() {
                        return Ok(array[0].clone());
                    }
                    return Ok(GCBox::new(NULL));
                }
                return Err(MonkeyErr::EvalErr {
                    msg: format!("Argument to `first` must be array, got {}", arg.r#type()),
                });
            }
            Self::Last => {
                check_arg_len!(args, 1);

                let arg = &args[0];
                if let Object::Array(array) = &**arg {
                    if !array.is_empty() {
                        return Ok(array[array.len() - 1].clone());
                    }
                    return Ok(GCBox::new(NULL));
                }
                return Err(MonkeyErr::EvalErr {
                    msg: format!("Argument to `last` must be array, got {}", arg.r#type()),
                });
            }
            Self::Rest => {
                check_arg_len!(args, 1);

                let arg = &args[0];
                if let Object::Array(array) = &**arg {
                    if !array.is_empty() {
                        return Ok(GCBox::new(Object::Array(array.get(1..).unwrap().to_vec())));
                    }
                    return Ok(GCBox::new(NULL));
                }
                return Err(MonkeyErr::EvalErr {
                    msg: format!("Argument to `rest` must be array, got {}", arg.r#type()),
                });
            }
            Self::Push => {
                check_arg_len!(args, 2);

                let arr = args[0].clone();
                let obj = &args[1];

                if let Object::Array(mut array) = (*arr).clone() {
                    array.push(obj.clone());
                    return Ok(GCBox::new(Object::Array(array)));
                }
                return Err(MonkeyErr::EvalErr {
                    msg: format!("Argument to `push` must be array, got {}", arr.r#type()),
                });
            }
            _ => Ok(GCBox::new(NULL)),
        }
    }
}
