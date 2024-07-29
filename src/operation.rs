use crate::error::{SML_Error, SML_Result};
use crate::value::Value;


#[derive(Clone, Debug)]
pub enum UnaryOperation {
    // Arithmetic
    Increment,
    Decrement,

    // Boolean
    Negate,
}


impl UnaryOperation {
    pub fn apply(&self, operand: &Value) -> SML_Result<Value> {
        match self {
            Self::Negate => {
                match operand {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(SML_Error::BadOperation("Negation only valid for boolean operands.".to_string()))
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
                    _ => Err(SML_Error::BadOperation("Incr/decrement only valid for numerical operands.".to_string()))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
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
    pub fn from_str(s: String) -> SML_Result<Self> {
        let rv = match s.as_str() {
            "=" => Self::Assign,

            // Arithmetic
            "+" => Self::Add,
            "-" => Self::Subtract,
            "*" => Self::Multiply,
            "/" => Self::Divide,
            "^" => Self::Power,

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

            s => { return Err(SML_Error::JsonFormatError(format!("BinaryOp got invalid value {s}"))); }
        };

        Ok(rv)
    }

    pub fn apply(&self, left: &Value, right: &Value) -> SML_Result<Value> {
        match self {
            Self::Assign => {
                panic!("assign handled elsewhere");
            },

            // Arithmetic ops
            Self::Add => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                    _ => Err(SML_Error::BadOperation("Arithmetic only valid for boolean operands.".to_string()))
                }
            },
            Self::Subtract => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                    _ => Err(SML_Error::BadOperation("Arithmetic only valid for boolean operands.".to_string()))
                }
            },
            Self::Multiply => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                    _ => Err(SML_Error::BadOperation("Arithmetic only valid for boolean operands.".to_string()))
                }
            },
            Self::Divide => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                    _ => Err(SML_Error::BadOperation("Arithmetic only valid for boolean operands.".to_string()))
                }
            },
            Self::Power => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left.powf(*right))),
                    _ => Err(SML_Error::BadOperation("Arithmetic only valid for boolean operands.".to_string()))
                }
            },

            // Comparison
            Self::LessThan => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left < right)),
                    _ => Err(SML_Error::BadOperation("Comparison only valid for boolean operands.".to_string()))
                }
            },
            Self::LessThanOrEqual => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left <= right)),
                    _ => Err(SML_Error::BadOperation("Comparison only valid for boolean operands.".to_string()))
                }
            },
            Self::GreaterThan => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left > right)),
                    _ => Err(SML_Error::BadOperation("Comparison only valid for boolean operands.".to_string()))
                }
            },
            Self::GreaterThanOrEqual => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left >= right)),
                    _ => Err(SML_Error::BadOperation("Comparison only valid for boolean operands.".to_string()))
                }
            },

            // Equality
            Self::Equal => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool( (left - right).abs() < 1e-5 )),
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(*left && *right)),
                    (Value::String(left), Value::String(right)) => Ok(Value::Bool(*left == *right)),
                    _ => Err(SML_Error::BadOperation("Comparison only valid for boolean operands.".to_string()))
                }
            },
            Self::NotEqual => {
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Bool( (left - right).abs() > 1e-5 )),
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(!(*left && *right))),
                    (Value::String(left), Value::String(right)) => Ok(Value::Bool(*left != *right)),
                    _ => Err(SML_Error::BadOperation("Comparison only valid for boolean operands.".to_string()))
                }
            },

            // Boolean ops
            Self::And => {
                match (left, right) {
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(*left && *right)),
                    _ => Err(SML_Error::BadOperation("Boolean ops only valid for boolean operands.".to_string()))
                }
            },
            Self::Or => {
                match (left, right) {
                    (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(*left || *right)),
                    _ => Err(SML_Error::BadOperation("Boolean ops only valid for boolean operands.".to_string()))
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
