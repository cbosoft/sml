use std::{collections::HashMap, fs::read_to_string};

use crate::StateMachine;


type Foo = HashMap<String, f64>;


#[test]
fn test_flip_flip() {
    let src = read_to_string("flip_flop.json").unwrap();
    let mut sm = StateMachine::from_src(&src).unwrap();

    let states = vec!["A", "B"];

    for j in 0..5 {
        let i: Foo = HashMap::new();
        let _: Option<Foo> = sm.run(i).unwrap();
        let expected_state = states[(j + 1) % 2];
        let actual_state = sm.current_state();
        assert_eq!(expected_state, &actual_state);
    }

}
