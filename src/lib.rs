use std::collections::HashMap;

use anyhow::anyhow;
use json;
use json::JsonValue;


pub struct Condition {
    json: JsonValue
}

impl Condition {
    pub fn new(json: JsonValue) -> Self {
        Self { json }
    }
}


pub struct Expression {
    json: JsonValue
}

impl Expression {
    pub fn new(json: JsonValue) -> Self {
        Self { json }
    }
}


pub struct State {
    default: String,
    head: Vec<Expression>,
    body: Vec<(Condition, Vec<Expression>)>
}

impl State {
    pub fn new(default: String, head: Vec<Expression>, body: Vec<(Condition, Vec<Expression>)>) -> Self {
        Self { default, head, body }
    }
}


pub struct SM /* <InputType, OutputType> */ {
    states: HashMap<String, State>,
    current_state: String,
}

impl SM {
    pub fn new(src: &str) -> anyhow::Result<Self> {
        let json = json::parse(src)?;
        if !json.is_object() {
            return Err(anyhow::anyhow!("json expected to be object"));
        }

        let mut current_state = None;
        let mut states = HashMap::new();
        for (state_name, state_data) in json.entries() {
            let state_name = state_name.to_string();
            if current_state.is_none() {
                current_state = Some(state_name.clone());
            }
            let default = state_data["default"].as_str().unwrap().to_string();

            let head = &state_data["head"];
            if !head.is_array() {
                return Err(anyhow::anyhow!("head expected to be array"));
            }
            let head: Vec<_> = head.members().map(|j| Expression::new(j.clone())).collect();

            let body = &state_data["body"];
            if !body.is_array() {
                return Err(anyhow::anyhow!("body expected to be object"));
            }
            let mut body_parsed = Vec::new();
            for item in body.members() {
                let condition = item["condition"];
                let condition = Condition::new(condition);
                let expressions = item["expressions"];
                let expressions: Vec<_> = expressions.members().map(|j| Expression::new(j.clone())).collect();
                body_parsed.push((condition, expressions));
            }
            
            let state = State::new(default, head, body_parsed);
            states.insert(state_name, state);
        }

        if current_state.is_none() {
            return Err(anyhow::anyhow!("no states defined!"));
        }

        Ok(Self { states, current_state })
    }

    pub fn run_node(node: JsonValue) -> anyhow::Result<JsonValue> {

        if !node.is_object() {
            return Err(anyhow::anyhow!("node expected to be object"));
        }

        todo!();
    }

    pub fn run_condition(node: JsonValue) -> anyhow::Result<JsonValue> {
        let left = node["left"];
        let op = node["op"];
        let right = node["right"];

        // 
    }
}




pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
