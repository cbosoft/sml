use std::collections::HashMap;

use serde::{Serialize, de::DeserializeOwned};
use json::JsonValue;

use crate::refcount::RefCount;
use crate::expression::Expression;
use crate::state::{State, StateOp};
use crate::error::{SML_Error, SML_Result};


type StateRef = RefCount<State>;


#[derive(Clone, Debug)]
pub struct StateMachine {
    globals: JsonValue,
    default_head: Vec<Expression>,
    states: HashMap<String, StateRef>,
    current_state: Option<StateRef>,
}


impl StateMachine {
    pub fn new(default_head: Vec<Expression>, states: HashMap<String, StateRef>, initial_state: StateRef) -> Self {
        let globals = json::object! { };
        let current_state = Some(RefCount::clone(&initial_state));
        Self { globals, default_head, states, current_state }
    }

    pub fn reinit<G: Serialize>(&mut self, g: G) -> SML_Result<()> {
        let s = serde_json::to_string(&g)?;
        let j = json::parse(&s)?;
        self.globals = j;
        Ok(())
    }

    fn get_state(&self, name: &String) -> SML_Result<StateRef> {
        match self.states.get(name) {
            Some(state) => Ok(RefCount::clone(state)),
            None => Err(SML_Error::NonexistantState(name.clone()))
        }
    }

    pub fn current_state(&self) -> String {
        self.current_state.as_ref().unwrap().name().clone()
    }

    pub fn run<I: Serialize, O: DeserializeOwned>(&mut self, i: I) -> SML_Result<Option<O>> {
        // Using `DeserializeOwned` instead of `Deserialize` and dealing with lifetime issues
        // https://users.rust-lang.org/t/lifetime-confusion-with-function-parameter-serde-deserialize/76842
        let (rv, state_op) = match &self.current_state {
            Some(current_state) => {
                let i = serde_json::to_string(&i)?;
                let i = json::parse(&i)?;
                let state = RefCount::clone(&current_state);
                let (o, state_op) = (*state).run(&i, &mut self.globals, &self.default_head)?;
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

    pub fn globals<G: DeserializeOwned>(&self) -> SML_Result<G> {
        let g = self.globals.to_string();
        let g: G = serde_json::from_str(&g)?;
        Ok(g)
    }
}

#[cfg(all(test, feature = "thread_safe"))]
mod thread_safety_tests {
    use super::StateMachine;

    fn is_send_sync<T: Send + Sync>() { }

    #[test]
    fn test_send_sync() {
        is_send_sync::<StateMachine>();
    }
}
