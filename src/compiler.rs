use std::collections::HashMap;

use crate::error::{SML_Error, SML_Result};
use crate::expression::Expression;
use crate::state::{State, StateOp};
use crate::value::Value;
use crate::StateMachine;
use crate::parse_expression::expr_from_str;


enum CompileState {
    TopLevel, // "state <name>:" or "default head:"
    State,
    StateHead,
    StateBranch,
    DefaultHead,
}


struct StateData {
    pub name: String,
    pub head: Vec<Expression>,
    pub branches: Vec<StateBranchData>,
    pub has_default: bool,
    pub has_otherwise: bool,
    pub has_always: bool,
}

impl StateData {
    fn new(name: String) -> Self {
        Self {
            name,
            head: Vec::new(),
            branches: Vec::new(),
            has_default: false,
            has_otherwise: false,
            has_always: false,
        }
    }
}

impl TryFrom<StateData> for State {
    type Error = SML_Error;

    fn try_from(state_data: StateData) -> Result<Self, Self::Error> {
        let name = state_data.name;
        let head = state_data.head;
        let default_branch = if state_data.has_default {
            let mut idx = state_data.branches.len();
            for (i, branch) in state_data.branches.iter().enumerate() {
                if branch.is_default {
                    idx = i;
                    break;
                }
            }

            // if idx still at initial value, no default branches were found.
            if idx >= state_data.branches.len() {
                panic!("state claimed to have default, but no default branches found!");
            }
            else {
                Some(idx)
            }
        }
        else {
            None
        };
        let body = state_data.branches.into_iter().map(|b| (b.condition, b.body, b.state_op)).collect();
        let mut rv = State::new(name, head, body);
        if let Some(idx) = default_branch {
            rv.set_default(idx)?;
        }
        Ok(rv)
    }
}

struct StateBranchData {
    condition: Expression,
    body: Vec<Expression>,
    state_op: StateOp,
    is_default: bool,
}

impl StateBranchData {
    fn new(condition: Expression) -> Self {
        Self {
            condition,
            body: Vec::new(),
            state_op: StateOp::Stay,
            is_default: false,
        }
    }
}


