use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use json;
use json::JsonValue;
use serde::{Serialize, de::DeserializeOwned};


pub enum IdentifierKind {
    Inputs,
    Outputs,
    Globals
}


pub enum Value {
    Identifier(IdentifierKind, String),
    String(String),
    Number(f64),
    Bool(bool)
}

impl Value {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap().to_string();
            Ok(Self::String(s))
        }
        else if json.is_object() {
            let store = &json["store"];
            if !store.is_string() {
                anyhow::bail!("Identifier missing store or is not correct type (string)");
            }
            let store = store.as_str().unwrap();
            let kind = match store {
                "inputs" => IdentifierKind::Inputs,
                "outputs" => IdentifierKind::Outputs,
                "globals" => IdentifierKind::Globals,
                s => { anyhow::bail!("identifier store wrong value: {s}") }
            };


            let name = &json["name"];
            if !name.is_string() {
                anyhow::bail!("Identifier missing name or is not correct type (string)");
            }
            let name = name.as_str().unwrap().to_string();

            Ok(Self::Identifier(kind, name))
        }
        else if json.is_number() {
            Ok(Self::Number(json.as_f64().unwrap()))
        }
        else if json.is_boolean() {
            Ok(Self::Bool(json.as_bool().unwrap()))
        }
        else {
            anyhow::bail!("Value expects a json number, string, object, or boolean. Got null, array, or empty.")
        }
    }
}

pub enum UnaryOperation {
    Negate,
    Increment,
    Decrement,
}

impl UnaryOperation {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap();
            let rv = match s {
                "!" | "^" | "not" => Self::Negate,
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
}

pub enum BinaryOperation {
    Add,
    Subtract,
    Divide,
    Multiply,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual
}

impl BinaryOperation {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap();
            let rv = match s {
                "+" => Self::Add,
                "-" => Self::Subtract,
                "*" => Self::Multiply,
                "/" => Self::Divide,
                "<" => Self::LessThan,
                "<=" => Self::LessThanOrEqual,
                ">" => Self::GreaterThan,
                ">=" => Self::GreaterThanOrEqual,
                "==" => Self::Equal,
                "!=" => Self::NotEqual,
                s => anyhow::bail!("BinaryOp got invalid value {s}")
            };

            Ok(rv)
        }
        else {
            anyhow::bail!("operation expects a string")
        }
    }
}

pub enum Expression {
    Unary(UnaryOperation, Value),
    Binary(BinaryOperation, Value, Value),
}

impl Expression {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if !json.is_object() {
            anyhow::bail!("expr expects object")
        }

        let op = &json["operation"];
        let left = &json["left"];
        let rv = if left.is_null() {
            let value = &json["value"];
            let value = Value::new(value)?;
            let op = UnaryOperation::new(op)?;
            Self::Unary(op, value)
        }
        else {
            let right = &json["right"];
            let left = Value::new(left)?;
            let right = Value::new(right)?;
            let op = BinaryOperation::new(op)?;
            Self::Binary(op, left, right)
        };

        Ok(rv)
    }

    pub fn execute(&self, i: &JsonValue, o: &mut JsonValue, g: &mut JsonValue) -> anyhow::Result<Value> {
        todo!()
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
            let head: Result<Vec<_>, _> = head.members().map(|j| Expression::new(j.clone())).collect();
            let head = head?;

            let body = &state_data["body"];
            if !body.is_array() {
                return Err(anyhow::anyhow!("body expected to be object"));
            }
            let mut body_parsed = Vec::new();
            for item in body.members() {
                let condition = item["condition"].clone();
                let condition = Expression::new(condition)?;
                let expressions = item["expressions"].clone();
                let expressions: Result<Vec<_>, _> = expressions.members().map(|j| Expression::new(j.clone())).collect();
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
        let o = Self::run_state(state, i, &mut self.globals)?;
        let o = o.to_string();
        let o: O = serde_json::from_str(&o)?;
        Ok(o)
    }

    fn run_state(state: &State, i: JsonValue, g: &mut JsonValue) -> anyhow::Result<JsonValue> {
        todo!();
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
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
