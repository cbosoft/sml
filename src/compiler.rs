use std::{collections::HashMap, rc::Rc};

use crate::{error::{SML_Error, SML_Result}, expression::Expression, identifier::Identifier, operation::BinaryOperation, state::{State, StateOp}, value::Value, StateMachine};

// Algorithm from: https://faculty.cs.niu.edu/~hutchins/csci241/eval.htm

enum CompileState {
    TopLevel, // "state <name>:" or "default head:"
    State,
    StateHead,
    StateBranch,
    DefaultHead,
}

#[derive(Debug, PartialEq)]
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

    pub fn precedence(&self) -> u8 {
        match self {
            Token::OpenParens | Token::CloseParens => 0,
            Token::Operator(op) => {
                match op.as_str() {
                    "*" | "/" | "^" => 1,
                    "+" | "-" => 2,
                    "==" | "<" | "<=" | ">" | ">=" => 3,
                    "&&" | "||" => 3,
                    "=" => 4,
                    _ => 4,
                }
            },
            Token::Identifier(_) | Token::Boolean(_) | Token::Number(_) | Token::String(_) => 5
        }
    }
}


/// Convert an expression string into a list of tokens.
/// "a = 1+1" -> [a, =, 1, +, 1]
fn tokenise(s: &str, lineno: usize) -> SML_Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote_stack = Vec::new();
    let mut pc = ' ';
    
    fn create_new_token(token: &mut String, tokens: &mut Vec<Token>) {
        // only create new token if current is not empty
        if !token.is_empty() {
            let token = std::mem::take(token);
            let token = Token::from_string(token);
            tokens.push(token);
        }
    }

    for (col, c) in s.chars().enumerate() {
        match (quote_stack.is_empty(), pc, c) {
            (true, _, ' ' | '\t') => {
                create_new_token(&mut current, &mut tokens);
            }
            (_, '\\', '\\') => { current.push(c); },
            (_, _, '\\') => {}, // escape char; wait to see what's next
            (_, '\\', '"' | '\'') => { current.push(c); },
            (_, _, '"' | '\'') => { 
                current.push(c);
                if quote_stack.is_empty() {
                    quote_stack.push((c, col));
                }
                else if quote_stack.last().unwrap().0 == c {
                    let _ = quote_stack.pop();
                    if quote_stack.is_empty() {
                        // that quote closes the token!
                        create_new_token(&mut current, &mut tokens);
                    }
                }
                else {
                    quote_stack.push((c, col));
                }
            },
            (true, _, '+' | '-' | '*' | '/' | '^' | '(' | ')') => {
                create_new_token(&mut current, &mut tokens);
                current.push(c);
                create_new_token(&mut current, &mut tokens);
            }
            // TODO: what if no whitespace between 2-char operators?
            _ => {
                current.push(c);
            }
        }
        pc = c;
    }

    if !quote_stack.is_empty() {
        let cols: Vec<_> = quote_stack.iter().map(|qi| { format!("{}", qi.1) }).collect();
        let s = if cols.len() > 1 { "s" } else { "" };
        let cols = cols.join(", ");
        return Err(SML_Error::SyntaxError(format!("Unmatched quote on line {lineno}, col{s} {cols}.")));
    }

    if !current.is_empty() {
        let token = Token::from_string(current);
        tokens.push(token);
    }

    Ok(tokens)
}


