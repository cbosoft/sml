use json::JsonValue;

use crate::error::{SML_Error, SML_Result};
use crate::expression::Expression;


#[derive(Clone, Debug)]
pub enum StateOp {
    Stay,
    ChangeTo(String),
    End
}


impl StateOp {
    pub fn from_str(s: &str) -> SML_Result<Self> {
        if let Some(s) = s.strip_prefix("changeto ") {
            Ok(Self::ChangeTo(s.to_string()))
        }
        else {
            match s {
                "stay" => Ok(Self::Stay),
                "end" => Ok(Self::End),
                s => Err(SML_Error::SyntaxError(format!("Unexpected StateOp: {s:?} (expected \"stay\", \"end\", or \"changeto <state>\".")))
            }
        }
    }
}


#[derive(Clone, Debug)]
pub struct State {
    name: String,

    /// Expressions evaluated when this state is visited
    head: Vec<Expression>,

    /// List of condition expressions and associated expressions.
    /// When the condition expression is true, the associated body of expressions is run.
    body: Vec<(Expression, Vec<Expression>, StateOp)>,

    default_branch: Option<usize>
}

pub type StateRef = Box<State>;

impl State {
    pub fn new(name: String, head: Vec<Expression>, body: Vec<(Expression, Vec<Expression>, StateOp)>) -> Self {
        Self { name, head, body, default_branch: None }
    }

    pub fn set_default(&mut self, i: usize) -> SML_Result<()> {
        if i >= self.body.len() {
            Err(SML_Error::CompilerError(format!("branch index out of range ({i} > {}) in {}", self.body.len(), self.name)))
        }
        else {
            Ok(self.default_branch = Some(i))
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn run(&self, i: &JsonValue, g: &mut JsonValue, default_head: &Vec<Expression>) -> SML_Result<(JsonValue, StateOp)> {
        self.run_or_advance(i, g, default_head, false)
    }
    
    pub fn run_default(&self, i: &JsonValue, g: &mut JsonValue, default_head: &Vec<Expression>) -> SML_Result<(JsonValue, StateOp)> {
        self.run_or_advance(i, g, default_head, true)
    }
    
    fn run_or_advance(&self, i: &JsonValue, g: &mut JsonValue, default_head: &Vec<Expression>, advance: bool) -> SML_Result<(JsonValue, StateOp)> {
        let mut o = json::object! { };

        for expr in default_head {
            expr.evaluate(i, &mut o, g)?;
        }

        for expr in &self.head {
            expr.evaluate(i, &mut o, g)?;
        }

        let mut state_op = StateOp::Stay;
        if advance {
            let (_, branch_body, branch_state_op) = &self.body[self.default_branch.unwrap()];
            for expr in branch_body {
                expr.evaluate(i, &mut o, g)?;
            }
            state_op = branch_state_op.clone();
        }
        else {
            for (cond, branch_body, branch_state_op) in &self.body {
                let v = cond.evaluate(i, &mut o, g)?;
                if v.as_bool() {
                    for expr in branch_body {
                        expr.evaluate(i, &mut o, g)?;
                    }
                    state_op = branch_state_op.clone();
                    break;
                }
            }
        }

        Ok((o, state_op))
    }
}
