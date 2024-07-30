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
                s => Err(SML_Error::JsonFormatError(format!("Unexpected StateOp: {s:?}")))
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
}

impl State {
    pub fn new(name: String, head: Vec<Expression>, body: Vec<(Expression, Vec<Expression>, StateOp)>) -> Self {
        Self { name, head, body }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    // return some output if not ended
    pub fn run(&self, i: &JsonValue, g: &mut JsonValue, default_head: &Vec<Expression>) -> SML_Result<(JsonValue, StateOp)> {
        let mut o = json::object! { };

        for expr in default_head {
            expr.evaluate(i, &mut o, g)?;
        }

        for expr in &self.head {
            expr.evaluate(i, &mut o, g)?;
        }

        let mut state_op = StateOp::Stay;
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

        Ok((o, state_op))
    }
}
