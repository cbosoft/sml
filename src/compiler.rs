use std::{collections::HashMap, rc::Rc};

use crate::{error::{SML_Error, SML_Result}, expression::Expression, state::{State, StateOp}, StateMachine, value::Value};


enum CompileState {
    TopLevel, // "state <name>:" or "globals:"
    State,
    StateHead,
    StateBranch,
    Globals,
}


fn expr_from_str(s: &str) -> SML_Result<Expression> {
    let _ = s;
    Ok(Expression::Value(Value::Bool(true)))
}


pub fn compile(s: &str) -> SML_Result<StateMachine> {
    let mut c_state_stack = vec![CompileState::TopLevel];
    let lines: Vec<_> = s.lines().collect();
    let mut i = 0usize;
    let mut state_data = None;
    let mut state_branch_data = None;
    let mut globals = Vec::new();
    let mut states = Vec::new();
    let mut leading_ws = None;

    loop {
        let line = lines[i];
        let cstate = c_state_stack.last().unwrap();

        if leading_ws.is_none() && matches!(cstate, CompileState::Globals | CompileState::State) {
            let line_no_ws = line.trim_start();
            let ws = line.strip_suffix(line_no_ws).unwrap();
            let ws2 = ws.to_string() + ws;
            leading_ws = Some((ws, ws2));
        }

        let adv = match cstate {
            CompileState::TopLevel => {
                // 
                if let Some(sname_colon) = line.strip_prefix("state ") {
                    if let Some(sname) = sname_colon.strip_suffix(":") {
                        state_data = Some((sname.to_string(), Vec::new(), Vec::new()));
                        c_state_stack.push(CompileState::State);
                        true
                    }
                    else {
                        true
                    }
                }
                else if line == "globals:" {
                    c_state_stack.push(CompileState::Globals);
                    true
                }
                else {
                    true
                }
            },
            CompileState::State => {
                if line.starts_with(&leading_ws.as_ref().unwrap().1) {
                    panic!();
                }
                else if line.starts_with(&leading_ws.as_ref().unwrap().0) {
                    let line = line.trim_start();
                    if line == "head:" {
                        c_state_stack.push(CompileState::StateHead);
                    }
                    else if let Some(expr_colon) = line.strip_prefix("when ") {
                        if let Some(expr) = expr_colon.strip_suffix(":") {
                            let cond = expr_from_str(expr)?;
                            state_branch_data = Some((cond, Vec::new(), StateOp::Stay));
                            c_state_stack.push(CompileState::StateBranch);
                        }
                        else {
                            panic!();
                        }
                    }
                    else {
                        panic!();
                    }
                    true
                }
                else {
                    let (name, head, body) = state_data.take().unwrap();
                    states.push(State::new(name, head, body));
                    c_state_stack.pop();
                    false
                }
            },
            CompileState::Globals => {
                if line.starts_with(leading_ws.as_ref().unwrap().0) {
                    let line = line.trim_start();
                    let expr = expr_from_str(line)?;
                    globals.push(expr);
                    true
                }
                else {
                    c_state_stack.pop();
                    false
                }
            },
            CompileState::StateHead => {
                if line.starts_with(&leading_ws.as_ref().unwrap().1) {
                    let line = line.trim_start();
                    let expr = expr_from_str(line)?;
                    state_data.as_mut().unwrap().1.push(expr);
                    true
                }
                else {
                    c_state_stack.pop();
                    false
                }
            },
            CompileState::StateBranch => {
                if line.starts_with(&leading_ws.as_ref().unwrap().1) {
                    let line = line.trim_start();
                    if let Some(state_name) = line.strip_prefix("changeto ") {
                        state_branch_data.as_mut().unwrap().2 = StateOp::ChangeTo(state_name.to_string());
                    }
                    else if line == "end" {
                        state_branch_data.as_mut().unwrap().2 = StateOp::End;
                    }
                    else if line == "stay" {
                        state_branch_data.as_mut().unwrap().2 = StateOp::Stay;
                    }
                    else {
                        let expr = expr_from_str(line)?;
                        state_branch_data.as_mut().unwrap().1.push(expr);
                    }
                    true
                }
                else {
                    let branch = state_branch_data.take().unwrap();
                    state_data.as_mut().unwrap().2.push(branch);
                    c_state_stack.pop();
                    false
                }
            }
        };

        if adv {
            i += 1;
            if i >= lines.len() {
                break;
            }
        }
    }

    let initial_state = states[0].name().clone();
    let states_iter = states.into_iter();
    let mut states = HashMap::new();
    for state in states_iter {
        states.insert(state.name().clone(), Rc::new(state));
    }
    let initial_state = states.get(&initial_state).unwrap().clone();

    Ok(StateMachine::new(
        json::object! { },
        states,
        initial_state,
    ))
}


#[cfg(test)]
mod tests {

    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    struct Foo {
        pub bar: u64,
    }

    #[test]
    fn test_compile() {
        const SRC: &'static str = r#"
state A:
    when true:
        outputs.bar = inputs.bar
        changeto B
state B:
    when true:
        outputs.bar = inputs.bar
        changetoA
"#;
        let mut sm = compile(SRC).unwrap();

        eprintln!("{:?}", sm);

        let i = Foo { bar: 0 };
        let o: Foo = sm.run(i).unwrap().unwrap();

    }

}
