use json::JsonValue;
use itertools::{Itertools, Position};

use crate::error::{SML_Error, SML_Result};
use crate::value::Value;


#[derive(Clone, Debug)]
pub enum IdentifierStore {
    Inputs,
    Outputs,
    Globals
}

#[derive(Clone, Debug)]
pub struct Identifier {
    store: IdentifierStore,
    name: String,
    path: Vec<String>,
}

impl Identifier {
    pub fn new(json: &JsonValue) -> SML_Result<Self> {
        let store = &json["store"];
        if !store.is_string() {
            return Err(SML_Error::JsonFormatError("Identifier missing `store` or is not correct type".to_string()));
        }
        let store = store.as_str().unwrap();
        let store = match store {
            "inputs" => IdentifierStore::Inputs,
            "outputs" => IdentifierStore::Outputs,
            "globals" => IdentifierStore::Globals,
            s => { return Err(SML_Error::JsonFormatError(format!("Identifier `store` got unexpected value {s:?}. "))); }
        };


        let name = &json["name"];
        if !name.is_string() {
            return Err(SML_Error::JsonFormatError("Identifier missing `name` or is not correct type".to_string()));
        }
        let name = name.as_str().unwrap().to_string();
        let path: Vec<_> = name.split(".").map(|s| s.to_string()).collect();

        if path.is_empty() {
            return Err(SML_Error::JsonFormatError(format!("Zero-length identifier: {name}")));
        }

        Ok(Self { store, name, path })
    }

    pub fn get(&self, i: &JsonValue, o: &JsonValue, g: &JsonValue) -> SML_Result<Value> {
        let mut store = match self.store {
            IdentifierStore::Inputs => i,
            IdentifierStore::Outputs => o,
            IdentifierStore::Globals => g,
        };

        for node in &self.path {
            if !store.is_object() || !store.has_key(node) {
                return Err(SML_Error::IdentifierNameError(self.name.clone()));
            }

            store = &store[node];
        }

        let value = Value::new(store)?;

        Ok(value)
    }

    pub fn set(&self, o: &mut JsonValue, g: &mut JsonValue, v: &Value) -> SML_Result<()> {
        let mut store = match self.store {
            IdentifierStore::Inputs => { return Err(SML_Error::InputsWriteError); },
            IdentifierStore::Outputs => o,
            IdentifierStore::Globals => g,
        };

        let mut key = None;
        for (pos, node) in self.path.iter().with_position() {
            if let Position::Last | Position::Only = pos {
                key = Some(node);
                break;
            }
            if !store.is_object() {
                if store.has_key(node) {
                    return Err(SML_Error::IdentifierError(format!("Cannot set sub-value of non-object. Identifier \"{}\" collides with another variable.", self.name)));
                }
                else {
                    store[node] = json::object! { };
                }
            }

            store = &mut store[node];
        }

        let json_value = match v {
            Value::Bool(b) => JsonValue::Boolean(*b),
            Value::String(s) => JsonValue::String(s.to_string()),
            Value::Number(n) => JsonValue::Number((*n).into()),
        };

        store[key.unwrap()] = json_value;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_set() {
        let mut g = json::object! { };
        let mut o = json::object! { };
        let ident = json::object! { store: "outputs", name: "foo.bar" };
        let ident = Identifier::new(&ident).unwrap();
        let v = Value::Number(1.0);
        ident.set(&mut o, &mut g, &v).unwrap();

        assert!(g.is_empty());
        assert!(!o.is_empty());
        assert!(o.has_key("foo"));
        assert!(o["foo"].has_key("bar"));
        assert!(matches!(o["foo"]["bar"], JsonValue::Number(_)));
    }
}
