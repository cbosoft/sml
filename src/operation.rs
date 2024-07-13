use json::JsonValue;

use crate::value::Value;


#[derive(Debug)]
pub enum UnaryOperation {
    // Arithmetic
    Increment,
    Decrement,

    // Boolean
    Negate,
}

impl UnaryOperation {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap();
            let rv = match s {
                "not" => Self::Negate,
                "++" => Self::Increment,
                "--" => Self::Decrement,
                s => anyhow::bail!("UnaryOp got invalid value {s}")
            };

            Ok(rv)
        }
        else {
            anyhow::bail!("operation expects a string")
        }
    }

    pub fn apply(&self, operand: &Value) -> anyhow::Result<Value> {
        todo!();
    }
}

#[derive(Debug)]
pub enum BinaryOperation {
    Assign,

    // Arithmetic
    Add,
    Subtract,
    Divide,
    Multiply,
    Power,

    // Comparison and equality
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,

    // Boolean
    And,
    Or,
}

impl BinaryOperation {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap();
            let rv = match s {
                "=" => Self::Assign,

                // Arithmetic
                "+" => Self::Add,
                "-" => Self::Subtract,
                "*" => Self::Multiply,
                "/" => Self::Divide,
                "**" | "^" => Self::Power,

                // Comparison and equality
                "<" => Self::LessThan,
                "<=" => Self::LessThanOrEqual,
                ">" => Self::GreaterThan,
                ">=" => Self::GreaterThanOrEqual,
                "==" => Self::Equal,
                "!=" => Self::NotEqual,

                // Boolean
                "and" => Self::And,
                "or" => Self::Or,

                s => anyhow::bail!("BinaryOp got invalid value {s}")
            };

            Ok(rv)
        }
        else {
            anyhow::bail!("operation expects a string")
        }
    }

    pub fn apply(&self, left: &Value, right: &Value) -> anyhow::Result<Value> {
        match self {
            Self::Add => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                    _ => todo!()
                }
            },
            _ => todo!()
        }
    }
}
