use json::JsonValue;

use crate::value::Value;
use crate::identifier::Identifier;
use crate::operation::{UnaryOperation, BinaryOperation};


#[derive(Debug)]
pub enum Expression {
    Value(Value),
    Identifier(Identifier),
    Unary(UnaryOperation, Box<Expression>),
    Binary(BinaryOperation, Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if !json.is_object() {
            anyhow::bail!("expr expects object")
        }

        let kind = &json["kind"];
        if !kind.is_string() {
            anyhow::bail!("expr expects key kind value to be string")
        }

        match kind.as_str().unwrap() {
            "value" => Self::new_value(json),
            "identifier" => Self::new_identifier(json),
            "unary op" => Self::new_unaryop(json),
            "binary op" => Self::new_binaryop(json),
            s => anyhow::bail!("unhandled expr kind {s}"),
        }
    }

    pub fn new_value(json: &JsonValue) -> anyhow::Result<Self> {
        let value = &json["value"];
        let value = Value::new(value)?;
        Ok(Self::Value(value))
    }

    pub fn new_identifier(json: &JsonValue) -> anyhow::Result<Self> {
        let identifier = Identifier::new(json)?;
        Ok(Self::Identifier(identifier))
    }

    pub fn new_unaryop(json: &JsonValue) -> anyhow::Result<Self> {
        let op = &json["operation"];
        let op = UnaryOperation::new(op)?;
        let operand = &json["operand"];
        let operand = Expression::new(operand)?;
        let operand = Box::new(operand);
        Ok(Self::Unary(op, operand))
    }

    pub fn new_binaryop(json: &JsonValue) -> anyhow::Result<Self> {
        let op = &json["operation"];
        let op = BinaryOperation::new(op)?;
        let left = &json["left"];
        let left = Expression::new(left)?;
        let left = Box::new(left);
        let right = &json["right"];
        let right = Expression::new(right)?;
        let right = Box::new(right);
        Ok(Self::Binary(op, left, right))
    }

    pub fn evaluate(&self, i: &JsonValue, o: &mut JsonValue, g: &mut JsonValue) -> anyhow::Result<Value> {
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
                        _ => anyhow::bail!("can only assign to identifier, got {left:?}")
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
