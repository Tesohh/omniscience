#let wasm = plugin("/resources/typst/lib/omniscience_typst.wasm") // TODO: replace with something else

#let ghost-link(body) = { text[#body] }

#let omni(
  title: "",
  tags: (),
  names: (),
  private: true,
  body,
) = {
  let nodes_toml = read("/build/nodes.toml", encoding: none)
  let config_toml = read("/omni.toml", encoding: none)

  let wasm = plugin.transition(
    wasm.init,
    nodes_toml,
    config_toml,
  )

  assert.ne(title, "", message: "empty title. please provide a title.")

  [#metadata((
    title: title,
    tags: tags,
    names: names,
    private: private,
  )) <omni-frontmatter>]

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

    assert(not res.starts-with("err: "), message: res.replace("err: ", ""))

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
        to: file_part,
        ghost: true,
      )) <omni-link>]
    } else {
      link(node.target)[#node.content]
      [#metadata((
        content: node.content,
        to: node.to,
        ghost: false,
      )) <omni-link>]
    }
  }
  body
}
