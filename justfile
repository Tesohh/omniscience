build_plugin:
    RUSTFLAGS="-C panic=abort" cargo build --release --package omniscience_typst --target wasm32-wasip1
    wasi-stub target/wasm32-wasip1/release/omniscience_typst.wasm -o target/wasm32-wasip1/release/omniscience_typst.wasm
