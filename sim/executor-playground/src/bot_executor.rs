use execution_data::SimulationStepper;
use wasmtime::{Engine, Store};

use crate::{
    bindings::{LineFollowerRobot, exports::robot::Configuration},
    bot_wasm_host::BotHost,
};

pub struct BotExecutor<S: SimulationStepper + 'static> {
    #[allow(unused)]
    engine: Engine,
    #[allow(unused)]
    store: Store<BotHost<S>>,
    #[allow(unused)]
    component: wasmtime::component::Component,
    #[allow(unused)]
    linker: wasmtime::component::Linker<BotHost<S>>,
    #[allow(unused)]
    robot_component: LineFollowerRobot,

    robot_configuration: Configuration,
}

impl<S: SimulationStepper + 'static> BotExecutor<S> {
    pub fn new(wasm_bytes: &[u8], stepper: S) -> wasmtime::Result<Self> {
        // Create engine and store
        let engine = Engine::default();
        let mut store = wasmtime::Store::new(&engine, BotHost::new(stepper));

        // Instantiate component
        let component = wasmtime::component::Component::new(&engine, wasm_bytes)?;

        // Configure the linker
        let mut linker = wasmtime::component::Linker::new(&engine);

        // Ignore unknown imports
        linker.define_unknown_imports_as_traps(&component)?;

        // Instantiate component host
        let robot_component = LineFollowerRobot::instantiate(&mut store, &component, &linker)?;

        let robot_configuration = robot_component.robot().call_setup(&mut store)?;

        Ok(Self {
            engine,
            store,
            component,
            linker,
            robot_component,
            robot_configuration,
        })
    }

    pub fn robot_configuration(&self) -> &Configuration {
        &self.robot_configuration
    }
}
