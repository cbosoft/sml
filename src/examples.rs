//! # Examples
//!
//!
//! Compile SML source into a [StateMachine] and run it.
//! ```
//! use shakemyleg::compile;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize)]
//! struct Globals {
//!   foo: u8
//! }
//!
//! #[derive(Serialize)]
//! struct Inputs {
//!   bar: u8
//! }
//!
//! #[derive(Deserialize, PartialEq, Debug)]
//! struct Outputs {
//!   state: String,
//!   foo: u8,
//!   bar: u8,
//! }
//!
//! let src = r#"
//! default head:
//!   outputs.foo = globals.foo
//!
//! state first:
//!   head:
//!     outputs.state = "first"
//!   when inputs.bar < 10:
//!     outputs.bar = inputs.bar + 1
//!   otherwise:
//!     changeto second
//!   
//! state second:
//!   head:
//!     outputs.state = "second"
//!   when inputs.bar > 1:
//!     outputs.bar = inputs.bar - 1
//!   otherwise:
//!     changeto first
//! "#;
//!
//! let mut sm = compile(src).unwrap();
//! sm.reinit(Globals { foo: 1u8 }).unwrap();
//!
//! let i = Inputs { bar: 3u8 };
//! let o: Outputs = sm.run(i).unwrap().unwrap();
//! assert_eq!(o, Outputs { state: "first".to_string(), foo: 1u8, bar: 4u8 });
//! ```
//!
//! We can't define a list literal in SML, but we can interact with lists passed into the machine:
//! ```
//! use shakemyleg::compile;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize)]
//! struct Globals {
//!   things: Vec<String>
//! }
//!
//! #[derive(Serialize)]
//! struct Inputs {
//!   thing: String
//! }
//!
//! #[derive(Deserialize, PartialEq, Debug)]
//! struct Outputs {
//!   things: Vec<String>
//! }
//!
//! let src = r#"
//! state ThingAccumulator:
//!   head:
//!     globals.things = globals.things + inputs.thing
//!     outputs.things = globals.things
//!   when globals.things contains "lastthing":
//!     end
//! "#;
//!
//! let mut sm = compile(src).unwrap();
//! sm.reinit(Globals { things: Vec::new() });
//!
//! let i = Inputs { thing: "FirstThing".to_string() };
//! let o: Outputs = sm.run(i).unwrap().unwrap();
//! assert_eq!(o, Outputs { things: vec![ "FirstThing".to_string() ] });
//! ```
