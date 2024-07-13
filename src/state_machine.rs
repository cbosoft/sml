use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use json::JsonValue;

use crate::expression::Expression;
use crate::state::{State, StateOp};


pub struct StateMachine {
    // Might need RefCell here
    states: HashMap<String, State>,
    initial_state: String,
    current_state: Option<String>, // if using RC<RefCell<>>, then can put the state directly here
                                   // instead of the name...
    globals: JsonValue,
}

impl StateMachine {
    pub fn new(states: HashMap<String, State>, initial_state: String) -> Self {
        let globals = json::object! { };
        let current_state = Some(initial_state.clone());
        Self { states, initial_state, current_state, globals }
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
        if !json.is_array() {
            return Err(anyhow::anyhow!("json expected to be array"));
        }

        let mut initial_state = None;
        let mut states = HashMap::new();
        for state_data in json.members() {
            let state_name = &state_data["name"];
            if !state_name.is_string() {
                anyhow::bail!("state name should be string");
            }
            let state_name = state_name.as_str().unwrap().to_string();

            if initial_state.is_none() {
                initial_state = Some(state_name.clone());
            }
            let default_next = match state_data["default_next"].as_str() {
                Some(s) => Some(s.to_string()),
                None => None,
            };

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

                let state_op = &item["state op"];
                if !state_op.is_string() {
                    anyhow::bail!("state op is expected to be string");
                }
                let state_op = StateOp::from_str(state_op.as_str().unwrap())?;
                body_parsed.push((condition, expressions, state_op));
            }
            
            let state = State::new(default_next, head, body_parsed);
            states.insert(state_name, state);
        }

        if initial_state.is_none() {
            return Err(anyhow::anyhow!("no states defined!"));
        }
        let initial_state = initial_state.unwrap();

        Ok(Self::new(states, initial_state))
    }


    fn get_current_state(&self) -> anyhow::Result<&State> {
        let state_name = match self.current_state.as_ref() {
            Some(s) => s,
            None => { return Err(anyhow::anyhow!("no current state! state machine is ended.")); },
        };

        let state = self.states.get(state_name);

        match state {
            None => Err(anyhow::anyhow!("state {state_name} not found")),
            Some(state) => Ok(state),
        }
    }

    // Use `DeserializeOwned` instead `Deserialize` and dealing with lifetime issues
    // https://users.rust-lang.org/t/lifetime-confusion-with-function-parameter-serde-deserialize/76842
    pub fn run<I: Serialize, O: DeserializeOwned>(&mut self, i: I) -> anyhow::Result<Option<O>> {
        let (rv, state_op) = match &self.current_state {
            Some(current_state) => {
                let i = serde_json::to_string(&i)?;
                let i = json::parse(&i)?;
                let state = self.get_current_state()?; // might need to look at RefCell for this, need
                                                       // immutable access to the state but don't
                                                       // want to have to clone it
                let (o, state_op) = state.run(&i, &mut self.globals)?;
                let o = o.to_string();
                let o: O = serde_json::from_str(&o)?;
                (Some(o), state_op)
            },
            None => {
                (None, StateOp::Stay)
            }
        };

        match state_op {
            StateOp::Stay => {},
            StateOp::End => { self.current_state = None; },
            StateOp::ChangeTo(state_name) => { self.current_state.insert(state_name); },
        }

        Ok(rv)
    }
}
