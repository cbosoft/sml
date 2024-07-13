use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use json::JsonValue;

use crate::expression::Expression;
use crate::state::State;


pub struct StateMachine {
    states: HashMap<String, State>,
    current_state: String,
    globals: JsonValue,
}

impl StateMachine {
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
