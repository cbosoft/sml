# SML - ShakeMyLeg, is that a State Machine Language?

A simple state machine definition language and interpreter.

A state machine is composed of states - stages of the machine which are run until an exit condition is met and the machine moves to the next stage. State machines in `shakemyleg` are defined as a series of expressions which are run every time the machine runs - the "head". Along with the head is the body - a list of conditions which, when evaluate `true`, run a series of expressions and a `StateOp` (`changeto <state>`, `stay`, `end`). If no condition is true, no action is taken. Conditions are visited in order.

A very simple example `shakemyleg` machine:
```sml
# flip_flip.sml

state A:
    when true:
        outputs.bar = inputs.bar
        changeto B

state B:
    when true:
        outputs.bar = inputs.bar
        changeto A
```

This machine alternates between states A and B.

For interest, the above machine is "compiled" to JSON as:
```json
// flip_flop.json
{
    "globals": {},
    "states": [
        {
            "name": "A",
            "head": [],
            "body": [{
                "condition": { "kind": "value", "value": true },
                "expressions": [{
                    "kind": "binary op",
                    "operation": "=",
                    "left": { "kind": "identifier", value: { "store": "outputs", "name": "bar" } },
                    "right": { "kind": "identifier", value: { "store": "inputs", "name": "bar" } }
                }],
                "state op": "changeto B"
            }]
        },
        ...
    ],
}
```

Running the machine in Rust, at the moment, requires the intermediate JSON as the compiler still lives in my head.
```rust
use shakemyleg::StateMachine;
let machine = StateMachine::from_src("flip_flop.json").unwrap();
machine.run(); // error
```

Ah, actually. That's an error. We almost certainly want to pass data into and out from the state machine. We need to define a type we can serialise (input to the machine) and a type we can deserialize (out from the machine).

```rust
use shakemyleg::StateMachine;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Foo {
    bar: u8
}

let machine = StateMachine::from_src("flip_flop.json").unwrap();

// machine state is A initially (first one defined)

let rv: Foo = machine.run(Foo{ bar: 0u8 }).unwrap().unwrap();
// now machine state is B

```
