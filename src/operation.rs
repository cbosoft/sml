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
        match self {
            Self::Negate => {
                match operand {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(anyhow::anyhow!("negation only valid on boolean"))
                }
            },
            _ => {
                match operand {
                    Value::Number(v) => {
                        match self {
                            Self::Decrement => Ok(Value::Number(*v - 1.0)),
                            Self::Increment => Ok(Value::Number(*v + 1.0)),
                            Self::Negate => panic!(),
                        }
                    },
                    _ => Err(anyhow::anyhow!("incr/decrement only valid for numbers"))
                }
            }
        }
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
            Self::Assign => {
                panic!("assign handled elsewhere");
            },

            // Arithmetic ops
            Self::Add => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                    _ => Err(anyhow::anyhow!("arithmetic only valid on numbers"))
                }
            },
            Self::Subtract => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                    _ => Err(anyhow::anyhow!("arithmetic only valid on numbers"))
                }
            },
            Self::Multiply => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                    _ => Err(anyhow::anyhow!("arithmetic only valid on numbers"))
                }
            },
            Self::Divide => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                    _ => Err(anyhow::anyhow!("arithmetic only valid on numbers"))
                }
            },
            Self::Power => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left.powf(*right))),
                    _ => Err(anyhow::anyhow!("arithmetic only valid on numbers"))
                }
            },

            // Comparison
            Self::LessThan => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left < right)),
                    _ => Err(anyhow::anyhow!("Comparison only valid on numbers"))
                }
            },
            Self::LessThanOrEqual => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left <= right)),
                    _ => Err(anyhow::anyhow!("Comparison only valid on numbers"))
                }
            },
            Self::GreaterThan => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left > right)),
                    _ => Err(anyhow::anyhow!("Comparison only valid on numbers"))
                }
            },
            Self::GreaterThanOrEqual => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left >= right)),
                    _ => Err(anyhow::anyhow!("Comparison only valid on numbers"))
                }
            },

            // Equality
            Self::Equal => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool( (left - right).abs() < 1e-5 )),
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(*left && *right)),
                    (Value::String(left), Value::String(right)) => Ok(Value::Bool(*left == *right)),
                    _ => Err(anyhow::anyhow!("Equality check only valid between values of the same types"))
                }
            },
            Self::NotEqual => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool( (left - right).abs() > 1e-5 )),
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(!(*left && *right))),
                    (Value::String(left), Value::String(right)) => Ok(Value::Bool(*left != *right)),
                    _ => Err(anyhow::anyhow!("Equality check only valid between values of the same types"))
                }
            },

            // Boolean ops
            Self::And => {
                match (left, right) {
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(*left && *right)),
                    _ => Err(anyhow::anyhow!("boolean ops only valid on boolean"))
                }
            },
            Self::Or => {
                match (left, right) {
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(*left || *right)),
                    _ => Err(anyhow::anyhow!("boolean ops only valid on boolean"))
                }
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_eq_num() {
        let left = Value::Number(1.0);
        let right = Value::Number(1.0);
        let op = BinaryOperation::Add;
        let result = op.apply(&left, &right).unwrap();
        match result {
            Value::Number(v) => {
                assert!( (v - 2.0).abs() < 1e-5 )
            },
            _ => { panic!(); }
        }

        let op2 = BinaryOperation::Equal;
        let expected = Value::Number(2.0);
        let result2 = op2.apply(&result, &expected).unwrap();
        assert!(matches!(result2, Value::Bool(true)));
    }

}