/// Take a string of SML source and compile to state machine.
/// ```
/// use shakemyleg::compile;
///
/// let src = r#"
/// state init:
///   when inputs.b <= 10:
///     outputs.b = inputs.b + 1
///   otherwise:
///     changeto second
/// state second:
///   always:
///     outputs.c = inputs.c + 2
/// "#;
///
/// let sm = compile(src).unwrap();
/// ```
pub fn compile(s: &str) -> SML_Result<StateMachine> {
    let mut c_state_stack = vec![CompileState::TopLevel];
    let lines: Vec<_> = s.lines().collect();
    let mut i = 0usize;
    let mut state_data: Option<StateData> = None;
    let mut state_branch_data: Option<StateBranchData> = None;
    let mut default_head = Vec::new();
    let mut states: Vec<State> = Vec::new();
    let mut leading_ws = None;

    let nlines = lines.len();
    while i < nlines {
        let line = lines[i];
        let cstate = c_state_stack.last().unwrap();

        if line.trim_start().starts_with("#") {
            i += 1;
            continue;
        }

        let line_empty = line.trim() == "";

        if leading_ws.is_none() && matches!(cstate, CompileState::DefaultHead | CompileState::State) {
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
                        state_data = Some(StateData::new(sname.to_string()));
                        c_state_stack.push(CompileState::State);
                        true
                    }
                    else {
                        return Err(SML_Error::SyntaxError(format!("State definition with no name on line {i}")));
                    }
                }
                else if line == "default head:" {
                    c_state_stack.push(CompileState::DefaultHead);
                    true
                }
                else if !line.is_empty() {
                    return Err(SML_Error::SyntaxError(format!("Unexpected value {line} on line {i}")));
                }
                else {
                    true
                }
            },
            CompileState::State => {
                if line.starts_with(&leading_ws.as_ref().unwrap().1) {
                    return Err(SML_Error::SyntaxError(format!("Unexpected indent on line {i}")));
                }
                else if line.starts_with(&leading_ws.as_ref().unwrap().0) {
                    if !line_empty {
                        let line_trim = line.trim_start();
                        if line_trim == "head:" {
                            c_state_stack.push(CompileState::StateHead);
                        }
                        else if let Some(expr_colon) = line_trim.strip_prefix("when ") {
                            let has_always = state_data.as_ref().unwrap().has_always;
                            let has_otherwise = state_data.as_ref().unwrap().has_otherwise;
                            if has_always || has_otherwise {
                                return Err(SML_Error::SyntaxError(format!("Branch defined after always or otherwise on line {i}")));
                            }

                            if let Some(expr) = expr_colon.strip_suffix(":") {
                                let cond = expr_from_str(expr, i)?;
                                state_branch_data = Some(StateBranchData::new(cond));
                                c_state_stack.push(CompileState::StateBranch);
                            }
                            else {
                                return Err(SML_Error::SyntaxError(format!("Missing colon on line {i}:{line}")));
                            }
                        }
                        else if line_trim == "always:" {
                            let has_always = state_data.as_ref().unwrap().has_always;
                            let has_otherwise = state_data.as_ref().unwrap().has_otherwise;
                            if has_always || has_otherwise {
                                return Err(SML_Error::SyntaxError(format!("Branch defined after always or otherwise on line {i}.")));
                            }

                            let has_other_branches = state_data.as_ref().unwrap().branches.len() > 0;
                            if has_other_branches {
                                return Err(SML_Error::SyntaxError(format!("Always defined after another branch on line {i}. Always must be the only branch.")));
                            }

                            let cond = Expression::Value(Value::Bool(true));
                            state_branch_data = Some(StateBranchData::new(cond));
                            state_data.as_mut().unwrap().has_always = true;
                            c_state_stack.push(CompileState::StateBranch);
                        }
                        else if line_trim == "normally:" {
                            let has_always = state_data.as_ref().unwrap().has_always;
                            let has_otherwise = state_data.as_ref().unwrap().has_otherwise;
                            if has_always || has_otherwise {
                                return Err(SML_Error::SyntaxError(format!("Branch defined after always or otherwise on line {i}.")));
                            }

                            let has_other_branches = state_data.as_ref().unwrap().branches.len() > 0;
                            if has_other_branches {
                                return Err(SML_Error::SyntaxError(format!("Normally defined after another branch on line {i}. Normally must be the first branch.")));
                            }

                            let cond = Expression::Value(Value::Bool(true));
                            state_branch_data = Some(StateBranchData::new(cond));
                            c_state_stack.push(CompileState::StateBranch);
                        }
                        else if line_trim == "otherwise:" {
                            let has_always = state_data.as_ref().unwrap().has_always;
                            let has_otherwise = state_data.as_ref().unwrap().has_otherwise;
                            if has_always || has_otherwise {
                                return Err(SML_Error::SyntaxError(format!("Branch defined after always or otherwise on line {i}.")));
                            }

                            let has_other_branches = state_data.as_ref().unwrap().branches.len() > 0;
                            if !has_other_branches {
                                return Err(SML_Error::SyntaxError(format!("Otherwise defined alone on line {i}. Otherwise must come after at least one other branch.")));
                            }

                            let cond = Expression::Value(Value::Bool(true));
                            state_branch_data = Some(StateBranchData::new(cond));
                            state_data.as_mut().unwrap().has_otherwise = true;
                            c_state_stack.push(CompileState::StateBranch);
                        }
                        else {
                            eprintln!("{}", lines[i-1]);
                            return Err(SML_Error::SyntaxError(format!("Expected ['head:', 'when <state>:', 'always:', 'otherwise:'] after state intro on line {i}:{line}")));
                        }
                    }
                    true
                }
                else {
                    if let Some(state_data) = state_data.take() {
                        states.push(state_data.try_into()?);
                        c_state_stack.pop();
                        false
                    }
                    else {
                        return Err(SML_Error::SyntaxError(format!("Unexpected de-dent on line {i}")));
                    }
                }
            },
            CompileState::DefaultHead => {
                if line.starts_with(leading_ws.as_ref().unwrap().0) {
                    let line = line.trim_start();
                    let expr = expr_from_str(line, i)?;
                    default_head.push(expr);
                    true
                }
                else {
                    // TODO warn if empty head?
                    c_state_stack.pop();
                    false
                }
            },
            CompileState::StateHead => {
                if line.starts_with(&leading_ws.as_ref().unwrap().1) {
                    let line = line.trim_start();
                    let expr = expr_from_str(line, i)?;
                    state_data.as_mut().unwrap().head.push(expr);
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
                        state_branch_data.as_mut().unwrap().state_op = StateOp::ChangeTo(state_name.to_string());
                    }
                    else if line == "end" {
                        state_branch_data.as_mut().unwrap().state_op = StateOp::End;
                    }
                    else if line == "stay" {
                        state_branch_data.as_mut().unwrap().state_op = StateOp::Stay;
                    }
                    else if line == "default" {
                        if state_data.as_ref().unwrap().has_default {
                            let name = &state_data.as_ref().unwrap().name;
                            return Err(SML_Error::SyntaxError(format!("Multiple branches marked as default in state {name}. On line {i}.")));
                        }
                        else {
                            state_branch_data.as_mut().unwrap().is_default = true;
                            state_data.as_mut().unwrap().has_default = true;
                        }
                    }
                    else {
                        let expr = expr_from_str(line, i)?;
                        state_branch_data.as_mut().unwrap().body.push(expr);
                    }
                    true
                }
                else {
                    let branch = state_branch_data.take().unwrap();
                    state_data.as_mut().unwrap().branches.push(branch);
                    c_state_stack.pop();
                    false
                }
            }
        };

        if adv {
            i += 1;
        }
    }

    if let Some(branch) = state_branch_data {
        state_data.as_mut().unwrap().branches.push(branch);
    }

    if let Some(state_data) = state_data {
        states.push(state_data.try_into()?);
    }

    let initial_state = states[0].name().clone();
    let states_iter = states.into_iter();
    let mut states = HashMap::new();
    for state in states_iter {
        states.insert(state.name().clone(), Box::new(state));
    }
    let initial_state = states.get(&initial_state).unwrap().clone();

    Ok(StateMachine::new(
        default_head,
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
    fn test_compile_1() {
        const SRC: &'static str = r#"
state A:
    when true:
        outputs.bar = inputs.bar+1
        changeto B
state B:
    when true:
        outputs.bar = inputs.bar + 1
        changeto A
"#;
        let mut sm = compile(SRC).unwrap();

        eprintln!("{:?}", sm);

        let i = Foo { bar: 0 };
        let o: Foo = sm.run(i).unwrap().unwrap();
        assert_eq!(o.bar, 1u64);
        assert_eq!(sm.current_state().unwrap(), "B".to_string());
    }

    #[test]
    #[should_panic]
    fn test_compile_parens_1() {
        const SRC: &'static str = r#"
state A:
    when true:
        outputs.bar = (inputs.bar+1)*2
        changeto B
state B:
    when true:
        outputs.bar = (inputs.bar + 1
        changeto A
"#;
        let _ = compile(SRC).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_compile_parens_2() {
        const SRC: &'static str = r#"
state A:
    when true:
        outputs.bar = (inputs.bar+1)*2
        changeto B
state B:
    when true:
        outputs.bar = inputs.bar + 1)
        changeto A
"#;
        let _ = compile(SRC).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_compile_quotes_1() {
        const SRC: &'static str = r#"
state A:
    when true:
        outputs.bar = (inputs.bar+1)*2
        changeto B
state B:
    when true:
        outputs.bar = inputs.bar + 1
        outputs.foo = "foo bar baz
        changeto A
"#;
        let _ = compile(SRC).unwrap();
    }

    #[test]
    fn test_compile_always_otherwise_1() {
        const SRC: &'static str = r#"
state A:
    always:
        changeto B
state B:
    when false:
        changeto A
    otherwise:
        changeto A
"#;
        let _ = compile(SRC).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_compile_always_otherwise_2() {
        const SRC: &'static str = r#"
state A:
    always:
        changeto B
    otherwise:
        changeto A
state B:
    always:
        changeto A
"#;
        let _ = compile(SRC).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_compile_always_otherwise_3() {
        const SRC: &'static str = r#"
state A:
    otherwise:
        changeto B
    when true:
        changeto A
state B:
    always:
        changeto A
"#;
        let _ = compile(SRC).unwrap();
    }

    #[test]
    fn test_compile_always_otherwise_4() {
        const SRC: &'static str = r#"
state A:
    when false:
        changeto A
    otherwise:
        changeto B
state B:
    always:
        changeto A
"#;
        let _ = compile(SRC).unwrap();
    }

    #[derive(Serialize)]
    struct InFoo {
        foo: Vec<u8>
    }

    #[derive(Deserialize)]
    struct OutBar {
        bar: u8
    }

    #[test]
    fn test_compile_end() {
        const SRC: &'static str = r#"
state final:
    always:
        outputs.bar = 1
        end
"#;
        let mut sm = compile(SRC).unwrap();

        let i = InFoo { foo: vec![0u8] };
        let o: OutBar = sm.run(i).unwrap().unwrap();
        assert_eq!(o.bar, 1u8);

        let i = InFoo { foo: vec![0u8] };
        let rv: SML_Result<Option<OutBar>> = sm.run(i);
        assert!(matches!(rv, Ok(None)));
    }

    #[test]
    fn test_compile_contains_1() {
        const SRC: &'static str = r#"
state final:
    when inputs.foo ^= 0:
        outputs.bar = 1
    otherwise:
        outputs.bar = 0
"#;
        let mut sm = compile(SRC).unwrap();

        let i = InFoo { foo: vec![0, 1, 2, 3] };
        let o: OutBar = sm.run(i).unwrap().unwrap();
        assert_eq!(o.bar, 1u8);
        
        let i = InFoo { foo: vec![1, 2, 3] };
        let o: OutBar = sm.run(i).unwrap().unwrap();
        assert_eq!(o.bar, 0u8);
    }

    #[test]
    #[should_panic]
    fn test_compile_contains_2() {
        const SRC: &'static str = r#"
state final:
    when outputs.bar ^= 0:
        outputs.bar = 1
    otherwise:
        outputs.bar = 0
"#;
        let mut sm = compile(SRC).unwrap();

        let i = InFoo { foo: vec![0, 1, 2, 3] };
        let o: OutBar = sm.run(i).unwrap().unwrap();
        assert_eq!(o.bar, 1u8);
    }

}
