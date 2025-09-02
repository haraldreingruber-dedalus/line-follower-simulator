use wasmtime::component::bindgen;

bindgen!({
    path: "../sample-robot-code/wit/"
});

use crate::{
    motors::MotorPower,
    sensors::{
        AsyncSensorResult, AsyncVoidResult, FutureHandle, ReadError, SensorIndexRange, SensorKind,
        SensorValues, TimeUs,
    },
};

pub struct BotHost {}

impl sensors::Host for BotHost {
    fn current_time(&mut self) -> TimeUs {
        0
    }

    fn sleep_blocking_for(&mut self, us: TimeUs) -> () {
        std::thread::sleep(std::time::Duration::from_micros(us as u64));
    }

    fn sleep_blocking_until(&mut self, _us: TimeUs) -> () {}

    fn read_sensor_blocking(
        &mut self,
        _sensor: SensorKind,
        indexes: SensorIndexRange,
    ) -> Result<SensorValues, ReadError> {
        Ok(vec![0; indexes.count as usize])
    }

    fn sleep_async_for(&mut self, _us: TimeUs) -> FutureHandle {
        0
    }

    fn sleep_async_until(&mut self, _us: TimeUs) -> FutureHandle {
        0
    }

    fn read_sensor_async(
        &mut self,
        _sensor: SensorKind,
        _sensor_index_range: SensorIndexRange,
    ) -> FutureHandle {
        0
    }

    fn poll_timer(&mut self, _handle: FutureHandle) -> Result<AsyncVoidResult, ReadError> {
        Ok(AsyncVoidResult::Blocked)
    }

    fn poll_sensor(&mut self, _handle: FutureHandle) -> Result<AsyncSensorResult, ReadError> {
        Ok(AsyncSensorResult::Blocked)
    }

    fn forget_handle(&mut self, _handle: FutureHandle) -> () {}
}

impl motors::Host for BotHost {
    fn set_power(&mut self, _left: MotorPower, _right: MotorPower) -> () {}
}

fn main() -> wasmtime::Result<()> {
    // Instantiate the engine and store
    let engine = wasmtime::Engine::default();
    let mut store = wasmtime::Store::new(&engine, BotHost {});

    // Load the component from disk
    let bytes =
        std::fs::read("sample-robot-code/target/wasm32-wasip1/release/line_follower_robot.wasm")?;
    let component = wasmtime::component::Component::new(&engine, bytes)?;

    // Configure the linker
    let mut linker = wasmtime::component::Linker::new(&engine);

    // Ignore unknown imports
    linker.define_unknown_imports_as_traps(&component)?;

    // Instantiate component host
    let robot_component = LineFollowerRobot::instantiate(&mut store, &component, &linker)?;

    let config = robot_component.robot().call_setup(&mut store)?;

    println!("Robot configuration: {:#?}", config);

    Ok(())
}
