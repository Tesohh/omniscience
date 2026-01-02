#import "/resources/typst/lib/omni.typ": omni


#let note(
  title: "",
  tags: (),
  names: (),
  body,
) = {
  show: omni.with(title: title, tags: tags, names: names)

  // your custom template here

  body
}
