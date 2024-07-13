use std::{any, collections::HashMap};
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use json;
use json::JsonValue;
use serde::{Serialize, de::DeserializeOwned};


#[derive(Debug)]
pub enum IdentifierStore {
    Inputs,
    Outputs,
    Globals
}

#[derive(Debug)]
pub struct Identifier {
    store: IdentifierStore,
    name: String
}

impl Identifier {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        let store = &json["store"];
        if !store.is_string() {
            anyhow::bail!("Identifier missing store or is not correct type (string)");
        }
        let store = store.as_str().unwrap();
        let store = match store {
            "inputs" => IdentifierStore::Inputs,
            "outputs" => IdentifierStore::Outputs,
            "globals" => IdentifierStore::Globals,
            s => { anyhow::bail!("identifier store wrong value: {s}") }
        };


        let name = &json["name"];
        if !name.is_string() {
            anyhow::bail!("Identifier missing name or is not correct type (string)");
        }
        let name = name.as_str().unwrap().to_string();

        Ok(Self { store, name })
    }

    pub fn get(&self, i: &JsonValue, o: &JsonValue, g: &JsonValue) -> anyhow::Result<Value> {
        todo!();
    }

    pub fn set(&self, o: &mut JsonValue, g: &mut JsonValue, v: &Value) -> anyhow::Result<()> {
        todo!();
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
}

impl Value {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap().to_string();
            Ok(Self::String(s))
        }
        else if json.is_number() {
            Ok(Self::Number(json.as_f64().unwrap()))
        }
        else if json.is_boolean() {
            Ok(Self::Bool(json.as_bool().unwrap()))
        }
        else {
            anyhow::bail!("Value expects a json number, string, or boolean. Got null, object, array, or empty.")
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Number(v) => *v != 0.0,
            Self::String(v) => v.is_empty(),
        }
    }
}

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


pub struct State {
    /// Name of state which is the default successor to this one
    default: String,

    /// Expressions evaluated when this state is visited
    head: Vec<Expression>,

    /// List of condition expressions and associated expressions.
    /// When the condition expression is true, the associated body of expressions is run.
    body: Vec<(Expression, Vec<Expression>)>,
}

impl State {
    pub fn new(default: String, head: Vec<Expression>, body: Vec<(Expression, Vec<Expression>)>) -> Self {
        Self { default, head, body }
    }

    pub fn run(&self, i: &JsonValue, g: &mut JsonValue) -> anyhow::Result<JsonValue> {
        let mut o = json::object! { };

        for expr in &self.head {
            expr.evaluate(i, &mut o, g)?;
        }

        todo!();


        Ok(o)
    }
}


pub struct SM {
    states: HashMap<String, State>,
    current_state: String,
    globals: JsonValue,
}

impl SM {
    pub fn new(states: HashMap<String, State>, current_state: String) -> Self {
        let globals = json::object! { };
        Self { states, current_state, globals }
    }

    pub fn from_file<P: Into<PathBuf>>(path: P) -> anyhow::Result<Self> {
        let path: PathBuf = path.into();
        let mut source = String::new();
        let mut f = File::options().read(true).open(path)?;
        let _ = f.read_to_string(&mut source)?;
        Self::from_src(&source)
    }

    pub fn from_src(src: &str) -> anyhow::Result<Self> {
        let json = json::parse(src)?;
        if !json.is_object() {
            return Err(anyhow::anyhow!("json expected to be object"));
        }

        let mut current_state = None;
        let mut states = HashMap::new();
        for (state_name, state_data) in json.entries() {
            let state_name = state_name.to_string();
            if current_state.is_none() {
                current_state = Some(state_name.clone());
            }
            let default = state_data["default"].as_str().unwrap().to_string();

            let head = &state_data["head"];
            if !head.is_array() {
                return Err(anyhow::anyhow!("head expected to be array"));
            }
            let head: Result<Vec<_>, _> = head.members().map(|j| Expression::new(j)).collect();
            let head = head?;

            let body = &state_data["body"];
            if !body.is_array() {
                return Err(anyhow::anyhow!("body expected to be object"));
            }
            let mut body_parsed = Vec::new();
            for item in body.members() {
                let condition = &item["condition"];
                let condition = Expression::new(condition)?;
                let expressions = &item["expressions"];
                let expressions: Result<Vec<_>, _> = expressions.members().map(|j| Expression::new(j)).collect();
                let expressions = expressions?;
                body_parsed.push((condition, expressions));
            }
            
            let state = State::new(default, head, body_parsed);
            states.insert(state_name, state);
        }

        if current_state.is_none() {
            return Err(anyhow::anyhow!("no states defined!"));
        }
        let current_state = current_state.unwrap();

        Ok(Self::new(states, current_state))
    }

    // Use `DeserializeOwned` instead `Deserialize` and dealing with lifetime issues
    // https://users.rust-lang.org/t/lifetime-confusion-with-function-parameter-serde-deserialize/76842
    pub fn run<I: Serialize, O: DeserializeOwned>(&mut self, i: I) -> anyhow::Result<O> {
        let i = serde_json::to_string(&i)?;
        let i = json::parse(&i)?;
        let state = self.states.get(&self.current_state).unwrap();
        let o = state.run(&i, &mut self.globals)?;
        let o = o.to_string();
        let o: O = serde_json::from_str(&o)?;
        Ok(o)
    }
}




pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let expr_src = r#"{ "kind": "binary op", "operation": "+", "left": { "kind": "value", "value": 5 }, "right": { "kind": "value", "value": 5 } }"#;
        let expr_json = json::parse(expr_src).unwrap();
        let expr = Expression::new(&expr_json).unwrap();

        let i = json::object! { };
        let mut o = json::object! { };
        let mut g = json::object! { };
        let result = expr.evaluate(&i, &mut o, &mut g).unwrap();

        match result {
            Value::Number(v) => {
                assert!( (v - 10.0).abs() < 1e-6 )
            },
            _ => panic!(),
        }
    }
}
