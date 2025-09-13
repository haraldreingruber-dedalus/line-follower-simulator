#[macro_use]
extern crate std;

pub mod bindings;
pub mod bot_executor;
pub mod bot_wasm_host;
pub mod mock_stepper;

fn main() -> wasmtime::Result<()> {
    // Load the component from disk
    let wasm_bytes = std::fs::read("../bot/target/wasm32-wasip1/release/line_follower_robot.wasm")?;

    // Create a mock stepper
    let stepper = mock_stepper::MockStepper::new();

    // Create the bot executor
    let bot_executor = bot_executor::BotExecutor::new(&wasm_bytes, stepper)?;

    println!(
        "Robot configuration: {:#?}",
        bot_executor.robot_configuration()
    );

    Ok(())
}
