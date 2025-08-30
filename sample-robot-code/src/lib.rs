#[allow(warnings)]
mod bindings;

use bindings::exports::robot::Guest;

struct Component;

impl Guest for Component {
    fn setup() -> bindings::exports::robot::Configuration {
        todo!()
    }

    fn run() -> () {
        todo!()
    }
}

bindings::export!(Component with_types_in bindings);
