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
    pub fn from_str(s: String) -> SML_Result<Self> {
        let parts: Vec<_> = s.split(".").collect();
        if parts.len() < 2 {
            return Err(SML_Error::SyntaxError(format!("Identifier must specify store location i.e., start with \"inputs.\", \"globals.\", or \"outputs.\". (Note full-stops.) Got {s:?}")));
        }

        let store = match parts[0] {
            "inputs" => IdentifierStore::Inputs,
            "outputs" => IdentifierStore::Outputs,
            "globals" => IdentifierStore::Globals,
            _ => {
                return Err(SML_Error::SyntaxError(format!("Identifier must specify store location i.e., start with \"inputs.\", \"globals.\", or \"outputs.\". Got: {:?}", parts[0])));
            }
        };

        let path: Vec<_> = parts[1..].iter().map(|s| { s.to_string() } ).collect();
        let name = path.join(".");

        Ok(Self { name, path, store })
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

        let json_value = v.as_json();
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
        let ident = Identifier::from_str("outputs.foo.bar".to_string()).unwrap();
        let v = Value::Number(1.0);
        ident.set(&mut o, &mut g, &v).unwrap();

        assert!(g.is_empty());
        assert!(!o.is_empty());
        assert!(o.has_key("foo"));
        assert!(o["foo"].has_key("bar"));
        assert!(matches!(o["foo"]["bar"], JsonValue::Number(_)));
    }
}
