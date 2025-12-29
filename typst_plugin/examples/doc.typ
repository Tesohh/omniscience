#let omni = plugin("../../target/wasm32-wasip1/release/omniscience_typst.wasm")

#let nodes_toml = bytes(
  "[[node]]
id = \"id1\"
path = \"cs/c/matrix.md\"
kind = \"file\"
names = [\"matrix\"]
tags = [\"programming\"]",
)

#let config_toml = bytes(
  "[project]
name = \"my_proj\"

[dir_aliases]
linalg = \"Linear Algebra\"",
)

#let omni = plugin.transition(
  omni.init,
  nodes_toml,
  config_toml,
)


#let res = omni.parse_link(
  bytes("cs.c.matrix"),
  bytes("operations.addition"),
  bytes("my alias"),
)

#str(res)

#let splits = str(res).split(",")
#link(splits.at(1))[#splits.at(0)]

