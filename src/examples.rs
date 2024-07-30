//! # Examples
//!
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
//!   when outputs.bar >= 10:
//!     changeto second
//!   
//! state second:
//!   head:
//!     outputs.state = "second"
//!   when inputs.bar > 1:
//!     outputs.bar = inputs.bar - 1
//!   when outputs.bar <= 1:
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
//!
