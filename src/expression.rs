use json::JsonValue;

use crate::error::{SML_Error, SML_Result};
use crate::value::Value;
use crate::identifier::Identifier;
use crate::operation::{UnaryOperation, BinaryOperation};


#[derive(Clone, Debug)]
pub enum Expression {
    Value(Value),
    Identifier(Identifier),
    Unary(UnaryOperation, Box<Expression>),
    Binary(BinaryOperation, Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn evaluate(&self, i: &JsonValue, o: &mut JsonValue, g: &mut JsonValue) -> SML_Result<Value> {
        let rv = match self {
            Self::Value(value) => value.clone(),
            Self::Identifier(identifier) => identifier.get(i, o, g)?,
            Self::Unary(op, operand) => {
                let operand = operand.evaluate(i, o, g)?;
                op.apply(&operand)?
            },
            Self::Binary(op, left, right) => {
                let right = right.evaluate(i, o, g)?;
                if matches!(op, BinaryOperation::Assign) {
                    match &**left {
                        Self::Identifier(identifier) => {
                            identifier.set(o, g, &right)?
                        },
                        _ => { return Err(SML_Error::BadOperation(format!("can only assign to identifier, got {left:?}"))); }
                    }
                    right
                }
                else {
                    let left = left.evaluate(i, o, g)?;
                    op.apply(&left, &right)?
                }
            }
        };

        Ok(rv)
    }
}
