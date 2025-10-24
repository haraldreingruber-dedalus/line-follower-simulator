use executor::wasmtime;

use crate::runner::{BotExecutionData, run_bot_from_code};

pub fn start_server(
    address: String,
    port: u16,
    period: u32,
    sender: std::sync::mpsc::Sender<wasmtime::Result<BotExecutionData>>,
) -> wasmtime::Result<()> {
    let server = tiny_http::Server::http(format!("{}:{}", address, port))
        .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
    std::thread::spawn(move || run_server(server, period, sender));
    Ok(())
}

fn run_server(
    server: tiny_http::Server,
    period: u32,
    sender: std::sync::mpsc::Sender<wasmtime::Result<BotExecutionData>>,
) {
    for mut request in server.incoming_requests() {
        let response = match request.method() {
            tiny_http::Method::Post => {
                let body_reader = request.as_reader();
                let mut wasm_bytes = Vec::new();
                match body_reader.read_to_end(&mut wasm_bytes) {
                    Ok(_) => {
                        let result_sender = sender.clone();
                        std::thread::spawn(move || {
                            result_sender
                                .send(run_bot_from_code(wasm_bytes, None, false, period))
                                .ok();
                        });
                        tiny_http::Response::from_string("Robot code received successfully")
                            .with_status_code(200)
                    }
                    Err(err) => tiny_http::Response::from_string(format!(
                        "Error reading request body: {}",
                        err
                    ))
                    .with_status_code(500),
                }
            }
            _ => tiny_http::Response::from_string(
                "Simulator ready.\nSend robot code with a POST request.\n",
            )
            .with_status_code(200),
        };
        request.respond(response).ok();
    }
}