fn expr_from_str(s: &str, lineno: usize) -> SML_Result<Expression> {
    let infix = tokenise(s, lineno)?;
    let mut postfix = Vec::new();
    let mut stack = Vec::new();

    // infix -> postfix
    for token in infix.into_iter() {
        match &token {
            Token::Number(_) | Token::Identifier(_) | Token::String(_) | Token::Boolean(_) => { postfix.push(token); },
            Token::OpenParens => { stack.push(token); },
            Token::CloseParens => { 
                if stack.is_empty() {
                    return Err(SML_Error::SyntaxError("unexpected right parens.".to_string()));
                }
                while !stack.is_empty() && !matches!(stack.last().unwrap(), Token::OpenParens) {
                    postfix.push(stack.pop().unwrap());
                }
                if !matches!(stack.last().unwrap(), Token::OpenParens) {
                    return Err(SML_Error::SyntaxError("unexpected right parens.".to_string()));
                }
                let _ = stack.pop().unwrap();
            },
            Token::Operator(_) => {
                if stack.is_empty() || matches!(stack.last().unwrap(), Token::OpenParens) {
                    stack.push(token);
                }
                else {
                    while !stack.is_empty() && !matches!(stack.last().unwrap(), Token::OpenParens) && (stack.last().unwrap().precedence() <= token.precedence()) {
                        postfix.push(stack.pop().unwrap());
                    }
                    stack.push(token);
                }
            }
        }
    }

    for token in stack.into_iter().rev() {
        match token {
            Token::OpenParens => {
                return Err(SML_Error::SyntaxError("unbalanced parens.".to_string()));
            },
            _ => { postfix.push(token); }
        }
    }

    // postfix -> call tree
    let mut exp_stack = Vec::new();
    for token in postfix.into_iter() {
        match token {
            Token::Number(v) => { let expr = Expression::Value(Value::Number(v)); exp_stack.push(expr); }
            Token::String(s) => { let expr = Expression::Value(Value::String(s)); exp_stack.push(expr); }
            Token::Boolean(b) => { let expr = Expression::Value(Value::Bool(b)); exp_stack.push(expr); }
            Token::Identifier(i) => { let expr = Expression::Identifier(Identifier::from_str(i)?); exp_stack.push(expr); }
            Token::OpenParens | Token::CloseParens => { eprintln!("{token:?}") },
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


/// Take a string of SML source and compile to state machine.
/// ```
/// use shakemyleg::compile;
///
/// let src = r#"
/// state init:
///   when inputs.b < 10:
///     outputs.b = inputs.b + 1
///   when inputs.b >= 10:
///     changeto second
/// state second:
///   when true:
///     outputs.c = inputs.c + 2
/// "#;
///
/// let sm = compile(src).unwrap();
/// ```
pub fn compile(s: &str) -> SML_Result<StateMachine> {
    let mut c_state_stack = vec![CompileState::TopLevel];
    let lines: Vec<_> = s.lines().collect();
    let mut i = 0usize;
    let mut state_data = None;
    let mut state_branch_data = None;
    let mut default_head = Vec::new();
    let mut states = Vec::new();
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
                        state_data = Some((sname.to_string(), Vec::new(), Vec::new()));
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
                            if let Some(expr) = expr_colon.strip_suffix(":") {
                                let cond = expr_from_str(expr, i)?;
                                state_branch_data = Some((cond, Vec::new(), StateOp::Stay));
                                c_state_stack.push(CompileState::StateBranch);
                            }
                            else {
                                return Err(SML_Error::SyntaxError(format!("Missing colon on line {i}:{line}")));
                            }
                        }
                        else {
                            eprintln!("{}", lines[i-1]);
                            return Err(SML_Error::SyntaxError(format!("Expect head or when after state intro on line {i}:{line}")));
                        }
                    }
                    true
                }
                else {
                    if let Some((name, head, body)) = state_data.take() {
                        states.push(State::new(name, head, body));
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
                        let expr = expr_from_str(line, i)?;
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
        }
    }

    if let Some((name, head, body)) = state_data {
        states.push(State::new(name, head, body));
    }

    let initial_state = states[0].name().clone();
    let states_iter = states.into_iter();
    let mut states = HashMap::new();
    for state in states_iter {
        states.insert(state.name().clone(), Rc::new(state));
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
    fn test_tokenise_1() {
        let input = "a = 1+1";
        let expected_output = vec![
            Token::Identifier("a".to_string()),
            Token::Operator("=".to_string()),
            Token::Number(1f64),
            Token::Operator("+".to_string()),
            Token::Number(1f64),
        ];
        let output = tokenise(input, 0).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_tokenise_2() {
        let input = r#"a = "1 + \"1\"""#;
        let expected_output = vec![
            Token::Identifier("a".to_string()),
            Token::Operator("=".to_string()),
            Token::String("\"1 + \"1\"\"".to_string()),
        ];
        let output = tokenise(input, 0).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_tokenise_3() {
        let input = r#"a = 1 == b"#;
        let expected_output = vec![
            Token::Identifier("a".to_string()),
            Token::Operator("=".to_string()),
            Token::Number(1f64),
            Token::Operator("==".to_string()),
            Token::Identifier("b".to_string()),
        ];
        let output = tokenise(input, 0).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_tokenise_4() {
        let input = r#"a = (1 + b)*3"#;
        let expected_output = vec![
            Token::Identifier("a".to_string()),
            Token::Operator("=".to_string()),
            Token::OpenParens,
            Token::Number(1f64),
            Token::Operator("+".to_string()),
            Token::Identifier("b".to_string()),
            Token::CloseParens,
            Token::Operator("*".to_string()),
            Token::Number(3f64),
        ];
        let output = tokenise(input, 0).unwrap();
        assert_eq!(output, expected_output);
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
        assert_eq!(sm.current_state(), "B".to_string());
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

}
