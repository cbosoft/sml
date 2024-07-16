use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use serde::{Serialize, de::DeserializeOwned};
use json::JsonValue;

use crate::expression::Expression;
use crate::state::{State, StateOp};
use crate::error::{SML_Error, SML_Result};


type StateRef = Rc<State>;


#[derive(Clone)]
pub struct StateMachine {
    globals: JsonValue,
    states: HashMap<String, StateRef>,
    current_state: Option<StateRef>,
}


impl StateMachine {
    pub fn new(globals: JsonValue, states: HashMap<String, StateRef>, initial_state: StateRef) -> Self {
        let current_state = Some(Rc::clone(&initial_state));
        Self { globals, states, current_state }
    }

    pub fn reinit<G: Serialize>(&mut self, g: G) -> SML_Result<()> {
        let s = serde_json::to_string(&g)?;
        let j = json::parse(&s)?;
        self.globals = j;
        Ok(())
    }

    pub fn from_file<P: Into<PathBuf>>(path: P) -> SML_Result<Self> {
        let path: PathBuf = path.into();
        let mut source = String::new();
        let mut f = File::options().read(true).open(path)?;
        let _ = f.read_to_string(&mut source)?;
        Self::from_src(&source)
    }

    pub fn from_src(src: &str) -> SML_Result<Self> {
        let machine_json = json::parse(src)?;
        let globals = machine_json["globals"].clone();


        let states_json = &machine_json["states"];
        if !states_json.is_array() {
            return Err(SML_Error::JsonFormatError("JSON .['states'] needs to be array".to_string()));
        }

        let mut initial_state = None;
        let mut states = HashMap::new();
        for state_data in states_json.members() {
            let state_name = &state_data["name"];
            if !state_name.is_string() {
                return Err(SML_Error::JsonFormatError(format!("JSON .['states'][]['state_name'] needs to be string, got {state_name:?}")));
            }
            let state_name = state_name.as_str().unwrap().to_string();

            if initial_state.is_none() {
                initial_state = Some(state_name.clone());
            }

            let head = &state_data["head"];
            if !head.is_array() {
                return Err(SML_Error::JsonFormatError(format!("JSON .['states'][]['head'] needs to be array of expressions.")));
            }
            let head: Result<Vec<_>, _> = head.members().map(|j| Expression::new(j)).collect();
            let head = head?;

            let body = &state_data["body"];
            if !body.is_array() {
                return Err(SML_Error::JsonFormatError(format!("JSON .['states'][]['body'] needs to be array.")));
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
                    return Err(SML_Error::JsonFormatError(format!("JSON .['states'][]['state op'] needs to be string.")));
                }
                let state_op = StateOp::from_str(state_op.as_str().unwrap())?;
                body_parsed.push((condition, expressions, state_op));
            }
            
            let state = State::new(state_name.clone(), head, body_parsed);
            let state = Rc::new(state);
            states.insert(state_name, state);
        }

        if initial_state.is_none() {
            return Err(SML_Error::JsonFormatError("JSON .['states'] is empty: no states defined!".to_string()));
        }
        let initial_state = initial_state.unwrap();
        let initial_state = Rc::clone(states.get(&initial_state).unwrap());
        Ok(Self::new(globals, states, initial_state))
    }

    fn get_state(&self, name: &String) -> SML_Result<StateRef> {
        match self.states.get(name) {
            Some(state) => Ok(Rc::clone(state)),
            None => Err(SML_Error::NonexistantState(name.clone()))
        }
    }

    pub fn current_state(&self) -> String {
        self.current_state.as_ref().unwrap().name().clone()
    }

    // Use `DeserializeOwned` instead `Deserialize` and dealing with lifetime issues
    // https://users.rust-lang.org/t/lifetime-confusion-with-function-parameter-serde-deserialize/76842
    pub fn run<I: Serialize, O: DeserializeOwned>(&mut self, i: I) -> SML_Result<Option<O>> {
        let (rv, state_op) = match &self.current_state {
            Some(current_state) => {
                let i = serde_json::to_string(&i)?;
                let i = json::parse(&i)?;
                let state = Rc::clone(&current_state);
                let (o, state_op) = (*state).run(&i, &mut self.globals)?;
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
            StateOp::ChangeTo(state_name) => {
                let state = self.get_state(&state_name)?;
                let _ = self.current_state.insert(state);
            },
        }

        Ok(rv)
    }
}
