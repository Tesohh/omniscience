#let plugin = plugin("../../target/wasm32-wasip1/release/omniscience_typst.wasm")

#str(
  plugin.parse_link(),
  plugin.hello(),
)
