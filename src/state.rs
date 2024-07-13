use json::JsonValue;

use crate::expression::Expression;


#[derive(Debug)]
pub enum StateOp {
    Stay,
    ChangeTo(String),
    End
}


impl StateOp {
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        let rv = if let Some(s) = s.strip_prefix("changeto ") {
            Self::ChangeTo(s.to_string())
        }
        else {
            match s {
                "stay" => Self::Stay,
                "end" => Self::End,
                s => anyhow::bail!("unexpected stateop {s:?}")
            }
        };

        Ok(rv)
    }
}


pub struct State {
    /// Name of state which is the default successor to this one
    default_next: Option<String>,

    /// Expressions evaluated when this state is visited
    head: Vec<Expression>,

    /// List of condition expressions and associated expressions.
    /// When the condition expression is true, the associated body of expressions is run.
    body: Vec<(Expression, Vec<Expression>, StateOp)>,
}

impl State {
    pub fn new(default_next: Option<String>, head: Vec<Expression>, body: Vec<(Expression, Vec<Expression>, StateOp)>) -> Self {
        Self { default_next, head, body }
    }

    // return some output if not ended
    pub fn run(&self, i: &JsonValue, g: &mut JsonValue) -> anyhow::Result<(JsonValue, StateOp)> {
        let mut o = json::object! { };

        for expr in &self.head {
            expr.evaluate(i, &mut o, g)?;
        }

        todo!();
    }
}
