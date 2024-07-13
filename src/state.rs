use json::JsonValue;

use crate::expression::Expression;


pub struct State {
    /// Name of state which is the default successor to this one
    default: String,

    /// Expressions evaluated when this state is visited
    head: Vec<Expression>,

    /// List of condition expressions and associated expressions.
    /// When the condition expression is true, the associated body of expressions is run.
    body: Vec<(Expression, Vec<Expression>)>,
}

impl State {
    pub fn new(default: String, head: Vec<Expression>, body: Vec<(Expression, Vec<Expression>)>) -> Self {
        Self { default, head, body }
    }

    pub fn run(&self, i: &JsonValue, g: &mut JsonValue) -> anyhow::Result<JsonValue> {
        let mut o = json::object! { };

        for expr in &self.head {
            expr.evaluate(i, &mut o, g)?;
        }

        todo!();


        Ok(o)
    }
}
