![tests badge](https://github.com/cbosoft/sml/actions/workflows/tests.yml/badge.svg)
[![Written up - here!](https://img.shields.io/static/v1?label=Written+up&message=here!&color=2ea44f)](https://cmjb.tech/blog/2024/08/02/shakemyleg/)

# SML - ShakeMyLeg, is that a State Machine Language?

A simple state machine definition language and interpreter.

A state machine is composed of states - stages of the machine which are run until an exit condition is met and the machine moves to the next stage. State machines in `shakemyleg` are defined as a series of expressions which are run every time the machine runs - the "head". Along with the head is the body - a list of conditions which, when evaluate `true`, run a series of expressions and a `StateOp` (`changeto <state>`, `stay`, `end`). If no condition is true, no action is taken. Conditions are visited in order. Comments start with a `#`.

A very simple example `shakemyleg` machine:
```sml
# flip_flip.sml

state A:
    always:
        outputs.bar = inputs.bar + 1
        changeto B

state B:
    always:
        outputs.bar = inputs.bar + 1
        changeto A
```

(Liberal whitespace around operators and brackets is **required** because the compiler is dumb.) This machine alternates between states A and B, and propagates the value `bar` from the input object to the output, and increments it.

We can "compile" this and run it:
```rust
use shakemyleg::compile;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Foo {
    bar: u8
}

let src = r#"
state A:
    always:
        outputs.bar = inputs.bar + 1
        changeto B
state B:
    always:
        outputs.bar = inputs.bar + 1
        changeto A
"#;

let mut machine = compile(src).unwrap();

let i = Foo { bar: 0 };
let o: Foo = machine.run(i).unwrap().unwrap();
// Two unwraps as the rv is Result<Option<Foo>>
// Result<...> checks if any errors occurred while running
// Option<...> checks if the machine is still running

// output.bar is incremented every time the machine is run
if o.bar != 1u8 {
    panic!();
}
```
