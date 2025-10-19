# wit-bindgen rust ../wit/world.wit --out-dir src
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/line_follower_robot.wasm ../sim
