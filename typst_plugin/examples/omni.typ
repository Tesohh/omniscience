#let wasm = plugin("/target/wasm32-wasip1/release/omniscience_typst.wasm")

#let ghost-link(body) = { text[#body] }

#let omni(
  title: str,
  tags: (),
  names: (),
  body,
) = {
  let nodes_toml = bytes(
    "[[node]]
id = \"id1\"
title = \"Matrix\"
path = \"cs/c/matrix.md\"
kind = \"file\"
names = [\"matrix\"]
tags = [\"programming\"]

  [[node]]
  id = \"id1\"
  title = \"Matrix\"
  path = \"cs/linalg/matrix.md\"
  kind = \"file\"
  names = [\"matrix\"]
  tags = []
  ",
  )

  let config_toml = bytes(
    "[project]
name = \"my_proj\"

[dir_aliases]
linalg = \"Linear Algebra\"",
  )

  let res = str(wasm.init(nodes_toml, config_toml))
  assert(not res.starts-with("err: "), message: res.replace("err: ", "omni: "))

  // let wasm = plugin.transition(
  //   wasm.init,
  //   nodes_toml,
  //   config_toml,
  // )

  assert.ne(title, "", message: "empty title. please provide a title.")

  show ref: it => {
    if not str(it.target).starts-with("omni.") {
      return it
    }

    let target = str(it.target)
    let splits = target.split(":")

    let file_part = splits.at(0).replace("omni.", "")
    let heading_part = splits.at(1, default: "")
    let alias = ""
    if type(it.supplement) == content or type(it.supplement) == str {
      alias = it.supplement
    }

    let res = str(wasm.parse_link(
      bytes(file_part),
      bytes(heading_part),
      bytes(alias),
    ))
    assert(not res.starts-with("err: "), message: res.replace("err: ", "omni: "))

    let splits = str(res).split(",")
    let node = (
      content: splits.at(0),
      target: splits.at(1),
      to: splits.at(2),
    )

    if node.target == "ghost" {
      ghost-link[#node.content]
      [#metadata((
        content: node.content,
        to: "ghost", // TODO: put file part here
      )) <omnilink>]
    } else {
      link(node.target)[#node.target]
      [#metadata((
        content: node.content,
        to: node.to,
      )) <omnilink>]
    }
  }
  body
}
