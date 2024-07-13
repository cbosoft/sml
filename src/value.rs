use json::JsonValue;


#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
}

impl Value {
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap().to_string();
            Ok(Self::String(s))
        }
        else if json.is_number() {
            Ok(Self::Number(json.as_f64().unwrap()))
        }
        else if json.is_boolean() {
            Ok(Self::Bool(json.as_bool().unwrap()))
        }
        else {
            anyhow::bail!("Value expects a json number, string, or boolean. Got null, object, array, or empty.")
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Number(v) => *v != 0.0,
            Self::String(v) => v.is_empty(),
        }
    }
}
