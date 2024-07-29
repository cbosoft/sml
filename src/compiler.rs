use std::{collections::HashMap, rc::Rc};

use crate::{error::{SML_Error, SML_Result}, expression::Expression, identifier::Identifier, operation::BinaryOperation, state::{State, StateOp}, value::Value, StateMachine};


enum CompileState {
    TopLevel, // "state <name>:" or "globals:"
    State,
    StateHead,
    StateBranch,
    Globals,
}

enum Token {
    Identifier(String),
    Number(f64),
    String(String),
    Operator(String),
    Boolean(bool),
    OpenParens,
    CloseParens,
}

impl Token {
    pub fn from_string(s: String) -> Self {
        if s.starts_with("'") || s.starts_with("\"") {
            // TODO: remove quotes
            Self::String(s)
        }
        else if let Ok(v) = s.parse::<f64>() {
            Self::Number(v)
        }
        else if let "+" | "-" | "*" | "/" | "=" | "==" | "<" | "<=" | ">" | ">=" | "!=" = s.as_str() {
            Self::Operator(s)
        }
        else if s == "(" {
            Self::OpenParens
        }
        else if s == ")" {
            Self::CloseParens
        }
        else if s == "true" {
            Self::Boolean(true)
        }
        else if s == "false" {
            Self::Boolean(false)
        }
        else {
            Self::Identifier(s)
        }
    }
}

fn tokenise(s: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for c in s.chars() {
        match c {
            ' ' | '\t' => {
                if !current.is_empty() {
                    let token_src = std::mem::take(&mut current);
                    let token = Token::from_string(token_src);
                    tokens.push(token);
                }
            }
            // TODO: what if no whitespace between operators?
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        let token = Token::from_string(current);
        tokens.push(token);
    }

    tokens
}


fn expr_from_str(s: &str) -> SML_Result<Expression> {
    let infix = tokenise(s);
    let mut postfix = Vec::new();
    let mut stack = Vec::new();

    // infix -> postfix
    for token in infix.into_iter() {
        match &token {
            Token::Number(_) | Token::Identifier(_) | Token::String(_) | Token::Boolean(_) => { postfix.push(token); },
            Token::OpenParens => { stack.push(token); },
            Token::CloseParens => { 
                while !stack.is_empty() && !matches!(stack.last().unwrap(), Token::OpenParens) {
                    postfix.push(stack.pop().unwrap());
                }
            },
            Token::Operator(_) => {
                if stack.is_empty() || matches!(stack.last().unwrap(), Token::OpenParens) {
                    stack.push(token);
                }
                else {
                    while !stack.is_empty() && !matches!(stack.last().unwrap(), Token::OpenParens) /* TODO: operator precedence */ {
                        postfix.push(stack.pop().unwrap());
                    }
                    stack.push(token);
                }
            }
        }
    }

    for token in stack.into_iter().rev() {
        postfix.push(token);
    }

    // postfix -> call tree
    let mut exp_stack = Vec::new();
    for token in postfix.into_iter() {
        match token {
            Token::Number(v) => { let expr = Expression::Value(Value::Number(v)); exp_stack.push(expr); }
            Token::String(s) => { let expr = Expression::Value(Value::String(s)); exp_stack.push(expr); }
            Token::Boolean(b) => { let expr = Expression::Value(Value::Bool(b)); exp_stack.push(expr); }
            Token::Identifier(i) => { let expr = Expression::Identifier(Identifier::from_str(i)?); exp_stack.push(expr); }
            Token::OpenParens | Token::CloseParens => (),
            Token::Operator(op) => {
                let a = exp_stack.pop().unwrap();
                let b = exp_stack.pop().unwrap();
                let expr = Expression::Binary(BinaryOperation::from_str(op)?, Box::new(b), Box::new(a));
                exp_stack.push(expr);
            }
        }
    }

    // there should only be one value in the stack
    if exp_stack.len() > 1 {
        Err(SML_Error::SyntaxError("Too many expressions left!".to_string()))
    }
    else if exp_stack.len() == 0 {
        Err(SML_Error::SyntaxError("No expression!".to_string()))
    }
    else {
        Ok(exp_stack.pop().unwrap())
    }
}


pub fn compile(s: &str) -> SML_Result<StateMachine> {
    let mut c_state_stack = vec![CompileState::TopLevel];
    let lines = {let mut lines: Vec<_> = s.lines().collect(); lines.push(""); lines };
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
                        return Err(SML_Error::SyntaxError(format!("State definition with no name on line {i}")));
                    }
                }
                else if line == "globals:" {
                    c_state_stack.push(CompileState::Globals);
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
                    return Err(SML_Error::SyntaxError(format!("Unexpected de-dent on line {i}")));
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
                            return Err(SML_Error::SyntaxError(format!("Missing colon on line {i}")));
                        }
                    }
                    else {
                        return Err(SML_Error::SyntaxError(format!("Expect head or when after state intro on line {i}")));
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
        outputs.bar = ( inputs.bar + 1 )
        changeto B
state B:
    when true:
        outputs.bar = ( inputs.bar + 1 )
        changeto A
"#;
        let mut sm = compile(SRC).unwrap();

        eprintln!("{:?}", sm);

        let i = Foo { bar: 0 };
        let o: Foo = sm.run(i).unwrap().unwrap();
        assert_eq!(o.bar, 1u64);
        assert_eq!(sm.current_state(), "B".to_string());

    }

}
