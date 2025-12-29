#let plugin = plugin("../../target/wasm32-unknown-unknown/release/omniscience_typst.wasm")

#str(
  plugin.parse_link(),
  plugin.hello(),
)
